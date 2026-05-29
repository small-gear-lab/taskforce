use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "taskforce")]
#[command(about = "Thin Taskwarrior-backed task workflow")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    List,
    Add { description: String },
    Done { id: u64 },
    Next,
}
