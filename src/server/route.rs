use crate::server::static_files::ServerStaticFiles;
use crate::http::request::HttpMethod;
use crate::cgi::handler::CGIHandler;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<HttpMethod>,
    pub static_files: Option<ServerStaticFiles>,
    pub cgi_handler: Option<CGIHandler>,
}