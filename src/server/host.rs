use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use crate::server::route::Route;
use crate::server::errors::ServerError;
use crate::server::uploader::Uploader;
use crate::server::logger::{Logger, LogLevel};
use crate::server::handlers::handlers::{
    Handler,
    StaticFileHandler,
    FileAPIHandler,
    CGIHandler,
    SessionHandler,
};
use crate::server::static_files::ErrorPages;
use crate::http::{
    request::{Request, HttpMethod},
    response::Response,
    status::HttpStatusCode,
    header::Header
};

use crate::server::errors::{HttpError, SessionError};

use crate::server::session::session::SessionManager;

#[derive(Debug)]
pub struct HostListener {
    pub fd: RawFd,
    pub listener: TcpListener,
    pub port: String,
}

impl Clone for HostListener {
    fn clone(&self) -> HostListener {
        HostListener {
            fd: self.fd,
            listener: self.listener.try_clone().unwrap(),
            port: self.port.clone(),
        }
    }
}

impl HostListener {
    pub fn new(port: String, server_address: String) -> Self {
        let addr = format!("{}:{}", server_address, port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind to address");
        listener.set_nonblocking(true).unwrap();
        let fd = listener.as_raw_fd();

        HostListener {
            fd,
            listener,
            port,
        }
    }

    pub fn accept_connection(&self) -> std::io::Result<TcpStream> {
        let logger = Logger::new(LogLevel::INFO);
        let (stream, addr) = self.listener.accept()?;
        logger.info(&format!("Accepted connection from {}", addr), "HostListener");
        stream.set_nonblocking(true)?;
        Ok(stream)
    }
}

/// Represents a virtual host configuration for the server
pub struct Host {
    pub server_address: String,
    pub server_name: String,
    pub listeners: Vec<HostListener>,
    pub routes: Vec<Route>,
    pub session_manager: Option<SessionManager>,
    pub logger: Logger,
    pub error_pages: Option<ErrorPages>,
    pub max_request_size: Option<usize>,
}

/// Core Host implementation
impl Host {
    pub fn new(
        server_address: &str,
        server_name: &str,
        ports: Vec<String>,
        routes: Vec<Route>,
        session_manager: Option<SessionManager>,
        error_pages: Option<ErrorPages>,
        max_request_size: Option<usize>,
    ) -> Result<Self, std::io::Error> {
        let mut listeners = Vec::new();
        let logger = Logger::new(LogLevel::INFO);

        for port in ports {
            listeners.push(HostListener::new(port, server_address.to_string()));
        }
        
        Ok(Host {
            server_address: server_address.to_string(),
            server_name: server_name.to_string(),
            listeners,
            routes,
            session_manager,
            logger,
            error_pages,
            max_request_size,
        })
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn get_listener(&self, fd: RawFd) -> Option<&HostListener> {
        self.listeners.iter().find(|listener| listener.fd == fd)
    }

    pub fn match_listener(&self, fd: RawFd) -> bool {
        self.listeners.iter().any(|listener| listener.fd == fd)
    }

    pub fn get_route(&self, path: &str) -> Option<&Route> {
        if let Some(route) = self.routes.iter().find(|r| r.path == path) {
            return Some(route);
        }

        let path_segments: Vec<_> = path.trim_end_matches('/').split('/').collect();
        for route in &self.routes {
            let route_segments: Vec<_> = route.path.trim_end_matches('/').split('/').collect();
            
            if path_segments.len() != route_segments.len() {
                continue;
            }
    
            let mut is_dynamic_match = true;
            for (p, r) in path_segments.iter().zip(route_segments.iter()) {
                if !r.starts_with(':') && r != p {
                    is_dynamic_match = false;
                    break;
                }
            }
    
            if is_dynamic_match {
                return Some(route);
            }
        }

        let file_route = self.routes.iter().find(|r| {
            if let Some(files) = r.static_files.as_ref() {
                let path_file = Path::new(path.trim_start_matches("/"));
                let rr = files.is_directory_contain_file(path_file);
                return rr;
            } else {
                return false;
            }
        });

        if file_route.is_some() {
            return file_route;
        }
    
        None
    }

    pub fn add_session_api(&mut self) {
        // Route for creating a session
        let create_session_route = Route {
            path: "/api/session/create".to_string(),
            methods: vec![HttpMethod::POST],
            session_required: Some(false),
            redirect: None,
            session_redirect: None,
            static_files: None,
            cgi_config: None,
            matcher: None,
            params: HashMap::new(),
        };
    
        // Route for deleting a session
        let delete_session_route = Route {
            path: "/api/session/delete".to_string(),
            methods: vec![HttpMethod::DELETE],
            session_required: Some(true),
            session_redirect: None,
            redirect: None,
            static_files: None,
            cgi_config: None,
            matcher: None,
            params: HashMap::new(),
        };
    
        // Add routes to this host
        self.add_route(create_session_route);
        self.add_route(delete_session_route);
    }


    pub fn route_request(&mut self, request: &Request, route: &Route, uploader: Option<Uploader>) -> Result<Response, ServerError> {
        // Handle redirects
        if request.uri == route.path {
            if let Some(redirect) = &route.redirect {
                if let Some(listing) = &route.static_files {
                    if !listing.is_directory_contain_file(Path::new(&listing.directory.join(&request.uri.trim_start_matches("/")))) {
                        self.logger.info(&format!("Redirecting to {}", redirect), "Host");
                        return Ok(self.redirect(&redirect));
                    }
                } else {
                    self.logger.info(&format!("Redirecting to {}", redirect), "Host");
                    return Ok(self.redirect(&redirect));
                }
            }
        }

        // Check if method is allowed for this route
        if !route.methods.contains(&request.method) && !route.methods.is_empty() {
            return Err(HttpError::MethodNotAllowed(format!(
                "Method {} not allowed for route {}", 
                request.method, route.path
            )).into());
        }

        // Route the request to the appropriate handler
        match (&request.method, &request.uri) {
            // Handle file API endpoints with FileApiHandler
            (_, uri) if uri.starts_with("/api/files") => {
                if let Some(uploader) = uploader {
                    // Create and use the file API handler
                    let mut handler = FileAPIHandler::new(uploader.clone())?;
                    handler.serve_http(request, route)
                } else {
                    // Return service unavailable if uploader is not configured
                    Err(HttpError::InternalServerError("File upload service is not available".to_string()).into())
                }
            },

            // Handle session requests with SessionHandler
            (_, uri) if uri.starts_with("/api/session") => {
                if let Some(session_manager) = self.session_manager.as_mut() {
                    let mut handler = SessionHandler::new(session_manager);
                    handler.serve_http(request, route)
                } else {
                    Err(HttpError::InternalServerError("Session service is not available".to_string()).into())
                }
            },

            // Handle requests based on the route configuration
            _ => {
                if let Some(cgi_config) = &route.cgi_config {
                    // Handle CGI script requests first
                    let mut handler = CGIHandler { 
                        cgi_config: cgi_config.clone()
                    };
                    handler.serve_http(request, route)
                } else if let Some(static_files) = &route.static_files {
                    // Fall back to static file requests if no CGI handler matches
                    let mut handler = StaticFileHandler { static_files: static_files.clone() };
                    handler.serve_http(request, route)
                } else {
                    // Return not found if no handler matches
                    Err(HttpError::NotFound(format!("No handler found for route: {}", request.uri)).into())
                }
            }
        }
    }

    fn redirect(&self, redirect: &str) -> Response {
        Response::new(
            HttpStatusCode::MovedPermanently,
            vec![Header::from_str("location", redirect)],
            None
        )
    }

}

