use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::path::Path;

// Structures existantes avec ajout des validations
#[derive(Deserialize, Debug)]
pub struct Route {
    pub path: String,
    pub methods: Vec<String>,
    pub root: String,
    pub default_page: Option<String>,
    pub directory_listing: bool,
    pub cgi_script: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Host {
    pub server_address: String,
    pub ports: Vec<String>,
    pub server_name: String,
    pub routes: Vec<Route>,
    pub error_pages: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub servers: Vec<Host>,
}

// Énumération des erreurs possibles
#[derive(Debug)]
pub enum ConfigError {
    // Erreurs IO et parsing
    IoError(std::io::Error),
    JsonParseError(serde_json::Error),
    
    // Erreurs de validation Host
    InvalidServerAddress(AddrParseError),
    InvalidPort(ParseIntError),
    EmptyPorts,
    DuplicatePorts,
    EmptyServerName,
    DuplicateServerName(String),
    NoRoutes,
    
    // Erreurs de validation Route
    InvalidPath(String),
    NoMethods,
    InvalidMethod(String),
    InvalidRoot(String),
    InvalidDefaultPage(String),
    InvalidCgiScript(String),
    
    // Erreurs de validation error_pages
    InvalidErrorPagesPath(String),
    ErrorPageNotFound(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "Erreur IO: {}", e),
            ConfigError::JsonParseError(e) => write!(f, "Erreur de parsing JSON: {}", e),
            ConfigError::InvalidServerAddress(e) => write!(f, "Adresse serveur invalide: {}", e),
            ConfigError::InvalidPort(e) => write!(f, "Port invalide: {}", e),
            ConfigError::EmptyPorts => write!(f, "Aucun port spécifié"),
            ConfigError::DuplicatePorts => write!(f, "Ports en double détectés"),
            ConfigError::EmptyServerName => write!(f, "Nom de serveur vide"),
            ConfigError::DuplicateServerName(name) => write!(f, "Nom de serveur en double: {}", name),
            ConfigError::NoRoutes => write!(f, "Aucune route spécifiée"),
            ConfigError::InvalidPath(path) => write!(f, "Chemin invalide: {}", path),
            ConfigError::NoMethods => write!(f, "Aucune méthode HTTP spécifiée"),
            ConfigError::InvalidMethod(method) => write!(f, "Méthode HTTP invalide: {}", method),
            ConfigError::InvalidRoot(root) => write!(f, "Répertoire racine invalide: {}", root),
            ConfigError::InvalidDefaultPage(page) => write!(f, "Page par défaut invalide: {}", page),
            ConfigError::InvalidCgiScript(script) => write!(f, "Script CGI invalide: {}", script),
            ConfigError::InvalidErrorPagesPath(path) => write!(f, "Chemin des pages d'erreur invalide: {}", path),
            ConfigError::ErrorPageNotFound(page) => write!(f, "Page d'erreur non trouvée: {}", page),
        }
    }
}

impl Error for ConfigError {}

// Extension des méthodes de validation
impl Route {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validation du chemin
        if !self.path.starts_with('/') {
            return Err(ConfigError::InvalidPath(self.path.clone()));
        }

        // Validation des méthodes
        if self.methods.is_empty() {
            return Err(ConfigError::NoMethods);
        }
        
        for method in &self.methods {
            match method.as_str() {
                "GET" | "POST" | "DELETE" => {},
                _ => return Err(ConfigError::InvalidMethod(method.clone()))
            }
        }

        // Validation du répertoire racine
        let root_path = Path::new(&self.root);
        if !root_path.exists() {
            return Err(ConfigError::InvalidRoot(self.root.clone()));
        }

        // Validation de la page par défaut si elle existe
        if let Some(default_page) = &self.default_page {
            let page_path = root_path.join(default_page);
            if !page_path.exists() {
                return Err(ConfigError::InvalidDefaultPage(default_page.clone()));
            }
        }

        // Validation du script CGI si présent
        if let Some(cgi_script) = &self.cgi_script {
            let script_path = root_path.join(cgi_script);
            if !script_path.exists() {
                return Err(ConfigError::InvalidCgiScript(cgi_script.clone()));
            }
        }

        Ok(())
    }
}

impl Host {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validation de l'adresse serveur
        self.server_address.parse::<std::net::IpAddr>()
            .map_err(ConfigError::InvalidServerAddress)?;

        // Validation des ports
        if self.ports.is_empty() {
            return Err(ConfigError::EmptyPorts);
        }

        let mut unique_ports = std::collections::HashSet::new();
        for port in &self.ports {
            let port_num = port.parse::<u16>()
                .map_err(ConfigError::InvalidPort)?;
            if !unique_ports.insert(port_num) {
                return Err(ConfigError::DuplicatePorts);
            }
        }

        // Validation du nom de serveur
        if self.server_name.is_empty() {
            return Err(ConfigError::EmptyServerName);
        }

        // Validation des routes
        if self.routes.is_empty() {
            return Err(ConfigError::NoRoutes);
        }

        // Validation de chaque route
        for route in &self.routes {
            route.validate()?;
        }

        // Validation du répertoire des pages d'erreur
        if let Some(error_pages) = &self.error_pages {
            let error_path = Path::new(error_pages);
            if !error_path.exists() {
                return Err(ConfigError::InvalidErrorPagesPath(error_pages.clone()));
            }
            
            // Vérifier l'existence des pages d'erreur communes
            for error_code in &["400", "404", "500", "502", "503"] {
                let page_path = error_path.join(format!("{}.html", error_code));
                if !page_path.exists() {
                    return Err(ConfigError::ErrorPageNotFound(format!("{}.html", error_code)));
                }
            }
        }

        Ok(())
    }
}

impl Server {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut server_names = std::collections::HashSet::new();
        
        for host in &self.servers {
            // Vérifier les doublons de noms de serveur
            if !server_names.insert(&host.server_name) {
                return Err(ConfigError::DuplicateServerName(host.server_name.clone()));
            }
            
            // Valider chaque host
            host.validate()?;
        }
        
        Ok(())
    }
}

// Fonction de chargement améliorée avec validation
pub fn load_config() -> Result<Server, ConfigError> {
    let config_content = fs::read_to_string("./src/config/config.json")
        .map_err(ConfigError::IoError)?;

        
    let config: Server = serde_json::from_str(&config_content)
        .map_err(ConfigError::JsonParseError)?;
    
    // Valider la configuration
    config.validate()?;
    
    
    Ok(config)
}

// Gestionnaire d'erreurs HTTP et du rendu des pages d'erreur
// #[derive(Debug)]
// pub enum HttpError {
//     BadRequest(String),          // 400
//     NotFound(String),           // 404
//     MethodNotAllowed(String),   // 405
//     InternalServerError(String), // 500
//     BadGateway(String),         // 502
//     ServiceUnavailable(String), // 503
// }

// impl HttpError {
//     pub fn status_code(&self) -> u16 {
//         match self {
//             HttpError::BadRequest(_) => 400,
//             HttpError::NotFound(_) => 404,
//             HttpError::MethodNotAllowed(_) => 405,
//             HttpError::InternalServerError(_) => 500,
//             HttpError::BadGateway(_) => 502,
//             HttpError::ServiceUnavailable(_) => 503,
//         }
//     }

//     pub fn get_error_page(&self, error_pages_path: &str) -> Option<String> {
//         let file_name = format!("{}.html", self.status_code());
//         let error_path = Path::new(error_pages_path).join(file_name);
        
//         fs::read_to_string(error_path).ok()
//     }
// }