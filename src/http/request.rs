use crate::http::header::Header;
use crate::http::body::{Body, FormUrlEncoded, BodyError};
use crate::http::header::{HeaderName, HeaderParsedValue, HeaderValue, ContentType};
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

impl HttpMethod {
    pub fn from_str(method: &str) -> HttpMethod {
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
        }
    }
    
}


#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub uri: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Option<Body>
}

pub struct RequestBuilder {
    method: HttpMethod,
    uri: String,
    version: String,
    headers: Vec<Header>,
    body: Option<Body>
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

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            method: HttpMethod::GET,
            uri: String::new(),
            version: "HTTP/1.1".to_string(),
            headers: Vec::new(),
            body: None
        }
    }

    pub fn method(mut self, method: &str) -> RequestBuilder {
        self.method = HttpMethod::from_str(method);
        self
    }

    pub fn uri(mut self, uri: &str) -> RequestBuilder {
        self.uri = uri.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> RequestBuilder {
        self.version = version.to_string();
        self
    }

    pub fn header(mut self, headers: Vec<Header>) -> RequestBuilder {
        self.headers = headers.to_vec();
        self
    }

    pub fn body(mut self, body: Body) -> RequestBuilder {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Request {
        Request::new(self.method, self.uri, self.version, self.headers, self.body)
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

    pub fn get_header(&self, name: HeaderName) -> Option<Header> {
        self.headers.iter().find(|&h| h.name == name).cloned()
    }


}


pub fn parse_request(request: &[u8]) -> Option<Request> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = HttparseRequest::new(&mut headers);

    let header_len = match req.parse(request) {
        Ok(httparse::Status::Complete(len)) => len,
        _ => return None,
    };


    let headers = req.headers
        .iter()
        .map(|h| Header::from_str(h.name, std::str::from_utf8(h.value).unwrap())).collect::<Vec<Header>>();

    let result = if request.len() > header_len {
        let body_data = &request[header_len..];
        let content_type = headers.iter()
            .find(|h| h.name == HeaderName::ContentType)?;

        let parsed_content_type = ContentType::parse_content_type(content_type).unwrap();
        let boundary = parsed_content_type.params.get("boundary")?;

        Body::from_mime(&parsed_content_type.mime, body_data.to_vec(), Some(boundary))
    } else {
        Err(BodyError::EmptyBody("No body data found".to_string()))
    };

    let body = match result {
        Ok(body) => Some(body),
        Err(_) => None
    };

    Some(Request::new(
        HttpMethod::from_str(req.method.unwrap()),
        req.path.unwrap().to_string(),
        req.version.unwrap().to_string(),
        headers,
        body
    ))
}