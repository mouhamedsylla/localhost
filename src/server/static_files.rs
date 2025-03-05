use std::{
    collections::HashMap, fs, io::{self, Read}, path::{Path, PathBuf}, env
};
use mime_guess::from_path;
use serde_json::{json, Value};
use crate::server::errors::ServerError;

// sites directory prefix

pub fn sites_dir() -> String {
    format!("{}/.cargo/localhost-cli/sites", env!("HOME"))
}

/// Type alias for MIME type strings
pub type mime = String;

/// Enum for file status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Ok,
    NotFound,
    DirectoryListingNotAllowed,
    Raw
}

#[derive(Debug, Clone)]
pub struct ErrorPages {
    pub custom_pages: HashMap<String, String>,
}

/// Configuration for serving static files
#[derive(Debug, Clone)]
pub struct ServerStaticFiles {
    pub directory: PathBuf,
    pub index: Option<String>,
    pub allow_directory_listing: bool,
    pub error_pages: Option<ErrorPages>,
    pub status: FileStatus,
}

/// Core implementation
impl ServerStaticFiles {
    pub fn new(
        directory: PathBuf,
        index: Option<String>,
        allow_directory_listing: bool,
        error_pages: Option<ErrorPages>,
    ) -> Result<Self, ServerError> {
        let binding = sites_dir();
        let dir_prefix = Path::new(&binding);
        let directory = dir_prefix.join(directory);
        // Validate directory exists
        if !directory.exists() {
            return Err(ServerError::FileNotFound(directory.clone()));
        }

        // Create default directory if missing
        let default_dir = directory.join(".default");

        if !default_dir.exists() {
            fs::create_dir(&default_dir).map_err(ServerError::from)?;
        }

        let value = env::var("LOCALHOST_RESOURCES").unwrap_or_else(|_| "src/.default".to_string());
        let src = std::path::PathBuf::from(value);
        copy_default_dir(&src, &default_dir).map_err(|e| 
            ServerError::DirectoryListingError(format!("Failed to copy default directory: {}", e))
        )?;

        Ok(ServerStaticFiles {
            directory,
            index,
            allow_directory_listing,
            error_pages,
            status: FileStatus::Raw,
        })
    }

    pub fn serve_static(&mut self, path: &str) -> Result<(Vec<u8>, Option<mime>, FileStatus), ServerError> {
        let defaultPath = self.directory.join(".default/index.html");


        let path = path.trim_start_matches('/');
        let full_path = self.directory.join(path);


        if full_path.is_dir() {
            if self.allow_directory_listing {
                return self.serve_directory(&full_path);
            }
        }

        if let Some(index) = &self.index  {
            let index_path = full_path.join(index);
            if index_path.is_file() && !self.allow_directory_listing {
                return self.serve_file(&index_path);
            }
        }

        if self.index.is_none() && full_path == self.directory {
            return self.serve_file(&defaultPath);
        }

        self.serve_file(&full_path)
    }

    pub fn is_directory_contain_file(&self, path: &Path) -> bool {
        // if self.directory.join(path).exists() {
        //     return true;
        // }
        self.directory.join(path).is_file()
    }
}

/// File serving implementation
impl ServerStaticFiles {
    /// Serves a static file
    pub fn serve_file(&mut self, path: &Path) -> Result<(Vec<u8>, Option<mime>, FileStatus), ServerError> {
        if !path.is_file() {
            self.set_status(FileStatus::NotFound);
            if let Some(error_page) = self.error_pages.clone() {
                if let Some(page) = error_page.custom_pages.get("404") {
                    let error_page = Path::new(page);
                    return self.serve_file(error_page);
                }
            }
            
            let error_page = self.directory.join(".default/error/error_template.html");            
            return self.serve_file(&error_page);    
        }

        let mut file = fs::File::open(path)
            .map_err(|e| ServerError::FileNotFound(path.to_path_buf()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(ServerError::from)?;
        
        let mime = self.get_mime_type(path);
        Ok((buffer, Some(mime), self.status.clone()))
    }

    /// Gets MIME type for a file path
    fn get_mime_type(&self, path: &Path) -> mime {
        from_path(path).first_or_octet_stream().to_string()
    }
}

/// Directory handling implementation
impl ServerStaticFiles {
    /// Serves a directory listing
    fn serve_directory(&mut self, path: &Path) -> Result<(Vec<u8>, Option<mime>, FileStatus), ServerError> {
        self.write_directory_data(path)?;

        let serve_dir_html = self
            .directory
            .join(".default")
            .join("directory_listing.html");

        self.serve_file(&serve_dir_html)
    }

    /// Generates directory listing data
    fn generate_directory_data(&self, dir_path: &Path) -> Result<Value, ServerError> {
        let mut items = Vec::new();
        
        for entry in fs::read_dir(dir_path).map_err(|e| {
            ServerError::DirectoryListingError(format!("Failed to read directory {}: {}", 
                dir_path.display(), e))
        })? {
            let entry = entry.map_err(ServerError::from)?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(ServerError::from)?;
            
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
    fn write_directory_data(&self, path: &Path) -> Result<(), ServerError> {
        let mut structure = std::collections::HashMap::new();

        // Generate data for current directory
        let current_dir_data = self.generate_directory_data(path)?;
        structure.insert(
            current_dir_data["path"].as_str().unwrap_or("/").to_string(),
            current_dir_data,
        );

        // Generate data for subdirectories
        for entry in fs::read_dir(path).map_err(|e| {
            ServerError::DirectoryListingError(format!("Failed to read directory {}: {}", 
                path.display(), e))
        })? {
            let entry = entry.map_err(ServerError::from)?;
            if entry.metadata().map_err(ServerError::from)?.is_dir() {
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
            serde_json::to_string_pretty(&structure).map_err(|e| 
                ServerError::DirectoryListingError(format!("Failed to serialize directory data: {}", e))
            )?
        );

        let data_js_path = self.directory.join(".default/js/directory/data.js");
        if data_js_path.exists() {
            fs::remove_file(&data_js_path).map_err(|e| 
                ServerError::DirectoryListingError(format!("Failed to remove old data file: {}", e))
            )?;
        }

        // Write to file
        let data_js_path = self.directory.join(".default/js/directory/data.js");
        fs::write(data_js_path, js_content).map_err(|e| 
            ServerError::DirectoryListingError(format!("Failed to write directory data: {}", e))
        )?;

        Ok(())
    }

    fn set_status(&mut self, status: FileStatus) {
        self.status = status;
    }
}

pub fn copy_default_dir(src: &Path, dst: &Path) -> Result<(), io::Error> {
    if !dst.exists() {
        fs::create_dir(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_default_dir(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

