mod http;
mod server;
mod config;

use server::server::Server;
use server::server::ServerError;
use crate::server::host::Host;
use crate::server::static_files::ServerStaticFiles;
use config::config::load_config;
use std::path::PathBuf;

fn main() -> Result<(), ServerError> {
    let mut servers = Server::new().unwrap();
    
    let config = load_config()?;
    
    for host_config in config.servers {
        let static_files = match host_config.static_files {
            Some(sf_config) => Some(ServerStaticFiles::new(
                PathBuf::from(sf_config.directory),
                sf_config.default_page,
                sf_config.list_directory
            ).unwrap()),
            None => None,
        };

        let host = Host::new(
            &host_config.port,
            &host_config.name,
            static_files
        ).expect("Failed to create host");

        let _ = servers.add_host(host);
    }

    servers.run()
}