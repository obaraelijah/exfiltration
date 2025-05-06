use std::fs::read_to_string;
use std::net::IpAddr;
use std::path::Path;

use serde::Deserialize;

/// Interface section
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Interface {
    /// The interface to listen on
    pub interface: String,
    /// The adddress to listen to
    pub listen_address: IpAddr,
}

/// The config of the aggregator
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    /// Interface section of the config
    pub interface: Interface,
}

impl TryFrom<&str> for Config {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = Path::new(value);

        if !path.exists() || !path.is_file() {
            return Err(format!("No config file found at: {value}"));
        }

        let config_str = read_to_string(path).map_err(|e| format!("Error reading {value}: {e}"))?;
        Ok(toml::from_str(&config_str).map_err(|e| format!("Error deserializing config: {e}"))?)
    }
}
