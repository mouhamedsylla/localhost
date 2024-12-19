use std::{io, fs, io::Read};
use std::path::{Path, PathBuf};
use mime_guess::from_path;

pub type mime = String;

#[derive(Debug, Clone)]
pub struct ServerStaticFiles {
    directory: PathBuf,
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

        // list directory content
        // if directory.is_dir() {
        //     for entry in fs::read_dir(&directory)? {
        //         let entry = entry?;
        //         let path = entry.path();
        //         println!("{:?}", path);
        //     }
        // }

        Ok(ServerStaticFiles {
            directory,
            index,
            allow_directory_listing
        })
    }

    fn serve_file(&self, path: &Path) -> io::Result<(Vec<u8>, Option<mime>)> {
        if !path.is_file() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }
        
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        let mime = self.get_mime_type(path.to_str().unwrap());
        file.read_to_end(&mut buffer)?;

        Ok((buffer, Some(mime)))
    }

    pub fn handle_stactic_file_serve(&self, path: &str) -> io::Result<(Vec<u8>, Option<mime>)> {
        let path = path.trim_start_matches('/');
        let full_path = self.directory.join(path);

        if full_path.is_dir() {
            let message = "Directory listing is not allowed";
            return Ok((message.as_bytes().to_vec(), None));
        }

        self.serve_file(&full_path)
    }

    fn get_mime_type(&self, path: &str) -> mime {
    from_path(path).first_or_octet_stream().to_string()
    }
}