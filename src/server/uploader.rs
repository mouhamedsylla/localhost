use crate::http::response::Response;
use crate::http::request::{Request, HttpMethod};
use crate::http::status::HttpStatusCode;
use crate::http::header::{Header, HeaderName};
use crate::http::body::Body;
use serde_json::{json, Value};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use std::io::Write;
use std::io::BufReader;

#[derive(Debug, Clone)]
struct File {
    id: i32,
    name: String,
    path: PathBuf,
    size: u64
}

#[derive(Debug)]
pub struct Uploader {
    database: Vec<File>,
}


pub trait Handler {
    fn serve_http(
        &mut self, 
        request: &Request
    ) -> Result<Response, io::Error>;
}


impl Handler for Uploader {
    fn serve_http(&mut self, request: &Request) -> Result<Response, io::Error> {
        match request.method {
            HttpMethod::GET => self.GET(request),
            HttpMethod::POST => self.POST(request),
            // HttpMethod::DELETE => self.DELETE(),
            _ => Ok(Response::new(
                HttpStatusCode::MethodNotAllowed, 
                vec![Header::from_str("content-type", "application/json")],
                Some(Body::json(json!({
                    "message": "method not allowed"
                }))) 
            ))

        }
    }
}


// Uploader API
impl Uploader {
    pub fn GET(&self, request: &Request) -> Result<Response, io::Error> {
        if request.uri != "/api/files" {
            return self.not_found()
        }

        let body = Body::json(self.list_json()?);

        return Ok(Response::new(
            HttpStatusCode::Ok, 
            vec![
                    Header::from_str("content-type", "application"), 
                    Header::from_str("content-length", &body.body_len().to_string())
                    ], 
            Some(body)
        ))
    } 

    pub fn POST(&mut self, request: &Request) -> Result<Response, io::Error> {
        if request.uri != "/api/upload" {
            return self.not_found();
        }

        if let Some(body) = &request.body {
            match body {
                Body::Multipart(form) => {
                    let mut uploaded_files = Vec::new();

                    for (field_name, file) in &form.files {
                        // Vérifier le type MIME
                        if !self.is_allowed_mime_type(&file.content_type) {
                            return Ok(Response::new(
                                HttpStatusCode::UnsupportedMediaType,
                                vec![Header::from_str("content-type", "application/json")],
                                Some(Body::json(json!({
                                    "error": format!("Unsupported file type: {}", file.content_type)
                                })))
                            ));
                        }

                        // Vérifier la taille du fichier
                        if file.data.len() > self.max_file_size() {
                            return Ok(Response::new(
                                HttpStatusCode::PayloadTooLarge,
                                vec![Header::from_str("content-type", "application/json")],
                                Some(Body::json(json!({
                                    "error": "File too large"
                                })))
                            ));
                        }

                        let file_name = file.filename.replace("\"", "");
                        let file_path = self.generate_unique_path(&file_name);

                        // Créer le répertoire si nécessaire
                        if let Some(parent) = file_path.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        // Écrire le fichier
                        fs::write(&file_path, &file.data)?;

                        // Mettre à jour la base de données
                        let new_file = File {
                            id: self.generate_next_id(),
                            name: file.filename.clone(),
                            path: file_path.clone(),
                            size: file.data.len() as u64,
                        };

                        self.database.push(new_file.clone());
                        uploaded_files.push(json!({
                            "field": field_name,
                            "filename": new_file.name,
                            "size": new_file.size,
                            "id": new_file.id,
                            "type": file.content_type
                        }));
                    }

                    return Ok(Response::new(
                        HttpStatusCode::Created,
                        vec![Header::from_str("content-type", "application/json")],
                        Some(Body::json(json!({
                            "message": "Files uploaded successfully",
                            "files": uploaded_files
                        })))
                    ));
                },
                _ => return Ok(Response::new(
                    HttpStatusCode::BadRequest,
                    vec![Header::from_str("content-type", "application/json")],
                    Some(Body::json(json!({
                        "error": "Invalid body format"
                    })))
                ))
            }
        }

        Ok(Response::new(
            HttpStatusCode::BadRequest,
            vec![Header::from_str("content-type", "application/json")],
            Some(Body::json(json!({
                "error": "Missing file data"
            })))
        ))
    }

    fn is_allowed_mime_type(&self, mime_type: &str) -> bool {
        let allowed_types = [
            "text/", "image/", "application/pdf", "application/json",
            "application/msword", "application/vnd.openxmlformats-officedocument",
            "audio/", "video/"
        ];

        allowed_types.iter().any(|&allowed| mime_type.starts_with(allowed))
    }

    fn max_file_size(&self) -> usize {
        // 10 MB par défaut
        10 * 1024 * 1024
    }
    
