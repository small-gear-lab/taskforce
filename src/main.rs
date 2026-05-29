use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = taskforce::cli::Cli::parse();
    taskforce::app::run(cli)
}
