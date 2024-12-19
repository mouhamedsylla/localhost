use std::default;
use std::{io, fs, io::Read};
use std::path::{Path, PathBuf};
use libc::NFT_CT_DST_IP;
use mime_guess::from_path;
use serde_json::{Value, json};

pub type mime = String;

#[derive(Debug, Clone)]
pub struct ServerStaticFiles {
    pub directory: PathBuf,
    index: String,
    allow_directory_listing: bool
}

impl ServerStaticFiles {
    pub fn new(directory: PathBuf, index: String, allow_directory_listing: bool) -> io::Result<Self> {
        if !directory.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Directory not found"));
        }

        if !index.is_empty() {
            if !directory.join(&index).exists() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Index file not found"));
            }
        }

        let default_dir = directory.join(".default");
        if (!default_dir.exists()) {
            fs::create_dir(&default_dir).unwrap();
        }

        let src = std::path::PathBuf::from("src/.default");

        copy_default_dir(&src, &default_dir);

        Ok(ServerStaticFiles {
            directory,
            index,
            allow_directory_listing
        })
    }

    fn serve_file(&mut self, path: &Path) -> io::Result<(Vec<u8>, Option<mime>)> {
        if !path.is_file() {
            println!("File not found: {:?}", path);
            if path.is_dir() {
                return  self.serve_directory(path);
            } else {
                let error_page = self.directory.join(".default/error/error_template.html");
                return  self.serve_file(&error_page)
            }
        }
        
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        let mime = self.get_mime_type(path.to_str().unwrap());
        file.read_to_end(&mut buffer)?;

        Ok((buffer, Some(mime)))
    }

    fn serve_directory(&mut self, path: &Path) -> io::Result<(Vec<u8>, Option<mime>)> {
        println!("Ok est on bon  : {:?}", path);
        self.write_directory_data(path)?;
        
        let serve_dir_html = self.directory.join(".default").join("directory_listing.html");

        println!("Serving directory: {:?}", serve_dir_html);
        self.serve_file(&serve_dir_html)
    }

    pub fn handle_stactic_file_serve(&mut self, path: &str) -> io::Result<(Vec<u8>, Option<mime>)> {
        let path = path.trim_start_matches('/');
        let full_path = self.directory.join(path);

        if full_path.is_dir() {
            println!("Serving directory 2: {:?}", full_path);
            if self.allow_directory_listing {
                return self.serve_directory(&full_path);
            } else {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Directory listing not allowed"));
            }
        }

        self.serve_file(&full_path)
    }

    fn get_mime_type(&self, path: &str) -> mime {
        from_path(path).first_or_octet_stream().to_string()
    }

    fn generate_directory_data(&self, dir_path: &Path) -> io::Result<Value> {
        let mut items = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let relative_path = path.strip_prefix(&self.directory)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            
            let item = if metadata.is_dir() {
                json!({
                    "name": name,
                    "type": "directory",
                    "path": format!("{}", relative_path)
                })
            } else {
                json!({
                    "name": name,
                    "type": "file",
                    "size": metadata.len(),
                    "path": format!("/{}", relative_path)
                })
            };
            
            items.push(item);
        }

        Ok(json!({
            "path": format!("/{}", dir_path.strip_prefix(&self.directory)
                .unwrap_or(dir_path)
                .to_string_lossy()),
            "items": items
        }))
    }

    fn write_directory_data(&self, path: &Path) -> io::Result<()> {
        let mut structure = std::collections::HashMap::new();
        
        // Generate data for current directory
        let current_dir_data = self.generate_directory_data(path)?;
        structure.insert(
            current_dir_data["path"].as_str().unwrap_or("/").to_string(),
            current_dir_data
        );
        
        // Generate data for subdirectories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                let subdir_data = self.generate_directory_data(&entry.path())?;
                structure.insert(
                    subdir_data["path"].as_str().unwrap_or("/").to_string(),
                    subdir_data
                );
            }
        }

        // Create data.js content
        let js_content = format!(
            "// Generated directory structure\nexport const directoryData = {};",
            serde_json::to_string_pretty(&structure)?
        );

        // remove .default/js/directory/data.js if it exists
        let data_js_path = self.directory.join(".default/js/directory/data.js");
        if data_js_path.exists() {
            fs::remove_file(data_js_path)?;
        }

        // Write to file
        let data_js_path = self.directory.join(".default/js/directory/data.js");
        fs::write(data_js_path, js_content)?;

        Ok(())
    }
}

pub fn copy_default_dir(src: &Path, dst: &Path) {
    if !dst.exists() {
        fs::create_dir(dst).unwrap();
    }

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_default_dir(&src_path, &dst_path);
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}