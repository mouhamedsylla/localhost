#[derive(Debug)]
pub enum ServerError {
    IoError(std::io::Error),
    EpollError(&'static str),
    ConnectionError(String),
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        ServerError::IoError(error)
    }
}

impl std::fmt::Display for ServerError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::IoError(e) => write!(f, "IO Error: {}", e),
            ServerError::EpollError(e) => write!(f, "Epoll Error: {}", e),
            ServerError::ConnectionError(e) => write!(f, "Connection Error: {}", e),
        }
    }
}