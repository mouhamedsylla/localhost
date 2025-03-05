use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::header::Header;
use crate::http::body::Body;
use crate::http::status::HttpStatusCode;
use crate::server::errors::{ServerError, CGIError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone)]
pub struct CGIConfig {
    pub interpreter: String,
    pub script_dir: String,
    pub allowed_extensions: Vec<String>
}

impl CGIConfig {
    pub fn new(script_dir: String) -> Self {
        CGIConfig {
            interpreter: String::from("/usr/bin/python3"),
            script_dir,
            allowed_extensions: vec!["py".to_string()]
        }
    }

    pub fn prepare_cgi_environment(&self, request: &Request) -> HashMap<String, String> {
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

    pub fn parse_cgi_output(&self, output: Output) -> Result<Response, ServerError> {
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(CGIError::ScriptOutputError(error_msg.to_string()).into());
        }

        // Parser les headers et le body
        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.split("\r\n\r\n").collect();
        
        if parts.len() < 2 {
            return Err(CGIError::InvalidOutputFormat.into());
        }

        // Construire la réponse
        let mut headers = Vec::new();
        let mut status_code = HttpStatusCode::Ok;

        for line in parts[0].lines() {
            // Vérifie si c'est une ligne Status pour extraire le code HTTP
            if line.to_lowercase().starts_with("status:") {
                if let Some(status_str) = line.splitn(2, ':').nth(1) {
                    if let Some(code_str) = status_str.trim().split_whitespace().next() {
                        if let Ok(code) = code_str.parse::<u16>() {
                            if !HttpStatusCode::from_code(code).is_none() {
                                status_code = HttpStatusCode::from_code(code).unwrap();
                            };
                        }
                    }
                }
            } else {
                let h_parts: Vec<&str> = line.splitn(2, ":").collect();
                if h_parts.len() == 2 {
                    headers.push(Header::from_str(h_parts[0].trim(), h_parts[1].trim()));
                }
            }
        }

        // Assurer qu'on a un Content-Type, sinon mettre text/plain par défaut
        if !headers.iter().any(|h| h.name.to_string().to_lowercase() == "content-type") {
            headers.push(Header::from_str("content-type", "text/plain"));
        }

        Ok(Response::new(
            status_code,
            headers,
            Some(Body::text(parts[1]))
        ))
    }

    pub fn is_allowed_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.allowed_extensions.contains(&ext.to_string()))
            .unwrap_or(false)
    }

    pub fn validate_script(&self, script_path: &Path) -> Result<(), ServerError> {
        if (!script_path.exists()) {
            return Err(CGIError::ScriptNotFound(script_path.to_path_buf()).into());
        }

        if (!self.is_allowed_extension(script_path)) {
            let ext = script_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string();
            return Err(CGIError::ExtensionNotAllowed(ext).into());
        }

        Ok(())
    }

    pub fn execute_script(&self, script_path: &Path, env_vars: &HashMap<String, String>) 
        -> Result<Output, ServerError> {
        Command::new(&self.interpreter)
            .arg(script_path)
            .envs(env_vars)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| CGIError::ExecutionFailed(e.to_string()).into())
    }
}