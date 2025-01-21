#![allow(warnings)]

mod http;
mod server;
mod config;

use std::fs::{OpenOptions};
use std::io::{Write, BufRead, BufReader};
use server::server::Server;
use server::errors::ServerError;
use server::{session, static_files};
use crate::server::host::Host;
use crate::server::static_files::{ServerStaticFiles, ErrorPages};
use core::error;
use chrono::Local;
use std::path::Path;
use std::path::PathBuf;
use crate::http::request::HttpMethod;
use crate::server::uploader::Uploader;
use crate::server::route::Route;
use crate::server::cgi::CGIConfig;
use crate::server::logger::{Logger, LogLevel};
use crate::config::config::ServerConfig;
use crate::server::session::session::{SessionManager, MemorySessionStore};



const BANNER: &str = r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║   ██╗      ██████╗  ██████╗ █████╗ ██╗     ██╗  ██╗ ██████╗ ███████╗████╗ ║
║   ██║     ██╔═══██╗██╔════╝██╔══██╗██║     ██║  ██║██╔═══██╗██╔════╝╚██║  ║
║   ██║     ██║   ██║██║     ███████║██║     ███████║██║   ██║███████╗ ██║  ║
║   ██║     ██║   ██║██║     ██╔══██║██║     ██╔══██║██║   ██║╚════██║ ██║  ║
║   ███████╗╚██████╔╝╚██████╗██║  ██║███████╗██║  ██║╚██████╔╝███████║ ██║  ║
║   ╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝ ╚═╝  ║
║                                                                           ║
║   Server Status: Running on http://localhost                              ║
║   Time: {current_time}                                               ║
║   Environment: Development                                                ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
"#;

fn display_banner() {
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{}", BANNER.replace("{current_time}", &current_time));
}

fn update_hosts_file(server_name: &str, ip_address: &str) -> Result<(), std::io::Error> {
    let hosts_path = "/etc/hosts";
    let hosts_file = OpenOptions::new().read(true).write(true).open(hosts_path)?;
    let logger = Logger::new(LogLevel::INFO);

    let reader = BufReader::new(&hosts_file);
    let entry_exists = reader
        .lines()
        .filter_map(Result::ok)
        .any(|line| line.contains(server_name));

    if !entry_exists {
        let mut file = OpenOptions::new().append(true).open(hosts_path)?;
        writeln!(file, "{}      {}", ip_address, server_name)?;
        logger.info(&format!("Added '{}' to /etc/hosts with IP address '{}'", server_name, ip_address), "INIT");
    } else {
        logger.warn(&format!("The entry '{}' already exists in /etc/hosts", server_name), "INIT");
    }

    Ok(())
}


fn main() -> Result<(), ServerError> {
    let mut active_warn_opt = false;
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&String::from("--warn")) {
        active_warn_opt = true;
    };

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    display_banner();
    
    let uploader = Uploader::new(Path::new("example/upload").to_path_buf());

    let mut servers = Server::new(Some(uploader)).unwrap();
    let load_config = ServerConfig::load_and_validate(active_warn_opt);

    match load_config {
        Ok(server_config) => {
            for host_config in server_config.servers {
                let mut routes: Vec<Route>  =  Vec::new();

                if let Some(tab_routes) = host_config.routes {
                    for r in tab_routes {
                        let methods = r.methods.iter()
                            .flat_map(|v| v.iter().map(|m| HttpMethod::from_str(m)))
                            .collect::<Vec<HttpMethod>>();
    
                        let error_pages = if let Some(ref pages) = host_config.error_pages {
                            Some(ErrorPages {
                                custom_pages: pages.custom_pages.clone(),
                            })
                        } else {
                            None
                        };


    
                        let results = ServerStaticFiles::new(
                            PathBuf::from(r.root.unwrap_or("".to_string())), r.default_page, r.directory_listing.unwrap_or(false), error_pages);
    
                        let static_files = match results {
                            Ok(files) => Some(files),
                            Err(e) => {
                                None
                            }
                            
                        };
    
                        let cgi_config = 
                            if let Some(cgi) = r.cgi {
                                Some(CGIConfig::new(cgi.script_path))
                            } else {
                                None
                            };
    
                        routes.push(Route { 
                            path: r.path.unwrap(), 
                            methods , 
                            static_files, 
                            cgi_config,
                            redirect: r.redirect.clone(), 
                            session_required: r.session_required, 
                            session_redirect: r.session_redirect.clone() 
                        });
                    }
                }

                let session_manager = if let Some(config) = host_config.session {
                        Some(SessionManager::new(config, Box::new(MemorySessionStore::new())))
                } else {
                    None
                };
                

                let mut host = Host::new(
                    host_config.server_address.as_deref().unwrap_or(""),
                    host_config.server_name.as_deref().unwrap_or(""),
                    host_config.ports.unwrap_or_default(),
                    routes,
                    session_manager.clone(),
                ).unwrap();

                if session_manager.is_some() {
                    host.add_session_api();
                }

                if let Some(ip) = host_config.server_address {
                    update_hosts_file(host_config.server_name.as_deref().unwrap_or(""), &ip).unwrap();
                }

                let _ = servers.add_host(host);

            }
        }
        Err(e) => {
            return Err(ServerError::ConfigError(e));
        }
    }

    servers.run()
}