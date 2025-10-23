//! # Configuración del Servidor
//!
//! Este módulo define la configuración del servidor HTTP.
//! La configuración puede venir de:
//! - Valores por defecto
//! - Variables de entorno
//! - Argumentos de línea de comandos (CLI)
//!
//! Por ahora empezamos simple, luego agregaremos más opciones.

/// Configuración del servidor HTTP
#[derive(Debug, Clone)]
pub struct Config {
    /// Puerto en el que escucha el servidor
    pub port: u16,
    
    /// Host/IP en el que escucha (por defecto 127.0.0.1)
    pub host: String,
    
    /// Directorio donde se guardan/leen archivos (data/)
    pub data_dir: String,
}

impl Config {
    /// Crea una nueva configuración con valores personalizados
    pub fn new(port: u16, host: String, data_dir: String) -> Self {
        Self {
            port,
            host,
            data_dir,
        }
    }
    
    /// Crea una configuración leyendo variables de entorno
    /// 
    /// Variables soportadas:
    /// - `HTTP_PORT`: Puerto (default: 8080)
    /// - `HTTP_HOST`: Host (default: 127.0.0.1)
    /// - `DATA_DIR`: Directorio de datos (default: ./data)
    pub fn from_env() -> Self {
        let port = std::env::var("HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        let host = std::env::var("HTTP_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let data_dir = std::env::var("DATA_DIR")
            .unwrap_or_else(|_| "./data".to_string());
        
        Self {
            port,
            host,
            data_dir,
        }
    }
    
    /// Obtiene la dirección completa para bind (host:port)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::config::Config;
    /// 
    /// let config = Config::default();
    /// assert_eq!(config.address(), "127.0.0.1:8080");
    /// ```
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    /// Configuración por defecto
    /// 
    /// - Puerto: 8080
    /// - Host: 127.0.0.1
    /// - Data dir: ./data
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            data_dir: "./data".to_string(),
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
        assert_eq!(config.data_dir, "./data");
    }
    
    #[test]
    fn test_new_config() {
        let config = Config::new(3000, "0.0.0.0".to_string(), "/tmp/data".to_string());
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.data_dir, "/tmp/data");
    }
    
    #[test]
    fn test_address() {
        let config = Config::default();
        assert_eq!(config.address(), "127.0.0.1:8080");
        
        let config2 = Config::new(3000, "0.0.0.0".to_string(), "./data".to_string());
        assert_eq!(config2.address(), "0.0.0.0:3000");
    }
}