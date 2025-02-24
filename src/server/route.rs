use crate::http::response::Response;
use crate::server::errors::ServerError;
use crate::server::static_files::ServerStaticFiles;
use crate::server::cgi::CGIConfig;
use crate::http::request::HttpMethod;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<HttpMethod>,
    pub static_files: Option<ServerStaticFiles>,
    pub cgi_config: Option<CGIConfig>,
    pub redirect: Option<String>,
    pub session_required: Option<bool>,
    pub session_redirect: Option<String>,
}

impl Route {
    pub fn is_method_allowed(&self, method: &HttpMethod) -> bool {
        self.methods.contains(method)
    }   
}