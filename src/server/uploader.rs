use serde_json::{json, Value};
use std::{
    fs::{self, read_dir},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct Uploader {
    database: Vec<File>,
    upload_dir: PathBuf,
}

impl Uploader {
    pub fn new(upload_dir: PathBuf) -> Self {
        let list_files = list_files(&upload_dir);
        Uploader { 
            database: match list_files {
                Ok(files) => files,
                Err(_) => Vec::new()
            }, 
            upload_dir 
        }
    }

    // Core business logic methods
    pub fn add_file(&mut self, name: String, data: &[u8]) -> io::Result<File> {
        self.sync_database()?;
        let clean_name = name.trim_matches('"').to_string();
        
        let file_path = self.generate_unique_path(&clean_name);
        
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&file_path, data)?;

        let new_file = File {
            id: self.generate_next_id(),
            name: clean_name,
            size: data.len() as u64,
            path: file_path,
        };

        self.database.push(new_file.clone());

        Ok(new_file)
    }

    pub fn delete_file(&mut self, file_id: i32) -> io::Result<File> {
        self.sync_database()?;
        let file_index = self.database.iter()
            .position(|f| f.id == file_id)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::NotFound,
                "File not found"
            ))?;

        let file = self.database[file_index].clone();
        fs::remove_file(&file.path)?;
        self.database.remove(file_index);
        
        Ok(file)
    }

    pub fn list_files(&self) -> Vec<&File> {
        self.database.iter().collect()
    }

    pub fn sync_database(&mut self) -> io::Result<()> {
        self.database.retain(|file| file.path.exists());

        if self.upload_dir.exists() {
            for entry in fs::read_dir(&self.upload_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if !self.database.iter().any(|f| f.path == path) {
                    let metadata = entry.metadata()?;
                    self.database.push(File {
                        id: self.generate_next_id(),
                        name: entry.file_name().to_string_lossy().into_owned(),
                        path,
                        size: metadata.len(),
                    });
                }
            }
        }
        Ok(())
    }

    // File validation methods
    pub fn is_allowed_mime_type(&self, mime_type: &str) -> bool {
        const ALLOWED_TYPES: [&str; 8] = [
            "text/", "image/", "application/pdf", "application/json",
            "application/msword", "application/vnd.openxmlformats-officedocument",
            "audio/", "video/"
        ];
        ALLOWED_TYPES.iter().any(|&allowed| mime_type.starts_with(allowed))
    }

    pub fn max_file_size(&self) -> usize {
        10 * 1024 * 1024 // 10MB
    }

    // Utility methods
    fn generate_next_id(&self) -> i32 {
        self.database.iter().map(|f| f.id).max().unwrap_or(-1) + 1
    }

    fn generate_unique_path(&self, original_name: &str) -> PathBuf {
        let mut counter = 0;
        let ext = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let base_name = Path::new(original_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        loop {
            let filename = if counter == 0 {
                format!("{}.{}", base_name, ext)
            } else {
                format!("{}_{}.{}", base_name, counter, ext)
            };
            
            let full_path = self.upload_dir.join(&filename);
            if !full_path.exists() {
                return full_path;
            }
            counter += 1;
        }
    }


}

fn list_files(dir_path: &Path) -> io::Result<Vec<File>> {
    let mut files = Vec::new();
    let mut id = 0;

    if dir_path.exists() {
        for entry in read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            let name = entry.file_name()
                .into_string()
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();

            files.push(File {
                id,
                name,
                path,
                size: metadata.len(),
            });

            id += 1;
        }
    }

    Ok(files)
}