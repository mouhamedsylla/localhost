use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::env;
use std::net::IpAddr;
use std::path::Path;
use std::collections::HashMap;
use crate::server::logger::{Logger, LogLevel};

const ALLOWED_EXTENSIONS: [&str; 1] = ["py"];
const ALLOWED_STATUS: [&str; 8] = ["400", "403", "404", "405", "413", "500", "502", "503"];
const ALLOWED_HTTP_METHODS: [&str; 3] = ["GET", "POST", "DELETE"];
const MODULE : &str = "CONFIG";

#[derive(Deserialize, Debug)]
pub struct CgiConfig {
    pub script_file_name: String,
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
    pub redirect: Option<String>,
    pub cgi: Option<CgiConfig>,
    pub session_required: Option<bool>,
    pub session_redirect: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionOptionsConfig {
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub max_age: Option<u64>,
    pub path: Option<String>,
    pub expires: Option<u64>,
    pub domain: Option<String>,
    pub same_site: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionConfig {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub options: Option<SessionOptionsConfig>,
}


#[derive(Deserialize, Debug)]
pub struct Host {
    pub server_address: Option<String>,
    pub ports: Option<Vec<String>>,
    pub server_name: Option<String>,
    pub routes: Option<Vec<Route>>,
    pub error_pages: Option<ErrorPages>,
    pub client_max_body_size: Option<String>,
    pub session: Option<SessionConfig>,
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

        // Vérifier si le nom du script est vide
        if self.script_file_name.is_empty() {
            errors.push(ConfigError::Warning("CgiConfig script_file_name is empty".to_string()));
            return errors;
        }
        
        // Extraire l'extension du fichier
        let extension = Path::new(&self.script_file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
            
        // Vérifier si l'extension est autorisée
        if extension.is_empty() {
            errors.push(ConfigError::Warning(format!(
                "CgiConfig script '{}' has no extension",
                self.script_file_name
            )));
        } else if !ALLOWED_EXTENSIONS.contains(&extension) {
            errors.push(ConfigError::Warning(format!(
                "CgiConfig script extension '{}' is not allowed. Allowed extensions: {:?}",
                extension, ALLOWED_EXTENSIONS
            )));
        }

        errors
    }
}


impl ErrorPages {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        for (status, path) in &self.custom_pages {
            // Validate status code
            if status.is_empty() {
                errors.push(ConfigError::Warning("ErrorPages status is empty".to_string()));
            } else if let Err(_) = status.parse::<u16>() {
                errors.push(ConfigError::Warning(format!(
                    "ErrorPages status '{}' is not a valid HTTP status code",
                    status
                )));
            } else if !ALLOWED_STATUS.contains(&status.as_str()) {
                errors.push(ConfigError::Warning(format!(
                    "ErrorPages status '{}' is not allowed. Allowed status codes: {:?}",
                    status, ALLOWED_STATUS
                )));
            }
        }
        errors
    }
}


impl Route {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        // Validate path
        match &self.path {
            None => errors.push(ConfigError::Warning("Route path is undefined".to_string())),
            Some(path) if path.is_empty() => {
                errors.push(ConfigError::Warning("Route path is empty".to_string()))
            }
            Some(path) if !path.starts_with('/') => {
                errors.push(ConfigError::Warning(format!(
                    "Route path '{}' must start with '/'",
                    path
                )))
            }
            _ => {}
        }

        // Validate methods
        match &self.methods {
            None => errors.push(ConfigError::Warning("Route methods is undefined".to_string())),
            Some(methods) if methods.is_empty() => {
                errors.push(ConfigError::Warning("Route methods is empty".to_string()))
            }
            Some(methods) => {
                for method in methods {
                    if !ALLOWED_HTTP_METHODS.contains(&method.as_str()) {
                        errors.push(ConfigError::Warning(format!(
                            "Invalid HTTP method '{}'. Allowed methods: {:?}",
                            method, ALLOWED_HTTP_METHODS
                        )));
                    }
                }
            }
        }

