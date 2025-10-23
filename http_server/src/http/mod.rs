//! # Módulo HTTP
//!
//! Este módulo implementa el protocolo HTTP/1.0 desde cero, sin usar
//! librerías de alto nivel. Incluye:
//!
//! - Parsing de requests HTTP/1.0
//! - Construcción de responses HTTP
//! - Manejo de status codes
//! - Extracción de query parameters
//!
//! ## Especificación HTTP/1.0
//!
//! El protocolo HTTP/1.0 (RFC 1945) es más simple que HTTP/1.1:
//! - No requiere el header `Host`
//! - No tiene chunked transfer encoding
//! - No mantiene conexiones persistentes por defecto
//!
//! ### Formato de Request
//!
//! ```text
//! GET /path?query=value HTTP/1.0\r\n
//! Header-Name: Header-Value\r\n
//! Another-Header: Value\r\n
//! \r\n
//! ```
//!
//! ### Formato de Response
//!
//! ```text
//! HTTP/1.0 200 OK\r\n
//! Content-Type: application/json\r\n
//! Content-Length: 13\r\n
//! \r\n
//! {"ok": true}
//! ```

// Submódulos del módulo HTTP
// Vamos a implementarlos uno por uno

pub mod request;   // Parsing de HTTP requests
pub mod response;  // Construcción de HTTP responses
pub mod status;    // Códigos de estado HTTP

// Re-exportamos los tipos principales para facilitar su uso
// Esto permite usar `http::Request` en vez de `http::request::Request`
pub use request::Request;
pub use response::Response;
pub use status::StatusCode;