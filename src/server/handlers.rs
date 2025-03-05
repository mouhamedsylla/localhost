/// This module provides the core handler functionality for the HTTP server.
/// It implements different types of request handlers following a common interface.
pub mod handlers {
    use crate::http::request::Request;
    use crate::http::response::Response;
    use crate::server::route::Route;
    use crate::server::errors::ServerError;

    /// The Handler trait defines the core interface for processing HTTP requests.
    /// All specific handlers must implement this trait to provide their unique
    /// request handling logic while maintaining a consistent interface.
    pub trait Handler {
        /// Processes an HTTP request and generates an appropriate response.
        ///
        /// # Arguments
        /// * `request` - The incoming HTTP request to be processed
        /// * `route` - The route configuration for this request
        ///
        /// # Returns
        /// * `Result<Response, ServerError>` - The response or a typed error if processing fails
        fn serve_http(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError>;
    }

    /// Handlers for serving static files from the filesystem
    pub mod static_files_api {
        use super::*;
        use crate::http::{
            body::Body,
            header::Header,
            response::{Response, ResponseBuilder},
            status::HttpStatusCode,
        };
        use crate::server::errors::{ServerError, HttpError};
        use crate::server::route::Route;
        use crate::server::static_files::{FileStatus, ServerStaticFiles};

        /// Handles requests for static files stored on the server
        pub struct StaticFileHandler {
            pub static_files: ServerStaticFiles,
        }

        impl Handler for StaticFileHandler {
            fn serve_http(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                // Directly return ServerError now that our trait supports it
                self.handle_static_file_request(request)
            }
        }

        impl StaticFileHandler {
            /// Creates a new StaticFileHandler with the given static files configuration
            pub fn new(static_files: ServerStaticFiles) -> Self {
                StaticFileHandler { static_files }
            }

            /// Handles a request for a static file
            fn handle_static_file_request(
                &mut self,
                request: &Request,
            ) -> Result<Response, ServerError> {
                match self.static_files.serve_static(&request.uri) {
                    Ok((content, mime, file_status)) => {
                        let mime_str = mime.as_deref().unwrap_or("text/plain");
                        let content_type = Header::from_mime(mime_str);

                        let body = Body::from_mime(mime_str, content, None).unwrap();
                        let content_length = Header::from_str("content-length", &body.body_len().to_string());

                        // Determine response status based on file_status
                        let status_code = if file_status == FileStatus::NotFound {
                            HttpStatusCode::NotFound
                        } else {
                            HttpStatusCode::Ok
                        };

                        // Build and return the response
                        Ok(ResponseBuilder::new()
                            .status_code(status_code)
                            .header(content_type)
                            .header(content_length)
                            .body(body)
                            .build())
                    }
                    Err(e) => Err(e), // Pass ServerError directly
                }
            }
        }
    }

    /// Handlers for executing CGI scripts
    pub mod cgi_api {
        use super::*;
        use crate::http::response::Response;
        use crate::server::cgi::CGIConfig;
        use crate::server::errors::{HttpError, ServerError};
        use std::path::Path;
        use std::process::{Command, Stdio};

        /// Handles requests for executing CGI scripts
        pub struct CGIHandler {
            pub cgi_config: CGIConfig,
        }

        impl Handler for CGIHandler {
            fn serve_http(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                // Execute the CGI script and return its response
                let script_path = Path::new(&self.cgi_config.script_dir);
                self.handle_request(request, &script_path)
            }
        }

        impl CGIHandler {
            /// Creates a new CGIHandler with the given configuration
            pub fn new(cgi_config: CGIConfig) -> Self {
                CGIHandler { cgi_config }
            }

