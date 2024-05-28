use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub init_path: PathBuf,
}

pub fn read_config() -> Config {
    let config_str = include_str!("../../config.toml");
    toml::from_str::<Config>(config_str).expect("Failed to parse userspace config.")
}
