use serde_json;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::str;
use crate::http::header::{ParsedContentDisposition, ParsedContentType, Header, HeaderName, ContentType};
// ============= Type Definitions =============
pub type JsonValue = serde_json::Value;
pub type BinaryData = Vec<u8>;
pub type FormData = HashMap<String, String>;


// ============= Main Structures =============
#[derive(Debug, Clone)]
pub enum Body {
    Text(String),
    Json(JsonValue),
    FormUrlEncoded(FormUrlEncoded),
    Binary(BinaryData),
    Multipart(MultipartForm),
    Empty,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormUrlEncoded {
    data: FormData,
}

#[derive(Debug, Clone)]
pub struct MultipartFile {
    pub filename: String,
    pub content_type: String,
    pub data: BinaryData,
}

#[derive(Debug, Clone)]
pub struct MultipartForm {
    pub fields: HashMap<String, String>,
    pub files: HashMap<String, MultipartFile>,
}

// ============= Error Handling =============
#[derive(Debug)]
pub enum BodyError {
    InvalidUtf8(String),
    InvalidJson(String),
    UnsupportedMimeType(String),
    ParseError(String),
    MultipartError(String),
    EmptyBody(String),
}

impl std::error::Error for BodyError {}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BodyError::InvalidUtf8(msg) => write!(f, "Invalid UTF-8: {}", msg),
            BodyError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            BodyError::UnsupportedMimeType(mime) => write!(f, "Unsupported MIME type: {}", mime),
            BodyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BodyError::MultipartError(msg) => write!(f, "Multipart error: {}", msg),
            BodyError::EmptyBody(msg) => write!(f, "Empty body: {}", msg),
        }
    }
}

