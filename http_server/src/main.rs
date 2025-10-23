//! # HTTP Server - Entry Point
//! src/main.rs
//! 
//! Punto de entrada del servidor HTTP/1.0.
//!
//! Por ahora usa configuración por defecto.
//! Luego agregaremos parsing de CLI arguments.

use http_server::config::Config;
use http_server::server::Server;

fn main() {
    println!("=================================");
    println!("  RedUnix HTTP/1.0 Server");
    println!("  Principios de Sistemas Operativos");
    println!("=================================\n");
    
    // Crear configuración (por defecto o desde env)
    let config = Config::from_env();
    
    println!("⚙️  Configuración:");
    println!("   Puerto: {}", config.port);
    println!("   Host: {}", config.host);
    println!("   Data Dir: {}", config.data_dir);
    println!();
    
    // Crear el servidor
    let mut server = Server::new(config);
    
    // Iniciar el servidor (esto bloqueará el thread)
    if let Err(e) = server.run() {
        eprintln!("💥 Error fatal: {}", e);
        std::process::exit(1);
    }
}