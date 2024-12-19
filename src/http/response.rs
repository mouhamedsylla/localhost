use crate::http::header::Header;
use crate::http::body::Body;
use crate::http::status::HttpStatusCode;
use crate::http::header::{HeaderName, HeaderValue, HeaderParsedValue, ContentType};

pub struct Response {
    pub version: String,
    pub status_code: HttpStatusCode,
    pub headers: Vec<Header>,
    pub body: Option<Body>
}

pub struct ResponseBuilder {
    version: String,
    status_code: HttpStatusCode,
    headers: Vec<Header>,
    body: Option<Body>
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder {
            version: "HTTP/1.1".to_string(),
            status_code: HttpStatusCode::Ok,
            headers: Vec::new(),
            body: None
        }
    }


    pub fn status_code(mut self, status_code: HttpStatusCode) -> ResponseBuilder {
        self.status_code = status_code;
        self
    }

    pub fn header(mut self, header: Header) -> ResponseBuilder {
        self.headers.push(header);
        self
    }

    pub fn body(mut self, body: Body) -> ResponseBuilder {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Response {
        Response::new(self.status_code, self.headers, self.body)
    }
}


impl Response {
    pub fn new(
        status_code: HttpStatusCode,
        headers: Vec<Header>,
        body: Option<Body>
    ) -> Response {
        Response {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers,
            body
        }
    }

    pub fn ok() -> Response {
        Response::new(
            HttpStatusCode::Ok,
            Vec::new(),
            None
        )
    }

    pub fn not_found() -> Response {
        Response::new(
            HttpStatusCode::NotFound,
            Vec::new(),
            None
        )
    }

    pub fn response_with_json(data: serde_json::Value) -> Response {
        let mut headers = Vec::new();
        headers.push(Header {
            name: HeaderName::ContentType,
            value: HeaderValue {
                value: "application/json".to_string(),
                parsed_value: Some(HeaderParsedValue::ContentType(ContentType::ApplicationJson)),
            },
        });

        Response {
            version: "HTTP/1.1".to_string(),
            status_code: HttpStatusCode::Ok,
            headers,
            body: Some(Body::json(data))
        }
    }

    pub fn response_with_html(data: &str) -> Response {
        let mut headers = Vec::new();
        headers.push(Header {
            name: HeaderName::ContentType,
            value: HeaderValue {
                value: "text/html".to_string(),
                parsed_value: Some(HeaderParsedValue::ContentType(ContentType::TextHtml))
            }
        });

        Response { 
            version: "HTTP/1.1".to_string(),
            status_code: HttpStatusCode::Ok,
            headers, 
            body: Some(Body::text(data))
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

}
