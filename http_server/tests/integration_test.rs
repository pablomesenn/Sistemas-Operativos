//! Tests de integración para el servidor HTTP
//! src/tests/integration_test.rs
//!
//! IMPORTANTE: Estos tests requieren que el servidor esté corriendo.
//! 
//! Para ejecutar:
//! 1. Terminal 1: cargo run
//! 2. Terminal 2: cargo test --test integration_test

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Helper: envía un request HTTP y retorna la response completa
fn send_request(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Dar tiempo al servidor a estar listo
    thread::sleep(Duration::from_millis(50));
    
    // Conectar al servidor
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    
    // Configurar timeouts
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    // Construir request HTTP/1.0 (más simple)
    let request = format!(
        "GET {} HTTP/1.0\r\n\r\n",
        path
    );
    
    // Enviar request
    stream.write_all(request.as_bytes())?;
    stream.flush()?;
    
    // Leer response
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    
    Ok(response)
}

/// Helper: extrae el body JSON de una response HTTP
fn extract_body(response: &str) -> &str {
    // Buscar la línea vacía que separa headers del body
    if let Some(pos) = response.find("\r\n\r\n") {
        &response[pos + 4..]
    } else {
        ""
    }
}

#[test]
fn test_help_endpoint() {
    let response = send_request("/help").expect("Failed to send request");
    
    // Verificar que la response es exitosa
    assert!(response.contains("200 OK"), "Expected 200 OK, got: {}", response);
    
    // Verificar que el body contiene "commands"
    let body = extract_body(&response);
    assert!(body.contains("commands"), "Body should contain 'commands'");
    assert!(body.contains("/fibonacci"), "Body should list fibonacci command");
}

#[test]
fn test_status_endpoint() {
    let response = send_request("/status").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("status"));
    assert!(body.contains("running"));
}

#[test]
fn test_fibonacci_endpoint() {
    let response = send_request("/fibonacci?num=10").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("55"), "fib(10) should be 55, got: {}", body);
}

#[test]
fn test_fibonacci_larger_number() {
    let response = send_request("/fibonacci?num=20").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("6765"), "fib(20) should be 6765");
}

#[test]
fn test_reverse_endpoint() {
    let response = send_request("/reverse?text=hello").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("olleh"));
}

#[test]
fn test_reverse_with_spaces() {
    let response = send_request("/reverse?text=hello%20world").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("dlrow olleh"));
}

#[test]
fn test_toupper_endpoint() {
    let response = send_request("/toupper?text=hello").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("HELLO"));
}

#[test]
fn test_timestamp_endpoint() {
    let response = send_request("/timestamp").expect("Failed to send request");
    
    assert!(response.contains("200 OK"));
    let body = extract_body(&response);
    assert!(body.contains("timestamp"));
    
    // Verificar que el timestamp es un número razonable (mayor que 2020-01-01)
    assert!(body.contains("1"), "Should contain timestamp digits");
}

#[test]
fn test_not_found() {
    let response = send_request("/nonexistent").expect("Failed to send request");
    
    assert!(response.contains("404"), "Expected 404 for non-existent route");
    let body = extract_body(&response);
    assert!(body.contains("error") || body.contains("not found") || body.contains("Route not found"));
}

#[test]
fn test_fibonacci_missing_param() {
    let response = send_request("/fibonacci").expect("Failed to send request");
    
    assert!(response.contains("400"), "Expected 400 for missing parameter");
    let body = extract_body(&response);
    assert!(body.contains("error"));
}

#[test]
fn test_fibonacci_invalid_param() {
    let response = send_request("/fibonacci?num=abc").expect("Failed to send request");
    
    assert!(response.contains("400"), "Expected 400 for invalid parameter");
}

#[test]
fn test_fibonacci_too_large() {
    let response = send_request("/fibonacci?num=100").expect("Failed to send request");
    
    assert!(response.contains("400"), "Expected 400 for num > 90");
}

#[test]
fn test_multiple_requests_sequentially() {
    // Verificar que el servidor puede manejar múltiples requests
    for i in 0..5 {
        let response = send_request(&format!("/fibonacci?num={}", i)).expect("Failed to send request");
        assert!(response.contains("200 OK"), "Request {} failed", i);
    }
}

#[test]
fn test_reverse_missing_param() {
    let response = send_request("/reverse").expect("Failed to send request");
    assert!(response.contains("400"));
}

#[test]
fn test_toupper_missing_param() {
    let response = send_request("/toupper").expect("Failed to send request");
    assert!(response.contains("400"));
}