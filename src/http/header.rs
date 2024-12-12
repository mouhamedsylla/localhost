use httpdate::parse_http_date;
use std::time::SystemTime;

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
pub enum ContentType {
    TextPlain,
    TextHtml,
    ApplicationJson,
    ApplicationXml,
    ApplicationFormUrlEncoded,
    MultipartFormData,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderName {
    // En-têtes généraux
    ContentType,
    ContentLength,
    TransferEncoding,
    Connection,
    Date,

    // En-têtes de requête
    Host,
    UserAgent,
    Accept,
    AcceptLanguage,
    AcceptEncoding,

    // En-têtes de réponse
    Server,
    StatusCode,

    // En-têtes de cache
    CacheControl,
    ETag,
    LastModified,

    // En-têtes de sécurité
    StrictTransportSecurity,
    Custom(String),
}

impl Header {
    pub fn new(name: HeaderName, value: HeaderValue) -> Header {
        Header { name, value }
    }
}

impl HeaderParsedValue {
    pub fn header_parsed_value(value: &str) -> HeaderParsedValue {
        match value {
            "text/plain" => HeaderParsedValue::ContentType(ContentType::TextPlain),
            "text/html" => HeaderParsedValue::ContentType(ContentType::TextHtml),
            "application/json" => HeaderParsedValue::ContentType(ContentType::ApplicationJson),
            "application/xml" => HeaderParsedValue::ContentType(ContentType::ApplicationXml),
            "application/x-www-form-urlencoded" => {
                HeaderParsedValue::ContentType(ContentType::ApplicationFormUrlEncoded)
            }
            "multipart/form-data" => HeaderParsedValue::ContentType(ContentType::MultipartFormData),
            "chunked" => HeaderParsedValue::TransferEncoding(TransferEncoding::Chunked),
            "compress" => HeaderParsedValue::TransferEncoding(TransferEncoding::Compress),
            "deflate" => HeaderParsedValue::TransferEncoding(TransferEncoding::Deflate),
            "gzip" => HeaderParsedValue::TransferEncoding(TransferEncoding::Gzip),
            "identity" => HeaderParsedValue::TransferEncoding(TransferEncoding::Identity),
            "keep-alive" => HeaderParsedValue::Connection(Connection::KeepAlive),
            "close" => HeaderParsedValue::Connection(Connection::Close),
            _ => HeaderParsedValue::Raw,
        }
    }

    // fn parse_content_type(value: &str) -> Option<HeaderParsedValue> {
    //     match value {
    //         "text/plain" => Some(HeaderParsedValue::ContentType(ContentType::TextPlain)),
    //         "text/html" => Some(HeaderParsedValue::ContentType(ContentType::TextHtml)),
    //         "application/json" => Some(HeaderParsedValue::ContentType(ContentType::ApplicationJson)),
    //         "application/xml" => Some(HeaderParsedValue::ContentType(ContentType::ApplicationXml)),
    //         "application/x-www-form-urlencoded" => {
    //             Some(HeaderParsedValue::ContentType(ContentType::ApplicationFormUrlEncoded))
    //         }
    //         "multipart/form-data" => Some(HeaderParsedValue::ContentType(ContentType::MultipartFormData)),
    //         _ => None,
    //     }
    // }

    // fn parse_date(value: &str) -> Option<HeaderParsedValue> {
    //     Some(HeaderParsedValue::Date(parse_http_date(value).unwrap()))
    // }

    // fn parse_content_length(value: &str) -> Option<HeaderParsedValue> {
    //     Some(HeaderParsedValue::ContentLength(value.parse().unwrap()))
    // }

    // fn parse_connection(value: &str) -> Option<HeaderParsedValue> {
    //     match value {
    //         "keep-alive" => Some(HeaderParsedValue::Connection(Connection::KeepAlive)),
    //         "close" => Some(HeaderParsedValue::Connection(Connection::Close)),
    //         _ => None,
    //     }
    // }

    // fn parse_transfer_encoding(value: &str) -> HeaderParsedValue {
    //     match value {
    //         "chunked" => HeaderParsedValue::TransferEncoding(TransferEncoding::Chunked),
    //         "compress" => HeaderParsedValue::TransferEncoding(TransferEncoding::Compress),
    //         "deflate" => HeaderParsedValue::TransferEncoding(TransferEncoding::Deflate),
    //         "gzip" => HeaderParsedValue::TransferEncoding(TransferEncoding::Gzip),
    //         "identity" => HeaderParsedValue::TransferEncoding(TransferEncoding::Identity),
    //         _ => HeaderParsedValue::Custom(value.to_string()),
    //     }
    // }

    // fn parse_server(value: &str) -> HeaderParsedValue {
    //     HeaderParsedValue::Server(value.to_string())
    // }
}

impl HeaderName {
    pub fn parse_header_name(name: &str) -> HeaderName {
        match name {
            "Content-Type" => HeaderName::ContentType,
            "Content-Length" => HeaderName::ContentLength,
            "Transfer-Encoding" => HeaderName::TransferEncoding,
            "Connection" => HeaderName::Connection,
            "Date" => HeaderName::Date,
            "Host" => HeaderName::Host,
            "User-Agent" => HeaderName::UserAgent,
            "Accept" => HeaderName::Accept,
            "Accept-Language" => HeaderName::AcceptLanguage,
            "Accept-Encoding" => HeaderName::AcceptEncoding,
            "Server" => HeaderName::Server,
            "Status-Code" => HeaderName::StatusCode,
            "Cache-Control" => HeaderName::CacheControl,
            "ETag" => HeaderName::ETag,
            "Last-Modified" => HeaderName::LastModified,
            "Strict-Transport-Security" => HeaderName::StrictTransportSecurity,
            _ => HeaderName::Custom(name.to_string()),
        }
    }
}