            /// Processes a CGI script execution request
            fn handle_request(
                &self,
                request: &Request,
                script_path: &Path,
            ) -> Result<Response, ServerError> {
                // Vérifier si le script existe
                if !script_path.exists() {
                    return Err(HttpError::NotFound(format!(
                        "CGI script not found: {}", 
                        script_path.display()
                    )).into());
                }

                // Vérifier l'extension du script
                println!("Checking extension: {:?}", script_path);
                if !self.cgi_config.is_allowed_extension(script_path) {
                    return Err(HttpError::Forbidden(format!(
                        "Script type not allowed: {}", 
                        script_path.display()
                    )).into());
                }

                // Préparer l'environnement CGI
                let env_vars = self.cgi_config.prepare_cgi_environment(request);

                // Exécuter le script CGI
                let output = Command::new(&self.cgi_config.interpreter)
                    .arg(script_path)
                    .envs(&env_vars)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .map_err(|e| HttpError::InternalServerError(
                        format!("Failed to execute CGI script: {}", e)
                    ))?;

                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(HttpError::InternalServerError(
                        format!("CGI script error: {}", error_msg)
                    ).into());
                }

                // Parser la sortie CGI
                self.cgi_config.parse_cgi_output(output)
                    .map_err(|e| HttpError::InternalServerError(
                        format!("Failed to parse CGI output: {}", e)
                    ).into())
            }
        }
    }

    /// Handlers for file upload and management API
    pub mod file_api {
        use super::Handler;
        use crate::http::{
            body::Body,
            request::{HttpMethod, Request},
            response::Response,
            status::HttpStatusCode,
        };
        use crate::server::errors::{ServerError, HttpError};
        use crate::server::uploader::Uploader;
        use crate::server::route::Route;
        use serde_json::json;
        

        pub struct FileAPIHandler {
            uploader: Uploader,
        }

        impl Handler for FileAPIHandler {
            fn serve_http(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                match request.method {
                    HttpMethod::GET => self.handle_get(request, route),
                    HttpMethod::POST => self.handle_post(request, route),
                    HttpMethod::DELETE => self.handle_delete(request, route),
                    _ => Err(HttpError::MethodNotAllowed(format!(
                        "Method {} not allowed for file API", 
                        request.method
                    )).into()),
                }
            }
        }

        impl FileAPIHandler {
            pub fn new(uploader: Uploader) -> Result<Self, ServerError> {
                Ok(FileAPIHandler { uploader })
            }

            // Request handlers
            fn handle_get(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                if request.uri != "/api/files/list" {
                    return Err(HttpError::NotFound(format!(
                        "API route not found: {}", 
                        request.uri
                    )).into());
                }

                match self.uploader.sync_database() {
                    Ok(_) => {
                        let files = self.uploader.list_files();
                        let files_json = json!({
                            "files": files.iter().map(|file| json!({
                                "id": file.id,
                                "name": file.name,
                                "path": file.path.to_string_lossy(),
                                "size": file.size
                            })).collect::<Vec<_>>()
                        });

                        Ok(Response::response_with_json(files_json, HttpStatusCode::Ok))
                    }
                    Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                }
            }

            fn handle_post(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                if request.uri != "/api/files/upload" {
                    return Err(HttpError::NotFound(format!(
                        "API route not found: {}", 
                        request.uri
                    )).into());
                }

                match &request.body {
                    Some(Body::Multipart(form)) => {
                        let mut uploaded_files = Vec::new();

                        for (_, file) in &form.files {
                            // Validate file type
                            self.uploader.validate_mime_type(&file.content_type)?;
                            
                            // Add file through uploader
                            match self.uploader.add_file(file.filename.clone(), &file.data) {
                                Ok(new_file) => {
                                    uploaded_files.push(json!({
                                        "id": new_file.id,
                                        "name": new_file.name,
                                        "path": new_file.path.to_string_lossy(),
                                        "size": new_file.size
                                    }));
                                }
                                Err(e) => {
                                    return Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut()));
                                }
                                
                            }
                        }

                        let body = json!({
                            "message": "Files uploaded successfully",
                            "files": uploaded_files
                        });
                        
                        Ok(Response::response_with_json(body, HttpStatusCode::Ok))
                    }
                    _ => Err(HttpError::BadRequest(
                        "Invalid request format: expected multipart form data".to_string()
                    ).into()),
                }
            }

