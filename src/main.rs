use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = taskforce::cli::Cli::parse();
    taskforce::app::run(cli).await
}
