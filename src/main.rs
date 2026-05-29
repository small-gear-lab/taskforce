use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    if let Some(path) = taskforce::config::env_file_path()
        && path.exists()
    {
        dotenvy::from_path(&path)?;
    }

    taskforce::i18n::init()?;

    let cli = taskforce::cli::Cli::parse();
    taskforce::app::run(cli).await
}
