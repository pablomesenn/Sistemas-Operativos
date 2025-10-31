//! # Servidor TCP Concurrente
//! src/server/tcp.rs
//!
//! Implementacion del servidor TCP que maneja mulltiples conexiones simultaneas
//! usando threads. Cada conexiÃ³n se procesa en su propio thread.

use crate::config::Config;
use crate::http::{Request, Response, StatusCode};
use crate::router::Router;
use crate::commands;
use crate::metrics::MetricsCollector;
use crate::jobs::{JobManager, handlers as job_handlers};
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
    job_manager: Arc<JobManager>,
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let mut router = Router::new();
        
        // Comandos bÃ¡sicos
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
        
        // Nota: /metrics y /jobs/* se manejarán especialmente en handle_connection_static
        
        // Inicializar Job Manager con configuración del CLI
        let job_manager_config = crate::jobs::manager::JobManagerConfig::from_config(&config);
        let job_manager = JobManager::new(job_manager_config);
        
        Self {
            config,
            router: Arc::new(router),
            metrics: Arc::new(MetricsCollector::new()),
            job_manager: Arc::new(job_manager),
            listener: None,
        }
    }
    
    pub fn run(&mut self) -> std::io::Result<()> {
        let address = self.config.address();
        println!("[*] Iniciando servidor en {}", address);
        
        let listener = TcpListener::bind(&address)?;
        println!("[+] Servidor escuchando en {}", address);
        println!("[*] Modo concurrente: un thread por conexion\n");
        
        self.listener = Some(listener);
        let listener = self.listener.as_ref().unwrap();
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let router = Arc::clone(&self.router);
                    let metrics = Arc::clone(&self.metrics);
                    let job_manager = Arc::clone(&self.job_manager);
                    
                    let peer_addr = stream.peer_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    println!(" ✅ Nueva conexión desde: {} (spawning thread)", peer_addr);
                    
                    // Incrementar contador de threads activos
                    metrics.increment_active_threads();
                    
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_connection_static(stream, router, metrics.clone(), job_manager) {
                            eprintln!("   ❌ Error en thread: {}", e);
                        }
                        // Decrementar al terminar
                        metrics.decrement_active_threads();
                    });
                }
                Err(e) => {
                    eprintln!("   ❌ Error al aceptar conexión: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_connection_static(
        mut stream: TcpStream, 
        router: Arc<Router>,
        metrics: Arc<MetricsCollector>,
        job_manager: Arc<JobManager>
    ) -> std::io::Result<()> {
        let start = Instant::now();
        
        // Generar Request ID Ãºnico
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
            println!("   ✅ Conexión cerrada");
            return Ok(());
        }
        
        println!("   ✅ {} bytes [req_id: {}]", bytes_read, &request_id[..8]);
        
        let (response, path) = match Request::parse(&buffer[..bytes_read]) {
            Ok(request) => {
                let path = request.path().to_string();
                println!("   ✅ {} {}", request.method().as_str(), path);
                
                // Manejar rutas especiales
                let response = if path == "/metrics" {
                    // MEJORADO: Incluir estadísticas de colas y workers del JobManager
                    let metrics_json = metrics.get_metrics_json();
                    let queue_stats = job_manager.get_queue_stats();
                    
                    // Combinar métricas del servidor con estadísticas de jobs
                    // Remover el último } del JSON de métricas
                    let metrics_without_closing = metrics_json.trim_end_matches('}').trim_end();
                    
                    // Agregar estadísticas de jobs
                    let combined = format!(
                        r#"{},
  "job_queues": {}
}}"#,
                        metrics_without_closing,
                        queue_stats
                    );
                    
                    Response::new(StatusCode::Ok)
                        .with_header("Content-Type", "application/json")
                        .with_body(&combined)
                } else if path.starts_with("/jobs/") {
                    // Despachar a handlers de jobs
                    if path == "/jobs/submit" {
                        job_handlers::submit_handler(&request, &job_manager)
                    } else if path == "/jobs/status" {
                        job_handlers::status_handler(&request, &job_manager)
                    } else if path == "/jobs/result" {
                        job_handlers::result_handler(&request, &job_manager)
                    } else if path == "/jobs/cancel" {
                        job_handlers::cancel_handler(&request, &job_manager)
                    } else {
                        Response::error(StatusCode::NotFound, "Unknown jobs endpoint")
                    }
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

        // NUEVO: Agregar PID del proceso actual (requerido por el proyecto)
        let process_id = std::process::id();
        response.add_header("X-Worker-Pid", &process_id.to_string());
        
        let response_bytes = response.to_bytes();
        stream.write_all(&response_bytes)?;
        stream.flush()?;
        
        let latency = start.elapsed();
        let status_code = response.status().as_u16();
        
        // Registrar mÃ©tricas
        metrics.record_request(&path, status_code, latency);
        
        println!("   ✅ {} ({:.2}ms)\n", response.status(), latency.as_secs_f64() * 1000.0);
        
        Ok(())
    }
}

