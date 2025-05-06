use clap::{Parser, Subcommand};

mod exfiltrators;
mod config;
mod logger;

#[derive(Parser)]
#[command(name = "exfiltration")]
#[command(about = "A pentesting toolbox for data exfiltration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Exfil {
        #[arg(short, long)]
        method: String, 
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Exfil { method } => {
            println!("Exfiltrating using method: {}", method);
        }
    }
}