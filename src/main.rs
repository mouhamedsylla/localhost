#![allow(warnings)]

mod http;
mod server;
mod config;

use server::server::Server;
use server::errors::ServerError;
use server::static_files;
use crate::server::host::Host;
use crate::server::static_files::{ServerStaticFiles, ErrorPages};
use core::error;
use std::path::Path;
use std::path::PathBuf;
use crate::http::request::HttpMethod;
use crate::server::uploader::Uploader;
use crate::server::route::Route;
use crate::server::cgi::CGIConfig;
use crate::config::config::ServerConfig;


fn main() -> Result<(), ServerError> {
    let uploader = Uploader::new(Path::new("example/upload").to_path_buf());

    let mut servers = Server::new(Some(uploader)).unwrap();
    let load_config = ServerConfig::load_and_validate();

    match load_config {
        Ok(server_config) => {
            for host_config in server_config.servers {
                let mut routes: Vec<Route>  =  Vec::new();

                for r in host_config.routes {
                    let methods = r.methods.iter()
                        .map(|m| HttpMethod::from_str(m))
                        .collect::<Vec<HttpMethod>>();

                    let error_pages = if let Some(ref pages) = host_config.error_pages {
                        Some(ErrorPages {
                            custom_pages: pages.custom_pages.clone(),
                        })
                    } else {
                        None
                    };

                    let results = ServerStaticFiles::new(
                        PathBuf::from(r.root), r.default_page, r.directory_listing, error_pages);

                    let static_files = match results {
                        Ok(files) => Some(files),
                        Err(e) => {
                            None
                        }
                        
                    };

                    let cgi_config = 
                        if let Some(cgi) = r.cgi {
                            Some(CGIConfig::new(cgi.scrpit_path))
                        } else {
                            None
                        };

                    routes.push(Route { path: r.path, methods , static_files: static_files, cgi_config });
                }

                let host = Host::new(
                    &host_config.server_address,
                    &host_config.server_name,
                    host_config.ports,
                    routes
                ).unwrap();

                let _ = servers.add_host(host);

            }
        }
        Err(e) => {
            eprintln!("Error 2: {}", e);
            return Err(ServerError::ConfigError(e));
        }
    }

    
    // let config = load_config().unwrap();
    
    // for host_config in config.servers {
    //     let mut routes: Vec<Route>  =  Vec::new();

    //     for r in host_config.routes {
    //         let methods = r.methods.iter()
    //             .map(|m| HttpMethod::from_str(m))
    //             .collect::<Vec<HttpMethod>>();

    //         let static_files = ServerStaticFiles::new(
    //             PathBuf::from(r.root), r.default_page, r.directory_listing, host_config.error_pages.clone()).unwrap();

    //         let cgi_config = 
    //             if let Some(cgi_script) = r.cgi_script {
    //                 Some(CGIConfig::new(cgi_script))
    //             } else {
    //                 None
    //            };

    //         routes.push(Route { path: r.path, methods , static_files: Some(static_files), cgi_config });
    //     }

    //     let host = Host::new(
    //         &host_config.server_address,
    //         &host_config.server_name,
    //         host_config.ports,
    //         routes
    //     ).unwrap();

    //     let _ = servers.add_host(host);

    // }

    servers.run()
}
