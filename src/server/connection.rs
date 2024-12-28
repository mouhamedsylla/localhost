use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};
use std::io::{Read, Write, ErrorKind};

const BUFFER_SIZE: usize = 4096;
const MAX_REQUEST_SIZE: usize = 8192;

#[derive(Debug)]
// pub struct Connection {
//     pub stream: TcpStream,
//     pub client_fd: RawFd,
//     pub host_name: String,
//     pub keep_alive: bool,
//     pub start_time: std::time::Instant,
// }

// impl Connection {
//     pub fn new(stream: TcpStream, host_name: String) -> Self {
//         let client_fd = stream.as_raw_fd();
//         Connection {
//             stream,
//             client_fd,
//             host_name,
//             keep_alive: true,
//             start_time: std::time::Instant::now(),
//         }
//     }

//     pub fn read_request(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
//         let mut buffer = [0; 1024];
//         match self.stream.read(&mut buffer) {
//             Ok(0) => Ok(None), // Connection closed
//             Ok(bytes_read) => Ok(Some(buffer[..bytes_read].to_vec())),
//             Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(Some(Vec::new())),
//             Err(e) => Err(e),
//         }

//     }

    // pub fn send_response(&mut self, response: String) -> std::io::Result<()> {
    //     self.stream.write_all(response.as_bytes())?;
    //     self.stream.flush()
    // }
// }

pub struct Connection {
    pub stream: TcpStream,
    pub client_fd: RawFd,
    pub host_name: String,
    pub keep_alive: bool,
    pub start_time: std::time::Instant,
    buffer: Vec<u8>,  // Ajouter un buffer persistant
}

impl Connection {
    pub fn new(stream: TcpStream, host_name: String) -> Self {
        let client_fd = stream.as_raw_fd();
        Connection {
            stream,
            client_fd,
            host_name,
            keep_alive: true,
            start_time: std::time::Instant::now(),
            buffer: Vec::new(),
        }
    }

    pub fn read_request(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        let mut temp_buffer = [0; BUFFER_SIZE];
        match self.stream.read(&mut temp_buffer) {
            Ok(0) => Ok(None),
            Ok(n) => {
                self.buffer.extend_from_slice(&temp_buffer[..n]);
                if self.buffer.len() > MAX_REQUEST_SIZE {
                    return Ok(None);  // Requête trop grande
                }
                if self.buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    let result = Ok(Some(self.buffer.clone()));
                    self.buffer.clear();
                    result
                } else {
                    Ok(Some(Vec::new()))  // Requête incomplète
                }
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(Some(Vec::new())),
            Err(e) => Err(e),
        }
    }

    pub fn send_response(&mut self, response: String) -> std::io::Result<()> {
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()
    }
}