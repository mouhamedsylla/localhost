use std::net::{TcpListener, TcpStream};
use crate::http::status::HttpStatusCode;
use std::io::{Read, Write};
use std::os::fd;
use std::os::unix::io::{AsRawFd, RawFd};

use libc::{epoll_create1, epoll_ctl, EPOLLIN, EPOLLET, EPOLL_CTL_ADD };

pub struct Host {
    pub port: String,
    pub server_name: String,
    pub listener: TcpListener,
}

pub struct Server {
    pub hosts: Vec<Host>,
}


impl Host {
    pub fn new(port: &str, server_name: &str) -> Host {
        Host {
            port: port.to_string(),
            server_name: server_name.to_string(),
            listener: TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap(),
        }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            handle_connection(&mut stream);
        }
    }
}

impl Server {
    pub fn new() -> Server {
        Server {
            hosts: Vec::new(),
        }
    }

    pub fn run(&self) {
        for host in &self.hosts {
            let epool_fd = unsafe {
                epoll_create1(0)
            };
            if epool_fd < 0 {
                panic!("Failed to create epoll file descriptor");
            }

            host.listener.set_nonblocking(true).unwrap();

            let fd = host.listener.as_raw_fd();
            
            let mut event = libc::epoll_event {
                events: (EPOLLIN | EPOLLET) as u32,
                u64: fd as u64,
            };

            unsafe {
                if epoll_ctl(epool_fd, EPOLL_CTL_ADD, fd, &mut event) < 0 {
                    panic!("Failed to add file descriptor to epoll");
                }
            }

            println!("Serveur HTTP {} écoutant sur le port {}", host.server_name, host.port);

            host.run();
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);



    let request = crate::http::request::parse_request(&request_str).unwrap();


    let mut headers: Vec<crate::http::header::Header> = Vec::new();
    headers.push(crate::http::header::Header {
        name: crate::http::header::HeaderName::ContentType,
        value: crate::http::header::HeaderValue {
            value: "application/json".to_string(),
            parsed_value: Some(crate::http::header::HeaderParsedValue::ContentType(crate::http::header::ContentType::ApplicationJson)),
        },
    });

    let response = crate::http::response::Response::new(
        HttpStatusCode::Ok,
        headers,
        Some(crate::http::body::Body::from_json(serde_json::json!({
            "message": "Hello!"
        })))
    );

    stream.write_all(response.to_string().as_bytes()).unwrap();
}