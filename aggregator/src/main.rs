use std::env;

pub mod config;

pub mod capture;
use clap::Parser;

use crate::capture::start_capture;
use crate::config::Config;

/// The aggregator to collect from the exfiltrator
#[derive(Parser)]
pub struct Cli {
    /// The path to the config file
    #[clap(long)]
    #[clap(default_value_t = String::from("/etc/aggregator/config.toml"))]
    config_path: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "INFO");
        }
    }

    env_logger::init();

    let cli = Cli::parse();

    let config = Config::try_from(cli.config_path.as_str())?;

    start_capture(&config).await?;

    Ok(())
}
