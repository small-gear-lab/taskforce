use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::taskwarrior::TaskwarriorClient;

pub fn run(cli: Cli) -> Result<()> {
    let client = TaskwarriorClient::new();

    match cli.command {
        Commands::List => {
            let tasks = client.list_pending()?;
            print_tasks(&tasks);
        }
        Commands::Add { description } => {
            let task = client.add(&description)?;
            println!("added {}: {}", task.id_text(), task.description);
        }
        Commands::Done { id } => {
            let task = client.mark_done(id)?;
            println!("done {}: {}", task.id_text(), task.description);
        }
        Commands::Next => match client.next_task()? {
            Some(task) => println!("next {}: {}", task.id_text(), task.description),
            None => println!("no pending tasks"),
        },
    }

    Ok(())
}

fn print_tasks(tasks: &[crate::taskwarrior::Task]) {
    if tasks.is_empty() {
        println!("no pending tasks");
        return;
    }

    for task in tasks {
        println!("{} {}", task.id_text(), task.description);
    }
}
