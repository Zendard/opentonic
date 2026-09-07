use crate::OpentonicError;
use serde::Deserialize;
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub host_address: std::net::IpAddr,
    pub host_port: u16,
    pub html_dir: std::path::PathBuf,
    pub static_dir: std::path::PathBuf,
    pub db_file: std::path::PathBuf,
}

const DEFAULT_CONFIG_LOCATION: &str = "/etc/opentonic/config.toml";

impl Config {
    pub fn get(location: Option<&str>) -> Result<Self, OpentonicError> {
        let location = location.unwrap_or(DEFAULT_CONFIG_LOCATION);
        let string = std::fs::read_to_string(location).unwrap_or_default();
        let config: Config = toml::de::from_str(&string)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host_address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            host_port: 80,
            html_dir: PathBuf::from_str("html").unwrap(),
            static_dir: PathBuf::from_str("static").unwrap(),
            db_file: PathBuf::from_str("db.sqlite").unwrap(),
        }
    }
}
