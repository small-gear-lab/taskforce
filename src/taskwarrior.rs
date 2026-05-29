use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

use crate::backend::{Task, TaskBackend};
use crate::config::{AppConfig, task_bin_env};

pub trait TaskRunner {
    fn command_output(&self, args: &[String]) -> Result<Vec<u8>>;
}

#[derive(Clone, Default)]
pub struct SystemTaskRunner {
    task_bin: PathBuf,
}

impl TaskRunner for SystemTaskRunner {
    fn command_output(&self, args: &[String]) -> Result<Vec<u8>> {
        let output = self.run_command(args)?;

        if output.status.success() {
            return Ok(output.stdout);
        }

        let detail = command_failure_detail(&output);

        if is_missing_rc_error(&detail) {
            bootstrap_taskwarrior_rc()?;
            let retry = self.run_command(args)?;
            if retry.status.success() {
                return Ok(retry.stdout);
            }

            bail!("task command failed: {}", command_failure_detail(&retry));
        }

        bail!("task command failed: {}", detail);
    }
}

impl SystemTaskRunner {
    fn run_command(&self, args: &[String]) -> Result<std::process::Output> {
        Command::new(&self.task_bin)
            .args(args)
            .output()
            .with_context(|| {
                format!(
                    "failed to spawn `{}`; is Taskwarrior installed?",
                    self.task_bin.display()
                )
            })
    }
}

#[derive(Clone)]
pub struct TaskwarriorClient<R = SystemTaskRunner> {
    runner: R,
}

impl TaskwarriorClient<SystemTaskRunner> {
    pub fn new() -> Result<Self> {
        let config = AppConfig::load()?;
        let task_bin = resolve_task_bin(&config)?;
        Ok(Self {
            runner: SystemTaskRunner { task_bin },
        })
    }
}

impl<R> TaskwarriorClient<R>
where
    R: TaskRunner + Clone,
{
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }

    fn list_pending_impl(&self) -> Result<Vec<Task>> {
        let mut tasks = self.export(["status:pending"])?;
        sort_tasks(&mut tasks);
        Ok(tasks)
    }

    fn add_impl(&self, description: &str) -> Result<Task> {
        let before = self.list_pending_impl()?;
        self.run(vec!["add".into(), description.into()])?;
        let mut after = self.list_pending_impl()?;

        after.retain(|task| !before.iter().any(|existing| existing.uuid == task.uuid));
        sort_tasks(&mut after);

        after
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("task was added but could not be identified"))
    }

    fn edit_impl(&self, id: u64, description: &str) -> Result<Task> {
        self.find_pending(id)?;
        self.run(vec![id.to_string(), "modify".into(), description.into()])?;
        self.find_pending(id)
    }

    fn delete_impl(&self, id: u64) -> Result<Task> {
        let task = self.find_pending(id)?;
        self.run(vec![id.to_string(), "delete".into()])?;
        Ok(task)
    }

    fn mark_done_impl(&self, id: u64) -> Result<Task> {
        let task = self.find_pending(id)?;
        self.run(vec![id.to_string(), "done".into()])?;
        Ok(task)
    }

    fn next_task_impl(&self) -> Result<Option<Task>> {
        let mut tasks = self.list_pending_impl()?;
        Ok(tasks.drain(..).next())
    }

    fn find_pending(&self, id: u64) -> Result<Task> {
        self.list_pending_impl()?
            .into_iter()
            .find(|task| task.id == Some(id))
            .ok_or_else(|| anyhow!("pending task {} not found", id))
    }

    fn export<const N: usize>(&self, filters: [&str; N]) -> Result<Vec<Task>> {
        let mut args = base_args();
        args.extend(filters.into_iter().map(str::to_owned));
        args.push("export".into());

        let output = self.runner.command_output(&args)?;
        let tasks: Vec<Task> =
            serde_json::from_slice(&output).context("failed to parse Taskwarrior JSON")?;
        Ok(tasks)
    }

    fn run(&self, args: Vec<String>) -> Result<()> {
        self.runner.command_output(&args)?;
        Ok(())
    }
}