// ============= MultipartForm Implementations =============
impl MultipartForm {
    pub fn new() -> Self {
        MultipartForm {
            fields: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn set_data(
        &mut self,
        data: BinaryData, 
        boundary: &str, 
    ) {
        let boundary = format!("--{}", boundary);
        let parts = split_multipart(&data, boundary.as_bytes());

        for part in parts {
            if let Some((headers, data)) = parse_part(&part) {
                if let Some(cd_header) = headers.iter().find(|h| h.name == HeaderName::ContentDisposition) {
                    if let Some(ct_header) = headers.iter().find(|h| h.name == HeaderName::ContentType) {
                        let parsed_content_disposition = ParsedContentDisposition::parse_content_disposition(cd_header).unwrap();
                        let parsed_content_type = ContentType::parse_content_type(ct_header).unwrap();
                        
                        if let Some(name) = parsed_content_disposition.params.get("name") {
                            if let Some(filename) = parsed_content_disposition.params.get("filename") {
                                let file = MultipartFile {
                                    filename: filename.to_string(),
                                    content_type: parsed_content_type.mime,
                                    data
                                };
                                self.files.insert(name.to_string(), file);
                            } else {
                                if let Ok(text) = std::str::from_utf8(&data) {
                                    self.fields.insert(name.to_string(), text.to_string());
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Failed to parse part");
            }
        }
    }

    pub fn add_field(&mut self, name: &str, value: &str) {
        self.fields.insert(name.to_string(), value.to_string());
    }

    pub fn add_file(&mut self, name: &str, file: MultipartFile) {
        self.files.insert(name.to_string(), file);
    }

    pub fn get_field(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }

    pub fn get_file(&self, name: &str) -> Option<&MultipartFile> {
        self.files.get(name)
    }
}

// ============= Body Implementations =============
impl Body {
    // Constructor methods
    pub fn text(content: &str) -> Self {
        Body::Text(content.to_string())
    }

    pub fn json(content: JsonValue) -> Self {
        Body::Json(content)
    }

    pub fn form(form: FormUrlEncoded) -> Self {
        Body::FormUrlEncoded(form)
    }

    pub fn binary(data: BinaryData) -> Self {
        Body::Binary(data)
    }

    pub fn empty() -> Self {
        Body::Empty
    }

    // Content-Type based creation
    pub fn from_mime(mime: &str, data: BinaryData, boundary: Option<&str>) -> Result<Body, BodyError> {
        match mime.to_lowercase().as_str() {
            // Texte
            "text/plain" | "text/html" | "text/css" | "text/javascript" | "text/csv" => {
                let text = std::str::from_utf8(&data)
                    .map_err(|_| BodyError::InvalidUtf8(mime.to_string()))?;
                Ok(Body::text(text))
            }

            // JSON
            "application/json" => {
                let json = serde_json::from_slice(&data)
                    .map_err(|e| BodyError::InvalidJson(e.to_string()))?;
                Ok(Body::json(json))
            }

            // Form data
            "application/x-www-form-urlencoded" => {
                let form_str = std::str::from_utf8(&data)
                    .map_err(|_| BodyError::InvalidUtf8("form data".to_string()))?;
                let mut form = FormUrlEncoded::new();
                form.parse_str(form_str)?;
                Ok(Body::form(form))
            }

            // Multipart form data
            "multipart/form-data" => {
                println!("Data length: {}", data.len());
                if let Some(boundary) = boundary {
                    let mut form = MultipartForm::new();
                    form.set_data(data, boundary);
                    Ok(Body::Multipart(form))
                } else {
                    Err(BodyError::MultipartError("Missing boundary".to_string()))
                }
            }

            // Images
            mime if mime.starts_with("image/") => {
                println!("Image length: {}", data.len());
                Ok(Body::Binary(data))
            }

            // Documents
            "application/pdf" |
            "application/msword" |
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" |
            "application/vnd.ms-excel" |
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
            "application/vnd.ms-powerpoint" |
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
                Ok(Body::Binary(data))
            }

            // Audio
            mime if mime.starts_with("audio/") => {
                Ok(Body::Binary(data))
            }

            // Vidéo
            mime if mime.starts_with("video/") => {
                Ok(Body::Binary(data))
            }

            // Archives
            "application/zip" |
            "application/x-rar-compressed" |
            "application/x-7z-compressed" |
            "application/x-tar" |
            "application/gzip" => {
                Ok(Body::Binary(data))
            }

            // Fallback pour les données binaires non identifiées
            "application/octet-stream" => Ok(Body::Binary(data)),

            // Type MIME non supporté
            _ => Err(BodyError::UnsupportedMimeType(mime.to_string())),
        }
    }

    pub fn body_len(&self) -> usize {
        match self {
            Body::Text(text) => text.len(),
            Body::Json(json) => json.to_string().len(),
            Body::FormUrlEncoded(form) => {
                let parts: Vec<String> = form
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                parts.join("&").len()
            }
            Body::Binary(data) => data.len(),
            Body::Multipart(data) => {
                let fields_len: usize = data.fields.iter().map(|(k, v)| k.len() + v.len()).sum();
                let files_len: usize = data.files.iter().map(|(k, v)| k.len() + v.filename.len() + v.content_type.len() + v.data.len()).sum();
                fields_len + files_len
            }
            Body::Empty => 0,
        }
    }

    // Conversion methods
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Body::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&JsonValue> {
        match self {
            Body::Json(json) => Some(json),
            _ => None,
        }
    }

    pub fn as_form(&self) -> Option<&FormUrlEncoded> {
        match self {
            Body::FormUrlEncoded(form) => Some(form),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&BinaryData> {
        match self {
            Body::Binary(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_multipart(&self) -> Option<&MultipartForm> {
        match self {
            Body::Multipart(form) => Some(form),
            _ => None,
        }
    }
}

// Body Parsing


// ============= FormUrlEncoded Implementations =============
impl FormUrlEncoded {
    pub fn new() -> Self {
        FormUrlEncoded {
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn parse_str(&mut self, input: &str) -> Result<(), BodyError> {
        for pair in input.split('&').filter(|s| !s.is_empty()) {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => self.add(key, value),
                _ => return Err(BodyError::ParseError("Invalid form data format".to_string())),
            }
        }
        Ok(())
    }


    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.data.iter()
    }
}

// ============= Display Implementations =============
impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Body::Text(text) => write!(f, "{}", text),
            Body::Json(json) => write!(f, "{}", json),
            Body::FormUrlEncoded(form) => {
                let parts: Vec<String> = form
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                write!(f, "{}", parts.join("&"))
            }
            Body::Binary(data) => write!(f, "<{} bytes of binary data>", data.len()),
            Body::Multipart(form) => {
                write!(f, "Multipart Form (Fields: {}, Files: {})", 
                    form.fields.len(), 
                    form.files.len()
                )
            }
            Body::Empty => write!(f, ""),
        }
    }
}

// ============= Utility functions for multipart parsing =============
fn split_multipart(data: &[u8], boundary: &[u8]) -> Vec<Vec<u8>> {
    let mut parts = Vec::new();
    let mut current_pos = 0;

    while let Some(pos) = find_subsequence(&data[current_pos..], boundary) {
        if current_pos > 0 {
            parts.push(data[current_pos..current_pos + pos - 2].to_vec());
        }
        current_pos += pos + boundary.len();
    }

    parts
}


fn parse_part(part: &[u8]) -> Option<(Vec<Header>, Vec<u8>)> {
    let mut headers = Vec::new();
    let mut pos = 0;
    let mut part_data = Vec::new();

    while let Some(line_end) = find_subsequence(&part[pos..], b"\r\n") {
        if line_end == 2 {
            pos += 4;
            break;
        }
        
        if let Ok(line) = str::from_utf8(&part[pos..pos + line_end]) {
            if let Some((name, value)) = line.split_once(':') {
                headers.push(Header::from_str(name, value));
            }
        } else {
            part_data.extend_from_slice(&part[pos..pos + line_end]);
        }
        pos += line_end + 2;
    }

    Some((headers, part_data))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
}