//! # Construcción de Respuestas HTTP
//!
//! Este módulo proporciona una API para construir respuestas HTTP/1.0
//! de forma programática y convertirlas a bytes para enviar al cliente.
//!
//! ## Formato de una respuesta HTTP/1.0
//!
//! ```text
//! HTTP/1.0 200 OK\r\n
//! Content-Type: application/json\r\n
//! Content-Length: 13\r\n
//! X-Request-Id: abc123\r\n
//! \r\n
//! {"ok": true}
//! ```
//!
//! ## Ejemplo de uso
//!
//! ```
//! use http_server::http::{Response, StatusCode};
//!
//! let response = Response::new(StatusCode::Ok)
//!     .with_header("Content-Type", "application/json")
//!     .with_body(r#"{"message": "Hello"}"#);
//!
//! let bytes = response.to_bytes();
//! // Ahora puedes enviar `bytes` por el socket
//! ```

use super::StatusCode;
use std::collections::HashMap;

/// Representa una respuesta HTTP/1.0 completa
#[derive(Debug, Clone)]
pub struct Response {
    /// Código de estado HTTP (200, 404, etc.)
    status: StatusCode,
    
    /// Headers HTTP (Content-Type, Content-Length, etc.)
    /// Usamos HashMap para evitar duplicados
    headers: HashMap<String, String>,
    
    /// Cuerpo de la respuesta (puede ser vacío)
    body: Vec<u8>,
}

impl Response {
    /// Crea una nueva respuesta con el código de estado especificado
    /// 
    /// Por defecto, la respuesta no tiene headers ni body.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let response = Response::new(StatusCode::Ok);
    /// ```
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
    
