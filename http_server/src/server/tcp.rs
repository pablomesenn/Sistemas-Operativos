//! # Servidor TCP Concurrente
//!
//! Implementación del servidor TCP que maneja múltiples conexiones simultáneas
//! usando threads. Cada conexión se procesa en su propio thread.

use crate::config::Config;
use crate::http::{Request, Response, StatusCode};
use crate::router::Router;
use crate::commands;
use crate::metrics::MetricsCollector;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Servidor HTTP/1.0 concurrente con métricas
pub struct Server {
    config: Config,
    router: Arc<Router>,
    metrics: Arc<MetricsCollector>,
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let mut router = Router::new();
        
        // Comandos básicos
        router.register("/status", commands::status_handler);
        router.register("/fibonacci", commands::fibonacci_handler);
        router.register("/reverse", commands::reverse_handler);
        router.register("/toupper", commands::toupper_handler);
        router.register("/timestamp", commands::timestamp_handler);
        router.register("/random", commands::random_handler);
        router.register("/hash", commands::hash_handler);
        router.register("/createfile", commands::createfile_handler);
        router.register("/deletefile", commands::deletefile_handler);
        router.register("/simulate", commands::simulate_handler);
        router.register("/sleep", commands::sleep_handler);
        router.register("/loadtest", commands::loadtest_handler);
        router.register("/help", commands::help_handler);
        
        // Comandos CPU-bound
        router.register("/isprime", commands::isprime_handler);
        router.register("/factor", commands::factor_handler);
        router.register("/pi", commands::pi_handler);
        router.register("/mandelbrot", commands::mandelbrot_handler);
        router.register("/matrixmul", commands::matrixmul_handler);
        
        // Comandos IO-bound
        router.register("/sortfile", commands::sortfile_handler);
        router.register("/wordcount", commands::wordcount_handler);
        router.register("/grep", commands::grep_handler);
        router.register("/compress", commands::compress_handler);
        router.register("/hashfile", commands::hashfile_handler);
        
        // Nota: /metrics se manejará especialmente en handle_connection_static
        
        Self {
            config,
            router: Arc::new(router),
            metrics: Arc::new(MetricsCollector::new()),
            listener: None,
        }
    }
    
    pub fn run(&mut self) -> std::io::Result<()> {
        let address = self.config.address();
        println!("🚀 Iniciando servidor en {}", address);
        
        let listener = TcpListener::bind(&address)?;
        println!("✅ Servidor escuchando en {}", address);
        println!("⚡ Modo concurrente: un thread por conexión\n");
        
        self.listener = Some(listener);
        let listener = self.listener.as_ref().unwrap();
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let router = Arc::clone(&self.router);
                    let metrics = Arc::clone(&self.metrics);
                    
                    let peer_addr = stream.peer_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    println!("📥 Nueva conexión desde: {} (spawning thread)", peer_addr);
                    
                    // Incrementar contador de threads activos
                    metrics.increment_active_threads();
                    
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_connection_static(stream, router, metrics.clone()) {
                            eprintln!("   ❌ Error en thread: {}", e);
                        }
                        // Decrementar al terminar
                        metrics.decrement_active_threads();
                    });
                }
                Err(e) => {
                    eprintln!("❌ Error al aceptar conexión: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_connection_static(
        mut stream: TcpStream, 
        router: Arc<Router>,
        metrics: Arc<MetricsCollector>
    ) -> std::io::Result<()> {
        let start = Instant::now();
        
        // Generar Request ID único
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        start.elapsed().as_nanos().hash(&mut hasher);
        thread::current().id().hash(&mut hasher);
        let request_id = format!("{:016x}", hasher.finish());
        let thread_id = format!("{:?}", thread::current().id());
        
        let mut buffer = [0u8; 8192];
        let bytes_read = stream.read(&mut buffer)?;
        
        if bytes_read == 0 {
            println!("   ⚠️  Conexión cerrada");
            return Ok(());
        }
        
        println!("   📨 {} bytes [req_id: {}]", bytes_read, &request_id[..8]);
        
        let (response, path) = match Request::parse(&buffer[..bytes_read]) {
            Ok(request) => {
                let path = request.path().to_string();
                println!("   ✅ {} {}", request.method().as_str(), path);
                
                // Manejar /metrics especialmente
                let response = if path == "/metrics" {
                    let json = metrics.get_metrics_json();
                    Response::json(&json)
                } else {
                    router.route(&request)
                };
                
                (response, path)
            }
            Err(e) => {
                println!("   ❌ Parse error: {}", e);
                (Response::error(StatusCode::BadRequest, &format!("Invalid: {}", e)), "/error".to_string())
            }
        };
        
        // Agregar headers de observabilidad
        let mut response = response;
        response.add_header("X-Request-Id", &request_id);
        response.add_header("X-Worker-Thread", &thread_id);
        
        let response_bytes = response.to_bytes();
        stream.write_all(&response_bytes)?;
        stream.flush()?;
        
        let latency = start.elapsed();
        let status_code = response.status().as_u16();
        
        // Registrar métricas
        metrics.record_request(&path, status_code, latency);
        
        println!("   📤 {} ({:.2}ms)\n", response.status(), latency.as_secs_f64() * 1000.0);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_creation() {
        let config = Config::default();
        let _server = Server::new(config);
    }
}