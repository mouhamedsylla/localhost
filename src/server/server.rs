use std::os::fd::AsRawFd;
use std::path::Path;
use std::time::{Instant, Duration};
use std::{collections::HashMap, os::unix::io::RawFd};
use crate::http::response;
use crate::http::{
    body::Body,
    status::HttpStatusCode,
    response::Response,
    header::{Header, HeaderName},
    request::{Request, HttpMethod},
};

use crate::server::{
    host::Host,
    route::Route,
    uploader::Uploader,
    errors::ServerError,
    connection::{Connection, ConnectionState},
    logger::{Logger, LogLevel},
};

use crate::server::stream::request_stream::unifiedReader::UnifiedReader;
use crate::server::session::session::SessionMiddleware;

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, 
    EPOLLET, EPOLLIN, EPOLLHUP, EPOLLERR,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL,
};

use serde_json::json;

const EPOLL_EVENTS: u32 = (EPOLLIN | EPOLLET) as u32;
const TIMEOUT_DURATION: Duration = Duration::from_secs(60);
const MAX_EVENTS: usize = 1024;

pub struct Server {
    hosts: Vec<Host>,
    connections: HashMap<RawFd, Connection>,
    epoll_fd: RawFd,
    logger: Logger,
    uploader: Option<Uploader>,
    session_middleware: SessionMiddleware,
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
            uploader,
            session_middleware: SessionMiddleware{},
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

        let reader = UnifiedReader::new(stream);

        let connection = Connection::new(client_fd, host.server_name.clone(), Box::new(reader));
        self.logger.debug(&format!("New connection on host: {} - {}", host.server_name, listener.port), "server");
        self.connections.insert(client_fd, connection);
        
        Ok(())
    }


    fn handle_connection_event(&mut self, fd: RawFd, events: u32, host: Host) -> Result<(), ServerError> {
        let connection = self.connections.get_mut(&fd)
            .ok_or(ServerError::ConnectionError("Connection not found".to_string()))?;
        let mut should_close = false;

    


        match connection.handle_event(events) {
            Ok(state) => {
                match state {
                    ConnectionState::Complete(request) => {
                        if let Some(route) = host.get_route(&request.uri) {
                            // precess middleware session
                            if let Some(session_manager) = host.session_manager.as_ref() {
                                match self.session_middleware.process(&request, route, session_manager) {
                                    Ok(session) => {
                                        if let Some(s) = session {
                                        }
                                    },
                                    Err(response) => {
                                        if let Err(e) = connection.send_response(response.to_string()) {
                                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                                self.logger.error(&format!("Failed to send response: {}", e), "Server");
                                                should_close = true;
                                            }
                                        }
                                        return Ok(());
                                    }
                                }
                            }

                            match host.route_request(&request, route, self.uploader.clone()) {
                                Ok(mut response) => {
                                    // Add CORS and Connection headers
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

                                    if let Err(e) = connection.send_response(response.clone().to_string()) {
                                        if e.kind() != std::io::ErrorKind::WouldBlock {
                                            self.logger.error(&format!("Failed to send response: {}", e), "Server");
                                            should_close = true;
                                        }
                                    }

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
                        } else {
                            let response = Response::not_found("Route not found");
                            if let Err(e) = connection.send_response(response.to_string()) {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    self.logger.error(&format!("Failed to send response: {}", e), "Server");
                                    should_close = true;
                                }
                            }
                            self.logger.warn(&format!("Route not found: {}", request.uri), "Server");
                        }
                        connection.start_time = Instant::now();
                        connection.keep_alive = want_keep_alive(request);
                        should_close = !connection.keep_alive;
                    },

                    ConnectionState::AwaitingRequest => {},
                    ConnectionState::Error(error) => {
                        self.logger.error(&error, "Server");
                        should_close = true;
                    }
                }
            }
            Err(e) => {
                // Only treat non-WouldBlock errors as actual errors
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    self.logger.error(&format!("Connection error: {}", e), "Server");
                    should_close = true;
                }
            }
        }
        if should_close {
            self.close_connection(fd)?;
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
                let fd = event.u64 as RawFd;

                if let Some(_) = self.find_host_by_fd(fd) {
                    if let Err(e) = self.handle_new_connection(fd) {
                        self.logger.error(&format!("New connection error: {:?}", e), "Server");
                    }
                } else {
                    let host = self.get_host_by_name(&self.connections.get(&fd).unwrap().host_name)
                        .ok_or_else(|| ServerError::ConnectionError("Host not found".to_string()))?;
                    if let Err(e) = self.handle_connection_event(fd, event.events, host.clone()) {
                        self.logger.error(&format!("Connection event error: {:?}", e), "Server");
                    }
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