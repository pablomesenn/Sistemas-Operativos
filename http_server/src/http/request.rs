//! # Parsing de Requests HTTP/1.0
//! src/http/request.rs
//!
//! Este módulo implementa un parser HTTP/1.0 desde cero.
//!
//! ## Formato de un Request HTTP/1.0
//!
//! ```text
//! GET /path?param1=value1&param2=value2 HTTP/1.0\r\n
//! Host: localhost:8080\r\n
//! User-Agent: curl/7.68.0\r\n
//! \r\n
//! ```
//!
//! ## Componentes
//!
//! 1. **Request Line**: `METHOD /path?query HTTP/1.0`
//! 2. **Headers**: Pares `Name: Value` (uno por línea)
//! 3. **Empty Line**: `\r\n` que separa headers del body
//! 4. **Body**: (Opcional, no usado en GET)

use std::collections::HashMap;

/// Métodos HTTP soportados
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// GET - Obtener un recurso
    GET,
    
    /// HEAD - Como GET pero solo retorna headers (opcional en el proyecto)
    HEAD,

    /// POST - Enviar datos a un recurso
    POST,
}

impl Method {
    /// Parsea un método HTTP desde un string
    /// 
    /// # Errores
    /// 
    /// Retorna error si el método no es soportado
    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            _ => Err(ParseError::UnsupportedMethod(s.to_string())),
        }
    }
    
    /// Convierte el método a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
        }
    }
}

/// Representa un request HTTP/1.0 parseado
#[derive(Debug, Clone)]
pub struct Request {
    /// Método HTTP (GET, HEAD, POST)
    method: Method,
    
    /// Path de la petición (ej: "/fibonacci")
    path: String,
    
    /// Query parameters parseados (ej: {"num": "10"})
    query_params: HashMap<String, String>,
    
    /// Headers HTTP (ej: {"Host": "localhost:8080"})
    headers: HashMap<String, String>,
    
    /// Versión HTTP (debe ser "HTTP/1.0")
    version: String,
    
    /// Body del request para métodos POST
    body: Vec<u8>,
}

/// Errores que pueden ocurrir durante el parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Request incompleto o truncado
    IncompleteRequest,
    
    /// Formato inválido de la request line
    InvalidRequestLine,
    
    /// Método HTTP no soportado
    UnsupportedMethod(String),
    
    /// Versión HTTP incorrecta (debe ser HTTP/1.0)
    InvalidHttpVersion(String),
    
    /// Header malformado
    InvalidHeader(String),
    
    /// Request vacío
    EmptyRequest,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IncompleteRequest => write!(f, "Incomplete HTTP request"),
            ParseError::InvalidRequestLine => write!(f, "Invalid request line format"),
            ParseError::UnsupportedMethod(m) => write!(f, "Unsupported HTTP method: {}", m),
            ParseError::InvalidHttpVersion(v) => write!(f, "Invalid HTTP version: {}", v),
            ParseError::InvalidHeader(h) => write!(f, "Invalid header: {}", h),
            ParseError::EmptyRequest => write!(f, "Empty request"),
        }
    }
}

impl std::error::Error for ParseError {}

