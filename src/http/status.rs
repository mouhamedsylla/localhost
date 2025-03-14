use std::fmt;

#[derive(Debug, Clone)]
pub enum HttpStatusCode {
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
}


impl HttpStatusCode {
    pub fn as_str(&self) -> &str {
        match self {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::Created => "201 Created",
            HttpStatusCode::Accepted => "202 Accepted",
            HttpStatusCode::NoContent => "204 No Content",
            HttpStatusCode::MovedPermanently => "301 Moved Permanently",
            HttpStatusCode::Found => "302 Found",
            HttpStatusCode::SeeOther => "303 See Other",
            HttpStatusCode::NotModified => "304 Not Modified",
            HttpStatusCode::TemporaryRedirect => "307 Temporary Redirect",
            HttpStatusCode::PermanentRedirect => "308 Permanent Redirect",
            HttpStatusCode::BadRequest => "400 Bad Request",
            HttpStatusCode::Unauthorized => "401 Unauthorized",
            HttpStatusCode::Forbidden => "403 Forbidden",
            HttpStatusCode::NotFound => "404 Not Found",
            HttpStatusCode::MethodNotAllowed => "405 Method Not Allowed",
            HttpStatusCode::RequestTimeout => "408 Request Timeout",
            HttpStatusCode::Conflict => "409 Conflict",
            HttpStatusCode::Gone => "410 Gone",
            HttpStatusCode::LengthRequired => "411 Length Required",
            HttpStatusCode::PreconditionFailed => "412 Precondition Failed",
            HttpStatusCode::PayloadTooLarge => "413 Payload Too Large",
            HttpStatusCode::URITooLong => "414 URI Too Long",
            HttpStatusCode::UnsupportedMediaType => "415 Unsupported Media Type",
            HttpStatusCode::RangeNotSatisfiable => "416 Range Not Satisfiable",
            HttpStatusCode::ExpectationFailed => "417 Expectation Failed",
            HttpStatusCode::InternalServerError => "500 Internal Server Error",
            HttpStatusCode::NotImplemented => "501 Not Implemented",
            HttpStatusCode::BadGateway => "502 Bad Gateway",
            HttpStatusCode::ServiceUnavailable => "503 Service Unavailable",
            HttpStatusCode::GatewayTimeout => "504 Gateway Timeout",
            HttpStatusCode::HTTPVersionNotSupported => "505 HTTP Version Not Supported",
        }
    }

    pub fn from_code(code: u16) -> Option<HttpStatusCode> {
        match code {
            200 => Some(HttpStatusCode::Ok),
            201 => Some(HttpStatusCode::Created),
            202 => Some(HttpStatusCode::Accepted),
            204 => Some(HttpStatusCode::NoContent),
            301 => Some(HttpStatusCode::MovedPermanently),
            302 => Some(HttpStatusCode::Found),
            303 => Some(HttpStatusCode::SeeOther),
            304 => Some(HttpStatusCode::NotModified),
            307 => Some(HttpStatusCode::TemporaryRedirect),
            308 => Some(HttpStatusCode::PermanentRedirect),
            400 => Some(HttpStatusCode::BadRequest),
            401 => Some(HttpStatusCode::Unauthorized),
            403 => Some(HttpStatusCode::Forbidden),
            404 => Some(HttpStatusCode::NotFound),
            405 => Some(HttpStatusCode::MethodNotAllowed),
            408 => Some(HttpStatusCode::RequestTimeout),
            409 => Some(HttpStatusCode::Conflict),
            410 => Some(HttpStatusCode::Gone),
            411 => Some(HttpStatusCode::LengthRequired),
            412 => Some(HttpStatusCode::PreconditionFailed),
            413 => Some(HttpStatusCode::PayloadTooLarge),
            414 => Some(HttpStatusCode::URITooLong),
            415 => Some(HttpStatusCode::UnsupportedMediaType),
            416 => Some(HttpStatusCode::RangeNotSatisfiable),
            417 => Some(HttpStatusCode::ExpectationFailed),
            500 => Some(HttpStatusCode::InternalServerError),
            501 => Some(HttpStatusCode::NotImplemented),
            502 => Some(HttpStatusCode::BadGateway),
            503 => Some(HttpStatusCode::ServiceUnavailable),
            504 => Some(HttpStatusCode::GatewayTimeout),
            505 => Some(HttpStatusCode::HTTPVersionNotSupported),
            _ => None,}
    }
}