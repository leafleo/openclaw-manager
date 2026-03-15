use std::time::Duration;
use std::net::TcpStream;
use std::io::{Write, Read};

const SERVICE_PORT: u16 = 18789;

/// Check if gateway responds to HTTP requests
fn check_gateway_http() -> bool {
    // Try to connect to the gateway
    let addr = format!("127.0.0.1:{}", SERVICE_PORT);
    match TcpStream::connect(&addr) {
        Ok(mut stream) => {
            // Set read timeout
            if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(2000))) {
                return false;
            }
            
            // Send a simple HTTP GET request
            let request = format!(
                "GET / HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
                SERVICE_PORT
            );
            if let Err(_) = stream.write_all(request.as_bytes()) {
                return false;
            }
            
            // Read response
            let mut buffer = [0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    // Check if we got any HTTP response
                    response.contains("HTTP/1.1") || response.contains("<!DOCTYPE") || response.contains("<html")
                }
                _ => false,
            }
        }
        Err(_) => false,
    }
}

fn main() {
    println!("Testing service status check...");
    
    let is_running = check_gateway_http();
    println!("Service running: {}", is_running);
    
    if is_running {
        println!("✅ Service is running!");
    } else {
        println!("❌ Service is not running.");
    }
}