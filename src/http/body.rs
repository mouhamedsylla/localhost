use serde_json;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;

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
    Empty,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormUrlEncoded {
    data: FormData,
}

// ============= Error Handling =============
#[derive(Debug)]
pub enum BodyError {
    InvalidUtf8(String),
    InvalidJson(String),
    UnsupportedMimeType(String),
    ParseError(String),
}

impl std::error::Error for BodyError {}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BodyError::InvalidUtf8(msg) => write!(f, "Invalid UTF-8: {}", msg),
            BodyError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            BodyError::UnsupportedMimeType(mime) => write!(f, "Unsupported MIME type: {}", mime),
            BodyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
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
    pub fn from_mime(mime: &str, data: BinaryData) -> Result<Body, BodyError> {
        match mime.to_lowercase().as_str() {
            "text/plain" | "text/html" | "text/css" | "text/javascript" => {
                let text = std::str::from_utf8(&data)
                    .map_err(|_| BodyError::InvalidUtf8(mime.to_string()))?;
                Ok(Body::text(text))
            }
            "application/json" => {
                let json = serde_json::from_slice(&data)
                    .map_err(|e| BodyError::InvalidJson(e.to_string()))?;
                Ok(Body::json(json))
            }
            "application/x-www-form-urlencoded" => {
                let form_str = std::str::from_utf8(&data)
                    .map_err(|_| BodyError::InvalidUtf8("form data".to_string()))?;
                let mut form = FormUrlEncoded::new();
                form.parse_str(form_str)?;
                Ok(Body::form(form))
            }
            "application/octet-stream" => Ok(Body::binary(data)),
            _ => Err(BodyError::UnsupportedMimeType(mime.to_string())),
        }
    }

    // Conversion methods
    // pub fn as_text(&self) -> Option<&str> {
    //     match self {
    //         Body::Text(text) => Some(text),
    //         _ => None,
    //     }
    // }

    // pub fn as_json(&self) -> Option<&JsonValue> {
    //     match self {
    //         Body::Json(json) => Some(json),
    //         _ => None,
    //     }
    // }

    // pub fn as_form(&self) -> Option<&FormUrlEncoded> {
    //     match self {
    //         Body::FormUrlEncoded(form) => Some(form),
    //         _ => None,
    //     }
    // }

    // pub fn as_binary(&self) -> Option<&BinaryData> {
    //     match self {
    //         Body::Binary(data) => Some(data),
    //         _ => None,
    //     }
    // }
}

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
            Body::Empty => write!(f, ""),
        }
    }
}