#[cfg(test)]
mod more_server_tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::io::{Read, Write};
    use std::time::Duration;

    fn ephemeral_listener() -> TcpListener {
        TcpListener::bind("127.0.0.1:0").expect("bind")
    }

    #[test]
    fn test_handle_connection_help_ok() {
        let listener = ephemeral_listener();
        let addr = listener.local_addr().unwrap();

        let router = Arc::new({
            let mut r = Router::new();
            r.register("/help", commands::help_handler);
            r
        });

        let metrics = Arc::new(MetricsCollector::new());
        let job_manager = Arc::new(JobManager::new(crate::jobs::manager::JobManagerConfig::from_config(&Config::default())));

        // Servidor: aceptar y procesar una conexión
        let t = thread::spawn({
            let router = Arc::clone(&router);
            let metrics = Arc::clone(&metrics);
            let job_manager = Arc::clone(&job_manager);
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                Server::handle_connection_static(stream.try_clone().unwrap(), router, metrics, job_manager).unwrap();
            }
        });

        // Cliente: enviar GET /help
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"GET /help HTTP/1.0\r\n\r\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);

        assert!(text.contains("200 OK"));
        assert!(text.contains("X-Request-Id:"));
        assert!(text.contains("X-Worker-Thread:"));
        assert!(text.contains("X-Worker-Pid:"));

        t.join().unwrap();
    }

    #[test]
    fn test_handle_connection_metrics_ok() {
        let listener = ephemeral_listener();
        let addr = listener.local_addr().unwrap();

        let mut router = Router::new();
        // (no importa registrar nada, vamos a /metrics)
        let router = Arc::new(router);
        let metrics = Arc::new(MetricsCollector::new());
        let job_manager = Arc::new(JobManager::new(crate::jobs::manager::JobManagerConfig::from_config(&Config::default())));

        let t = thread::spawn({
            let router = Arc::clone(&router);
            let metrics = Arc::clone(&metrics);
            let job_manager = Arc::clone(&job_manager);
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                Server::handle_connection_static(stream.try_clone().unwrap(), router, metrics, job_manager).unwrap();
            }
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"GET /metrics HTTP/1.0\r\n\r\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);

        assert!(text.contains("200 OK"));
        assert!(text.contains("\"job_queues\"")); // se unió con get_queue_stats()

        t.join().unwrap();
    }

    #[test]
    fn test_handle_connection_jobs_unknown_endpoint() {
        let listener = ephemeral_listener();
        let addr = listener.local_addr().unwrap();

        let router = Arc::new(Router::new());
        let metrics = Arc::new(MetricsCollector::new());
        let job_manager = Arc::new(JobManager::new(crate::jobs::manager::JobManagerConfig::from_config(&Config::default())));

        let t = thread::spawn({
            let router = Arc::clone(&router);
            let metrics = Arc::clone(&metrics);
            let job_manager = Arc::clone(&job_manager);
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                Server::handle_connection_static(stream.try_clone().unwrap(), router, metrics, job_manager).unwrap();
            }
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"GET /jobs/unknown HTTP/1.0\r\n\r\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);

        assert!(text.contains("404 Not Found"));
        assert!(text.contains("Unknown jobs endpoint"));

        t.join().unwrap();
    }

    #[test]
    fn test_handle_connection_parse_error() {
        let listener = ephemeral_listener();
        let addr = listener.local_addr().unwrap();

        let router = Arc::new(Router::new());
        let metrics = Arc::new(MetricsCollector::new());
        let job_manager = Arc::new(JobManager::new(crate::jobs::manager::JobManagerConfig::from_config(&Config::default())));

        let t = thread::spawn({
            let router = Arc::clone(&router);
            let metrics = Arc::clone(&metrics);
            let job_manager = Arc::clone(&job_manager);
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                Server::handle_connection_static(stream.try_clone().unwrap(), router, metrics, job_manager).unwrap();
            }
        });

        // Enviar bytes no-HTTP para disparar error de parseo
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"\x00\x01\x02\x03garbage").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);

        assert!(text.contains("400 Bad Request"));
        assert!(text.contains("Invalid:"));

        t.join().unwrap();
    }

    #[test]
    fn test_handle_connection_peer_closed_immediately() {
        // Cubre rama bytes_read == 0
        let listener = ephemeral_listener();
        let addr = listener.local_addr().unwrap();

        let router = Arc::new(Router::new());
        let metrics = Arc::new(MetricsCollector::new());
        let job_manager = Arc::new(JobManager::new(crate::jobs::manager::JobManagerConfig::from_config(&Config::default())));

        let t = thread::spawn({
            let router = Arc::clone(&router);
            let metrics = Arc::clone(&metrics);
            let job_manager = Arc::clone(&job_manager);
            move || {
                let (mut stream, _) = listener.accept().unwrap();
                // No se envía nada desde el peer: el read retorna 0 y la función debe terminar Ok(())
                Server::handle_connection_static(stream, router, metrics, job_manager).unwrap();
            }
        });

        // Cliente que conecta y cierra inmediatamente sin mandar datos
        drop(TcpStream::connect(addr).unwrap());

        t.join().unwrap();
    }
}
