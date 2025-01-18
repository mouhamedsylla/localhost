use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Instant, Duration};
use std::io::{self, Read, Write, Error};
use crate::http::header;
use crate::http::{
    request::Request,
    header::{HeaderName, HeaderParsedValue},
    request::parse_request
};

use libc::{
    epoll_event,
    EPOLLET, EPOLLIN, EPOLLHUP, EPOLLERR,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL,
};

use crate::server::stream::request_stream::{
    RequestStream,
    RequestState,
    RequestData,
};

#[derive(Debug, Clone)]
pub enum ConnectionState {
    AwaitingRequest,
    Complete(Request),
    Error(String),
}

pub struct Connection {
    pub client_fd: RawFd,
    pub host_name: String,
    pub keep_alive: bool,
    pub reader: Box<dyn RequestStream>,
    pub state: ConnectionState,
    pub start_time: std::time::Instant,
}

impl Connection {
    pub fn new(client_fd: RawFd, host_name: String, reader: Box<dyn RequestStream>) -> Self {
        
        Connection {
            client_fd,
            host_name,
            keep_alive: true,
            reader,
            state: ConnectionState::AwaitingRequest,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: u32) -> io::Result<ConnectionState> {
        if event & EPOLLIN as u32 != 0 {
            match self.reader.read_next() {
                Ok(request_state) => {
                    match request_state {
                        RequestState::Complete(data) => {
                            match self.process_complete_request(data) {
                                Ok(request) => {
                                    
                                    self.state = ConnectionState::Complete(request);
                                    Ok(self.state.clone())
                                }
                                Err(e) => {
                                    self.state = ConnectionState::Error(e.to_string());
                                    Ok(self.state.clone())
                                }
                            }
                        }
                        RequestState::ProcessingBody {..} => {
                            Ok(self.state.clone())
                        }
                        RequestState::AwaitingHeaders => {
                            Ok(ConnectionState::AwaitingRequest)
                        }
                        RequestState::EndOfStream => {
                            self.state = ConnectionState::Error("End of stream".to_string());
                            Ok(self.state.clone())
                        }
                    }
                }
                Err(e ) => {
                    self.state = ConnectionState::Error(e.to_string());
                    Ok(self.state.clone())
                }
            }
        } else {
            Ok(self.state.clone())
        }
    }

    fn process_complete_request(&mut self, data: RequestData) -> io::Result<Request> {
        match parse_request(&data.data) {
            Some(request) => {
                self.reset();
                Ok(request)
            },
            None => Err(io::Error::new(io::ErrorKind::InvalidData, "failed parsed request"))
        }
    }

    pub fn reset(&mut self) {
        self.reader.reset();
        self.state = ConnectionState::AwaitingRequest;
        self.start_time = Instant::now();
    }

    pub fn send_response(&mut self, response: String) -> std::io::Result<()> {
        if let Err(e) = self.reader.write(response.as_bytes()) {
            println!("erreur to write: {}", e);
        };
        self.reader.flush()
    }
}

fn want_keep_alive(request: Request) -> bool {
    match request.get_header(HeaderName::Connection) {
        Some(header) => header.value.value.to_lowercase() == "keep-alive",
        None => true
    }
}