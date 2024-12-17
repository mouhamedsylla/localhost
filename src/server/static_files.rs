use std::path::PathBuf;

pub struct StaticFiles {
    directory: PathBuf,
    index: String,
    allow_directory_listing: bool
}