use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::net::AddrParseError;
use std::path::Path;
use std::collections::HashMap;
use crate::server::logger::{Logger, LogLevel};

use crate::server::route;

const ALLOWED_EXTENSIONS: [&str; 1] = ["py"];
const ALLOWED_STATUS: [&str; 8] = ["400", "403", "404", "405", "413", "500", "502", "503"];
const MODULE : &str = "CONFIG";

#[derive(Deserialize, Debug)]
pub struct CgiConfig {
    pub extension: String,
    pub scrpit_path: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorPages {
    pub custom_pages: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Route {
    pub path: Option<String>,
    pub methods: Option<Vec<String>>,
    pub root: Option<String>,
    pub default_page: Option<String>,
    pub directory_listing: Option<bool>,
    pub cgi: Option<CgiConfig>,
}

#[derive(Deserialize, Debug)]
pub struct Host {
    pub server_address: Option<String>,
    pub ports: Option<Vec<String>>,
    pub server_name: Option<String>,
    pub routes: Option<Vec<Route>>,
    pub error_pages: Option<ErrorPages>,
    pub client_max_body_size: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub servers: Vec<Host>,
    #[serde(skip)]
    pub validation_errors: Vec<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Critical(String),
    Warning(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Critical(s) => write!(f, "Critical error: {}", s),
            ConfigError::Warning(s) => write!(f, "Warning: {}", s),
        }
    }
}

impl CgiConfig {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        if self.extension.is_empty() {
            errors.push(ConfigError::Warning("CgiConfig extension is empty".to_string()));
        }
        if self.scrpit_path.is_empty() {
            errors.push(ConfigError::Warning("CgiConfig interpreter_path is empty".to_string()));
        }
        if !ALLOWED_EXTENSIONS.contains(&self.extension.as_str()) {
            errors.push(ConfigError::Warning("CgiConfig extension is not allowed".to_string()));
        }
        errors
    }
}

impl ErrorPages {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        for (status, path) in &self.custom_pages {
            if status.is_empty() {
                errors.push(ConfigError::Warning("ErrorPages status is empty".to_string()));
            }
            if path.is_empty() {
                errors.push(ConfigError::Warning("ErrorPages path is empty".to_string()));
            }
            if !Path::new(path).exists() {
                errors.push(ConfigError::Warning("ErrorPages path does not exist".to_string()));
            }

            if status.parse::<u16>().is_err() {
                errors.push(ConfigError::Warning("ErrorPages status is not a number".to_string()));
            }

            if !ALLOWED_STATUS.contains(&status.as_str()) {
                errors.push(ConfigError::Warning("ErrorPages status is not allowed".to_string()));
            }
        }
        errors
    }
}


impl Route {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        if let Some(path) = &self.path {
            if path.is_empty() {
                errors.push(ConfigError::Warning("Route path is empty".to_string()));
            }
        } else {
            errors.push(ConfigError::Warning("Route is undefined".to_string()));
        }
        if let Some(path_start) = &self.path {
            if !path_start.starts_with("/") {
                errors.push(ConfigError::Warning("Route path does not start with /".to_string()));
            }
        }
        if let Some(method) = &self.methods {
            if method.is_empty() {
                errors.push(ConfigError::Warning("Route methods is empty".to_string()));
            }
        } else {
            errors.push(ConfigError::Warning("Route methods is undefined".to_string()));
        }
        if let Some(root) = &self.root {
            if root.is_empty() {
                errors.push(ConfigError::Warning("Route root is empty".to_string()));
            }
        } else {
            errors.push(ConfigError::Warning("Route root is undefined".to_string()));
        }
        if let Some(root) = &self.root {
            if !Path::new(root).exists() {
                errors.push(ConfigError::Warning("Route root does not exist".to_string()));
            }
        }
        if self.default_page.is_some() {
            let default_page = self.default_page.as_ref().unwrap();
            if !Path::new(default_page).exists() {
                errors.push(ConfigError::Warning("Route default_page does not exist".to_string()));
            }
        }
        if let Some(cgi) = &self.cgi {
            errors.append(&mut cgi.validate());
        }
        errors
    }
    
}