        // Validate root directory
        match &self.root {
            None => errors.push(ConfigError::Warning("Route root is undefined".to_string())),
            Some(root) if root.is_empty() => {
                errors.push(ConfigError::Warning("Route root is empty".to_string()))
            }
            Some(root) if !Path::new(root).exists() => {
                errors.push(ConfigError::Warning(format!(
                    "Route root directory '{}' does not exist",
                    root
                )))
            }
            _ => {}
        }

        // Validate default page if specified
        if let Some(ref page) = self.default_page {
            if !Path::new(page).exists() {
                errors.push(ConfigError::Warning(format!(
                    "Route default_page '{}' does not exist",
                    page
                )));
            }
        }

        // Validate redirect if specified
        if let Some(redirect) = &self.redirect {
            if redirect.is_empty() {
                errors.push(ConfigError::Warning("Route redirect URL is empty".to_string()));
            } else if !redirect.starts_with('/') && !redirect.starts_with("http") {
                errors.push(ConfigError::Warning(format!(
                    "Route redirect '{}' must start with '/' or 'http'",
                    redirect
                )));
            }
        }

        // Validate session redirect if required
        if self.session_required.unwrap_or(false) {
            if let Some(redirect) = &self.session_redirect {
                if redirect.is_empty() {
                    errors.push(ConfigError::Warning(
                        "Session redirect URL is empty but session is required".to_string(),
                    ));
                }
            } else {
                errors.push(ConfigError::Warning(
                    "Session redirect is required when session_required is true".to_string(),
                ));
            }
        }

        // Validate CGI configuration if present
        if let Some(ref cgi) = self.cgi {
            errors.extend(cgi.validate());
        }

        errors
    }
}

impl SessionOptionsConfig {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        // Validate path only if defined
        if let Some(path) = &self.path {
            if path.is_empty() {
                errors.push(ConfigError::Warning("Session path is empty".to_string()));
            } else if !path.starts_with('/') {
                errors.push(ConfigError::Warning(format!(
                    "Session path '{}' must start with '/'",
                    path
                )));
            }
        }

        // Validate domain only if defined
        if let Some(domain) = &self.domain {
            if domain.is_empty() {
                errors.push(ConfigError::Warning("Session domain is empty".to_string()));
            }
        }

        // Validate same_site only if defined
        if let Some(same_site) = &self.same_site {
            match same_site.to_lowercase().as_str() {
                "strict" | "lax" | "none" => {}
                _ => errors.push(ConfigError::Warning(format!(
                    "Invalid same_site value '{}'. Must be 'Strict', 'Lax', or 'None'",
                    same_site
                ))),
            }
        }

        // Validate max_age only if defined
        if let Some(max_age) = self.max_age {
            if max_age == 0 {
                errors.push(ConfigError::Warning("Session max_age must be greater than 0".to_string()));
            }
        }

        // Validate expires only if defined
        if let Some(expires) = self.expires {
            if expires == 0 {
                errors.push(ConfigError::Warning("Session expires must be greater than 0".to_string()));
            }
        }

        errors
    }
}


impl SessionConfig {
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        if self.enabled.unwrap_or(false) {
            match &self.name {
                Some(name) if !name.is_empty() => {},
                Some(_) => errors.push(ConfigError::Critical("Session cookie name is empty".to_string())),
                None => errors.push(ConfigError::Critical("Session cookie name is required when sessions are enabled".to_string())),
            }

            if let Some(options) = &self.options {
                errors.extend(options.validate());
            }
        }

        errors
    }
}


