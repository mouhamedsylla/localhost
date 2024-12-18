use crate::http::header::Header;
use crate::http::body::Body;
use crate::http::header::{HeaderName, HeaderParsedValue, HeaderValue};
use httparse::Request as HttparseRequest;
use std::fmt;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
    OPTIONS,
    HEAD,
    CONNECT,
    TRACE    
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub uri: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Option<Body>
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::CONNECT => write!(f, "CONNECT"),
            HttpMethod::TRACE => write!(f, "TRACE")
        }
    }
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


    pub fn to_string(&self) -> String {
        let mut request = format!("{} {} {}\r\n", self.method, self.uri, self.version);
        for header in self.headers.clone() {
            request.push_str(&header.to_string());
            request.push_str("\r\n");
        }

        request.push_str("\r\n");
        match &self.body {
            Some(body) => {
                request.push_str(&body.to_string());
            }
            None => {}
        }
        request
    }

}

pub fn parse_request(request: &str) -> Option<Request> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = HttparseRequest::new(&mut headers);
    let _ = req.parse(request.as_bytes());
    let method = req.method.unwrap();
    let uri = req.path.unwrap();
    let version = req.version.unwrap();
    let headers = req.headers.iter().map(|h| Header::new(
        HeaderName::parse_header_name(h.name),
        HeaderValue {
            value: String::from_utf8_lossy(h.value).to_string(),
            parsed_value: Some(HeaderParsedValue::header_parsed_value(String::from_utf8_lossy(h.value).trim()))
        }
    )).collect::<Vec<Header>>();

    let body = None;

    let request = Request::new(
        match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "DELETE" => HttpMethod::DELETE,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "OPTIONS" => HttpMethod::OPTIONS,
            "HEAD" => HttpMethod::HEAD,
            "CONNECT" => HttpMethod::CONNECT,
            "TRACE" => HttpMethod::TRACE,
            _ => HttpMethod::GET
        },
        uri.to_string(),
        version.to_string(),
        headers,
        body
    );

    Some(request)
}


// pub fn parse_request(resquest: &str) -> Option<Request> {
//     let mut lines = resquest.lines();

//     let first_line = lines.next().unwrap();
//     let mut parts: Vec<&str> = first_line.split_whitespace().collect();

//     if parts.len() != 3 {
//         return None;
//     }

//     let method = match parts[0] {
//         "GET" => HttpMethod::GET,
//         "POST" => HttpMethod::POST,
//         "DELETE" => HttpMethod::DELETE,
//         "PUT" => HttpMethod::PUT,
//         "PATCH" => HttpMethod::PATCH,
//         "OPTIONS" => HttpMethod::OPTIONS,
//         "HEAD" => HttpMethod::HEAD,
//         "CONNECT" => HttpMethod::CONNECT,
//         "TRACE" => HttpMethod::TRACE,
//         _ => return None
//     };

//     let mut headers = Vec::new();
//     let mut lines_num = 0;

//     for line in lines.clone() {
//         lines_num += 1;
//         if line.is_empty() {
//             break;
//         }

//         let mut parts = line.splitn(2, ':');
//         let name = parts.next().unwrap();
//         let value = parts.next().unwrap();

//         let header = Header::new(
//             HeaderName::parse_header_name(name), 
//             HeaderValue {
//                 value: value.to_string(),
//                 parsed_value: Some(HeaderParsedValue::header_parsed_value(value.trim_ascii_start()))
//             }
//         );
//         headers.push(header);
//     }

//     let mut body = None;

//     if let Some(content_type) = get_header(headers.clone(), HeaderName::ContentType) {
//         body = Some(Body::from_parsing(content_type.value.value.as_str(), lines.collect::<Vec<&str>>()[lines_num..].join("\n").into_bytes()));
//     }

//     Some(Request::new(method, parts[1].to_string(), parts[2].to_string(), headers, body))
    
// }

// fn get_header(headers: Vec<Header>, name: HeaderName) -> Option<Header> {
//     headers.iter().find(|&h| h.name == name).cloned()
// }