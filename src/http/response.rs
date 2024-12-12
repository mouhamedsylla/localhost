use crate::http::header::Header;
use crate::http::body::Body;

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
    ) -> Response {
        Response {
            version,
            status_code,
            headers: Vec::new(),
            body: None
        }
    }

    pub fn add_header() {
    
    }

    /*
        - Add a method to set the body of the response

    **/ 
}