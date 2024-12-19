use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize)]
pub struct StaticFiles {
    pub directory: String,
    pub default_page: String,
    pub list_directory: bool,
}

#[derive(Deserialize)]
pub struct Host {
    pub port: String,
    pub name: String,
    pub static_files: Option<StaticFiles>,
}

#[derive(Deserialize)]
pub struct Server {
    pub servers: Vec<Host>,
}

pub fn load_config() -> std::io::Result<Server> {
    let config_content = fs::read_to_string("./src/config/config.json")?;
    let config: Server = serde_json::from_str(&config_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(config)
}