use serde_json;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
    FormUrlEncoded(FormUrlEncoded),
  //  MultipartFormData(MultipartFormData),
    Binary(Vec<u8>),
    Empty
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormUrlEncoded {
    data: HashMap<String, String>
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

    pub fn from_parsing(content_type: &str, data: Vec<u8>) -> Body {
        let data_str = String::from_utf8_lossy(&data);
        println!("Raw data: {}", data_str);
        match content_type.trim() {
            "application/json" => {
                let json = serde_json::from_slice(&data).expect("Error parsing JSON");
                println!("JSON: {:?}", json);
                Body::Json(json)
            },
            "application/x-www-form-urlencoded" => {
                let form = FormUrlEncoded::new();
                Body::FormUrlEncoded(form)
            },
            "text/plain" => {
                let text = String::from_utf8(data).unwrap();
                Body::Text(text)
            },
            _ => Body::Binary(data)
        }
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



