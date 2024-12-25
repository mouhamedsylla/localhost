use serde::Deserialize;
use std::fs;


#[derive(Deserialize, Debug)]
pub struct Route {
    pub path: String,
    pub methods: Vec<String>,
    pub root: String,
    pub default_page: String,
    pub directory_listing: bool,
}

#[derive(Deserialize, Debug)]
pub struct Host {
    pub server_address: String,
    pub ports: Vec<String>,
    pub server_name: String,
    pub routes: Vec<Route>,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub servers: Vec<Host>,
}

pub fn load_config() -> std::io::Result<Server> {
    let config_content = fs::read_to_string("./src/config/config.json")?;
    let config: Server = serde_json::from_str(&config_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(config)
}
