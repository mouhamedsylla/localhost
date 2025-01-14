use std::hash::Hash;
use std::time::SystemTime;
use std::fmt;
use std::collections::HashMap;

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
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderName {
    // Content headers
    ContentType,
    ContentLength,
    ContentDisposition,

    // Connection headers
    Connection,
    TransferEncoding,
    Host,
    
    // Accept headers
    Accept,
    AcceptLanguage,
    AcceptEncoding,
    
    // Response headers
    Server,
    StatusCode,
    Date,
    
    // Cache headers
    CacheControl,
    ETag,
    LastModified,
    
    // Security headers
    StrictTransportSecurity,
    
    // Custom headers
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
            HeaderName::Server => HeaderParsedValue::Server(value.to_string()),
            HeaderName::Date => {
                HeaderParsedValue::Raw
            }
            _ => HeaderParsedValue::Custom(value.to_string()),
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