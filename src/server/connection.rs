use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};
use std::io::{Read, Write, ErrorKind};
use crate::http::request::Request;
use std::borrow::Cow;
use crate::http::header::{HeaderName, HeaderParsedValue};
use crate::http::request::parse_request;

const BUFFER_SIZE: usize = 4096;
const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug)]

pub struct Connection {
    pub stream: TcpStream,
    pub client_fd: RawFd,
    pub host_name: String,
    pub keep_alive: bool,
    pub start_time: std::time::Instant,
    buffer: Vec<u8>,
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

    pub fn read_request(&mut self) -> Result<Request, std::io::Error> {
        let mut temp_buffer = vec![0; BUFFER_SIZE];
        
        loop {
            match self.stream.read(&mut temp_buffer) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "Connection closed by peer"
                    ));
                }
                Ok(n) => {
                    // Ajouter les données au buffer
                    self.buffer.extend_from_slice(&temp_buffer[..n]);
    
                    // Chercher la fin des headers
                    if let Some(headers_end) = find_headers_end(&self.buffer) {
                        // Convertir UNIQUEMENT les headers en String
                        let headers_data = &self.buffer[..headers_end];
                        if let Ok(headers_str) = String::from_utf8(headers_data.to_vec()) {
                            if let Some(content_length) = get_content_length(&headers_str) {
                                let total_length = headers_end + content_length;
                                
                                // Vérifier si on a reçu toutes les données
                                if self.buffer.len() >= total_length {
                                    // Garder les données brutes pour le parsing
                                    println!("Total length: {}", total_length);
                                    println!("Buffer length: {}", self.buffer.len());
                                    let request_data = self.buffer[..total_length].to_vec();
                                    self.buffer.clear();
                                    
                                    // Utiliser parse_request avec les données brutes
                                    return match parse_request(&request_data) {
                                        Some(request) => Ok(request),
                                        None => Err(std::io::Error::new(
                                            ErrorKind::InvalidData,
                                            "Failed to parse request"
                                        ))
                                    };
                                }
                            } else {
                                // Pas de Content-Length, la requête est complète après les headers
                                let request_data = self.buffer[..headers_end].to_vec();
                                self.buffer.clear();
                                
                                return match parse_request(&request_data) {
                                    Some(request) => Ok(request),
                                    None => Err(std::io::Error::new(
                                        ErrorKind::InvalidData,
                                        "Failed to parse request"
                                    ))
                                };
                            }
                        }
                    }
    
                    // Vérifier la taille maximale
                    if self.buffer.len() > MAX_REQUEST_SIZE {
                        self.buffer.clear();
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Request exceeded maximum size"
                        ));
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    if !self.buffer.is_empty() {
                        continue;
                    }
                    return Err(e);
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn send_response(&mut self, response: String) -> std::io::Result<()> {
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()
    }

}

fn find_headers_end(data: &[u8]) -> Option<usize> {
    data.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn get_content_length(headers: &str) -> Option<usize> {
    headers.lines()
        .find(|line| line.to_lowercase().starts_with("content-length:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|len| len.trim().parse().ok())
}