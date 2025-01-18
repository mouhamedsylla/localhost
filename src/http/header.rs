use std::hash::Hash;
use std::time::SystemTime;
use std::fmt;
use std::collections::HashMap;

// ============= Type Imports & Re-exports =============
mod types {
    use super::*;
    
    // Déplacer les types de base ici
    pub use super::{Header, HeaderValue, HeaderName, HeaderParsedValue};
}

// ============= Content Types =============
mod content {
    use super::*;
    
    pub use super::{ParsedContentType, ParsedContentDisposition, ContentType};
}

// ============= Cookie Handling =============
mod cookie {
    use super::*;
    
    pub use super::{Cookie, CookieOptions, SameSitePolicy};
}

// ============= Main Structures =============
#[derive(Debug, Clone)]
pub struct Header {
    pub name: HeaderName,
    pub value: HeaderValue,
}

#[derive(Debug, Clone)]
pub struct HeaderValue {
    pub value: String,
    pub parsed_value: Option<HeaderParsedValue>,
}

#[derive(Debug)]
pub struct ParsedContentType {
    pub mime: String,
    pub params: HashMap<String, String>,
}

pub struct ParsedContentDisposition {
    pub disposition: String,
    pub params: HashMap<String, String>,
}

// ============= Header Type Enums =============
#[derive(Debug, Clone)]
pub enum HeaderParsedValue {
    ContentType(ContentType),
    ContentLength(u64),
    Connection(Connection),
    TransferEncoding(TransferEncoding),
    Server(String),
    Date(SystemTime),
    Custom(String),
    Cookie(Cookie),
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderName {
    // Standard headers
    ContentType,
    ContentLength,
    ContentDisposition,
    Connection,
    TransferEncoding,
    Cookie,
    SetCookie,
    CacheControl,
    Date,
    Host,
    
    // Accept headers
    Accept,
    AcceptLanguage,
    AcceptEncoding,
    
    // Response headers
    Server,
    StatusCode,
    
    // Cache headers
    ETag,
    LastModified,
    
    // Security headers
    StrictTransportSecurity,
    
    // Custom header
    Custom(String),
}

// ============= Specific Value Enums =============
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    TextPlain,
    TextHtml,
    ApplicationJson,
    ApplicationXml,
    ApplicationFormUrlEncoded,
    MultipartFormData,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferEncoding {
    Chunked,
    Compress,
    Deflate,
    Gzip,
    Identity,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    options: CookieOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CookieOptions {
    http_only: bool,
    secure: bool,
    max_age: Option<u64>,
    path: Option<String>,
    expires: Option<SystemTime>,
    domain: Option<String>,
    pub same_site: SameSitePolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Connection {
    KeepAlive,
    Close,
}

// ============= Header Implementations =============
impl Header {
    pub fn new(name: HeaderName, value: HeaderValue) -> Header {
        Header { name, value }
    }

    pub fn from_mime(mime: &str) -> Header {
        Header {
            name: HeaderName::ContentType,
            value: HeaderValue {
                value: mime.to_string(),
                parsed_value: Some(HeaderParsedValue::ContentType(
                    ContentType::from_str(mime)
                )),
            },
        }
    }

    pub fn from_str(name: &str, value: &str) -> Header {
        let header_name = HeaderName::from_str(name);
        let header_value = HeaderValue {
            value: value.to_string(),
            parsed_value: Some(HeaderParsedValue::from_str(&header_name, value)),
        };

        Header::new(header_name, header_value)
    }



}

// ============= HeaderName Implementations =============
impl HeaderName {
    pub fn from_str(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "content-type" => HeaderName::ContentType,
            "content-length" => HeaderName::ContentLength,
            "content-disposition" => HeaderName::ContentDisposition,
            "transfer-encoding" => HeaderName::TransferEncoding,
            "connection" => HeaderName::Connection,
            "date" => HeaderName::Date,
            "host" => HeaderName::Host,
            "accept" => HeaderName::Accept,
            "accept-language" => HeaderName::AcceptLanguage,
            "accept-encoding" => HeaderName::AcceptEncoding,
            "server" => HeaderName::Server,
            "status-code" => HeaderName::StatusCode,
            "cache-control" => HeaderName::CacheControl,
            "etag" => HeaderName::ETag,
            "last-modified" => HeaderName::LastModified,
            "strict-transport-security" => HeaderName::StrictTransportSecurity,
            _ => HeaderName::Custom(name.to_string()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HeaderName::ContentType => "Content-Type",
            HeaderName::ContentLength => "Content-Length",
            HeaderName::ContentDisposition => "Content-Disposition",
            HeaderName::TransferEncoding => "Transfer-Encoding",
            HeaderName::Connection => "Connection",
            HeaderName::Cookie => "Cookie",
            HeaderName::SetCookie => "Set-Cookie",
            HeaderName::Date => "Date",
            HeaderName::Host => "Host",
            HeaderName::Accept => "Accept",
            HeaderName::AcceptLanguage => "Accept-Language",
            HeaderName::AcceptEncoding => "Accept-Encoding",
            HeaderName::Server => "Server",
            HeaderName::StatusCode => "Status-Code",
            HeaderName::CacheControl => "Cache-Control",
            HeaderName::ETag => "ETag",
            HeaderName::LastModified => "Last-Modified",
            HeaderName::StrictTransportSecurity => "Strict-Transport-Security",
            HeaderName::Custom(_) => "", // Returns empty string for custom headers
        }
    }
}

// ============= ContentType Implementations =============
impl ContentType {
    pub fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "text/plain" => ContentType::TextPlain,
            "text/html" => ContentType::TextHtml,
            "application/json" => ContentType::ApplicationJson,
            "application/xml" => ContentType::ApplicationXml,
            "application/x-www-form-urlencoded" => ContentType::ApplicationFormUrlEncoded,
            "multipart/form-data" => ContentType::MultipartFormData,
            _ => ContentType::Raw,
        }
    }

    pub fn parse_content_type(header: &Header) -> Option<ParsedContentType> {
        if header.name != HeaderName::ContentType {
            return None;
        }

        let parts = header.value.value.split(';').map(|part| part.trim()).collect::<Vec<&str>>();
        let mime = parts[0].to_string();
        let mut params = HashMap::new();
        
        for part in parts[1..].iter() {
            let mut parts = part.split('=');
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap_or("").to_string();
            params.insert(key, value);
        }

        Some(ParsedContentType { mime, params })
    }
}

// ============= Content-Disposition Implementation   =============

impl ParsedContentDisposition {
    pub fn parse_content_disposition(header: &Header) -> Option<ParsedContentDisposition> {
        if header.name != HeaderName::ContentDisposition {
            return None;
        }

        let parts = header.value.value.split(';').map(|part| part.trim()).collect::<Vec<&str>>();
        let disposition = parts[0].to_string();
        let mut params = HashMap::new();
        parts[1..].iter().for_each(|part| {
            let mut parts = part.split('=');
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap_or("").to_string();
            params.insert(key, value);
        }); 

        Some(ParsedContentDisposition { disposition, params })
    }
}

// ============= HeaderParsedValue Implementations =============
impl HeaderParsedValue {
    pub fn from_str(header_name: &HeaderName, value: &str) -> Self {
        match header_name {
            HeaderName::ContentType => {
                HeaderParsedValue::ContentType(ContentType::from_str(value))
            }
            HeaderName::ContentLength => {
                if let Ok(length) = value.parse() {
                    HeaderParsedValue::ContentLength(length)
                } else {
                    HeaderParsedValue::Raw
                }
            }
            
            HeaderName::TransferEncoding => match value.to_lowercase().as_str() {
                "chunked" => HeaderParsedValue::TransferEncoding(TransferEncoding::Chunked),
                "compress" => HeaderParsedValue::TransferEncoding(TransferEncoding::Compress),
                "deflate" => HeaderParsedValue::TransferEncoding(TransferEncoding::Deflate),
                "gzip" => HeaderParsedValue::TransferEncoding(TransferEncoding::Gzip),
                "identity" => HeaderParsedValue::TransferEncoding(TransferEncoding::Identity),
                _ => HeaderParsedValue::Raw,
            },
            HeaderName::Connection => match value.to_lowercase().as_str() {
                "keep-alive" => HeaderParsedValue::Connection(Connection::KeepAlive),
                "close" => HeaderParsedValue::Connection(Connection::Close),
                _ => HeaderParsedValue::Raw,
            },
            HeaderName::Cookie => {
                if let Some(cookie) = Cookie::parse(value) {
                    HeaderParsedValue::Cookie(cookie)
                } else {
                    HeaderParsedValue::Raw
                }
            }
            HeaderName::Server => HeaderParsedValue::Server(value.to_string()),
            HeaderName::Date => {
                HeaderParsedValue::Raw
            }
            _ => HeaderParsedValue::Custom(value.to_string()),
        }
    }
}

// ============= Cookie Implementations  =============
impl Cookie {
    pub fn new(name: &str, value: &str) -> Cookie {
        Cookie { 
            name: name.to_string(), 
            value: value.to_string(), 
            options: CookieOptions::default()
        }
    }

