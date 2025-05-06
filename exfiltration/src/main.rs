use std::{net::Ipv6Addr, path::Path};

use clap::Parser;

/// Exfiltration toolbox
#[derive(Parser)]
#[clap(version)]
pub struct Cli {
    /// Target to send the file to
    target: Ipv6Addr,

    /// Path to the file to exfiltrate
    file_path: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let path = Path::new(&cli.file_path);

    if !path.exists() {
        return Err(format!("File {} does not exist", cli.file_path));
    }

    if !path.is_file() {
        return Err(format!("Path {} is not a file", cli.file_path));
    }

    Ok(())
}
