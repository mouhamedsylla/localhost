use crate::config::config::ConfigError;
use crate::http::body::Body;
use crate::http::header::Header;
use crate::http::response::{Response, ResponseBuilder};
use crate::http::status::HttpStatusCode;
use std::path::{PathBuf, Path};
use std::env;
use serde_json::json;

use super::static_files::ServerStaticFiles;

#[derive(Debug)]
pub enum ServerError {
    IoError(std::io::Error),
    EpollError(&'static str),
    ConnectionError(String),
    ConfigError(ConfigError),

    FileNotFound(PathBuf),
    DirectoryAccessDenied(PathBuf),
    DirectoryListingError(String),

    SessionError(SessionError),
    UploaderError(UploaderError),
    CGIError(CGIError),  // Nouvelle erreur ajoutée
    HttpError(HttpError),
}

#[derive(Debug)]
pub enum SessionError {
    InvalidSession(String),
    SessionExpired(String),
    SessionStorageError(String),
    SessionExpiredRedirect(String),
    AuthenticationRequired,
}

#[derive(Debug)]
pub enum UploaderError {
    FileTooLarge { size: usize, max_size: usize },
    UnsupportedFileType(String),
    UploadProcessingError(String),
    FileNotFound(i32), // ID du fichier
    DeleteError(i32),  // ID du fichier
    DatabaseSyncError(String),
}

#[derive(Debug, Clone)]
pub enum HttpError {
    BadRequest(String),
    Forbidden(String),
    NotFound(String),
    MethodNotAllowed(String),
    PayloadTooLarge(String),
    UnsupportedMediaType(String),
    InternalServerError(String),
    Found(String),
}

// Ajoutez ce nouvel enum dans la partie des définitions d'erreurs
#[derive(Debug)]
pub enum CGIError {
    ScriptNotFound(PathBuf),
    ExtensionNotAllowed(String),
    ExecutionFailed(String),
    ScriptOutputError(String),
    InvalidOutputFormat,
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => ServerError::FileNotFound(PathBuf::from(error.to_string())),
            _ => ServerError::IoError(error)
        }
    }
}

impl From<SessionError> for ServerError {
    fn from(error: SessionError) -> Self {
        ServerError::SessionError(error)
    }
}

impl From<UploaderError> for ServerError {
    fn from(error: UploaderError) -> Self {
        ServerError::UploaderError(error)
    }
}

impl From<HttpError> for ServerError {
    fn from(error: HttpError) -> Self {
        ServerError::HttpError(error)
    }
}

// Ajoutez l'implémentation From pour convertir CGIError en ServerError
impl From<CGIError> for ServerError {
    fn from(error: CGIError) -> Self {
        ServerError::CGIError(error)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::IoError(e) => write!(f, "IO Error: {}", e),
            ServerError::EpollError(e) => write!(f, "Epoll Error: {}", e),
            ServerError::ConnectionError(e) => write!(f, "Connection Error: {}", e),
            ServerError::ConfigError(e) => write!(f, "Config Error: {}", e),
            ServerError::FileNotFound(path) => write!(f, "File not found: {}", path.display()),
            ServerError::DirectoryAccessDenied(path) => write!(f, "Directory access denied: {}", path.display()),
            ServerError::DirectoryListingError(e) => write!(f, "Directory listing error: {}", e),
            ServerError::SessionError(e) => write!(f, "Session Error: {}", e),
            ServerError::UploaderError(e) => write!(f, "Uploader Error: {}", e),
            ServerError::CGIError(e) => write!(f, "CGI Error: {}", e),
            ServerError::HttpError(e) => write!(f, "HTTP Error: {}", e),
        }
    }
}

// Implémentations Display pour les sous-erreurs
impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::InvalidSession(msg) => write!(f, "Invalid session: {}", msg),
            SessionError::SessionExpired(id) => write!(f, "Session expired: {}", id),
            SessionError::SessionStorageError(msg) => write!(f, "Session storage error: {}", msg),
            SessionError::AuthenticationRequired => write!(f, "Authentication required"),
            SessionError::SessionExpiredRedirect(url) => write!(f, "Session expired, redirecting to: {}", url),
        }
    }
}

