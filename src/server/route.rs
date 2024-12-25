use crate::server::static_files::ServerStaticFiles;
use crate::http::request::HttpMethod;


#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<HttpMethod>,
    pub static_files: Option<ServerStaticFiles>,
   // pub handler: fn(&mut Connection, &str) -> std::io::Result<()>,
}