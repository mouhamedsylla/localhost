use crate::http::header::Header;
use crate::http::body::Body;

use super::header;

pub enum HttpStatusCode {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
    
}

pub struct Response {
    pub version: String,
    pub status_code: HttpStatusCode,
    pub headers: Vec<Header>,
    pub body: Option<Body>
}

impl Response {
    pub fn new(
        version: String,
        status_code: HttpStatusCode,
        headers: Vec<Header>,
        body: Option<Body>
    ) -> Response {
        Response {
            version,
            status_code,
            headers,
            body
        }
    }

    pub fn to_string(self) -> String {
        let mut response = format!("{} {}\r\n", self.version, self.status_code as u16);
        for header in self.headers {
            response.push_str(&header.to_string());
            response.push_str("\r\n");
        }
        response.push_str("\r\n");
        match self.body {
            Some(body) => {
                response.push_str(&body.to_string());
            }
            None => {}
        }
        response
    }

    pub fn add_header() {
    
    }
}