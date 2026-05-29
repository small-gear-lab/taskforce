use anyhow::Result;
use chrono::NaiveDate;

use crate::backend::{NewTaskInput, Task, TaskBackend, UpdateTaskInput};
use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::local_backend::LocalBackend;

pub async fn run(cli: Cli) -> Result<()> {
    let config = AppConfig::load()?;
    let client = LocalBackend::new(config.resolve_sqlite_path()?)?;

    match cli.command {
        Commands::List => {
            let tasks = client.list_pending()?;
            print_tasks(&tasks);
        }
        Commands::Add {
            title,
            target_date,
            deadline,
            launch_date,
            target_time_hint,
            deadline_time_hint,
            launch_time_hint,
            project,
            tags,
        } => {
            let task = client.add(NewTaskInput {
                title,
                target_date: parse_optional_date(target_date)?,
                deadline: parse_optional_date(deadline)?,
                launch_date: parse_optional_date(launch_date)?,
                target_time_hint,
                deadline_time_hint,
                launch_time_hint,
                project,
                tags,
            })?;
            println!("added {}: {}", task.id_text(), task.title());
        }
        Commands::Edit {
            id,
            title,
            target_date,
            deadline,
            launch_date,
            target_time_hint,
            deadline_time_hint,
            launch_time_hint,
            project,
            tags,
        } => {
            let task = client.edit(
                id,
                UpdateTaskInput {
                    title,
                    target_date: parse_optional_date(target_date)?,
                    deadline: parse_optional_date(deadline)?,
                    launch_date: parse_optional_date(launch_date)?,
                    target_time_hint,
                    deadline_time_hint,
                    launch_time_hint,
                    project,
                    tags: (!tags.is_empty()).then_some(tags),
                },
            )?;
            println!("updated {}: {}", task.id_text(), task.title());
        }
        Commands::Delete { id } => {
            let task = client.delete(id)?;
            println!("deleted {}: {}", task.id_text(), task.title());
        }
        Commands::Done { id } => {
            let task = client.mark_done(id)?;
            println!("done {}: {}", task.id_text(), task.title());
        }
        Commands::Next => match client.next_task()? {
            Some(task) => println!("next {}: {}", task.id_text(), task.title()),
            None => println!("no pending tasks"),
        },
        Commands::Serve => {
            crate::web::serve(client, config.server.resolve()?).await?;
        }
    }

    Ok(())
}

fn print_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("no pending tasks");
        return;
    }

    for task in tasks {
        println!("{} {}", task.id_text(), task.title());
    }
}

fn parse_optional_date(value: Option<String>) -> Result<Option<NaiveDate>> {
    value
        .map(|text| {
            NaiveDate::parse_from_str(&text, "%Y-%m-%d")
                .map_err(|err| anyhow::anyhow!("invalid date `{text}`: {err}"))
        })
        .transpose()
}
