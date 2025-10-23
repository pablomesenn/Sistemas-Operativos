//! # HTTP Server - Entry Point
//! src/main.rs
//! 
//! Punto de entrada del servidor HTTP/1.0.
//!
//! Soporta configuración via CLI arguments y variables de entorno.

use http_server::config::Config;
use http_server::server::Server;

fn main() {
    println!("=================================");
    println!("  RedUnix HTTP/1.0 Server");
    println!("  Principios de Sistemas Operativos");
    println!("=================================\n");
    
    // Parsear configuración desde CLI/env
    let config = Config::new();
    
    // Validar configuración
    if let Err(e) = config.validate() {
        eprintln!("❌ Error de configuración: {}", e);
        eprintln!("\nUsa --help para ver las opciones disponibles");
        std::process::exit(1);
    }
    
    // Imprimir resumen de configuración
    config.print_summary();
    
    // Crear el servidor
    let mut server = Server::new(config);
    
    // Iniciar el servidor (esto bloqueará el thread)
    if let Err(e) = server.run() {
        eprintln!("💥 Error fatal: {}", e);
        std::process::exit(1);
    }
}