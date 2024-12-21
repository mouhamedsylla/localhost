use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use crate::server::static_files::ServerStaticFiles;

/// Default host address for server binding
const LOCAL_HOST: &str = "127.0.0.1";

/// Represents a virtual host configuration for the server
#[derive(Debug)]
pub struct Host {
    pub port: String,
    pub server_name: String,
    pub listener: TcpListener,
    pub fd: RawFd,
    pub static_files: Option<ServerStaticFiles>,
}

/// Clone implementation for Host
impl Clone for Host {
    fn clone(&self) -> Host {
        Host {
            port: self.port.clone(),
            server_name: self.server_name.clone(),
            listener: self.listener.try_clone().unwrap(),
            fd: self.fd,
            static_files: self.static_files.clone(),
        }
    }
}

/// Core Host implementation
impl Host {
    pub fn new(
        port: &str, 
        server_name: &str, 
        server_directory: Option<ServerStaticFiles>
    ) -> Result<Self, std::io::Error> {
        let addr = format!("{}:{}", LOCAL_HOST, port);
        let listener = TcpListener::bind(&addr)?;
        listener.set_nonblocking(true)?;
        
        Ok(Host {
            port: port.to_string(),
            server_name: server_name.to_string(),
            fd: listener.as_raw_fd(),
            listener,
            static_files: server_directory,
        })
    }

    pub fn accept_connection(&self) -> std::io::Result<TcpStream> {
        let (stream, _) = self.listener.accept()?;
        stream.set_nonblocking(true)?;
        Ok(stream)
    }
}

/// Connection management implementation
impl Host {
    /// Checks if host matches given server name
    pub fn matches_name(&self, name: &str) -> bool {
        self.server_name == name
    }

    /// Gets the host's listening port
    pub fn get_port(&self) -> &str {
        &self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_creation() {
        let host = Host::new("8080", "test.local", None);
        assert!(host.is_ok());
    }

    #[test]
    fn test_host_name_matching() {
        let host = Host::new("8080", "test.local", None).unwrap();
        assert!(host.matches_name("test.local"));
        assert!(!host.matches_name("other.local"));
    }
}