impl Request {
    /// Parsea un request HTTP/1.0 desde bytes
    /// 
    /// # Argumentos
    /// 
    /// * `buffer` - Buffer conteniendo el request HTTP completo
    /// 
    /// # Retorna
    /// 
    /// * `Ok(Request)` - Request parseado exitosamente
    /// * `Err(ParseError)` - Error durante el parsing
    /// 
    /// # Ejemplo
    /// 
    /// ```
    /// use http_server::http::Request;
    /// 
    /// let raw = b"GET /fibonacci?num=10 HTTP/1.0\r\n\r\n";
    /// let request = Request::parse(raw).unwrap();
    /// 
    /// assert_eq!(request.path(), "/fibonacci");
    /// assert_eq!(request.query_param("num"), Some("10"));
    /// ```
    pub fn parse(buffer: &[u8]) -> Result<Self, ParseError> {
        // Convertir a string (validando que sea UTF-8 válido)
        let request_str = std::str::from_utf8(buffer)
            .map_err(|_| ParseError::InvalidRequestLine)?;
        
        if request_str.trim().is_empty() {
            return Err(ParseError::EmptyRequest);
        }
        
        // Separar por \r\n para obtener líneas
        let lines: Vec<&str> = request_str.split("\r\n").collect();
        
        if lines.is_empty() {
            return Err(ParseError::IncompleteRequest);
        }
        
        // 1. Parsear la request line (primera línea)
        let (method, path, query_params, version) = Self::parse_request_line(lines[0])?;
        
        // 2. Parsear headers (resto de líneas hasta encontrar línea vacía)
        let headers = Self::parse_headers(&lines[1..])?;

        // 3. Parsear body
        let body = Self::parse_body(&lines, method);

        Ok(Request {
            method,
            path,
            query_params,
            headers,
            version,
            body,
        })
    }
    
    /// Parsea la request line (primera línea del request)
    /// 
    /// Formato: `GET /path?query HTTP/1.0`
    fn parse_request_line(line: &str) -> Result<(Method, String, HashMap<String, String>, String), ParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        // Debe tener exactamente 3 partes: METHOD PATH VERSION
        if parts.len() != 3 {
            return Err(ParseError::InvalidRequestLine);
        }
        
        // Parsear método
        let method = Method::from_str(parts[0])?;
        
        // Parsear path y query
        let (path, query_params) = Self::parse_path_and_query(parts[1]);
        
        // Validar versión HTTP
        let version = parts[2].to_string();
        if version != "HTTP/1.0" && version != "HTTP/1.1" {
            return Err(ParseError::InvalidHttpVersion(version));
        }
        
