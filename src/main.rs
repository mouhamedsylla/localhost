mod http;
mod server;
use server::server::Server;

fn main() -> std::io::Result<()> {
    let mut servers = Server::new();
    
    let host1 = server::server::Host::new("8080", "Serveur HTTP 1");
    let host2 = server::server::Host::new("8081", "Serveur HTTP 2");
    
    servers.add_host(host1);
    servers.add_host(host2);
    
    servers.run()
}