            fn handle_delete(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                if !request.uri.starts_with("/api/files/delete/") {
                    return Err(HttpError::NotFound(format!(
                        "API route not found: {}", 
                        request.uri
                    )).into());
                }

                let file_id = request
                    .uri
                    .strip_prefix("/api/files/delete/")
                    .and_then(|id| id.parse::<i32>().ok())
                    .ok_or_else(|| HttpError::BadRequest("Invalid file ID".to_string()))?;

                match self.uploader.delete_file(file_id) {
                    Ok(file) => {
                        let body = json!({
                            "message": "File deleted successfully",
                            "id": file.id
                        });

                        Ok(Response::response_with_json(body, HttpStatusCode::Ok))
                    }
                    Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                }
            }
        }
    }

    pub mod session_api {
        use super::*;

        use crate::server::session::session::{Session, SessionManager};
        use serde_json::json;
        use crate::http::{
            request::{Request, HttpMethod},
            response::{Response, ResponseBuilder},
            status::HttpStatusCode,
            header::Header,
            body::Body,
        };
        use crate::server::route::Route;
        use crate::server::errors::{ServerError, HttpError, SessionError};

        pub struct SessionHandler<'a> {
            session_manager: &'a mut SessionManager,
        }

        impl<'a> Handler for SessionHandler<'a> {
            fn serve_http(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                match request.method {
                    HttpMethod::POST => self.handle_create_session(request, route),
                    HttpMethod::DELETE => self.handle_destroy_session(request, route),
                    _ => Err(HttpError::MethodNotAllowed(format!(
                        "Method {} not allowed for session API", 
                        request.method
                    )).into()),
                }
            }
        }

        impl<'a> SessionHandler<'a> {
            pub fn new(session_manager: &'a mut SessionManager) -> Self {
                SessionHandler { session_manager }
            }

            fn handle_create_session(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                if request.uri != "/api/session/create" {
                    return Err(HttpError::NotFound(format!(
                        "API route not found: {}", 
                        request.uri
                    )).into());
                }

                match self.session_manager.create_session() {
                    Ok((session, cookie_header)) => {
                        match self.session_manager.store.set(session.clone()) {
                            Ok(_) => {
                                let body = Body::json(json!({
                                    "message": "Session created",
                                    "session_id": session.id
                                }));

                                Ok(ResponseBuilder::new()
                                    .status_code(HttpStatusCode::Ok)
                                    .header(cookie_header)
                                    .header(Header::from_str("content-type", "application/json"))
                                    .header(Header::from_str("content-length", &body.body_len().to_string()))
                                    .body(body)
                                    .build())
                            }
                            Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                            
                        }
                    }
                    Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                }
            }


            fn handle_destroy_session(&mut self, request: &Request, route: &Route) -> Result<Response, ServerError> {
                if request.uri != "/api/session/delete" {
                    return Err(HttpError::NotFound(format!(
                        "API route not found: {}", 
                        request.uri
                    )).into());
                }
    
                let cookie_header = request.headers.iter().find(|h| h.name.to_string() == "cookie");
                
                match self.session_manager.get_session(cookie_header) {
                    Ok(Some(session)) => {
                        match self.session_manager.destroy_session(&session.id) {
                            Ok(cookie_header) => {
                                let body = Body::json(json!({
                                    "message": "Session destroyed successfully",
                                    "session_id": session.id
                                }));

                                Ok(ResponseBuilder::new()
                                    .status_code(HttpStatusCode::Ok)
                                    .header(cookie_header)
                                    .header(Header::from_str("content-type", "application/json"))
                                    .header(Header::from_str("content-length", &body.body_len().to_string()))
                                    .body(body)
                                    .build())
                            }
                            Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                        }
                    }
                    Ok(None) => Ok(Response::response_with_json(json!({
                        "message": "No valid session found"
                    }), HttpStatusCode::Unauthorized)),
                    Err(e) => Ok(HttpError::new(e).to_response(route.static_files.clone().as_mut())),
                }
            }
        }
    }

    // Re-export the handlers for easier access
    pub use cgi_api::CGIHandler;
    pub use file_api::FileAPIHandler;
    pub use static_files_api::StaticFileHandler;
    pub use session_api::SessionHandler;
}