    // pub fn POST(&mut self, request: &Request) -> Result<Response, io::Error> {
    //     if request.uri != "/api/upload" {
    //         return self.not_found();
    //     }
    
    //     // Parser le contenu multipart
    //     if let Some(body) = &request.body {
    //         match body {
    //             Body::Multipart(form) => {
    //                 let mut uploaded_files = Vec::new();
    
    //                 // Traiter chaque fichier dans le formulaire multipart
    //                 for (field_name, file) in &form.files {
    //                     // Générer un chemin unique pour le fichier
    //                     let file_name = file.filename.replace("\"", "");
    //                     let file_path = self.generate_unique_path(&file_name);
    
    //                     // Créer le répertoire parent si nécessaire
    //                     if let Some(parent) = file_path.parent() {
    //                         fs::create_dir_all(parent)?;
    //                     }
    
    //                     // Écrire le fichier
    //                     fs::write(&file_path, &file.data)?;
    
    //                     // Créer l'entrée dans la base de données
    //                     let new_file = File {
    //                         id: self.generate_next_id(),
    //                         name: file.filename.clone(),
    //                         path: file_path.clone(),
    //                         size: file.data.len() as u64,
    //                     };
    
    //                     // Ajouter à la base de données
    //                     self.database.push(new_file.clone());
    
    //                     // Ajouter aux informations de réponse
    //                     uploaded_files.push(json!({
    //                         "field": field_name,
    //                         "filename": new_file.name,
    //                         "size": new_file.size,
    //                         "id": new_file.id
    //                     }));
    //                 }
    
    //                 // Retourner la réponse avec les informations sur les fichiers uploadés
    //                 return Ok(Response::new(
    //                     HttpStatusCode::Created,
    //                     vec![Header::from_str("content-type", "application/json")],
    //                     Some(Body::json(json!({
    //                         "message": "Files uploaded successfully",
    //                         "files": uploaded_files
    //                     })))
    //                 ));
    //             },
    //             _ => return Ok(Response::new(
    //                 HttpStatusCode::BadRequest,
    //                 vec![Header::from_str("content-type", "application/json")],
    //                 Some(Body::json(json!({
    //                     "error": "Invalid body format"
    //                 })))
    //             ))
    //         }
    //     }
    
    //     Ok(Response::new(
    //         HttpStatusCode::BadRequest,
    //         vec![Header::from_str("content-type", "application/json")],
    //         Some(Body::json(json!({
    //             "error": "Missing file data"
    //         })))
    //     ))
    // }
    
    // pub fn DELETE(&self) -> Result<Response, io::Error> {

    // }

    fn not_found(&self) -> Result<Response, io::Error> {
        return Ok(Response::new(
            HttpStatusCode::NotFound, 
            vec![Header::from_str("content-type", "application/json")], 
            Some(Body::json(json!({
                "message": "route not found"
            })))
        ))
    }

    // Helper methods
    fn generate_next_id(&self) -> i32 {
        self.database.iter()
            .map(|f| f.id)
            .max()
            .unwrap_or(-1) + 1
    }

    fn generate_unique_path(&self, original_name: &str) -> PathBuf {
        let mut counter = 0;
        let mut file_name = original_name.to_string();
        let fname = file_name.clone();
        let ext = Path::new(&fname)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        
        let base_name = Path::new(&fname)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        loop {
            let path = Path::new("example/upload");
            let full_path = path.join(&file_name.clone());
            if !full_path.exists() {
                return full_path;
            }
            counter += 1;
            file_name = format!("{}_{}.{}", base_name, counter, ext);
        }
    }
}

impl Uploader {
    pub fn new(dir_path: &Path) -> Self {
        let list_files = list_files(dir_path);
        Uploader {
            database: match list_files {
                Ok(files) => files,
                Err(_) => Vec::new()
            }
        }
    }

    // pub fn addFile(&mut self) {

    // }

    // pub fn removeFile(&mut self) {

    // }

    pub fn list_json(&self) -> io::Result<Value> {
        let mut items_file = Vec::new();

        for file in &self.database {
            items_file.push(json!({
                "name": file.name,
                "path": format!("{}", file.path.to_string_lossy()),
                "size": file.size
            }));
        }

        Ok(json!({
            "files": items_file
        })) 
    }
}



fn list_files(dir_path: &Path) -> io::Result<Vec<File>> {
    let mut files= Vec::new();
    let prefix_dir = Path::new("example");
    let mut id = 0;

    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        files.push(File {
            id, 
            name: entry.file_name().into_string().unwrap(), 
            path: path,
            size: metadata.len() 
        });

        id += 1;
    }

    Ok(files)
}