    pub fn with_options(name: &str, value: &str, options: CookieOptions) -> Cookie {
        Cookie { name: name.to_string(), value: value.to_string(), options }
    }

    pub fn parse(cookie_str: &str) -> Option<Cookie> {
        let mut parts = cookie_str.split(';');

        //Parse name=value
        let name_value = parts.next()?;
        let mut name_value_parts = name_value.split('=');
        let name = name_value_parts.next()?.trim();
        let value = name_value_parts.next()?.trim();

        let mut cookie = Cookie::new(name, value);

        //Parse options
        for opt in parts {
            let mut opt_parts = opt.split('=');
            let key = opt_parts.next()?.trim();
            let value = opt_parts.next().unwrap_or("").trim();

            match key.to_lowercase().as_str() {
                "httponly" => cookie.options.http_only = true,
                "secure" => cookie.options.secure = true,
                "max-age" => cookie.options.max_age = Some(value.parse().unwrap_or(0)),
                "path" => cookie.options.path = Some(value.to_string()),
                "expires" => cookie.options.expires = Some(SystemTime::now()),
                "domain" => cookie.options.domain = Some(value.to_string()),
                "samesite" => cookie.options.same_site = match value.to_lowercase().as_str() {
                    "strict" => SameSitePolicy::Strict,
                    "lax" => SameSitePolicy::Lax,
                    _ => SameSitePolicy::None,
                },
                _ => {}
            }
        }

        Some(cookie)
    }

