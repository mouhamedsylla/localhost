//! Request Stream Module
//!
//! This module provides functionality for handling HTTP request streams in a non-blocking manner.
//! It supports both standard Content-Length based requests and chunked transfer encoding.
//! The module implements buffering and state management for processing HTTP requests.

pub mod request_stream {
    use std::io::{self, Read};

    /// Size of the read buffer for processing requests
    const BUFFER_SIZE: usize = 4096;
    /// Maximum allowed size for a complete request
    const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB

    /// Represents the complete request data including headers and body
    #[derive(Debug, Clone)]
    pub struct RequestData {
        /// Raw data containing both headers and body
        pub data: Vec<u8>,
        /// Position marking the end of headers and start of body
        headers_end: usize,
    }

    /// Methods for accessing request data components
    impl RequestData {
        /// Returns a slice containing only the headers portion of the request
        pub fn get_headers(&self) -> &[u8] {
            &self.data[..self.headers_end]
        }

        /// Returns a slice containing only the body portion of the request
        pub fn get_body(&self) -> &[u8] {
            &self.data[self.headers_end..]
        }
    }

    /// Represents different types of request body handling
    #[derive(Debug)]
    enum ReaderType {
        /// Initial state before determining the type of request
        Unknown,
        /// Standard request with Content-Length header
        Standard { content_length: usize },
        /// Request using chunked transfer encoding
        Chunked,
    }

    /// Represents the current state of request processing
    #[derive(Debug, Clone)]
    pub enum RequestState {
        /// Waiting to receive and parse headers
        AwaitingHeaders,
        /// Currently processing the request body
        ProcessingBody { 
            /// Accumulated data including headers and partial body
            accumulated_data: Vec<u8>,
            /// Position marking end of headers
            headers_end: usize,
        },
        /// Request is fully received and parsed
        Complete(RequestData),
        /// Connection has been closed
        EndOfStream,
    }

    /// Core trait defining request stream behavior
    pub trait RequestStream {
        /// Attempts to read the next portion of the request
        /// Returns the current state of request processing
        fn read_next(&mut self) -> io::Result<RequestState>;
        
        /// Writes data to the underlying stream
        fn write(&mut self, buf: &[u8]) -> io::Result<()>;
        
        /// Flushes any buffered data to the underlying stream
        fn flush(&mut self) -> io::Result<()>;
        
        /// Resets the stream state for processing a new request
        fn reset(&mut self);
        
        /// Returns true if a complete request has been received
        fn is_complete(&self) -> bool;
    }

    /// Implementation of unified request reading with support for both
    /// standard and chunked transfer encoding
    pub mod unifiedReader {
        use super::*;
        use std::io::{self, Read, Write, ErrorKind};

        /// Unified reader for handling both standard and chunked requests
        pub struct UnifiedReader<S: Read + Write> {
            /// Underlying stream for I/O operations
            stream: S,
            /// Buffer for accumulating request data
            buffer: Vec<u8>,
            /// Current state of request processing
            state: RequestState,
            /// Type of request being processed
            reader_type: ReaderType,
            /// Temporary storage for chunk headers
            temp_chunk_headers: Option<Vec<u8>>,
            /// Size of the current chunk being processed
            current_chunk_size: Option<usize>,
        }

        /// Implementation of UnifiedReader for handling HTTP request streams
        /// 
        /// # Type Parameters
        /// * `S` - A type that implements both Read and Write traits for I/O operations
        /// 
        /// # Examples
        /// ```
        /// use std::net::TcpStream;
        /// 
        /// let stream = TcpStream::new();
        /// let reader = UnifiedReader::new(stream);
        /// ```
        /// 
        /// This implementation provides:
        /// - Non-blocking I/O operations
        /// - Support for both standard and chunked transfer encodings
        /// - Buffer management for partial reads
        /// - State management for request processing
        impl<S: Read + Write> UnifiedReader<S> {
            
            /// Creates a new UnifiedReader with the given stream
            /// 
            /// # Arguments
            /// * `stream` - An object implementing both Read and Write traits
            /// 
            /// # Returns
            /// A new UnifiedReader instance initialized with default values
            pub fn new(stream: S) -> Self {
                UnifiedReader {
                    stream,
                    buffer: Vec::new(),
                    state: RequestState::AwaitingHeaders,
                    reader_type: ReaderType::Unknown,
                    temp_chunk_headers: None,
                    current_chunk_size: None,
                }
            }

            fn determine_reader_type(data: &[u8], headers_end: usize) -> ReaderType {
                if let Ok(headers_str) = String::from_utf8(data[..headers_end].to_vec()) {
                    if headers_str.lines().any(|line| line.to_lowercase().contains("transfer-encoding: chunked")) {
                        return ReaderType::Chunked;
                    }

                    if let Some(content_length) = headers_str.lines()
                        .find(|line| line.to_lowercase().starts_with("content-length:"))
                        .and_then(|line| line.split(':').nth(1))
                        .and_then(|len| len.trim().parse::<usize>().ok()) {
                        return ReaderType::Standard { content_length };
                    }
                }
                ReaderType::Standard { content_length: 0 }
            }

            fn process_standard_body(
                &mut self,
                mut accumulated_data: Vec<u8>,
                headers_end: usize,
                content_length: usize,
            ) -> io::Result<RequestState> {
                let total_expected = headers_end + content_length;
            
                while accumulated_data.len() < total_expected {
                    let mut temp_buffer = [0u8; BUFFER_SIZE];
                    let bytes_read = self.stream.read(&mut temp_buffer)?;
                    if bytes_read == 0 {
                        // End of stream
                        return Ok(RequestState::EndOfStream);
                    }
                    accumulated_data.extend_from_slice(&temp_buffer[..bytes_read]);
                }
            
                // We have all the data we need
                let request_data = RequestData {
                    data: accumulated_data[..total_expected].to_vec(),
                    headers_end,
                };
                self.buffer = accumulated_data[total_expected..].to_vec();
                self.state = RequestState::Complete(request_data);
            
                Ok(self.state.clone())
            }
            

