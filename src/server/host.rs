use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use crate::server::route::Route;
use crate::server::errors::ServerError;
use crate::server::uploader::Uploader;
use serde_json::json;
use crate::server::handlers::handlers::{
    Handler,
    StaticFileHandler,
    FileAPIHandler,
    CGIHandler
};
use crate::http::{
    body::Body,
    request::{Request, HttpMethod},
    response::{Response, ResponseBuilder},
    status::HttpStatusCode,
    header::Header
};

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
        let (stream, addr) = self.listener.accept()?;
        println!("Connection from: {}", addr);
        stream.set_nonblocking(true)?;
        Ok(stream)
    }
}

/// Represents a virtual host configuration for the server
#[derive(Debug)]
pub struct Host {
    pub server_address: String,
    pub server_name: String,
    pub listeners: Vec<HostListener>,
    pub routes: Vec<Route>,
}

/// Clone implementation for Host
impl Clone for Host {
    fn clone(&self) -> Host {
        Host {
            server_address: self.server_address.clone(),
            server_name: self.server_name.clone(),
            listeners: self.listeners.clone(),
            routes: self.routes.clone(),
        }
    }
}

/// Core Host implementation
impl Host {
    pub fn new(
        server_address: &str,
        server_name: &str,
        ports: Vec<String>,
        routes: Vec<Route>, 
    ) -> Result<Self, std::io::Error> {
        let mut listeners = Vec::new();
        for port in ports {
            listeners.push(HostListener::new(port, server_address.to_string()));
        }
        
        Ok(Host {
            server_address: server_address.to_string(),
            server_name: server_name.to_string(),
            listeners,
            routes,
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
        self.routes.iter().find(|route| { 
            &route.path == path 
        })
    }

    pub fn route_request(&self, request: &Request, route: &Route, uploader: Option<Uploader>) -> Result<Response, ServerError> {
        match (&request.method, &request.uri) {
            // Handle file API endpoints with FileApiHandler
            (_, uri) if uri.starts_with("/api") => {
                if let Some(uploader) = uploader {
                    // Create and use the file API handler
                    let handler_result = FileAPIHandler::new(uploader.clone());
                    match handler_result {
                        Ok(mut handler) => {
                            handler.serve_http(request)
                            .map_err(|e| ServerError::ConnectionError(e.to_string()))
                        },
                        Err(err) => {
                            return Err(ServerError::ConnectionError("not available service".to_string()));
                        }
                    }
                } else {
                    // Return service unavailable if uploader is not configured
                    let body = json!({
                        "message": "File upload service is not available"
                    });
                    Ok(Response::response_with_json(body, HttpStatusCode::ServiceUnavailable))
                }
            },

            //Handle GET requests with appropriate handler
            (HttpMethod::GET, _) => {
                if let Some(static_files) = &route.static_files {
                    // Handle static file requests
                    let mut handler = StaticFileHandler { static_files: static_files.clone() };
                    handler.serve_http(request)
                        .map_err(|e| ServerError::ConnectionError(e.to_string()))
                } else if let Some(cgi_config) = &route.cgi_config {
                    // Handle CGI script requests
                    let mut handler = CGIHandler { 
                        cgi_config: cgi_config.clone()
                    };
                    handler.serve_http(request)
                        .map_err(|e| ServerError::ConnectionError(e.to_string()))
                } else {
                    // Return not found if no handler matches
                    let body = Body::Text("Not Found".to_string());
                    Ok(ResponseBuilder::new()
                        .status_code(HttpStatusCode::NotFound)
                        .header(Header::from_str("Content-Type", "text/plain"))
                        .header(Header::from_str("Content-Length", &body.body_len().to_string()))
                        .body(body)
                        .build())
                }
            },
            // Handle unsupported HTTP methods
            _ => {
                let body = Body::Text("Method Not Allowed".to_string());
                Ok(ResponseBuilder::new()
            .status_code(HttpStatusCode::MethodNotAllowed)
            .header(Header::from_str("Content-Type", "text/plain"))
            .header(Header::from_str("Content-Length", &body.body_len().to_string()))
            .body(body)
            .build())    
            }
        }
    }

}

