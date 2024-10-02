use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub env_vars_path: PathBuf,
}

pub fn read() -> Config {
    let config_str = include_str!("../../config.toml");
    toml::from_str::<Config>(config_str).expect("Failed to parse userspace config.")
}