    /// Agrega un header a la respuesta
    /// 
    /// Si el header ya existe, se sobrescribe.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let response = Response::new(StatusCode::Ok)
    ///     .with_header("Content-Type", "application/json");
    /// ```
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Agrega un header a una respuesta existente (versión mutable)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let mut response = Response::new(StatusCode::Ok);
    /// response.add_header("Content-Type", "application/json");
    /// ```
    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }
    
    /// Establece el cuerpo de la respuesta desde un string
    /// 
    /// Automáticamente calcula y agrega el header `Content-Length`.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let response = Response::new(StatusCode::Ok)
    ///     .with_body("Hello World");
    /// ```
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self.headers.insert(
            "Content-Length".to_string(),
            self.body.len().to_string()
        );
        self
    }
    
    /// Establece el cuerpo de la respuesta desde bytes
    /// 
    /// Útil para respuestas binarias (imágenes, archivos comprimidos, etc.)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let binary_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
    /// let response = Response::new(StatusCode::Ok)
    ///     .with_body_bytes(binary_data);
    /// ```
    pub fn with_body_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self.headers.insert(
            "Content-Length".to_string(),
            self.body.len().to_string()
        );
        self
    }
    
    /// Crea una respuesta JSON exitosa (200 OK)
    /// 
    /// Automáticamente establece `Content-Type: application/json`.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::Response;
    /// 
    /// let response = Response::json(r#"{"status": "ok"}"#);
    /// ```
    pub fn json(body: &str) -> Self {
        Self::new(StatusCode::Ok)
            .with_header("Content-Type", "application/json")
            .with_body(body)
    }
    
    /// Crea una respuesta de error con mensaje JSON
    /// 
    /// Formato del JSON: `{"error": "mensaje"}`
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let response = Response::error(
    ///     StatusCode::BadRequest,
    ///     "Invalid parameter: num must be positive"
    /// );
    /// ```
    pub fn error(status: StatusCode, message: &str) -> Self {
        let body = format!(r#"{{"error": "{}"}}"#, message);
        Self::new(status)
            .with_header("Content-Type", "application/json")
            .with_body(&body)
    }
    
    /// Convierte la respuesta a bytes listos para enviar por el socket
    /// 
    /// Genera el formato completo HTTP/1.0:
    /// - Status line: `HTTP/1.0 200 OK\r\n`
    /// - Headers: `Header-Name: Value\r\n`
    /// - Línea vacía: `\r\n`
    /// - Body: contenido binario
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::{Response, StatusCode};
    /// 
    /// let response = Response::new(StatusCode::Ok)
    ///     .with_body("Hello");
    /// 
    /// let bytes = response.to_bytes();
    /// // bytes contiene: "HTTP/1.0 200 OK\r\n...\r\n\r\nHello"
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // 1. Status line
        // Formato: HTTP/1.0 200 OK\r\n
        let status_line = format!(
            "HTTP/1.0 {}\r\n",
            self.status
        );
        result.extend_from_slice(status_line.as_bytes());
        
        // 2. Headers
        // Formato: Header-Name: Value\r\n
        for (name, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            result.extend_from_slice(header_line.as_bytes());
        }
        
        // 3. Línea vacía que separa headers del body
        result.extend_from_slice(b"\r\n");
        
        // 4. Body (si existe)
        result.extend_from_slice(&self.body);
        
        result
    }
    
    /// Obtiene el código de estado de la respuesta
    pub fn status(&self) -> StatusCode {
        self.status
    }
    
    /// Obtiene una referencia a los headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    
    /// Obtiene una referencia al body
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_response() {
        let response = Response::new(StatusCode::Ok);
        assert_eq!(response.status(), StatusCode::Ok);
        assert!(response.headers().is_empty());
        assert!(response.body().is_empty());
    }
    
    #[test]
    fn test_with_header() {
        let response = Response::new(StatusCode::Ok)
            .with_header("Content-Type", "text/plain")
            .with_header("X-Custom", "value");
        
        assert_eq!(response.headers().get("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(response.headers().get("X-Custom"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_with_body() {
        let response = Response::new(StatusCode::Ok)
            .with_body("Hello World");
        
        assert_eq!(response.body(), b"Hello World");
        assert_eq!(response.headers().get("Content-Length"), Some(&"11".to_string()));
    }
    
    #[test]
    fn test_json_response() {
        let response = Response::json(r#"{"status": "ok"}"#);
        
        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(response.headers().get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(response.body(), br#"{"status": "ok"}"#);
    }
    
    #[test]
    fn test_error_response() {
        let response = Response::error(StatusCode::BadRequest, "Invalid input");
        
        assert_eq!(response.status(), StatusCode::BadRequest);
        assert_eq!(response.headers().get("Content-Type"), Some(&"application/json".to_string()));
        
        let body_str = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body_str.contains("Invalid input"));
    }
    
    #[test]
    fn test_to_bytes() {
        let response = Response::new(StatusCode::Ok)
            .with_header("Content-Type", "text/plain")
            .with_body("Test");
        
        let bytes = response.to_bytes();
        let text = String::from_utf8(bytes).unwrap();
        
        // Verificar que contiene los elementos clave
        assert!(text.starts_with("HTTP/1.0 200 OK\r\n"));
        assert!(text.contains("Content-Type: text/plain\r\n"));
        assert!(text.contains("Content-Length: 4\r\n"));
        assert!(text.ends_with("\r\n\r\nTest"));
    }
    
    #[test]
    fn test_empty_body_response() {
        let response = Response::new(StatusCode::NoContent);
        let bytes = response.to_bytes();
        let text = String::from_utf8(bytes).unwrap();
        
        // Debe terminar con \r\n\r\n (sin body)
        assert!(text.ends_with("\r\n\r\n"));
    }
    
    #[test]
    fn test_with_body_bytes() {
        let binary_data = vec![0x00, 0x01, 0x02, 0xFF];
        let response = Response::new(StatusCode::Ok)
            .with_body_bytes(binary_data.clone());
        
        assert_eq!(response.body(), &binary_data[..]);
        assert_eq!(response.headers().get("Content-Length"), Some(&"4".to_string()));
    }
}