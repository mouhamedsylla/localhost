use serde_json;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
    FormUrlEncoded(FormUrlEncoded),
  //MultipartFormData(MultipartFormData),
    Binary(Vec<u8>),
    Empty
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormUrlEncoded {
    data: HashMap<String, String>
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Body::Text(text) => write!(f, "{}", text),
            Body::Json(json) => write!(f, "{}", json),
            Body::FormUrlEncoded(form) => {
                let mut form_str = String::new();
                for (key, value) in form.data.iter() {
                    form_str.push_str(&format!("{}={}&", key, value));
                }
                write!(f, "{}", form_str)
            },
            Body::Binary(data) => write!(f, "{:?}", data),
            Body::Empty => write!(f, "")
        }
    }
}

impl FormUrlEncoded {
    pub fn new() -> FormUrlEncoded {
        FormUrlEncoded {
            data: HashMap::new()
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}


impl Body {
    pub fn from_text(text: &str) -> Body {
        Body::Text(text.to_string())
    }

    pub fn from_json(json: serde_json::Value) -> Body {
        Body::Json(json)
    }

    pub fn from_form_urlencoded(form: FormUrlEncoded) -> Body {
        Body::FormUrlEncoded(form)
    }

    pub fn from_binary(data: Vec<u8>) -> Body {
        Body::Binary(data)
    }

    pub fn from_empty() -> Body {
        Body::Empty
    }
}

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct MultipartFormData {
//     fileds: Vec<MutipartData>    
// }


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct MutipartData {
//     name: String,
//     content_type: Option<Mime>,
//     data: Vec<u8>
// }



