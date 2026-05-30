use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    if let Some(path) = taskforce::config::base_env_file_path()
        && path.exists()
    {
        dotenvy::from_path(&path)?;
    }

    let cli_env = taskforce::config::bootstrap_environment_from_args(std::env::args_os())?;
    if let Some(environment) = taskforce::config::resolve_environment_from_sources(cli_env)? {
        unsafe {
            std::env::set_var("TASKFORCE_ENV", environment);
        }
    }

    if let Some(path) = taskforce::config::env_file_path()
        && path.exists()
    {
        dotenvy::from_path_override(&path)?;
    }

    taskforce::i18n::init()?;

    let cli = taskforce::cli::Cli::parse();
    taskforce::app::run(cli).await
}
