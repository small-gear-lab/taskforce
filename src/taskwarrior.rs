use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

pub struct TaskwarriorClient;

impl Default for TaskwarriorClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskwarriorClient {
    pub fn new() -> Self {
        Self
    }

    pub fn list_pending(&self) -> Result<Vec<Task>> {
        let mut tasks = self.export(["status:pending"])?;
        sort_tasks(&mut tasks);
        Ok(tasks)
    }

    pub fn add(&self, description: &str) -> Result<Task> {
        let before = self.list_pending()?;
        self.run(["add", description])?;
        let mut after = self.list_pending()?;

        after.retain(|task| !before.iter().any(|existing| existing.uuid == task.uuid));
        sort_tasks(&mut after);

        after
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("task was added but could not be identified"))
    }

    pub fn mark_done(&self, id: u64) -> Result<Task> {
        let task = self.find_pending(id)?;
        self.run([&id.to_string(), "done"])?;
        Ok(task)
    }

    pub fn next_task(&self) -> Result<Option<Task>> {
        let mut tasks = self.list_pending()?;
        Ok(tasks.drain(..).next())
    }

    fn find_pending(&self, id: u64) -> Result<Task> {
        self.list_pending()?
            .into_iter()
            .find(|task| task.id == Some(id))
            .ok_or_else(|| anyhow!("pending task {} not found", id))
    }

    fn export<const N: usize>(&self, filters: [&str; N]) -> Result<Vec<Task>> {
        let mut args = vec!["rc.confirmation=off", "rc.verbose=nothing"];
        args.extend(filters);
        args.push("export");

        let output = self.command_output(&args)?;
        let tasks: Vec<Task> =
            serde_json::from_slice(&output).context("failed to parse Taskwarrior JSON")?;
        Ok(tasks)
    }

    fn run<const N: usize>(&self, args: [&str; N]) -> Result<()> {
        self.command_output(&args)?;
        Ok(())
    }

    fn command_output(&self, args: &[&str]) -> Result<Vec<u8>> {
        let output = Command::new("task")
            .args(args)
            .output()
            .context("failed to spawn `task`; is Taskwarrior installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let detail = if !stderr.is_empty() { stderr } else { stdout };
            bail!("task command failed: {}", detail);
        }

        Ok(output.stdout)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: Option<u64>,
    pub uuid: String,
    pub description: String,
    #[serde(default)]
    pub urgency: f64,
}

impl Task {
    pub fn id_text(&self) -> String {
        self.id
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.uuid.clone())
    }
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