impl Host {
    pub fn is_valid_essential_config(&mut self) -> Result<(), ConfigError> {
        // Validation du server_name
        let server_name = self.server_name
            .as_ref()
            .ok_or_else(|| ConfigError::Critical("Host server_name is undefined".to_string()))?;
        
        if server_name.trim().is_empty() {
            return Err(ConfigError::Critical("Host server_name is empty".to_string()));
        }
        
        // Validation du server_address
        let server_address = self.server_address
            .as_ref()
            .ok_or_else(|| ConfigError::Critical("Host server_address is undefined".to_string()))?;
        
        if server_address.parse::<IpAddr>().is_err() {
            return Err(ConfigError::Critical(format!("Host server_address is invalid: {}", server_address)));
        }
        
        // Validation des ports
        let ports = self.ports
            .as_ref()
            .ok_or_else(|| ConfigError::Critical("Host ports is undefined".to_string()))?;
        
        if ports.is_empty() {
            return Err(ConfigError::Critical("Host ports is empty".to_string()));
        }
        
        let (valid_ports, invalid_ports) = ports.iter()
            .fold((HashSet::new(), Vec::new()), |(mut valid, mut invalid), port_str| {
                match port_str.parse::<u16>() {
                    Ok(port) => { valid.insert(port); },
                    Err(_) => { invalid.push(port_str); }
                }
                (valid, invalid)
            });
        
        if !invalid_ports.is_empty() {

            return Err(ConfigError::Critical("Host ports contains an invalid port".to_string()));
        }
        
        let logger = Logger::new(LogLevel::DEBUG);

        if valid_ports.len() < ports.len() {
            logger.warn(&format!("Host ports contain duplicate values on {:?}", self.server_name.as_ref().unwrap()), MODULE);
        }

        self.ports = Some(valid_ports.into_iter().map(|port| port.to_string()).collect());
        
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

        if let Some(session_config) = &self.session {
            warnings.extend(session_config.validate());
        }

        if let Some(routes) = &self.routes {
            for route in routes {
                warnings.extend(route.validate());
            }
        } else {
            warnings.push(ConfigError::Warning("Host routes is undefined".to_string()));
        }

        if let Some(error_pages) = &self.error_pages {
            warnings.extend(error_pages.validate());
        }

        warnings
    }
}

impl ServerConfig {
    pub fn load_and_validate(with_warn: bool) -> Result<ServerConfig, ConfigError> {
        let logger = Logger::new(LogLevel::DEBUG);
    
        let home_dir = env::var("HOME").expect("Failed to get home directory");
        let config_content = fs::read_to_string(&format!("{}/.cargo/localhost-cli/config.json", home_dir))
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
    
        config.servers = config.servers.into_iter().filter_map(|mut host| {
            match host.is_valid_essential_config() {
                Ok(()) => {
                    if !server_names.insert(host.server_name.clone()) {
                        validation_errors.push(format!("Duplicate server name: {:?}", host.server_name));
                        None
                    } else {
                        let warnings = host.collect_warnings();
                        for warn in warnings {
                            match warn {
                                ConfigError::Critical(msg) => {
                                    logger.error(&msg, &format!("{} - {}", MODULE, host.server_name.clone().unwrap_or_default()));
                                },
                                ConfigError::Warning(msg) => {
                                    if with_warn {
                                        logger.warn(&msg, &format!("{} - {}", MODULE, host.server_name.clone().unwrap_or_default()));
                                    }
                                }
                            }
                        }
                        Some(host) // Conserve l'hôte valide
                    }
                },
                Err(e) => {
                    match e {
                        ConfigError::Critical(msg) => {
                            logger.error(&msg, &format!("{} - {}", MODULE, host.server_name.clone().unwrap_or_default()));
                        },
                        ConfigError::Warning(msg) => {
                            if with_warn {
                                logger.warn(&msg, &format!("{} - {}", MODULE, host.server_name.clone().unwrap_or_default()));
                            }
                        }
                    }
                    None // Supprime l'hôte invalide
                }
            }
        }).collect();
    
        if config.servers.is_empty() {
            let msg = "No valid server configuration found";
            logger.error(&msg, MODULE);
            return Err(ConfigError::Critical(msg.to_string()));
        }
    
        if !validation_errors.is_empty() {
            for error in &validation_errors {
                logger.error(error, MODULE);
            }
            return Err(ConfigError::Critical("Invalid configuration".to_string()));
        }
    
        Ok(config)
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            enabled: None,
            name: None,
            options: None,
        }
    }
}