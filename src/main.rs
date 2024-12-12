mod http;

use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use http::header::{self, Header, HeaderName, HeaderValue, ContentType};
use http::request::parse_request;
use http::body::Body;
use http::response::{Response, HttpStatusCode};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    //stream.read(&mut buffer).unwrap();
    let bytes_read = stream.read(&mut buffer).unwrap();

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

    println!("Requête : {}", request_str);

    let request = parse_request(&request_str).unwrap();

    //println!("Body: {:?}", request.body.unwrap());

    if let Body::Text(ref text) = request.body.clone().unwrap() {
        println!("Text: {:?}", text);
    } else {
        println!("Not text");
    }

    if let Body::Json(ref json) = request.body.unwrap() {
        println!("JSON: {:?}", json);
    } else {
        println!("Not JSON");
    }

    let mut headers: Vec<Header> = Vec::new();
    headers.push(Header {
        name: header::HeaderName::ContentType,
        value: HeaderValue {
            value: "application/json".to_string(),
            parsed_value: Some(header::HeaderParsedValue::ContentType(ContentType::ApplicationJson)),
        },
    });

    let response = Response::new(
        "HTTP/1.1".to_string(), 
        HttpStatusCode::Ok,
        headers,
        Some(Body::from_json(serde_json::json!({
            "message": "Hello!"
        })))
    );

    stream.write_all(response.to_string().as_bytes()).unwrap();
}



fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Serveur HTTP écoutant sur le port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Erreur de connexion : {}", e);
            }
        }
    }

    Ok(())
}