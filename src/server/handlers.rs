/// This module provides the core handler functionality for the HTTP server.
/// It implements different types of request handlers following a common interface.
pub mod handlers {
    use super::*;
    use crate::http::request::Request;
    use crate::http::response::Response;
    use std::io;
    use std::path::Path;

    /// The Handler trait defines the core interface for processing HTTP requests.
    /// All specific handlers must implement this trait to provide their unique
    /// request handling logic while maintaining a consistent interface.
    pub trait Handler {
        /// Processes an HTTP request and generates an appropriate response.
        ///
        /// # Arguments
        /// * `request` - The incoming HTTP request to be processed
        ///
        /// # Returns
        /// * `Result<Response, io::Error>` - The response or an error if processing fails
        fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error>;
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
        use crate::server::errors::ServerError;
        use crate::server::static_files::{FileStatus, ServerStaticFiles};

        /// Handles requests for static files stored on the server
        pub struct StaticFileHandler {
            pub static_files: ServerStaticFiles,
        }

        impl Handler for StaticFileHandler {
            fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error> {
                // Convert the generic io::Error into our specific ServerError if needed
                self.handle_static_file_request(request)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
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

                        let body = Body::from_mime(mime_str, content, None).map_err(|e| {
                            ServerError::ConnectionError(format!("Body creation error: {}", e))
                        })?;

                        let content_length =
                            Header::from_str("content-length", &body.body_len().to_string());

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
                    Err(e) => Err(ServerError::ConnectionError(format!(
                        "Static file error: {}",
                        e
                    ))),
                }
            }
        }
    }

    /// Handlers for executing CGI scripts
    pub mod cgi_api {
        use super::*;
        use crate::http::{body::Body, header::Header, response::Response, status::HttpStatusCode};
        use crate::server::cgi::CGIConfig;
        use std::path::Path;
        use std::process::{Command, Stdio};

        /// Handles requests for executing CGI scripts
        pub struct CGIHandler {
            pub cgi_config: CGIConfig,
        }

        impl Handler for CGIHandler {
            fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error> {
                // Execute the CGI script and return its response
                let script_path = Path::new(&self.cgi_config.script_dir).join(&request.uri);
                self.handle_request(request, &script_path)
            }
        }

        impl CGIHandler {
            /// Creates a new CGIHandler with the given configuration
            pub fn new(cgi_config: CGIConfig) -> Self {
                CGIHandler { cgi_config }
            }

            /// Processes a CGI script execution request
            ///
            /// # Arguments
            /// * `request` - The HTTP request containing CGI parameters
            /// * `script_path` - Path to the CGI script to execute
            fn handle_request(
                &self,
                request: &Request,
                script_path: &Path,
            ) -> Result<Response, io::Error> {
                // Vérifier si le script existe
                if !script_path.exists() {
                    return Ok(Response::new(
                        HttpStatusCode::NotFound,
                        vec![Header::from_str("content-type", "text/plain")],
                        Some(Body::text("CGI script not found")),
                    ));
                }

                // Vérifier l'extension du script
                if !self.cgi_config.is_allowed_extension(script_path) {
                    return Ok(Response::new(
                        HttpStatusCode::Forbidden,
                        vec![Header::from_str("content-type", "text/plain")],
                        Some(Body::text("Script type not allowed")),
                    ));
                }

                // Préparer l'environnement CGI
                let env_vars = self.cgi_config.prepare_cgi_environment(request);

                // Exécuter le script CGI
                match Command::new(&self.cgi_config.interpreter)
                    .arg(script_path)
                    .envs(&env_vars)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                {
                    Ok(output) => {
                        if !output.status.success() {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            let body = Body::text(&format!("CGI script error: {}", error_msg));
                            return Ok(Response::new(
                                HttpStatusCode::InternalServerError,
                                vec![
                                    Header::from_str("content-type", "text/plain"),
                                    Header::from_str(
                                        "content-length",
                                        &body.body_len().to_string(),
                                    ),
                                ],
                                Some(body),
                            ));
                        }

                        // Parser la sortie CGI
                        match self.cgi_config.parse_cgi_output(output) {
                            Ok(response) => Ok(response),
                            Err(e) => {
                                let body =
                                    Body::text(&format!("Failed to parse CGI output: {}", e));
                                Ok(Response::new(
                                    HttpStatusCode::InternalServerError,
                                    vec![
                                        Header::from_str("content-type", "text/plain"),
                                        Header::from_str(
                                            "content-length",
                                            &body.body_len().to_string(),
                                        ),
                                    ],
                                    Some(body),
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        let body = Body::text(&format!("Failed to execute CGI script: {}", e));
                        Ok(Response::new(
                            HttpStatusCode::InternalServerError,
                            vec![
                                Header::from_str("content-type", "text/plain"),
                                Header::from_str("content-length", &body.body_len().to_string()),
                            ],
                            Some(body),
                        ))
                    }
                }
            }
        }
    }

    /// Handlers for file upload and management API
    pub mod file_api {
        use super::Handler;
        use crate::http::{
            body::Body,
            header::Header,
            request::{HttpMethod, Request},
            response::Response,
            status::HttpStatusCode,
        };
        use crate::server::uploader::Uploader;
        use serde_json::json;
        use std::{io, path::PathBuf};

        pub struct FileAPIHandler {
            uploader: Uploader,
        }

        impl Handler for FileAPIHandler {
            fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error> {
                match request.method {
                    HttpMethod::GET => self.handle_get(request),
                    HttpMethod::POST => self.handle_post(request),
                    HttpMethod::DELETE => self.handle_delete(request),
                    _ => Ok(self.method_not_allowed_response()),
                }
            }
        }

        impl FileAPIHandler {
            pub fn new(uploader: Uploader) -> io::Result<Self> {
                Ok(FileAPIHandler { uploader })
            }

            // Request handlers
            fn handle_get(&mut self, request: &Request) -> Result<Response, io::Error> {
                if request.uri != "/api/files/list" {
                    return Ok(self.not_found_response());
                }

                self.uploader.sync_database()?;
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

            fn handle_post(&mut self, request: &Request) -> Result<Response, io::Error> {
                if request.uri != "/api/files/upload" {
                    return Ok(self.not_found_response());
                }

                match &request.body {
                    Some(Body::Multipart(form)) => {
                        let mut uploaded_files = Vec::new();

                        for (field_name, file) in &form.files {
                            // Validate file
                            if let Some(error_response) =
                                self.validate_upload(&file.content_type, file.data.len())
                            {
                                return Ok(error_response);
                            }

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
                                    return Ok(self.error_response(
                                        HttpStatusCode::InternalServerError,
                                        &format!("Failed to save file: {}", e),
                                    ))
                                }
                            }
                        }

                        let body = json!({
                            "message": "Files uploaded successfully",
                            "files": uploaded_files
                        });
                        Ok(Response::response_with_json(body, HttpStatusCode::Ok))
                    }
                    _ => Ok(self.bad_request_response("Invalid request format")),
                }
            }

            fn handle_delete(&mut self, request: &Request) -> Result<Response, io::Error> {
                if !request.uri.starts_with("/api/files/delete") {
                    return Ok(self.not_found_response());
                }

                let file_id = match request
                    .uri
                    .strip_prefix("/api/files/delete/")
                    .and_then(|id| id.parse::<i32>().ok())
                    {
                    Some(id) => id,
                    None => return Ok(self.bad_request_response("Invalid file ID")),
                };

                match self.uploader.delete_file(file_id) {
                    Ok(file) => {
                        let body = json!({
                            "message": "File deleted successfully",
                            "id": file.id
                        });

                        Ok(Response::response_with_json(body, HttpStatusCode::Ok))
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::NotFound {
                            println!("error: {}", e.to_string());
                            Ok(self.not_found_response())
                        } else {
                            Ok(self.error_response(
                                HttpStatusCode::InternalServerError,
                                &format!("Failed to delete file: {}", e),
                            ))
                        }
                    }
                }
            }

            // Validation helpers
            fn validate_upload(&self, content_type: &str, file_size: usize) -> Option<Response> {
                if !self.uploader.is_allowed_mime_type(content_type) {
                    return Some(self.error_response(
                        HttpStatusCode::UnsupportedMediaType,
                        &format!("Unsupported file type: {}", content_type),
                    ));
                }

                if file_size > self.uploader.max_file_size() {
                    return Some(
                        self.error_response(HttpStatusCode::PayloadTooLarge, "File too large"),
                    );
                }

                None
            }

            // Response helpers
            fn not_found_response(&self) -> Response {
                self.error_response(HttpStatusCode::NotFound, "Route not found")
            }

            fn bad_request_response(&self, message: &str) -> Response {
                self.error_response(HttpStatusCode::BadRequest, message)
            }

            fn method_not_allowed_response(&self) -> Response {
                self.error_response(HttpStatusCode::MethodNotAllowed, "Method not allowed")
            }

            fn error_response(&self, status: HttpStatusCode, message: &str) -> Response {
                Response::response_with_json(json!({ "error": message }), status)
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

        pub struct SessionHandler {
            session_manager: SessionManager,
        }

        impl Handler for SessionHandler {
            fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error> {
                match request.method {
                    HttpMethod::POST => self.handle_create_session(request),
                    HttpMethod::DELETE => self.handle_destroy_session(request),
                    _ => Ok(self.method_not_allowed_response()),
                }
            }
        }

        impl SessionHandler {
            pub fn new(session_manager: SessionManager) -> Self {
                SessionHandler { session_manager }
            }

            fn handle_create_session(&mut self, request: &Request) -> Result<Response, io::Error> {
                if request.uri != "/api/session/create" {
                    return Ok(self.not_found_response());
                }
    
                let (session, cookie_header) = self.session_manager.create_session();

                self.session_manager.store.set(session.clone());

                let body = Body::json(json!({
                    "message": "Session created",
                    "session_id": session.id
                }));

                let response_builder = self.with_session(ResponseBuilder::new(), cookie_header);

                Ok(response_builder
                    .status_code(HttpStatusCode::Ok)
                    .header(Header::from_str("content-type", "application/json"))
                    .header(Header::from_str("content-length", &body.body_len().to_string()))
                    .body(body)
                    .build())
            }


            fn handle_destroy_session(&mut self, request: &Request) -> Result<Response, io::Error> {
                if request.uri != "/api/session/delete" {
                    return Ok(self.not_found_response());
                }
    
                let cookie_header = request.headers.iter().find(|h| h.name.to_string() == "cookie");
                
                match self.session_manager.get_session(cookie_header) {
                    Some(session) => {
                        let cookie_header = self.session_manager.destroy_session(&session.id);
                        let body = json!({
                            "message": "Session destroyed successfully",
                            "session_id": session.id
                        });
    
                        let mut response = Response::response_with_json(body, HttpStatusCode::Ok);
                        response.headers.push(cookie_header);
                        
                        Ok(response)
                    }
                    None => Ok(Response::response_with_json(
                        json!({"error": "Session not found"}),
                        HttpStatusCode::NotFound
                    ))
                }
            }

            fn with_session(&self, response_builder: ResponseBuilder, cookie: Header) -> ResponseBuilder {
                response_builder.header(cookie)
            }

            fn not_found_response(&self) -> Response {
                Response::response_with_json(
                    json!({"error": "Route not found"}),
                    HttpStatusCode::NotFound
                )
            }

            fn method_not_allowed_response(&self) -> Response {
                Response::response_with_json(
                    json!({"error": "Method not allowed"}),
                    HttpStatusCode::MethodNotAllowed
                )
            }
        }
    }



    // Re-export the handlers for easier access
    pub use cgi_api::CGIHandler;
    pub use file_api::FileAPIHandler;
    pub use static_files_api::StaticFileHandler;
    pub use session_api::SessionHandler;
}
