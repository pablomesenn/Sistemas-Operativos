//! # Configuración del Servidor
//! src/config.rs
//! 
//! Este módulo define la configuración del servidor HTTP con soporte completo
//! para argumentos CLI y variables de entorno.
//!
//! ## Ejemplos de uso
//!
//! ### CLI
//! ```bash
//! ./http_server --port 8080 \
//!   --workers-cpu 4 \
//!   --workers-io 4 \
//!   --queue-cpu 64 \
//!   --timeout-cpu 60000
//! ```
//!
//! ### Variables de entorno
//! ```bash
//! HTTP_PORT=8080 HTTP_HOST=0.0.0.0 ./http_server
//! ```

use clap::Parser;

/// Configuración del servidor HTTP/1.0
#[derive(Debug, Clone, Parser)]
#[command(name = "http_server")]
#[command(about = "Servidor HTTP/1.0 concurrente para Principios de Sistemas Operativos")]
#[command(version = "0.1.0")]
pub struct Config {
    /// Puerto en el que escucha el servidor
    #[arg(short, long, default_value = "8080", env = "HTTP_PORT")]
    pub port: u16,
    
    /// Host/IP en el que escucha
    #[arg(long, default_value = "127.0.0.1", env = "HTTP_HOST")]
    pub host: String,
    
    /// Directorio donde se guardan/leen archivos
    #[arg(long, default_value = "./data", env = "DATA_DIR")]
    pub data_dir: String,
    
    // === Workers ===
    
    /// Número de workers para comandos CPU-bound (isprime, factor, pi, etc.)
    #[arg(long = "workers-cpu", default_value = "4", env = "WORKERS_CPU")]
    pub cpu_workers: usize,
    
    /// Número de workers para comandos IO-bound (sortfile, compress, etc.)
    #[arg(long = "workers-io", default_value = "4", env = "WORKERS_IO")]
    pub io_workers: usize,
    
    /// Número de workers para comandos básicos (fibonacci, reverse, etc.)
    #[arg(long = "workers-basic", default_value = "2", env = "WORKERS_BASIC")]
    pub basic_workers: usize,
    
    // === Colas ===
    
    /// Capacidad máxima de la cola CPU-bound
    #[arg(long = "queue-cpu", default_value = "1000", env = "QUEUE_CPU")]
    pub cpu_queue_capacity: usize,
    
    /// Capacidad máxima de la cola IO-bound
    #[arg(long = "queue-io", default_value = "1000", env = "QUEUE_IO")]
    pub io_queue_capacity: usize,
    
    /// Capacidad máxima de la cola básica
    #[arg(long = "queue-basic", default_value = "500", env = "QUEUE_BASIC")]
    pub basic_queue_capacity: usize,
    
    // === Timeouts ===
    
    /// Timeout para jobs CPU-bound en milisegundos
    #[arg(long = "timeout-cpu", default_value = "60000", env = "TIMEOUT_CPU")]
    pub cpu_timeout_ms: u64,
    
    /// Timeout para jobs IO-bound en milisegundos
    #[arg(long = "timeout-io", default_value = "120000", env = "TIMEOUT_IO")]
    pub io_timeout_ms: u64,
    
    /// Timeout para jobs básicos en milisegundos
    #[arg(long = "timeout-basic", default_value = "30000", env = "TIMEOUT_BASIC")]
    pub basic_timeout_ms: u64,
    
    // === Backpressure ===
    
    /// Umbral de cola para activar backpressure (porcentaje 0-100)
    /// Cuando la cola supera este porcentaje, se devuelve 503
    #[arg(long = "backpressure-threshold", default_value = "90", env = "BACKPRESSURE_THRESHOLD")]
    pub backpressure_threshold: u8,
    
    /// Tiempo de reintento sugerido en milisegundos cuando hay backpressure
    #[arg(long = "retry-after-ms", default_value = "5000", env = "RETRY_AFTER_MS")]
    pub retry_after_ms: u64,
    
    // === Rate Limiting ===
    
    /// Máximo de requests por segundo por IP (0 = sin límite)
    #[arg(long = "rate-limit", default_value = "0", env = "RATE_LIMIT")]
    pub rate_limit_per_sec: u32,
    
    // === Storage ===
    
    /// Ruta del archivo de persistencia de jobs
    #[arg(long = "jobs-storage", default_value = "./data/jobs.json", env = "JOBS_STORAGE")]
    pub jobs_storage_path: String,
    
