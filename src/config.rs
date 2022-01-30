use std::fs::File;
use std::io::{self, Read};
use serde::Deserialize;

pub fn load_config_from_path(path: &str) -> io::Result<Config> {
    let mut reader = io::BufReader::new(File::open(path)?);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(serde_json::from_str::<Config>(&contents).unwrap())
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub caseta: CasetaConfig,
    pub aurora: AuroraConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CasetaConfig {
    pub ca_cert_path: String,
    pub cert_path: String,
    pub key_path: String,
    pub address: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuroraConfig {
    pub address: String,
    pub token: String,
}
