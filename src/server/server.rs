use std::os::fd::AsRawFd;
use std::path::Path;
use std::time::{Instant, Duration};
use std::{collections::HashMap, os::unix::io::RawFd};
use crate::http::{
    body::Body,
    status::HttpStatusCode,
    header::{Header, HeaderName},
    request::{Request, HttpMethod},
};

use crate::server::{
    host::Host,
    route::Route,
    uploader::Uploader,
    errors::ServerError,
    connection::Connection,
    logger::{Logger, LogLevel},
};

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, 
    EPOLLET, EPOLLIN, EPOLLHUP, EPOLLERR,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL,
};

use serde_json::json;

const EPOLL_EVENTS: u32 = (EPOLLIN | EPOLLET) as u32;
const TIMEOUT_DURATION: Duration = Duration::from_secs(60);
const MAX_EVENTS: usize = 1024;

#[derive(Debug)]
pub struct Server {
    hosts: Vec<Host>,
    connections: HashMap<RawFd, Connection>,
    epoll_fd: RawFd,
    logger: Logger,
    uploader: Option<Uploader>
}

impl Server {
    pub fn new(uploader: Option<Uploader>) -> Result<Self, ServerError> {
        let epoll_fd = Self::create_epoll()?;
        let logger = Logger::new(LogLevel::DEBUG);

        Ok(Server {
            hosts: Vec::new(),
            connections: HashMap::new(),
            epoll_fd,
            logger,
            uploader
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
        // Find host
        let host = self.find_host_by_fd(fd)
            .ok_or_else(|| {
                self.logger.error(&format!("Host not found for fd: {}", fd), "Server");
                ServerError::ConnectionError("Host not found".to_string())
            })?;

        // Get listener
        let listener = host.get_listener(fd)
            .ok_or_else(|| {
                self.logger.error(&format!("Listener not found for fd: {}", fd), "Server");
                ServerError::ConnectionError("Listener not found".to_string())
            })?;

            
            // Accept connection
        let stream = match listener.accept_connection() {
            Ok(s) => s,
            Err(e) => {
                self.logger.error(&format!("Failed to accept connection: {}", e), "Server");
                return Err(ServerError::ConnectionError(e.to_string()));
            }
        };

        // Set non-blocking
        if let Err(e) = stream.set_nonblocking(true) {
            self.logger.error(&format!("Failed to set non-blocking: {}", e), "Server");
            return Err(ServerError::ConnectionError(e.to_string()));
        }

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
        self.logger.debug(&format!("New connection on host: {} - {}", host.server_name, listener.port), "server");
        self.connections.insert(client_fd, connection);
        
        Ok(())
    }

    fn handle_request(&mut self, client_fd: RawFd, host: Host) -> Result<(), ServerError> {
        let connection = self.connections.get_mut(&client_fd).unwrap();
        let mut should_close = false;

        match connection.read_request() {
            Ok(request) => {
                if let Some(route) = host.get_route(&request.uri) {
                    match host.route_request(&request, route, self.uploader.clone()) {
                        Ok(mut response) => {
                            // Ajouter les headers CORS et Connection
                            let connection_header = if connection.keep_alive && want_keep_alive(request.clone()) {
                                "keep-alive"
                            } else {
                                "close"
                            };
                            response.headers.extend(vec![
                                Header::from_str("Connection", connection_header),
                                Header::from_str("Access-Control-Allow-Origin", "*"),
                                Header::from_str("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS"),
                                Header::from_str("Access-Control-Allow-Headers", "Content-Type"),
                            ]);

                            connection.send_response(response.clone().to_string());
                            let message = format!("{} - {} - {}", 
                                request.method,
                                &request.uri, 
                                response.status_code.as_str()
                            );
                            self.logger.info(&message, "Server");
                        },
                        Err(error) => {
                            self.logger.error(&error.to_string(), "Server");
                        }
                    }
                }
                connection.start_time = Instant::now();
                connection.keep_alive = want_keep_alive(request);
                should_close = !connection.keep_alive;
            },
            Err(error) => should_close = true
        }

        if should_close {
            self.close_connection(client_fd)?;
        }

        Ok(())
    }


    fn close_connection(&mut self, client_fd: RawFd) -> Result<(), ServerError> {
        unsafe {
            if epoll_ctl(self.epoll_fd, EPOLL_CTL_DEL, client_fd, std::ptr::null_mut()) < 0 {
                self.logger.error(&format!(
                    "Failed to remove client {} from epoll", client_fd
                ), "Server");
                return Err(ServerError::EpollError("Failed to remove client from epoll"));
            }
        }

        if let Some(connection) = self.connections.remove(&client_fd) {
            self.logger.info(&format!(
                "Connection closed - Host: {} Client fd: {}", 
                connection.host_name, client_fd
            ), "Server");
        }

        Ok(())
    }

    fn handle_event(&mut self, event: epoll_event) -> Result<(), ServerError> {
        let fd = event.u64 as RawFd;

        if let Some(_) = self.find_host_by_fd(fd) {
            return self.handle_new_connection(fd);
        }

        if event.events & EPOLLIN as u32 != 0 {
            let host = self.get_host_by_name(&self.connections.get(&fd).unwrap().host_name)
                .ok_or_else(|| ServerError::ConnectionError("Host not found".to_string()))?;
            
            return self.handle_request(fd, host.clone());
        }

        Ok(())
    }

    fn cleanup_timeouts(&mut self) -> Result<(), ServerError> {
        let timed_out: Vec<RawFd> = self
            .connections
            .iter()
            .filter(|(_, conn)| {
                let is_timeout = Instant::now().duration_since(conn.start_time) > TIMEOUT_DURATION;
                if is_timeout {
                    self.logger.warn(&format!(
                        "Connection timeout - Host: {} Client fd: {}", 
                        conn.host_name, conn.client_fd
                    ), "Server");
                }
                is_timeout
            })
            .map(|(fd, _)| *fd)
            .collect();

        for fd in timed_out {
            self.close_connection(fd)?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        self.logger.info("Starting server...", "Server");
        let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

        loop {
            let num_events = unsafe {
                epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    MAX_EVENTS as i32,
                    1000 // Poll every second for timeouts
                )
            };

            if num_events < 0 {
                self.logger.error("Failed to wait for events", "Server");
                return Err(ServerError::EpollError("Failed to wait for events"));
            }

            // Handle events
            for event in &events[..num_events as usize] {
                if let Err(e) = self.handle_event(*event) {
                    self.logger.error(&format!("Event handling error: {:?}", e), "Server");
                }
            }

            // Cleanup timeouts
            if let Err(e) = self.cleanup_timeouts() {
                self.logger.error(&format!("Timeout cleanup error: {:?}", e), "Server");
            }
        }
    }
}

fn want_keep_alive(request: Request) -> bool {
    match request.get_header(HeaderName::Connection) {
        Some(header) => header.value.value.to_lowercase() == "keep-alive",
        None => true
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


impl Drop for Server {
    fn drop(&mut self) {
        // Clean up epoll file descriptor
        unsafe {
            libc::close(self.epoll_fd);
        }
    }
}