impl std::fmt::Display for UploaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploaderError::FileTooLarge { size, max_size } => 
                write!(f, "File too large: {} bytes (max: {} bytes)", size, max_size),
            UploaderError::UnsupportedFileType(mime) => write!(f, "Unsupported file type: {}", mime),
            UploaderError::UploadProcessingError(msg) => write!(f, "Upload processing error: {}", msg),
            UploaderError::FileNotFound(id) => write!(f, "File with ID {} not found", id),
            UploaderError::DeleteError(id) => write!(f, "Failed to delete file with ID: {}", id),
            UploaderError::DatabaseSyncError(msg) => write!(f, "Database sync error: {}", msg),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            HttpError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            HttpError::NotFound(msg) => write!(f, "Not found: {}", msg),
            HttpError::MethodNotAllowed(msg) => write!(f, "Method not allowed: {}", msg),
            HttpError::PayloadTooLarge(msg) => write!(f, "Payload too large: {}", msg),
            HttpError::UnsupportedMediaType(msg) => write!(f, "Unsupported media type: {}", msg),
            HttpError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            HttpError::Found(msg) => write!(f, "Found: {}", msg),
        }
    }
}

// Ajoutez l'implémentation Display pour CGIError
impl std::fmt::Display for CGIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CGIError::ScriptNotFound(path) => write!(f, "CGI script not found: {}", path.display()),
            CGIError::ExtensionNotAllowed(ext) => write!(f, "CGI script extension not allowed: {}", ext),
            CGIError::ExecutionFailed(msg) => write!(f, "Failed to execute CGI script: {}", msg),
            CGIError::ScriptOutputError(msg) => write!(f, "CGI script error: {}", msg),
            CGIError::InvalidOutputFormat => write!(f, "Invalid CGI output format"),
        }
    }
}

// Mettez à jour la méthode to_http_status pour gérer les erreurs CGI
impl ServerError {
    pub fn to_http_status(&self) -> HttpStatusCode {
        match self {
            ServerError::FileNotFound(_) => HttpStatusCode::NotFound,
            ServerError::DirectoryAccessDenied(_) => HttpStatusCode::Forbidden,
            ServerError::SessionError(SessionError::SessionExpired(_)) => HttpStatusCode::Unauthorized,
            ServerError::SessionError(SessionError::AuthenticationRequired) => HttpStatusCode::Unauthorized,
            ServerError::SessionError(SessionError::SessionExpiredRedirect(_)) => HttpStatusCode::Found,
            ServerError::UploaderError(UploaderError::FileTooLarge { .. }) => HttpStatusCode::PayloadTooLarge,
            ServerError::UploaderError(UploaderError::UnsupportedFileType(_)) => HttpStatusCode::UnsupportedMediaType,
            ServerError::UploaderError(UploaderError::FileNotFound(_)) => HttpStatusCode::NotFound,
            ServerError::HttpError(HttpError::BadRequest(_)) => HttpStatusCode::BadRequest,
            ServerError::HttpError(HttpError::Forbidden(_)) => HttpStatusCode::Forbidden,
            ServerError::HttpError(HttpError::NotFound(_)) => HttpStatusCode::NotFound,
            ServerError::HttpError(HttpError::MethodNotAllowed(_)) => HttpStatusCode::MethodNotAllowed,
            ServerError::HttpError(HttpError::PayloadTooLarge(_)) => HttpStatusCode::PayloadTooLarge,
            ServerError::HttpError(HttpError::UnsupportedMediaType(_)) => HttpStatusCode::UnsupportedMediaType,
            ServerError::CGIError(CGIError::ScriptNotFound(_)) => HttpStatusCode::NotFound,
            ServerError::CGIError(CGIError::ExtensionNotAllowed(_)) => HttpStatusCode::Forbidden,
            ServerError::CGIError(CGIError::ExecutionFailed(_)) => HttpStatusCode::InternalServerError,
            ServerError::CGIError(CGIError::ScriptOutputError(_)) => HttpStatusCode::InternalServerError,
            ServerError::CGIError(CGIError::InvalidOutputFormat) => HttpStatusCode::InternalServerError,
            _ => HttpStatusCode::InternalServerError,
        }
    }