    /// Tiempo en segundos para limpiar jobs antiguos
    #[arg(long = "jobs-cleanup-age", default_value = "3600", env = "JOBS_CLEANUP_AGE")]
    pub jobs_cleanup_age_secs: u64,
}

impl Config {
    /// Crea una nueva configuración parseando argumentos CLI
    /// 
    /// # Ejemplo
    /// ```rust
    /// use http_server::config::Config;
    /// 
    /// let config = Config::new();
    /// println!("Server listening on {}", config.address());
    /// ```
    pub fn new() -> Self {
        Config::parse()
    }
    
    /// Obtiene la dirección completa para bind (host:port)
    /// 
    /// # Ejemplo
    /// ```rust
    /// use http_server::config::Config;
    /// 
    /// let config = Config::new();
    /// assert_eq!(config.address(), "127.0.0.1:8080");
    /// ```
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    
    /// Valida la configuración
    /// 
    /// Retorna errores si hay valores inválidos
    pub fn validate(&self) -> Result<(), String> {
        // Validar workers
        if self.cpu_workers == 0 {
            return Err("CPU workers must be >= 1".to_string());
        }
        if self.io_workers == 0 {
            return Err("IO workers must be >= 1".to_string());
        }
        if self.basic_workers == 0 {
            return Err("Basic workers must be >= 1".to_string());
        }
        
        // Validar colas
        if self.cpu_queue_capacity == 0 {
            return Err("CPU queue capacity must be >= 1".to_string());
        }
        if self.io_queue_capacity == 0 {
            return Err("IO queue capacity must be >= 1".to_string());
        }
        
        // Validar timeouts
        if self.cpu_timeout_ms == 0 {
            return Err("CPU timeout must be > 0".to_string());
        }
        if self.io_timeout_ms == 0 {
            return Err("IO timeout must be > 0".to_string());
        }
        
        // Validar backpressure threshold
        if self.backpressure_threshold > 100 {
            return Err("Backpressure threshold must be 0-100".to_string());
        }
        
        Ok(())
    }
    
    /// Imprime un resumen de la configuración
    pub fn print_summary(&self) {
        println!("⚙️  Configuración del Servidor:");
        println!("   📍 Dirección: {}", self.address());
        println!("   📁 Data dir: {}", self.data_dir);
        println!();
        println!("   👷 Workers:");
        println!("      CPU-bound: {}", self.cpu_workers);
        println!("      IO-bound:  {}", self.io_workers);
        println!("      Básicos:   {}", self.basic_workers);
        println!();
        println!("   📦 Capacidad de Colas:");
        println!("      CPU:    {}", self.cpu_queue_capacity);
        println!("      IO:     {}", self.io_queue_capacity);
        println!("      Básica: {}", self.basic_queue_capacity);
        println!();
        println!("   ⏱️  Timeouts (ms):");
        println!("      CPU:    {}", self.cpu_timeout_ms);
        println!("      IO:     {}", self.io_timeout_ms);
        println!("      Básico: {}", self.basic_timeout_ms);
        println!();
        println!("   🚦 Backpressure:");
        println!("      Umbral:      {}%", self.backpressure_threshold);
        println!("      Retry after: {}ms", self.retry_after_ms);
        
        if self.rate_limit_per_sec > 0 {
            println!();
            println!("   🛡️  Rate Limit: {} req/sec por IP", self.rate_limit_per_sec);
        }
        println!();
    }
}

impl Default for Config {
    /// Configuración por defecto
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            data_dir: "./data".to_string(),
            cpu_workers: 4,
            io_workers: 4,
            basic_workers: 2,
            cpu_queue_capacity: 1000,
            io_queue_capacity: 1000,
            basic_queue_capacity: 500,
            cpu_timeout_ms: 60_000,
            io_timeout_ms: 120_000,
            basic_timeout_ms: 30_000,
            backpressure_threshold: 90,
            retry_after_ms: 5_000,
            rate_limit_per_sec: 0,
            jobs_storage_path: "./data/jobs.json".to_string(),
            jobs_cleanup_age_secs: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.cpu_workers, 4);
    }
    
    #[test]
    fn test_address() {
        let config = Config::default();
        assert_eq!(config.address(), "127.0.0.1:8080");
    }
    
    #[test]
    fn test_validate_success() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validate_invalid_workers() {
        let mut config = Config::default();
        config.cpu_workers = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_validate_invalid_backpressure() {
        let mut config = Config::default();
        config.backpressure_threshold = 150;
        assert!(config.validate().is_err());
    }
}