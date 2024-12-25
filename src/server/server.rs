use std::os::fd::AsRawFd;
use std::{collections::HashMap, os::unix::io::RawFd};
use std::io::Write;
use crate::http::body::Body;
use crate::http::response::ResponseBuilder;
use crate::server::static_files::ServerStaticFiles;
use crate::http::request::HttpMethod;
use crate::http::header::Header;
use crate::http::request::Request;
use crate::server::connection::Connection;
use crate::server::host::Host;

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, 
    EPOLLET, EPOLLIN, EPOLLHUP, EPOLLERR,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL,
};


const EPOLL_EVENTS: u32 = (EPOLLIN | EPOLLET) as u32;
const MAX_EVENTS: usize = 64;

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

/// Server structure managing multiple hosts and connections
#[derive(Debug)]
pub struct Server {
    hosts: Vec<Host>,
    connections: HashMap<RawFd, Connection>,
    epoll_fd: RawFd,
}

/// Implementation of server creation and initialization
impl Server {
    pub fn new() -> Result<Self, ServerError> {
        let epoll_fd = Self::create_epoll()?;

        Ok(Server {
            hosts: Vec::new(),
            connections: HashMap::new(),
            epoll_fd
        })
    }

    fn create_epoll() -> Result<RawFd, ServerError> {
        let epoll_fd = unsafe { epoll_create1(0) };

        if epoll_fd < 0 {
            return Err(ServerError::EpollError("Failed to create epoll file descriptor"));
        }

        Ok(epoll_fd)
    }
}

/// Host management implementation
impl Server {
    pub fn add_host(&mut self, host: Host) -> Result<(), ServerError> {
        self.register_host_with_epoll(&host)?;
        println!("Added host: {} with ports {:?}", host.server_name, host.listeners);
    
        self.hosts.push(host);
        Ok(())
    }

    fn register_host_with_epoll(&self, host: &Host) -> Result<(), ServerError> {

        for listener in &host.listeners {
            let mut event = epoll_event {
                events: EPOLL_EVENTS,
                u64: listener.fd as u64
            };

            unsafe {
                if epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, listener.fd, &mut event) < 0 {
                    return Err(ServerError::EpollError("Failed to add listener to epoll"));
                }
            }
        }

        Ok(())
    }

    fn handle_new_connection(&mut self, fd: RawFd) -> Result<(), ServerError> {
        let host = self.find_host_by_fd(fd)
            .ok_or_else(|| ServerError::ConnectionError("Host not found".to_string())).unwrap();

        let listener = host.get_listener(fd)
            .ok_or_else(|| ServerError::ConnectionError("Listener not found".to_string())).unwrap();
        
        let stream = listener.accept_connection().unwrap();
        let client_fd = stream.as_raw_fd();

        let mut event = epoll_event {
            events: (EPOLLIN | EPOLLET) as u32,
            u64: client_fd as u64
        };

        unsafe {
            if epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, client_fd, &mut event) < 0 {
                return Err(ServerError::EpollError("Failed to add client to epoll"));
            }
        }

        let connection = Connection::new(stream, host.server_name.clone());
        self.connections.insert(client_fd, connection);
        Ok(())
    }


    fn handle_request(&mut self, client_fd: RawFd, host: Host) -> Result<(), ServerError> {
        let connection = self.connections.get_mut(&client_fd).unwrap();

        println!("Handling request for host: {}", host.server_name);

        match connection.read_request()? {
            Some(buffer) if !buffer.is_empty() => {
                let request_str = String::from_utf8_lossy(&buffer);
                println!("Received request: {}", request_str);
                if let Some(request) = crate::http::request::parse_request(&request_str) {
                    println!("URI: {}", request.uri);
                    if let Some(route) = host.get_route(&request.uri) {
                        println!("Handling route: {:#?}", route);
                        if request.method == HttpMethod::GET {
                            println!("Handling GET request: {}", request.uri);
                            
                            if let Some(mut static_files) = route.static_files.clone() {
                                println!("Handling static file request: {}", request.uri);
                                handle_static_file_request(&mut static_files, request, connection)?;
                            }
                        }
                    }
                }
            },
            Some(_) => (),
            None => self.close_connection(client_fd)?
        }

        unsafe {
            epoll_ctl(self.epoll_fd, EPOLL_CTL_DEL, client_fd, std::ptr::null_mut());
        }
        self.connections.remove(&client_fd);
        Ok(())
    }

    fn close_connection(&mut self, client_fd: RawFd) -> Result<(), ServerError> {
        unsafe {
            if epoll_ctl(self.epoll_fd, EPOLL_CTL_DEL, client_fd, std::ptr::null_mut()) < 0 {
                return Err(ServerError::EpollError("Failed to remove client from epoll"));
            }
        }
        self.connections.remove(&client_fd);
        Ok(())
    }

    fn handle_event(&mut self, event: epoll_event) -> Result<(), ServerError> {
        let fd = event.u64 as RawFd;

        if (event.events & (EPOLLHUP | EPOLLERR) as u32) != 0 {
            return self.close_connection(fd);
        }

        if let Some(_) = self.find_host_by_fd(fd) {
            return self.handle_new_connection(fd);
        }

        if let Some(connection) = self.connections.get(&fd) {
            let host = self.get_host_by_name(&connection.host_name).unwrap();
            return self.handle_request(fd, host.clone());
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        println!("Starting server with {} hosts", self.hosts.len());

        let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

        loop {
            let num_events = unsafe {
                epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    MAX_EVENTS as i32,
                    -1
                )
            };

            if num_events < 0 {
                return Err(ServerError::EpollError("Failed to wait for events"));
            }

            for event in &events[..num_events as usize] {
                if let Err(e) = self.handle_event(*event) {
                    eprintln!("Error handling event: {:?}", e);
                }
            }
        }
    }
}

/// Host lookup implementation
impl Server {
    pub fn get_host_by_name(&self, name: &str) -> Option<&Host> {
        self.hosts.iter().find(|&host| host.server_name == name)
    }


    fn find_host_by_fd(&self, fd: RawFd) -> Option<&Host> {
        self.hosts.iter().find(|&host| host.match_listener(fd))
    }
}

fn handle_static_file_request(static_files: &mut ServerStaticFiles, request: Request, connection: &mut Connection)
-> Result<(), ServerError> 
{
   match static_files.serve_static(&request.uri) {
       Ok((content, mime)) => {
           let mime_str = mime.map_or_else(|| "text/plain".to_string(), |m| m.to_string());
           let content_type = Header::from_mime(&mime_str);

           let body = Body::from_mime(&mime_str, content);
           let response_builder = ResponseBuilder::new;
       
           match body {
               Ok(body) => {
                   let response = response_builder().body(body).header(content_type).build().to_string();
                   connection.send_response(response)?;
               },
               Err(e) => {
                   let response = response_builder()
                        .body(Body::text(&e.to_string()))
                        .header(Header::from_mime("text/plain"))
                        .build()
                        .to_string();
                   connection.send_response(response)?;
               }
           }

           connection.stream.flush()?;
           Ok(())
       },
       Err(e) => Err(ServerError::ConnectionError(format!("Static file error: {}", e))),
   }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Clean up epoll file descriptor
        unsafe {
            libc::close(self.epoll_fd);
        }
    }
}