    pub fn to_response(&self) -> Response {
        match self {
            ServerError::FileNotFound(_) => HttpError::NotFound("File not found".to_string()),
            ServerError::DirectoryAccessDenied(_) => HttpError::Forbidden("Directory access denied".to_string()),
            ServerError::SessionError(SessionError::SessionExpired(_)) => HttpError::Forbidden("Session expired".to_string()),
            ServerError::SessionError(SessionError::AuthenticationRequired) => HttpError::Forbidden("Authentication required".to_string()),
            ServerError::SessionError(SessionError::SessionExpiredRedirect(url)) => HttpError::Found(url.to_string()),
            ServerError::UploaderError(UploaderError::FileTooLarge { size, max_size }) => {
                HttpError::PayloadTooLarge(format!("File too large: {} bytes (max: {} bytes)", size, max_size))
            },
            ServerError::UploaderError(UploaderError::UnsupportedFileType(mime)) => {
                HttpError::UnsupportedMediaType(format!("Unsupported file type: {}", mime))
            },
            ServerError::UploaderError(UploaderError::FileNotFound(id)) => {
                HttpError::NotFound(format!("File with ID {} not found", id))
            },
            ServerError::HttpError(e) => e.clone(),
            ServerError::CGIError(e) => HttpError::InternalServerError(format!("{}", e)),
            _ => HttpError::InternalServerError("Internal server error".to_string()),
        }.to_response(None)
    }
}


impl HttpError {
    pub fn new(e: ServerError) -> Self {
        let status = e.to_http_status();
        let message = format!("{}", e);
        match status {
            HttpStatusCode::BadRequest => HttpError::BadRequest(message.to_string()),
            HttpStatusCode::Forbidden => HttpError::Forbidden(message.to_string()),
            HttpStatusCode::NotFound => HttpError::NotFound(message.to_string()),
            HttpStatusCode::MethodNotAllowed => HttpError::MethodNotAllowed(message.to_string()),
            HttpStatusCode::PayloadTooLarge => HttpError::PayloadTooLarge(message.to_string()),
            HttpStatusCode::UnsupportedMediaType => HttpError::UnsupportedMediaType(message.to_string()),
            HttpStatusCode::InternalServerError => HttpError::InternalServerError(message.to_string()),
            _ => HttpError::InternalServerError(message.to_string()),
        }
    }

    pub fn status_code(&self) -> HttpStatusCode {
        match self {
            HttpError::BadRequest(_) => HttpStatusCode::BadRequest,
            HttpError::Forbidden(_) => HttpStatusCode::Forbidden,
            HttpError::NotFound(_) => HttpStatusCode::NotFound,
            HttpError::MethodNotAllowed(_) => HttpStatusCode::MethodNotAllowed,
            HttpError::PayloadTooLarge(_) => HttpStatusCode::PayloadTooLarge,
            HttpError::UnsupportedMediaType(_) => HttpStatusCode::UnsupportedMediaType,
            HttpError::InternalServerError(_) => HttpStatusCode::InternalServerError,
            HttpError::Found(_) => HttpStatusCode::Found,
        }
    }

    fn message(&self) -> &str {
        match self {
            HttpError::BadRequest(msg) => msg,
            HttpError::Forbidden(msg) => msg,
            HttpError::NotFound(msg) => msg,
            HttpError::MethodNotAllowed(msg) => msg,
            HttpError::PayloadTooLarge(msg) => msg,
            HttpError::UnsupportedMediaType(msg) => msg,
            HttpError::InternalServerError(msg) => msg,
            HttpError::Found(msg) => msg,
        }
    }


