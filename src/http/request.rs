use crate::http::header::Header;
use crate::http::body::Body;
use crate::http::header::{HeaderName, HeaderParsedValue, HeaderValue};


#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,    
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub uri: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Option<Body>
}

impl Request {
    pub fn new(
        method: HttpMethod,
        uri: String,
        version: String,
        headers: Vec<Header>,
        body: Option<Body>
    ) -> Request {
        Request {
            method,
            uri,    
            version,
            headers,
            body
        }
    }


    pub fn add_header(mut self, header: Header) {
        self.headers.push(header);
    }

    // fn get_header(&self, name: HeaderName) -> Option<&Header> {
    //     self.headers.iter().find(|h| h.name == name)
    // }   


}


pub fn parse_request(resquest: &str) -> Option<Request> {
    let mut lines = resquest.lines();

    let first_line = lines.next().unwrap();
    let mut parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() != 3 {
        return None;
    }

    let method = match parts[0] {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "DELETE" => HttpMethod::DELETE,
        _ => return None
    };

    let mut headers = Vec::new();

    let mut lines_num = 0;

    for line in lines.clone() {
        lines_num += 1;
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, ':');
        let name = parts.next().unwrap();
        let value = parts.next().unwrap();

        let header = Header::new(
            HeaderName::parse_header_name(name), 
            HeaderValue {
                value: value.to_string(),
                parsed_value: Some(HeaderParsedValue::header_parsed_value(value.trim_ascii_start()))
            }
        );
        headers.push(header);
    }

    let mut body = None;

    if let Some(content_type) = get_header(headers.clone(), HeaderName::ContentType) {
        body = Some(Body::from_parsing(content_type.value.value.as_str(), lines.collect::<Vec<&str>>()[lines_num..].join("\n").into_bytes()));
    }

    Some(Request::new(method, parts[1].to_string(), parts[2].to_string(), headers, body))
    
}

fn get_header(headers: Vec<Header>, name: HeaderName) -> Option<Header> {
    headers.iter().find(|&h| h.name == name).cloned()
}