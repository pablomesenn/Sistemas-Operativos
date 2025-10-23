//! # Módulo del Servidor HTTP
//! src/server/mod.rs
//!
//! Este módulo implementa el servidor TCP que:
//! 1. Escucha en un puerto
//! 2. Acepta conexiones entrantes
//! 3. Lee y parsea requests HTTP
//! 4. Genera y envía responses HTTP
//!
//! Por ahora implementaremos una versión básica que maneja
//! una conexión a la vez. Luego la haremos concurrente.

pub mod tcp;

// Re-exportar para facilitar el uso
pub use tcp::Server;