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
}