            fn process_chunked_body(
                &mut self,
                mut accumulated_data: Vec<u8>,
                headers_end: usize,
            ) -> io::Result<RequestState> {
                loop {
                    // Read chunk size if necessary
                    if self.current_chunk_size.is_none() {
                        if let Some(line_end) = find_line_end(&self.buffer) {
                            let size_line = &self.buffer[..line_end - 2];
                            if let Some(size) = parse_chunk_size(size_line) {
                                if size == 0 {
                                    // Final chunk - complete request
                                    self.state = RequestState::Complete(RequestData {
                                        data: accumulated_data,
                                        headers_end,
                                    });
                                    return Ok(self.state.clone());
                                }
                                self.current_chunk_size = Some(size);
                                self.buffer = self.buffer[line_end..].to_vec();
                            }
                        }
                    }
            
                    if let Some(chunk_size) = self.current_chunk_size {
                        if self.buffer.len() >= chunk_size + 2 {
                            // Append chunk data to accumulated data
                            accumulated_data.extend_from_slice(&self.buffer[..chunk_size]);
                            self.buffer = self.buffer[chunk_size + 2..].to_vec();
                            self.current_chunk_size = None;
                            
                        } else {
                            let mut temp_buffer = [0u8; BUFFER_SIZE];
                            let bytes_read = self.stream.read(&mut temp_buffer)?;
                            if bytes_read == 0 {
                                return Ok(RequestState::EndOfStream);
                            }
                            self.buffer.extend_from_slice(&temp_buffer[..bytes_read]);
                        }
                    }
                }
            }
            
        }

        impl<S: Read + Write> RequestStream for UnifiedReader<S> {
            /// Reads the next chunk of data from the stream and processes it according to the current state
            /// 
            /// # Returns
            /// - `Ok(RequestState)` - The new state after processing the data
            /// - `Err(io::Error)` - If an error occurs during reading or processing
            fn read_next(&mut self) -> io::Result<RequestState> {
                let mut temp_buffer = [0u8; BUFFER_SIZE];
                match self.state.clone() {
                    /// When awaiting headers, try to read until we find the header boundary
                    RequestState::AwaitingHeaders => {
                        match self.stream.read(&mut temp_buffer)? {
                            0 => Ok(RequestState::EndOfStream),
                            n => {
                                self.buffer.extend_from_slice(&temp_buffer[..n]);
                                if let Some(headers_end) = find_headers_end(&self.buffer) {
                                    let accumulated_data = self.buffer.clone();
                                    self.reader_type = Self::determine_reader_type(&accumulated_data, headers_end);
                                    self.buffer = self.buffer[headers_end..].to_vec();
                                    self.state = RequestState::ProcessingBody { 
                                        accumulated_data,
                                        headers_end,
                                    };
                                    self.read_next()
                                } else {
                                    Ok(RequestState::AwaitingHeaders)
                                }
                            }
                        }
                    }

                    /// When processing body, handle according to transfer type (chunked or standard)
                    RequestState::ProcessingBody { accumulated_data, headers_end } => {
                        match self.reader_type {
                            ReaderType::Unknown => Ok(RequestState::EndOfStream),
                            ReaderType::Standard { content_length } => {
                                self.process_standard_body(accumulated_data, headers_end, content_length)
                            },
                            ReaderType::Chunked => {
                                self.process_chunked_body(accumulated_data, headers_end)
                            }
                        }
                    }

                    /// For completed requests, just return the current state
                    RequestState::Complete(data) => Ok(RequestState::Complete(data)),

                    /// For end of stream, return the end state
                    RequestState::EndOfStream => Ok(RequestState::EndOfStream),
                }
            }

            /// Writes data to the underlying stream
            /// 
            /// # Arguments
            /// * `buf` - The buffer to write
            /// 
            /// # Returns
            /// - `Ok(())` if write was successful
            /// - `Err(io::Error)` if write failed
            fn write(&mut self, buf: &[u8]) -> io::Result<()> {
                self.stream.write_all(buf)
            }

            /// Flushes any buffered data to the underlying stream
            /// 
            /// # Returns
            /// - `Ok(())` if flush was successful
            /// - `Err(io::Error)` if flush failed
            fn flush(&mut self) -> io::Result<()> {
                self.stream.flush()
            }

            /// Resets the stream state for processing a new request
            fn reset(&mut self) {
                self.buffer.clear();
                self.state = RequestState::AwaitingHeaders;
                self.reader_type = ReaderType::Unknown;
                self.current_chunk_size = None;
                self.temp_chunk_headers = None;
            }

            /// Returns true if a complete request has been received
            fn is_complete(&self) -> bool {
                matches!(self.state, RequestState::Complete(_))
            }
        }
    }

    fn find_headers_end(data: &[u8]) -> Option<usize> {
        data.windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
    }

    fn parse_chunk_size(line: &[u8]) -> Option<usize> {
        if let Ok(size_str) = String::from_utf8(line.to_vec()) {
            return usize::from_str_radix(&size_str.trim(), 16).ok();
        }
        None
    }

    fn find_line_end(data: &[u8]) -> Option<usize> {
        data.windows(2)
            .position(|window| window == b"\r\n")
            .map(|pos| pos + 2)
    }

    pub use unifiedReader::UnifiedReader;
}