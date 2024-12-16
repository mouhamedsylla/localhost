use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

pub struct Host {
    port: String,
    server_name: String,
    listener: TcpListener,
}

pub struct Server {
    hosts: Vec<Host>,
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
        println!("Serveur HTTP {} écoutant sur le port {}", self.server_name, self.port);
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
        "HTTP/1.1".to_string(), 
        crate::http::response::HttpStatusCode::Ok,
        headers,
        Some(crate::http::body::Body::from_json(serde_json::json!({
            "message": "Hello!"
        })))
    );

    stream.write_all(response.to_string().as_bytes()).unwrap();
}