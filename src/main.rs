mod http;
mod server;

use std::net::{TcpListener, TcpStream};
use server::server::Server;
use libc::{epoll_create1};

fn main() {

    let host1 = server::server::Host::new("8080", "Serveur HTTP 1");
    let host2 = server::server::Host::new("8081", "Serveur HTTP 2");


    let mut servers = Server::new();

    servers.hosts.push(host1);
    servers.hosts.push(host2);

    servers.run();
}