impl Host {
    pub fn is_valid_essential_config(&self) -> Result<(), ConfigError> {
        let mut has_valid_port = false;

        if let Some(server_name) = &self.server_name {
            if server_name.is_empty() {
                return Err(ConfigError::Critical("Host server_name is empty".to_string()));
            }
        } else {
            return Err(ConfigError::Critical("Host server_name is undefined".to_string()));
        }

        if let Some(server_address) = &self.server_address {
            match server_address.parse::<std::net::IpAddr>() {
                Ok(_) => {},
                Err(e) => return Err(ConfigError::Critical(format!("Host server_address is invalid: {}", e))),
            }
        } else {
            return Err(ConfigError::Critical("Host server_address is undefined".to_string()));
        }

        if let Some(ports) = &self.ports {
            if ports.is_empty() {
                return Err(ConfigError::Critical("Host ports is empty".to_string()));
            }
        } else {
            return Err(ConfigError::Critical("Host ports is undefined".to_string()));
        }

        if let Some(ports) = &self.ports {
            has_valid_port = ports.iter().all(|port| {
                port.parse::<u16>().is_ok()
            });
        }

        if !has_valid_port {
            return Err(ConfigError::Critical("Host ports contains invalid port".to_string()));
        } 

        Ok(())
    }

    pub fn collect_warnings(&self) -> Vec<ConfigError> {
        let mut warnings = Vec::new();

        let mut unique_ports = std::collections::HashSet::new();
        if let Some(ports) = &self.ports {
            for port in ports {
                if let Ok(port_num) = port.parse::<u16>() {
                    if !unique_ports.insert(port_num) {
                        warnings.push(ConfigError::Warning("Host ports contains duplicate port".to_string()));
                    }
                }
            }
        }

        if let Some(size) = &self.client_max_body_size {
            if !size.ends_with("k") && !size.ends_with("m") {
                warnings.push(ConfigError::Warning("Host client_max_body_size is not in k or m".to_string()));
            }
        }

        if let Some(routes) = &self.routes {
            for route in routes {
                warnings.extend(route.validate());
            }
        }

        if let Some(error_pages) = &self.error_pages {
            warnings.extend(error_pages.validate());
        }

        warnings
    }
}

impl ServerConfig {
    pub fn load_and_validate() -> Result<ServerConfig, ConfigError> {
        let logger = Logger::new(LogLevel::DEBUG);

        let config_content = fs::read_to_string("./src/config/config.json")
            .map_err(|e| {
                logger.error(&format!("Cannot read config file: {}", e), MODULE);
                ConfigError::Critical(format!("Cannot read config file: {}", e))
            })?;

        let mut config: ServerConfig = serde_json::from_str(&config_content)
            .map_err(|e| {
                logger.error(&format!("Cannot parse config file: {}", e), MODULE);
                ConfigError::Critical(format!("Cannot parse config file: {}", e))
            })?;

        let mut server_names = std::collections::HashSet::new();
        let mut validation_errors = Vec::new();

        config.servers.retain(|host| {
            match host.is_valid_essential_config() {
                Ok(()) => {
                    if !server_names.insert(host.server_name.clone()) {
                        validation_errors.push(format!("Duplicate server name: {:?}", host.server_name));
                        
                        return false;
                    } else {
                        let warnings = host.collect_warnings();
                        if !warnings.is_empty() {
                            for warn in warnings {
                                match warn {
                                    ConfigError::Critical(msg) => {
                                        logger.warn(&msg, MODULE);
                                    },
                                    ConfigError::Warning(msg) => {
                                        logger.error(&msg, MODULE);
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                Err(e) => {
                    match e {
                        ConfigError::Critical(msg) => {
                            logger.error(&msg, MODULE);
                        },
                        ConfigError::Warning(msg) => {
                            logger.warn(&msg, MODULE);
                        }
                    }
                    false
                }
                
            }
        });

        if config.servers.is_empty() {
            let msg = "No valid server configuration found";
            logger.error(&msg, MODULE);
            return Err(ConfigError::Critical("No valid server configuration found".to_string()));
        }

        if !config.validation_errors.is_empty() {
            for error in &config.validation_errors {
                logger.error(&error, MODULE);
            }
            return Err(ConfigError::Critical("Invalid configuration".to_string()));
        }

        Ok(config)
    }
}