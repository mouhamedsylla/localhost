use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};
use std::io::{Read, Write, ErrorKind};

#[derive(Debug)]
pub struct Connection {
    pub stream: TcpStream,
    pub client_fd: RawFd,
    pub host_name: String,
}

impl Connection {
    pub fn new(stream: TcpStream, host_name: String) -> Self {
        let client_fd = stream.as_raw_fd();
        Connection {
            stream,
            client_fd,
            host_name,
        }
    }

    pub fn read_request(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        let mut buffer = [0; 1024];
        match self.stream.read(&mut buffer) {
            Ok(0) => Ok(None), // Connection closed
            Ok(bytes_read) => Ok(Some(buffer[..bytes_read].to_vec())),
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(Some(Vec::new())),
            Err(e) => Err(e),
        }
    }

    pub fn send_response(&mut self, response: String) -> std::io::Result<()> {
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()
    }
}