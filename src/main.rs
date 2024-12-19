mod http;
mod server;
use serde::de;
use server::server::Server;

use crate::server::static_files::ServerStaticFiles;

fn main() -> std::io::Result<()> {
    let mut servers = Server::new();

    let server1_dir = ServerStaticFiles::new(
        std::path::PathBuf::from("static"),
        "index.html".to_string(),
        true
    )?;

    let server2_dir = ServerStaticFiles::new(
        std::path::PathBuf::from("static"),
        "index.html".to_string(),
        true
    )?;
    
    let host1 = server::server::Host::new("8080", "Serveur HTTP 1", Some(server1_dir));
    let host2 = server::server::Host::new("8081", "Serveur HTTP 2", None);
    
    servers.add_host(host1);
    servers.add_host(host2);
    
    servers.run()
}