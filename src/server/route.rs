use crate::server::static_files::ServerStaticFiles;
use crate::server::cgi::CGIConfig;
use crate::http::request::HttpMethod;
use std::collections::HashMap;
//use regex::Regex;
use std::sync::Arc;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum RouteMatcher {
    /// Exact string match
    Exact(String),

    /// Dynamic parameter match (e.g., /:id/)
    Dynamic(Vec<String>),

    /// Static file match
    StaticFile(Arc<Path>),
}


impl RouteMatcher {
    pub fn from_path(path: &str) -> Self {
        if path.contains(':') {
            let segments = path.split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            RouteMatcher::Dynamic(segments)
        } else {
            RouteMatcher::Exact(path.to_string())
        }
    }

    pub fn matches(&self, path: &str) -> bool {
        match self {
            RouteMatcher::Exact(exact) => exact == path,
            RouteMatcher::Dynamic(segments) => {
                let path_segments = path.split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<&str>>();

                if path_segments.len() != segments.len() {
                    return false;
                }

                segments.iter().zip(path_segments.iter()).all(|(segment, path_segment)| {
                    segment.starts_with(':') || segment == path_segment
                })
            },
            RouteMatcher::StaticFile(base_path) => {
                let path_file = Path::new(path.trim_start_matches("/"));
                base_path.join(path_file).exists()
            }
        }
    }

    pub fn extract_params(&self, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        if let RouteMatcher::Dynamic(segments) = self {
            let path_segments = path.split('/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>();

            for (i, segment) in segments.iter().enumerate() {
                if segment.starts_with(':') && i < path_segments.len() {
                    params.insert(segment[1..].to_string(), path_segments[i].to_string());
                }
            }
        }

        params
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<HttpMethod>,
    pub static_files: Option<ServerStaticFiles>,
    pub cgi_config: Option<CGIConfig>,
    pub redirect: Option<String>,
    pub session_required: Option<bool>,
    pub session_redirect: Option<String>,
    pub matcher: Option<RouteMatcher>,
    pub params: HashMap<String, String>,
}

impl Route {
    pub fn is_method_allowed(&self, method: &HttpMethod) -> bool {
        self.methods.contains(method)
    }   
}