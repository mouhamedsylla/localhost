use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use crate::server::static_files::ServerStaticFiles;
use crate::server::route::Route;

/// Default host address for server binding
const LOCAL_HOST: &str = "1.0.0.1";

#[derive(Debug)]
pub struct HostListener {
    pub fd: RawFd,
    pub listener: TcpListener,
    pub port: String,
}

impl Clone for HostListener {
    fn clone(&self) -> HostListener {
        HostListener {
            fd: self.fd,
            listener: self.listener.try_clone().unwrap(),
            port: self.port.clone(),
        }
    }
}

impl HostListener {
    pub fn new(port: String, server_address: String) -> Self {
        let addr = format!("{}:{}", server_address, port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind to address");
        listener.set_nonblocking(true);
        let fd = listener.as_raw_fd();
        HostListener {
            fd,
            listener,
            port,
        }
    }

    pub fn accept_connection(&self) -> std::io::Result<TcpStream> {
        let (stream, _) = self.listener.accept()?;
        stream.set_nonblocking(true)?;
        Ok(stream)
    }
}

/// Represents a virtual host configuration for the server
#[derive(Debug)]
pub struct Host {
    pub server_address: String,
    pub server_name: String,
    pub static_files: Option<ServerStaticFiles>,
    pub listeners: Vec<HostListener>,
    pub routes: Vec<Route>,
}

/// Clone implementation for Host
impl Clone for Host {
    fn clone(&self) -> Host {
        Host {
            server_address: self.server_address.clone(),
            server_name: self.server_name.clone(),
            listeners: self.listeners.clone(),
            static_files: self.static_files.clone(),
            routes: self.routes.clone(),
        }
    }
}

/// Core Host implementation
impl Host {
    pub fn new(
        server_address: &str,
        server_name: &str,
        ports: Vec<String>, 
        server_directory: Option<ServerStaticFiles>
    ) -> Result<Self, std::io::Error> {
        let mut listeners = Vec::new();
        for port in ports {
            listeners.push(HostListener::new(port, server_address.to_string()));
        }
        
        Ok(Host {
            server_address: server_address.to_string(),
            server_name: server_name.to_string(),
            listeners,
            static_files: server_directory,
            routes: Vec::new(),
        })
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

}