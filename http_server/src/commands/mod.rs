//! # Comandos del Servidor
//!
//! Este módulo contiene la implementación de todos los comandos
//! que el servidor puede ejecutar.
//!
//! ## Categorías de comandos
//!
//! - **basic**: Comandos básicos (fibonacci, reverse, toupper, etc.)
//! - **cpu_bound**: Comandos intensivos en CPU (isprime, factor, pi, etc.)
//! - **io_bound**: Comandos intensivos en I/O (sortfile, compress, etc.)
//!
//! Cada comando es una función handler que recibe un Request
//! y retorna una Response.

pub mod basic;
pub mod cpu_bound;
pub mod io_bound;

// Re-exportar funciones útiles
pub use basic::*;
pub use cpu_bound::*;
pub use io_bound::*;