    pub fn to_string(&self) -> String {
        let mut cookie_str = format!("{}={}", self.name, self.value);

        if self.options.http_only {
            cookie_str.push_str("; HttpOnly");
        }

        if self.options.secure {
            cookie_str.push_str("; Secure");
        }

        if let Some(max_age) = self.options.max_age {
            cookie_str.push_str(&format!("; Max-Age={}", max_age));
        }

        if let Some(ref path) = self.options.path {
            cookie_str.push_str(&format!("; Path={}", path));
        }

        if let Some(ref expires) = self.options.expires {
            cookie_str.push_str(&format!("; Expires={:?}", expires));
        }

        if let Some(ref domain) = self.options.domain {
            cookie_str.push_str(&format!("; Domain={}", domain));
        }

        match self.options.same_site {
            SameSitePolicy::Strict => cookie_str.push_str("; SameSite=Strict"),
            SameSitePolicy::Lax => cookie_str.push_str("; SameSite=Lax"),
            SameSitePolicy::None => cookie_str.push_str("; SameSite=None"),
        }

        cookie_str
    }
}

impl Default for CookieOptions {
    fn default() -> Self {
        CookieOptions {
            http_only: false,
            secure: false,
            max_age: None,
            path: Some("/".to_string()),
            expires: None,
            domain: None,
            same_site: SameSitePolicy::Lax
        }
    }
    
}


// ============= Display Implementations =============
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeaderName::Custom(ref name) => write!(f, "{}", name),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}