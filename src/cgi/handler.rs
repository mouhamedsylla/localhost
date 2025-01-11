use std::process::{Command, Stdio};
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::header::{self, Header, HeaderName};
use crate::http::body::Body;
use crate::http::status::HttpStatusCode;
use std::collections::HashMap;
use std::io::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CGIConfig {
    interpreter: String,
    pub script_dir: String,
    allowed_extensions: Vec<String>
}

impl CGIConfig {
    pub fn new(script_dir: String) -> Self {
        CGIConfig {
            interpreter: String::from("/usr/bin/python3"),
            script_dir,
            allowed_extensions: vec!["py".to_string()]
        }
    }

    pub fn handle_request(&self, request: &Request, script_path: &Path) -> Result<Response, Error> {
        if !script_path.exists() {
            return Ok(Response::new(
                HttpStatusCode::NotFound,
                vec![Header::from_str("content-type", "text/plain")],
                Some(Body::text("CGI script not found"))
            ));
        }
    
        // Vérifier l'extension
        if !self.is_allowed_extension(script_path) {
            return Ok(Response::new(
                HttpStatusCode::Forbidden,
                vec![Header::from_str("content-type", "text/plain")],
                Some(Body::text("Script type not allowed"))
            ));
        }
    
        // Préparer les variables d'environnement CGI
        let env_vars = self.prepare_cgi_environment(request);
    
        // Exécuter le script avec timeout
        match Command::new(&self.interpreter)
            .arg(script_path)
            .envs(&env_vars)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output() {
                Ok(output) => {let script = Path::new(&self.script_dir).join(&request.uri);
                    if !output.status.success() {
                        // Le script a retourné une erreur
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        let body = Body::text(&format!("CGI script error: {}", error_msg));
                        return Ok(Response::new(
                            HttpStatusCode::InternalServerError,
                            vec![Header::from_str("content-type", "text/plain"), Header::from_str("content-length", &body.body_len().to_string())],
                            Some(body)
                        ));
                    }
    
                    // Parser la sortie CGI
                    match self.parse_cgi_output(output) {
                        Ok(response) => Ok(response),
                        Err(e) => {
                            let body = Body::text(&format!("Failed to parse CGI output: {}", e));
                            Ok(Response::new(
                                HttpStatusCode::InternalServerError,
                                vec![Header::from_str("content-type", "text/plain"), Header::from_str("content-length", &body.body_len().to_string())],
                                Some(body)
                            ))
                        }
                    }
                },
                Err(e) => {
                    let body = Body::text(&format!("Failed to execute CGI script: {}", e));
                    Ok(Response::new(
                        HttpStatusCode::InternalServerError,
                        vec![Header::from_str("content-type", "text/plain"), 
                        Header::from_str("content-length", &body.body_len().to_string())],
                        Some(body)
                    ))
                }
            }
    }

    fn prepare_cgi_environment(&self, request: &Request) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("GATEWAY_INTERFACE".to_string(), "CGI/1.1".to_string());
        env.insert("SERVER_PROTOCOL".to_string(), request.version.to_string());
        env.insert("SERVER_SOFTWARE".to_string(), "Rust HTTP Server".to_string());
        env.insert("REQUEST_METHOD".to_string(), request.method.to_string());
        env.insert("SCRIPT_NAME".to_string(), request.uri.to_string());
        env.insert("QUERY_STRING".to_string(), "".to_string());

        // Headers HTTP -> Variables CGI
        for header in &request.headers {
            let env_name = format!("HTTP_{}", 
                header.name.to_string()
                    .replace("-", "_")
                    .to_uppercase());
            env.insert(env_name, header.value.value.clone());
        }
        env
    }

    fn parse_cgi_output(&self, output: std::process::Output) -> Result<Response, std::io::Error> {
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("CGI script error: {}", 
                    String::from_utf8_lossy(&output.stderr))
            ));
        }

        // Parser les headers et le body
        let output_str = String::from_utf8_lossy(&output.stdout);
        println!("CGI output: {}", output_str);
        let parts: Vec<&str> = output_str.split("\r\n\r\n").collect();
        
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid CGI output format"
            ));
        }

        // Construire la réponse
        let mut headers = Vec::new();
        for line in parts[0].lines() {
            let h_parts: Vec<&str> = line.splitn(2, ":").collect();
            if h_parts.len() == 2 {
                headers.push(Header::from_str(h_parts[0].trim(), h_parts[1].trim()));
            }
        }

        Ok(Response::new(
            HttpStatusCode::Ok,
            headers,
            Some(Body::text(parts[1]))
        ))
    }

    fn is_allowed_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.allowed_extensions.contains(&ext.to_string()))
            .unwrap_or(false)
    }
}