#[cfg(test)]
mod tests {

    use crate::http::header::{Header, HeaderName, HeaderValue, HeaderParsedValue, ContentType};
    use crate::http::request::{Request, HttpMethod};
    use crate::http::response::Response;
    use crate::http::status::HttpStatusCode;
    use crate::http::request::parse_request;
    use crate::http::body::Body;

    #[test]
    fn test_create_request() {
        let headers = Vec::new();
        let request = Request::new(
            HttpMethod::GET,
            "/test".to_string(),
            "HTTP/1.1".to_string(),
            headers,
            None
        );

        assert_eq!(request.uri, "/test");
        assert_eq!(request.version, "HTTP/1.1");
        matches!(request.method, HttpMethod::GET);
    }

    #[test]
    fn test_create_response() {
        let mut headers = Vec::new();
        headers.push(Header {
            name: HeaderName::ContentType,
            value: HeaderValue {
                value: "application/json".to_string(),
                parsed_value: Some(HeaderParsedValue::ContentType(ContentType::ApplicationJson)),
            },
        });

        let response = Response::new(
            HttpStatusCode::Ok,
            headers.clone(),
            Some(Body::json(serde_json::json!({"message": "test"})))
        );

        assert_eq!(response.version, "HTTP/1.1");
        matches!(response.status_code, HttpStatusCode::Ok);
        assert_eq!(response.headers.len(), 1);
    }

    #[test]
    fn test_parse_request() {
        let request_str = "GET /test HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";
        let request = parse_request(request_str);

        assert!(request.is_some());
        let request = request.unwrap();
        assert_eq!(request.uri, "/test");
        matches!(request.method, HttpMethod::GET);
    }

    #[test]
    fn test_response_to_string() {
        let mut headers = Vec::new();
        headers.push(Header {
            name: HeaderName::ContentType,
            value: HeaderValue {
                value: "text/plain".to_string(),
                parsed_value: Some(HeaderParsedValue::ContentType(ContentType::TextPlain)),
            },
        });

        let response = Response::new(
            HttpStatusCode::Ok,
            headers,
            Some(Body::text("Hello"))
        );

        let response_str = response.to_string();
        assert!(response_str.contains("HTTP/1.1 200"));
        assert!(response_str.contains("Content-Type: text/plain"));
        assert!(response_str.contains("Hello"));
    }

    // #[test]
    // fn test_invalid_http_version() {
    //     let headers = Vec::new();
    //     let request = Request::new(
    //         HttpMethod::GET,
    //         "/test".to_string(),
    //         "HTTP/2.0".to_string(), // Invalid version
    //         headers,
    //         None
    //     );
        
    //     assert!(request.is_valid().is_err());
    // }

    // #[test]
    // fn test_malformed_request() {
    //     let request_str = "INVALID /test\r\nHost: localhost\r\n\r\n";
    //     let result = parse_request(request_str);
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn test_request_body_too_large() {
    //     let large_body = "x".repeat(1024 * 1024 * 10); // 10MB body
    //     let request_str = format!(
    //         "POST /test HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
    //         large_body.len(),
    //         large_body
    //     );
    //     let result = parse_request(&request_str);
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn test_missing_required_headers() {
    //     let headers = Vec::new(); // No Content-Type header
    //     let response = Response::new(
    //         "HTTP/1.1".to_string(),
    //         HttpStatusCode::Ok,
    //         headers,
    //         Some(Body::from_json(serde_json::json!({"message": "test"})))
    //     );
    //     assert!(response.validate().is_err());
    // }

    // #[test]
    // fn test_invalid_content_type() {
    //     let mut headers = Vec::new();
    //     headers.push(Header {
    //         name: HeaderName::ContentType,
    //         value: HeaderValue {
    //             value: "invalid/content-type".to_string(),
    //             parsed_value: None,
    //         },
    //     });
        
    //     let result = Response::new(
    //         "HTTP/1.1".to_string(),
    //         HttpStatusCode::Ok,
    //         headers,
    //         Some(Body::from_text("test"))
    //     ).validate();
        
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn test_response_body_content_type_mismatch() {
    //     let mut headers = Vec::new();
    //     headers.push(Header {
    //         name: HeaderName::ContentType,
    //         value: HeaderValue {
    //             value: "application/json".to_string(),
    //             parsed_value: Some(HeaderParsedValue::ContentType(ContentType::ApplicationJson)),
    //         },
    //     });
        
    //     let response = Response::new(
    //         "HTTP/1.1".to_string(),
    //         HttpStatusCode::Ok,
    //         headers,
    //         Some(Body::from_text("not json")) // String body with JSON content-type
    //     );
        
    //     assert!(response.validate().is_err());
    // }

    // #[test]
    // #[should_panic]
    // fn test_stream_read_timeout() {
    //     use std::net::TcpStream;
    //     use std::time::Duration;
    //     use std::io::Read;
        
    //     let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    //     stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        
    //     let mut buffer = [0; 1024];
    //     stream.read(&mut buffer).unwrap(); // Should timeout and panic
    // }
}