impl<R> TaskBackend for TaskwarriorClient<R>
where
    R: TaskRunner + Clone,
{
    fn list_pending(&self) -> Result<Vec<Task>> {
        self.list_pending_impl()
    }

    fn add(&self, description: &str) -> Result<Task> {
        self.add_impl(description)
    }

    fn edit(&self, id: u64, description: &str) -> Result<Task> {
        self.edit_impl(id, description)
    }

    fn delete(&self, id: u64) -> Result<Task> {
        self.delete_impl(id)
    }

    fn mark_done(&self, id: u64) -> Result<Task> {
        self.mark_done_impl(id)
    }

    fn next_task(&self) -> Result<Option<Task>> {
        self.next_task_impl()
    }
}

fn base_args() -> Vec<String> {
    vec!["rc.confirmation=off".into(), "rc.verbose=nothing".into()]
}

fn sort_tasks(tasks: &mut [Task]) {
    tasks.sort_by(|left, right| {
        right
            .urgency
            .partial_cmp(&left.urgency)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn resolve_task_bin(config: &AppConfig) -> Result<PathBuf> {
    if let Some(task_bin) = task_bin_env() {
        return Ok(task_bin);
    }

    if let Some(task_bin) = config.task_bin.clone() {
        return Ok(task_bin);
    }

    if let Some(path) = find_in_path("task") {
        return Ok(path);
    }

    bail!(
        "could not find `task`; set TASKFORCE_TASK_BIN, configure task_bin in XDG_CONFIG_HOME/taskforce/config.toml, or add it to PATH"
    )
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;

    std::env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|candidate| is_executable_file(candidate))
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn command_failure_detail(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stderr.is_empty() { stderr } else { stdout }
}

fn is_missing_rc_error(detail: &str) -> bool {
    detail.contains("Cannot proceed without rc file.")
        || detail.contains("A configuration file could not be found")
}

fn bootstrap_taskwarrior_rc() -> Result<()> {
    let rc_path = default_taskrc_path()?;
    if rc_path.exists() {
        return Ok(());
    }

    if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let data_dir = default_taskdata_path()?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("failed to create {}", data_dir.display()))?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&rc_path)
        .with_context(|| format!("failed to create {}", rc_path.display()))?;

    use std::io::Write;
    file.write_all(default_taskrc_contents().as_bytes())
        .with_context(|| format!("failed to write {}", rc_path.display()))?;

    Ok(())
}

fn default_taskrc_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".taskrc"))
}

fn default_taskdata_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".task"))
}

fn default_taskrc_contents() -> &'static str {
    "# [Created by taskforce]\n# Minimal Taskwarrior configuration.\ndata.location=~/.task\n"
}

#[derive(Clone)]
pub struct FixtureRunner {
    #[cfg(test)]
    tasks: std::sync::Arc<Vec<Task>>,
}

#[cfg(not(test))]
impl TaskRunner for FixtureRunner {
    fn command_output(&self, _args: &[String]) -> Result<Vec<u8>> {
        unreachable!("FixtureRunner is only available in tests")
    }
}

#[cfg(test)]
impl FixtureRunner {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks: std::sync::Arc::new(tasks),
        }
    }
}

#[cfg(test)]
impl TaskRunner for FixtureRunner {
    fn command_output(&self, _args: &[String]) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self.tasks.as_ref())?)
    }
}