    pub fn to_response(&self, static_files: Option<&mut ServerStaticFiles>) -> Response {
        let message = self.message();
        let status = self.status_code();
        let status_code = status.as_str();

        // If we have access to static files, try to serve an error page
        if let Some(sf) = static_files {
            // Try to serve the appropriate error page
            let error_code = status_code.to_string();
            
            // First check for custom error page
            if let Some(error_page) = sf.error_pages.as_ref().and_then(|ep| ep.custom_pages.get(&error_code)) {
                if let Ok((content, _, _)) = sf.clone().serve_file(Path::new(error_page)) {
                    return ResponseBuilder::new()
                        .status_code(status)
                        .header(Header::from_str("content-type", "text/html; charset=UTF-8"))
                        .header(Header::from_str("content-length", &content.len().to_string()))
                        .body(Body::text(&String::from_utf8_lossy(&content)))
                        .build();
                }
            }
            
            // Fall back to default error page
            // Try default error template
            let default_error_path = sf.directory.join(".default/error/error_template.html");
            if let Ok((content, _, _)) = sf.serve_file(&default_error_path) {
                // Since the error template reads the code from URL params, we need to inject a script
                // that will set the error information directly
                let html_str = String::from_utf8_lossy(&content);
                
                // Find the closing head tag to inject our script
                let modified_html = if let Some(head_pos) = html_str.find("</head>") {
                    let (before_head, after_head) = html_str.split_at(head_pos);
                    format!(
                        "{}<script>
                        window.ERROR_CODE = '{}';  // Just the numeric part 
                        window.ERROR_MESSAGE = '{}';
                        </script>{}",
                        before_head, 
                        status_code.split_whitespace().next().unwrap_or(status_code), // Extract just the numeric code
                        message.replace("'", "\\'"), 
                        after_head
                    )
                } else {
                    html_str.into_owned()
                };
                
                // We also need to update the initialization script to use our injected variables
                let modified_html = modified_html.replace(
                    "const errorCode = urlParams.get('code') || '404';",
                    "const errorCode = window.ERROR_CODE || urlParams.get('code') || '404';"
                );
                
                return ResponseBuilder::new()
                    .status_code(status)
                    .header(Header::from_str("content-type", "text/html"))
                    .header(Header::from_str("content-length", &modified_html.len().to_string()))
                    .body(Body::text(&modified_html))
                    .build();
            }
        }

        if let HttpError::Found(url) = self {
            println!("Redirecting to: {}", url);
            return ResponseBuilder::new()
                .status_code(status)
                .header(Header::from_str("location", url))  
                .body(Body::empty()) 
                .build();
        }
        
        // Fallback to JSON response if error pages can't be served
        let json_body = json!({ "error": message });
        let body = Body::json(json_body);
        
        ResponseBuilder::new()
            .status_code(status)
            .header(Header::from_str("content-type", "application/json"))
            .header(Header::from_str("content-length", &body.body_len().to_string()))
            .body(body)
            .build()
    }


    pub fn not_found(page: Option<String>) -> Response {
        if let Some(error_page_path) = page {
            // Try to read the custom error page file
            match std::fs::read(&format!("{}/{}", sites_dir(), error_page_path)) {
                Ok(content) => {
                    // We know it's HTML, so no need for conditional checks
                    return ResponseBuilder::new()
                        .status_code(HttpStatusCode::NotFound)
                        .header(Header::from_str("content-type", "text/html; charset=UTF-8"))
                        .header(Header::from_str("content-length", &content.len().to_string()))
                        .body(Body::text(&String::from_utf8_lossy(&content)))  // Convert bytes to string for text body
                        .build();
                }
                Err(_) => {
                    // Fall through to default response if error page can't be read
                }
            }
        }

        let d_dir = env::var("LOCALHOST_RESOURCES").unwrap_or_else(|_| "src/.default".to_string());
        let default_error_path = format!("{}/error/error_template.html", d_dir);

        // Try to read the default error page file
        match std::fs::read(&default_error_path) {
            Ok(content) => {
                // We know it's HTML, so no need for conditional checks
                return ResponseBuilder::new()
                    .status_code(HttpStatusCode::NotFound)
                    .header(Header::from_str("content-type", "text/html; charset=UTF-8"))
                    .header(Header::from_str("content-length", &content.len().to_string()))
                    .body(Body::text(&String::from_utf8_lossy(&content)))  // Convert bytes to string for text body
                    .build();
            }
            Err(_) => {
                // Fall through to default response if error page can't be read
            }
        }

        // Default 404 response with simple HTML
        let html = r#"<!DOCTYPE html>
            <html>
            <head>
                <title>404 Not Found</title>
                <style>
                    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; 
                            text-align: center; padding: 40px; }
                    h1 { color: #333; font-size: 32px; }
                    p { color: #555; font-size: 16px; }
                </style>
            </head>
            <body>
                <h1>404 Not Found</h1>
                <p>The requested resource could not be found on this server.</p>
                <p><a href="/">Return to Home</a></p>
            </body>
            </html>"#;
    
        ResponseBuilder::new()
            .status_code(HttpStatusCode::NotFound)
            .header(Header::from_str("content-type", "text/html; charset=UTF-8"))
            .header(Header::from_str("content-length", &html.len().to_string()))
            .body(Body::text(html))
            .build()
    }
}

pub fn sites_dir() -> String {
    format!("{}/.cargo/localhost-cli/sites", env!("HOME"))
}