use std::{
    path::{Path, PathBuf},
    fs, io,
    io::Read
};
use mime_guess::from_path;
use serde_json::{json, Value};

/// Type alias for MIME type strings
pub type mime = String;

/// Configuration for serving static files
#[derive(Debug, Clone)]
pub struct ServerStaticFiles {
    pub directory: PathBuf,
    index: String,
    allow_directory_listing: bool,
}

/// Core implementation
impl ServerStaticFiles {
    pub fn new(
        directory: PathBuf,
        index: String,
        allow_directory_listing: bool,
    ) -> io::Result<Self> {
        // Validate directory exists
        if !directory.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Directory not found",
            ));
        }

        // Validate index file if specified
        if !index.is_empty() {
            if !directory.join(&index).exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Index file not found",
                ));
            }
        }

        // Create default directory if missing
        let default_dir = directory.join(".default");
        if !default_dir.exists() {
            fs::create_dir(&default_dir).unwrap();
        }

        let src = std::path::PathBuf::from("src/.default");
        copy_default_dir(&src, &default_dir);

        Ok(ServerStaticFiles {
            directory,
            index,
            allow_directory_listing,
        })
    }

    pub fn serve_static(&mut self, path: &str) -> io::Result<(Vec<u8>, Option<mime>)> {
        let path = path.trim_start_matches('/');
        let full_path = self.directory.join(path);

        if full_path.is_dir() {
            if self.allow_directory_listing {
                return self.serve_directory(&full_path);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Directory listing not allowed",
                ));
            }
        }

        self.serve_file(&full_path)
    }
}

/// File serving implementation
impl ServerStaticFiles {
    /// Serves a static file
    pub fn serve_file(&self, path: &Path) -> io::Result<(Vec<u8>, Option<mime>)> {
        if !path.is_file() {
            if path.is_dir() {
                return self.serve_directory(path);
            } else {
                let error_page = self.directory.join(".default/error/error_template.html");
                return self.serve_file(&error_page);
            }
        }

        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let mime = self.get_mime_type(path);
        Ok((buffer, Some(mime)))
    }

    /// Gets MIME type for a file path
    fn get_mime_type(&self, path: &Path) -> mime {
        from_path(path).first_or_octet_stream().to_string()
    }
}

/// Directory handling implementation
impl ServerStaticFiles {
    /// Serves a directory listing
    fn serve_directory(&self, path: &Path) -> io::Result<(Vec<u8>, Option<mime>)> {
        self.write_directory_data(path)?;

        let serve_dir_html = self
            .directory
            .join(".default")
            .join("directory_listing.html");

        self.serve_file(&serve_dir_html)
    }

    /// Generates directory listing data
    fn generate_directory_data(&self, dir_path: &Path) -> io::Result<Value> {
        let mut items = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            
            items.push(json!({
                "name": entry.file_name().to_string_lossy(),
                "type": if metadata.is_dir() { "directory" } else { "file" },
                "size": metadata.len(),
                "path": format!("/{}", path.strip_prefix(&self.directory)
                    .unwrap_or(&path)
                    .to_string_lossy())
            }));
        }

        Ok(json!({
            "path": format!("/{}", dir_path.strip_prefix(&self.directory)
                .unwrap_or(dir_path)
                .to_string_lossy()),
            "items": items
        }))
    }

    /// Writes directory listing data to a file
    fn write_directory_data(&self, path: &Path) -> io::Result<()> {
        let mut structure = std::collections::HashMap::new();

        // Generate data for current directory
        let current_dir_data = self.generate_directory_data(path)?;
        structure.insert(
            current_dir_data["path"].as_str().unwrap_or("/").to_string(),
            current_dir_data,
        );

        // Generate data for subdirectories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                let subdir_data = self.generate_directory_data(&entry.path())?;
                structure.insert(
                    subdir_data["path"].as_str().unwrap_or("/").to_string(),
                    subdir_data,
                );
            }
        }

        // Create data.js content
        let js_content = format!(
            "// Generated directory structure\nexport const directoryData = {};",
            serde_json::to_string_pretty(&structure)?
        );

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

