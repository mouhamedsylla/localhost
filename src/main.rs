mod http;
mod server;
mod config;

use server::server::Server;
use server::server::ServerError;
use server::static_files;
use crate::server::host::Host;
use crate::server::static_files::ServerStaticFiles;
use config::config::load_config;
use std::path::PathBuf;
use crate::server::route::Route;
use crate::http::request::HttpMethod;

fn main() -> Result<(), ServerError> {
    let mut servers = Server::new().unwrap();
    
    let config = load_config()?;

    println!("Config: {:#?}", config);
    
    for host_config in config.servers {
        let mut routes: Vec<Route>  =  Vec::new();

        for r in host_config.routes {
            let methods = r.methods.iter()
                .map(|m| HttpMethod::from_str(m))
                .collect::<Vec<HttpMethod>>();



                
            // let static_files = match r.static_files {
            //     Some(st) => {
            //         Some(ServerStaticFiles::new(PathBuf::from(st.root), st.default_page, st.directory_listing).unwrap())
            //     },
            //     None => None
            // };
            let static_files = ServerStaticFiles::new(
                PathBuf::from(r.root), r.default_page, r.directory_listing).unwrap();

           // let static_files = ServerStaticFiles::new(PathBuf::from(static_files), st.default_page, st.directory_listing).unwrap();
            routes.push(Route { path: r.path, methods , static_files: Some(static_files) });
        }

        println!("Routes: {:#?}", routes);

        let host = Host::new(
            &host_config.server_address,
            &host_config.server_name,
            host_config.ports,
            routes
        )?;

        let _ = servers.add_host(host);

    }

    servers.run()
}