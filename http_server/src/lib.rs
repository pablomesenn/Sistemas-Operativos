//! # HTTP Server
//! src/lib.rs
//!
//! Servidor HTTP/1.0 concurrente implementado desde cero para demostrar
//! conceptos de sistemas operativos: concurrencia, sincronización, 
//! planificación y manejo de recursos.
//!
//! ## Arquitectura
//!
//! El servidor está dividido en módulos especializados:
//! - `http`: Parsing y manejo del protocolo HTTP/1.0
//! - `server`: Lógica del servidor TCP y manejo de conexiones
//! - `router`: Enrutamiento de peticiones a handlers
//! - `commands`: Implementación de comandos (básicos, CPU-bound, IO-bound)
//! - `workers`: Sistema de pools de workers por tipo de tarea
//! - `jobs`: Sistema asíncrono de trabajos largos
//! - `metrics`: Recolección de métricas y observabilidad
//!
//! ## Ejemplo de uso
//!
//! ```ignore
//! // Ejemplo será habilitado cuando implementemos server y config
//! use http_server::server::Server;
//! use http_server::config::Config;
//!
//! let config = Config::default();
//! let server = Server::new(config);
//! server.run().expect("Error al iniciar servidor");
//! ```

// Iremos agregando más módulos conforme los implementemos
pub mod http;
pub mod config;
pub mod server;
pub mod router;
pub mod commands;
pub mod metrics;
pub mod jobs;

// Módulos que agregaremos después (comentados por ahora)
// pub mod router;
// pub mod config;
// pub mod commands;
// pub mod workers;
// pub mod jobs;
// pub mod metrics;
// pub mod utils;