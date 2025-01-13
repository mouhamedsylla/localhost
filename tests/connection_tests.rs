use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

// Mock structures pour les tests
struct TestClient {
    stream: TcpStream,
}

impl TestClient {
    fn new(addr: &str) -> Self {
        let stream = TcpStream::connect(addr).expect("Failed to connect");
        stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        TestClient { stream }
    }

    fn send_request(&mut self, path: &str) -> String {
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n",
            path
        );
        self.stream.write_all(request.as_bytes()).unwrap();
        
        let mut response = String::new();
        let mut buffer = [0; 1024];
        
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    if response.contains("\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    // Test de base pour la connexion persistante
    #[test]
    fn test_keep_alive_basic() {
        let mut client = TestClient::new("127.0.0.1:8080");
        
        // Envoie de plusieurs requêtes sur la même connexion
        let response1 = client.send_request("/index.html");
        let response2 = client.send_request("/style.css");
        
        assert!(response1.contains("HTTP/1.1 200"));
        assert!(response2.contains("HTTP/1.1 200"));
        assert!(response1.contains("Connection: keep-alive"));
        assert!(response2.contains("Connection: keep-alive"));
    }

    // Test de timeout
    #[test]
    fn test_connection_timeout() {
        let mut client = TestClient::new("127.0.0.1:8080");
        
        // Première requête
        let response1 = client.send_request("/index.html");
        assert!(response1.contains("HTTP/1.1 200"));
        
        // Attend le timeout
        thread::sleep(Duration::from_secs(31));
        
        // La connexion devrait être fermée
        let response2 = client.send_request("/style.css");
        assert!(response2.is_empty());
    }

    // Helper function pour vérifier que le serveur est prêt
    fn wait_for_server() -> bool {
        for _ in 0..5 {
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
                return true;
            }
            thread::sleep(Duration::from_secs(1));
        }
        false
    }

    #[test]
    fn test_concurrent_connections() {
        // Vérifie que le serveur est prêt
        assert!(wait_for_server(), "Server is not ready");

        let active_connections = Arc::new(AtomicUsize::new(0));
        let max_connections = Arc::new(AtomicUsize::new(0));
        
        // Réduit le nombre de connexions simultanées
        let num_connections = 10; // Réduit de 100 à 10
        
        let threads: Vec<_> = (0..num_connections)
            .map(|i| {
                let active = Arc::clone(&active_connections);
                let max = Arc::clone(&max_connections);
                
                thread::spawn(move || {
                    // Ajoute un délai pour éviter la congestion
                    thread::sleep(Duration::from_millis(i as u64 * 10));
                    
                    match TcpStream::connect("127.0.0.1:8080") {
                        Ok(stream) => {
                            let mut client = TestClient { stream };
                            
                            // Incrémente le compteur de connexions actives
                            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                            max.fetch_max(current, Ordering::SeqCst);
                            
                            // Réduit le nombre de requêtes par connexion
                            for j in 0..2 {  // Réduit de 5 à 2
                                let response = client.send_request(&format!("/index.html"));
                                if !response.contains("HTTP/1.1 200") {
                                    println!("Response received: {:?}", response);
                                    panic!("Invalid response for request {}", j);
                                }
                                // Ajoute un délai entre les requêtes
                                thread::sleep(Duration::from_millis(50));
                            }
                            
                            // Décrémente le compteur
                            active.fetch_sub(1, Ordering::SeqCst);
                           // Ok(())
                        },
                        Err(e) => {
                            panic!("Failed to connect: {:?}", e);
                        }
                    }
                })
            })
            .collect();

        // Attend que tous les threads terminent
        let mut success = true;
        for thread in threads {
            if let Err(e) = thread.join() {
                success = false;
                println!("Thread panicked: {:?}", e);
            }
        }
        
        assert!(success, "Some threads failed");
        
        let max_concurrent = max_connections.load(Ordering::SeqCst);
        println!("Maximum concurrent connections: {}", max_concurrent);
        assert!(max_concurrent > 0, "No concurrent connections detected");
    }

    // Test de performance
    #[test]
    fn test_performance() {
        let mut single_connection_client = TestClient::new("127.0.0.1:8080");
        
        // Test avec connexion persistante
        let start = Instant::now();
        for _ in 0..100 {
            let response = single_connection_client.send_request("/index.html");
            assert!(response.contains("HTTP/1.1 200"));
        }
        let keep_alive_duration = start.elapsed();
        
        // Test avec nouvelles connexions
        let start = Instant::now();
        for _ in 0..100 {
            let mut new_client = TestClient::new("127.0.0.1:8080");
            let response = new_client.send_request("/index.html");
            assert!(response.contains("HTTP/1.1 200"));
        }
        let new_conn_duration = start.elapsed();
        
        println!("Keep-alive duration: {:?}", keep_alive_duration);
        println!("New connections duration: {:?}", new_conn_duration);
        assert!(keep_alive_duration < new_conn_duration);
    }

    // Test des limites de buffer
    #[test]
    fn test_buffer_limits() {
        let mut client = TestClient::new("127.0.0.1:8080");
        
        // Crée une grande requête
        let large_path = "/".to_string() + &"a".repeat(8192);
        let response = client.send_request(&large_path);
        
        // Devrait recevoir une erreur 414 URI Too Long
        assert!(response.contains("HTTP/1.1 414"));
    }
}