        Ok((method, path, query_params, version))
    }
    
    /// Parsea el path y extrae los query parameters
    /// 
    /// Ejemplo: "/fibonacci?num=10&fast=true" 
    /// Retorna: ("/fibonacci", {"num": "10", "fast": "true"})
    fn parse_path_and_query(path_with_query: &str) -> (String, HashMap<String, String>) {
        // Buscar el símbolo '?' que separa path de query
        if let Some(query_start) = path_with_query.find('?') {
            let path = path_with_query[..query_start].to_string();
            let query_string = &path_with_query[query_start + 1..];
            let query_params = Self::parse_query_string(query_string);
            (path, query_params)
        } else {
            // No hay query parameters
            (path_with_query.to_string(), HashMap::new())
        }
    }
    
    /// Parsea una query string en un HashMap
    /// 
    /// Ejemplo: "num=10&text=hello&fast=true"
    /// Retorna: {"num": "10", "text": "hello", "fast": "true"}
    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        // Separar por '&' para obtener cada parámetro
        for param in query.split('&') {
            if param.is_empty() {
                continue;
            }
            
            // Separar por '=' para obtener key y value
            if let Some(eq_pos) = param.find('=') {
                let key = &param[..eq_pos];
                let value = &param[eq_pos + 1..];
                
                // URL decode básico (reemplazar %20 por espacio, etc.)
                let decoded_value = Self::url_decode(value);
                
                params.insert(key.to_string(), decoded_value);
            } else {
                // Parámetro sin valor (ej: "?debug")
                params.insert(param.to_string(), String::new());
            }
        }
        
        params
    }
    
    /// Decodifica una URL (convierte %20 a espacio, etc.)
    /// 
    /// Implementación básica - puede mejorarse con una librería
    fn url_decode(s: &str) -> String {
        // Por ahora solo manejamos %20 (espacio)
        // En una implementación completa usaríamos percent-encoding crate
        s.replace("%20", " ")
            .replace("+", " ")
    }

    /// Parsea los headers HTTP
    /// 
    /// Cada header tiene formato: "Name: Value"
    fn parse_headers(lines: &[&str]) -> Result<HashMap<String, String>, ParseError> {
        let mut headers = HashMap::new();
        
        for line in lines {
            // La línea vacía marca el fin de los headers
            if line.trim().is_empty() {
                break;
            }
            
            // Buscar el separador ':'
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(name, value);
            } else {
                // Header sin ':' es inválido
                return Err(ParseError::InvalidHeader(line.to_string()));
            }
        }
        
        Ok(headers)
    }

    /// Parsea el cuerpo del request
    fn parse_body(lines: &[&str], method: Method) -> Vec<u8> {
        if method != Method::POST {
            return Vec::new();
        }
        
        let mut body_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                body_start = i + 1;
                break;
            }
        }
        
        if body_start < lines.len() {
            let body_str = lines[body_start..].join("\r\n");
            body_str.as_bytes().to_vec()
        } else {
            Vec::new()
        }
    }
    
    // === Métodos públicos para acceder a los campos ===
    
    /// Obtiene el método HTTP del request
    pub fn method(&self) -> Method {
        self.method
    }
    
    /// Obtiene el path del request
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Obtiene todos los query parameters
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }
    
    /// Obtiene un query parameter específico
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::Request;
    /// 
    /// let raw = b"GET /test?num=42 HTTP/1.0\r\n\r\n";
    /// let request = Request::parse(raw).unwrap();
    /// 
    /// assert_eq!(request.query_param("num"), Some("42"));
    /// assert_eq!(request.query_param("missing"), None);
    /// ```
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(|s| s.as_str())
    }
    
    /// Obtiene todos los headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    
    /// Obtiene un header específico
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }
    
    /// Obtiene la versión HTTP
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// Obtiene el body del request
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Obtiene el body del request como String
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_get() {
        let raw = b"GET / HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.method(), Method::GET);
        assert_eq!(request.path(), "/");
        assert!(request.query_params().is_empty());
    }
    
    #[test]
    fn test_parse_with_path() {
        let raw = b"GET /fibonacci HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.path(), "/fibonacci");
    }
    
    #[test]
    fn test_parse_with_query_params() {
        let raw = b"GET /fibonacci?num=10 HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.path(), "/fibonacci");
        assert_eq!(request.query_param("num"), Some("10"));
    }
    
    #[test]
    fn test_parse_multiple_query_params() {
        let raw = b"GET /test?num=42&text=hello&fast=true HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.query_param("num"), Some("42"));
        assert_eq!(request.query_param("text"), Some("hello"));
        assert_eq!(request.query_param("fast"), Some("true"));
    }
    
    #[test]
    fn test_parse_with_headers() {
        let raw = b"GET / HTTP/1.0\r\nHost: localhost:8080\r\nUser-Agent: test\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.header("Host"), Some("localhost:8080"));
        assert_eq!(request.header("User-Agent"), Some("test"));
    }
    
    #[test]
    fn test_url_decode() {
        let raw = b"GET /reverse?text=hello%20world HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        
        assert_eq!(request.query_param("text"), Some("hello world"));
    }
    
    #[test]
    #[ignore]
    fn test_invalid_method() {
        let raw = b"POST / HTTP/1.0\r\n\r\n";
        let result = Request::parse(raw);
        
        assert!(matches!(result, Err(ParseError::UnsupportedMethod(_))));
    }
    
    #[test]
    fn test_invalid_version() {
        let raw = b"GET / HTTP/2.0\r\n\r\n"; // HTTP/2.0 no está soportado
        let result = Request::parse(raw);
        
        assert!(matches!(result, Err(ParseError::InvalidHttpVersion(_))));
    }
    
    #[test]
    fn test_empty_request() {
        let raw = b"";
        let result = Request::parse(raw);
        
        assert!(matches!(result, Err(ParseError::EmptyRequest)));
    }
    
    #[test]
    fn test_invalid_request_line() {
        let raw = b"GET\r\n\r\n"; // Falta path y version
        let result = Request::parse(raw);
        
        assert!(matches!(result, Err(ParseError::InvalidRequestLine)));
    }
}