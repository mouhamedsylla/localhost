#![allow(warnings)]

mod http;
mod server;
mod config;
mod cgi;

use server::server::Server;
use server::server::ServerError;
use crate::server::host::Host;
use crate::server::static_files::ServerStaticFiles;
use crate::cgi::handler::CGIHandler;
use config::config::load_config;
use std::path::Path;
use std::path::PathBuf;
use crate::server::route::Route;
use crate::http::request::HttpMethod;
use crate::server::uploader::Uploader;


fn main() -> Result<(), ServerError> {
    let uploader = Uploader::new(Path::new("example/upload"));

    let mut servers = Server::new(Some(uploader)).unwrap();
    
    let config = load_config().unwrap();
    
    for host_config in config.servers {
        let mut routes: Vec<Route>  =  Vec::new();

        for r in host_config.routes {
            let methods = r.methods.iter()
                .map(|m| HttpMethod::from_str(m))
                .collect::<Vec<HttpMethod>>();

            let static_files = ServerStaticFiles::new(
                PathBuf::from(r.root), r.default_page, r.directory_listing, host_config.error_pages.clone()).unwrap();

            let cgi_handler = 
                if let Some(cgi_script) = r.cgi_script {
                    Some(CGIHandler::new(cgi_script))
                } else {
                    None
                };

            routes.push(Route { path: r.path, methods , static_files: Some(static_files), cgi_handler });
        }

        let host = Host::new(
            &host_config.server_address,
            &host_config.server_name,
            host_config.ports,
            routes
        ).unwrap();

        let _ = servers.add_host(host);

    }

    servers.run()
}
