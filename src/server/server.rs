use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::io::{Read, Write, ErrorKind};
use crate::http::body::Body;
use crate::http::response::ResponseBuilder;
use crate::server::static_files::ServerStaticFiles;
use crate::http::request::HttpMethod;
use crate::http::header::Header;

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, 
    EPOLLET, EPOLLIN, EPOLLHUP, EPOLLERR, EPOLLOUT,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL,
};

const MAX_EVENTS: usize = 64;

pub struct Connection {
    pub stream: TcpStream,
    pub client_fd: RawFd,
    pub host_name: String,
}

impl Clone for Host {
    fn clone(&self) -> Host {
        Host {
            port: self.port.clone(),
            server_name: self.server_name.clone(),
            listener: self.listener.try_clone().unwrap(),
            fd: self.fd,
            static_files: self.static_files.clone(),
        }
    }
}
impl Connection {
    pub fn new(stream: TcpStream, host_name: String) -> Connection {
        let client_fd = stream.as_raw_fd();
        Connection {
            stream,
            client_fd,
            host_name,
        }
    }
}

pub struct Host {
    pub port: String,
    pub server_name: String,
    pub listener: TcpListener,
    pub fd: RawFd,
    pub static_files: Option<ServerStaticFiles>,
}

impl Host {
    pub fn new(port: &str, server_name: &str, server_directory: Option<ServerStaticFiles>) -> Host {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        listener.set_nonblocking(true).unwrap();
        
        let fd = listener.as_raw_fd();

        Host {
            port: port.to_string(),
            server_name: server_name.to_string(),
            listener,
            fd,
            static_files: server_directory,
        }
    }
}

pub struct Server {
    pub hosts: Vec<Host>,
    pub connections: HashMap<RawFd, Connection>,
    pub epool_fd: RawFd,
}

impl Server {
    pub fn new() -> Server {
        let epool_fd = unsafe { epoll_create1(0) };
        
        if epool_fd < 0 {
            panic!("Failed to create epoll file descriptor");
        }

        Server {
            hosts: Vec::new(),
            connections: HashMap::new(),
            epool_fd,
        }
    }

    fn get_host_by_name(&self, name: &str) -> Option<&Host> {
        self.hosts.iter().find(|&host| host.server_name == name)
    }

    pub fn add_host(&mut self, host: Host) {
        // Enregistrer le listener de l'hôte dans l'epoll
        let mut event = libc::epoll_event {
            events: (EPOLLIN | EPOLLET) as u32,
            u64: host.fd as u64,
        };

        unsafe {
            if epoll_ctl(self.epool_fd, EPOLL_CTL_ADD, host.fd, &mut event) < 0 {
                panic!("Failed to add listener to epoll");
            }
        }

        self.hosts.push(host);
    }

    fn find_host_by_fd(&self, fd: RawFd) -> Option<&Host> {
        self.hosts.iter().find(|&host| host.fd == fd)
    }

    fn handle_new_connection(&mut self, fd: RawFd) -> std::io::Result<()> {
        let host = self.find_host_by_fd(fd).unwrap();
        match host.listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(true)?;
                
                let client_fd = stream.as_raw_fd();

                let mut event = libc::epoll_event {
                    events: (EPOLLIN | EPOLLET) as u32,
                    u64: client_fd as u64,
                };

                unsafe {
                    if epoll_ctl(self.epool_fd, EPOLL_CTL_ADD, client_fd, &mut event) < 0 {
                        eprintln!("Failed to add client to epoll");
                    }
                }

                let connection = Connection::new(stream, host.server_name.clone());
                self.connections.insert(client_fd, connection);
            },
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn handle_request(&mut self, client_fd: RawFd, host: Host) -> std::io::Result<()> {
        if let Some(connection) = self.connections.get_mut(&client_fd) {
            let mut buffer = [0; 1024];
            match connection.stream.read(&mut buffer) {
                Ok(0) => {
                    // Connection closed by client
                    unsafe {
                        epoll_ctl(self.epool_fd, EPOLL_CTL_DEL, client_fd, std::ptr::null_mut());
                    }
                    self.connections.remove(&client_fd);
                    return Ok(());
                }
                Ok(bytes_read) => {
                    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
                    if let Some(request) = crate::http::request::parse_request(&request_str) {
                        if let Some(mut static_files) = host.static_files {
                            if request.method == HttpMethod::GET {
                                

                                match static_files.handle_stactic_file_serve(&request.uri) {
                                    Ok(result) => {
                                        let (content, mime) = result;
                                        let mime_str = match mime {
                                            Some(mime) => mime.to_string(),
                                            None => "text/plain".to_string(),
                                        };
    
                                        let content_type = Header::from_mime(&mime_str);
    
                                        let body = Body::from_mime(&mime_str, content);
                                        let response_builder = ResponseBuilder::new();
                                        
                                        
                                        match body {
                                            Ok(body) => {
                                                let response = response_builder.body(body).header(content_type);
                                                connection.stream.write_all(response.build().to_string().as_bytes())?;
                                            },
                                            Err(e) => {
                                                let response = response_builder.body(Body::text(&e.to_string())).header(Header::from_mime("text/plain"));
                                                connection.stream.write_all(response.build().to_string().as_bytes())?;
                                            }
                                        }
    
                                        connection.stream.flush()?;
                                    },
                                    Err(e) => {
                                        eprintln!("Error handling static file: {}", e);
                                    }
                                }
                            }
                        }

                    }

                    // Fermer la connexion après l'envoi
                    unsafe {
                        epoll_ctl(self.epool_fd, EPOLL_CTL_DEL, client_fd, std::ptr::null_mut());
                    }
                    self.connections.remove(&client_fd);                    
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        println!("Démarrage du serveur avec {} hôtes", self.hosts.len());

        let mut events = vec![
            epoll_event {
                events: 0,
                u64: 0,
            }; 
            MAX_EVENTS
        ];

        loop {
            let num_events = unsafe {
                epoll_wait(
                    self.epool_fd, 
                    events.as_mut_ptr(), 
                    MAX_EVENTS as i32, 
                    -1
                )
            };

            if num_events < 0 {
                panic!("Failed to wait for events");
            }

            for i in 0..num_events as usize {
                let event = events[i];
                let fd = event.u64 as i32;

                if (event.events & (EPOLLHUP | EPOLLERR) as u32) != 0 {
                    // Handle error or hung up events
                    unsafe {
                        epoll_ctl(self.epool_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
                    }
                    self.connections.remove(&(fd as RawFd));
                    continue;
                }

                if let Some(_host) = self.find_host_by_fd(fd as RawFd) {
                    // New connection on listener socket
                    if let Err(e) = self.handle_new_connection(fd as RawFd) {
                        eprintln!("Error accepting connection: {}", e);
                        unsafe {
                            epoll_ctl(self.epool_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
                        }
                        self.connections.remove(&(fd as RawFd));
                    }
                } else {
                    // Existing connection
                    // let host = self.find_host_by_fd(fd as RawFd).unwrap();
                    let connection = self.connections.get(&(fd as RawFd)).unwrap();
                    let host = self.get_host_by_name(&connection.host_name).unwrap();
                    if let Err(e) = self.handle_request(fd as RawFd, host.clone()) {
                        eprintln!("Error handling client connection: {}", e);
                        unsafe {
                            epoll_ctl(self.epool_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut());
                        }
                        self.connections.remove(&(fd as RawFd));
                    }
                }
            }
        }
    }
}