#[cfg(test)]
impl TaskwarriorClient<FixtureRunner> {
    pub fn from_tasks(tasks: Vec<Task>) -> Self {
        TaskwarriorClient::with_runner(FixtureRunner::new(tasks))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use anyhow::{Result, anyhow};

    use crate::backend::TaskBackend;
    use crate::config::AppConfig;

    use super::{
        TaskRunner, TaskwarriorClient, bootstrap_taskwarrior_rc, default_taskrc_contents,
        find_in_path, resolve_task_bin,
    };

    #[derive(Clone)]
    struct MockRunner {
        commands: Arc<Mutex<Vec<Vec<String>>>>,
        outputs: Arc<Mutex<VecDeque<Vec<u8>>>>,
    }

    impl MockRunner {
        fn new(outputs: Vec<Vec<u8>>) -> Self {
            Self {
                commands: Arc::new(Mutex::new(Vec::new())),
                outputs: Arc::new(Mutex::new(outputs.into())),
            }
        }

        fn commands(&self) -> Vec<Vec<String>> {
            self.commands.lock().expect("commands lock").clone()
        }
    }

    impl TaskRunner for MockRunner {
        fn command_output(&self, args: &[String]) -> Result<Vec<u8>> {
            self.commands
                .lock()
                .expect("commands lock")
                .push(args.to_vec());

            self.outputs
                .lock()
                .expect("outputs lock")
                .pop_front()
                .ok_or_else(|| anyhow!("missing mock output"))
        }
    }

    #[test]
    fn edit_updates_task_description() -> Result<()> {
        let runner = MockRunner::new(vec![
            br#"[{"id":4,"uuid":"before","description":"Old title","urgency":2.0}]"#.to_vec(),
            Vec::new(),
            br#"[{"id":4,"uuid":"before","description":"New title","urgency":2.0}]"#.to_vec(),
        ]);
        let client = TaskwarriorClient::with_runner(runner.clone());

        let task = client.edit(4, "New title")?;

        assert_eq!(task.description, "New title");
        assert_eq!(
            runner.commands(),
            vec![
                vec![
                    String::from("rc.confirmation=off"),
                    String::from("rc.verbose=nothing"),
                    String::from("status:pending"),
                    String::from("export"),
                ],
                vec![
                    String::from("4"),
                    String::from("modify"),
                    String::from("New title"),
                ],
                vec![
                    String::from("rc.confirmation=off"),
                    String::from("rc.verbose=nothing"),
                    String::from("status:pending"),
                    String::from("export"),
                ],
            ]
        );

        Ok(())
    }

    #[test]
    fn delete_issues_taskwarrior_delete() -> Result<()> {
        let runner = MockRunner::new(vec![
            br#"[{"id":9,"uuid":"deadbeef","description":"Obsolete","urgency":1.0}]"#.to_vec(),
            Vec::new(),
        ]);
        let client = TaskwarriorClient::with_runner(runner.clone());

        let task = client.delete(9)?;

        assert_eq!(task.description, "Obsolete");
        assert_eq!(
            runner.commands(),
            vec![
                vec![
                    String::from("rc.confirmation=off"),
                    String::from("rc.verbose=nothing"),
                    String::from("status:pending"),
                    String::from("export"),
                ],
                vec![String::from("9"), String::from("delete")],
            ]
        );

        Ok(())
    }

    #[test]
    fn resolve_task_bin_prefers_config() -> Result<()> {
        let config = AppConfig {
            task_bin: Some(PathBuf::from("/tmp/custom-task")),
            server: Default::default(),
        };

        let resolved = resolve_task_bin(&config)?;

        assert_eq!(resolved, PathBuf::from("/tmp/custom-task"));
        Ok(())
    }

    #[test]
    fn resolve_task_bin_prefers_env_override() -> Result<()> {
        let original_path = std::env::var_os("PATH");
        let original_env = std::env::var_os("TASKFORCE_TASK_BIN");
        unsafe { std::env::set_var("PATH", "/nonexistent") };
        unsafe { std::env::set_var("TASKFORCE_TASK_BIN", "/tmp/taskforce-task-from-env") };

        let resolved = resolve_task_bin(&AppConfig::default())?;

        match original_path {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        match original_env {
            Some(value) => unsafe { std::env::set_var("TASKFORCE_TASK_BIN", value) },
            None => unsafe { std::env::remove_var("TASKFORCE_TASK_BIN") },
        }

        assert_eq!(resolved, PathBuf::from("/tmp/taskforce-task-from-env"));
        Ok(())
    }

    #[test]
    fn find_in_path_detects_binary() -> Result<()> {
        let dir = unique_temp_dir("taskforce-path");
        let task_bin = dir.join("task");
        fs::create_dir_all(&dir)?;
        fs::write(&task_bin, "#!/bin/sh\n")?;

        let original_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &dir);
        }

        let found = find_in_path("task");

        match original_path {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert_eq!(found, Some(task_bin));
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn bootstrap_taskwarrior_rc_creates_minimal_files() -> Result<()> {
        let home = unique_temp_dir("taskforce-home");
        fs::create_dir_all(&home)?;
        let original_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", &home) };

        bootstrap_taskwarrior_rc()?;

        let rc_path = home.join(".taskrc");
        let data_dir = home.join(".task");
        let rc_contents = fs::read_to_string(&rc_path)?;

        match original_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(rc_contents, default_taskrc_contents());
        assert!(data_dir.is_dir());
        fs::remove_dir_all(home)?;
        Ok(())
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }
}
