//! # Códigos de Estado HTTP
//!
//! Este módulo define los códigos de estado HTTP/1.0 que usará el servidor.
//! Según el RFC 1945, HTTP/1.0 define códigos en 5 categorías:
//!
//! - **1xx**: Informacional (no se usan en HTTP/1.0)
//! - **2xx**: Éxito (200 OK)
//! - **3xx**: Redirección (no implementadas por ahora)
//! - **4xx**: Error del cliente (400, 404, 409, 429)
//! - **5xx**: Error del servidor (500, 503)

/// Representa los códigos de estado HTTP que soporta nuestro servidor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// 200 OK - La petición fue exitosa
    Ok = 200,
    
    /// 204 No Content - Petición exitosa sin contenido en el body
    NoContent = 204,
    
    /// 400 Bad Request - Parámetros inválidos o malformados
    BadRequest = 400,
    
    /// 404 Not Found - Ruta o recurso no encontrado
    NotFound = 404,
    
    /// 409 Conflict - Conflicto en el estado del recurso (ej: job no disponible aún)
    Conflict = 409,
    
    /// 429 Too Many Requests - Rate limiting activado
    TooManyRequests = 429,
    
    /// 500 Internal Server Error - Error interno del servidor
    InternalServerError = 500,
    
    /// 503 Service Unavailable - Colas llenas o servidor sobrecargado
    ServiceUnavailable = 503,
}

impl StatusCode {
    /// Convierte el código a su valor numérico
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::StatusCode;
    /// assert_eq!(StatusCode::Ok.as_u16(), 200);
    /// ```
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
    
    /// Retorna el texto de razón (reason phrase) asociado al código
    /// 
    /// Estos textos están definidos en el RFC 1945 y son estándares.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::StatusCode;
    /// assert_eq!(StatusCode::Ok.reason_phrase(), "OK");
    /// assert_eq!(StatusCode::NotFound.reason_phrase(), "Not Found");
    /// ```
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::NoContent => "No Content",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::NotFound => "Not Found",
            StatusCode::Conflict => "Conflict",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }
    
    /// Verifica si el código indica éxito (2xx)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::StatusCode;
    /// assert!(StatusCode::Ok.is_success());
    /// assert!(StatusCode::NoContent.is_success());
    /// assert!(!StatusCode::NotFound.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        matches!(self, StatusCode::Ok | StatusCode::NoContent)
    }
    
    /// Verifica si el código indica error del cliente (4xx)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::StatusCode;
    /// assert!(StatusCode::BadRequest.is_client_error());
    /// assert!(!StatusCode::Ok.is_client_error());
    /// ```
    pub fn is_client_error(&self) -> bool {
        let code = self.as_u16();
        (400..500).contains(&code)
    }
    
    /// Verifica si el código indica error del servidor (5xx)
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::http::StatusCode;
    /// assert!(StatusCode::InternalServerError.is_server_error());
    /// assert!(!StatusCode::BadRequest.is_server_error());
    /// ```
    pub fn is_server_error(&self) -> bool {
        let code = self.as_u16();
        (500..600).contains(&code)
    }
}

impl std::fmt::Display for StatusCode {
    /// Formatea el código de estado para mostrarlo
    /// 
    /// Formato: "200 OK"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.as_u16(), self.reason_phrase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_status_code_values() {
        assert_eq!(StatusCode::Ok.as_u16(), 200);
        assert_eq!(StatusCode::BadRequest.as_u16(), 400);
        assert_eq!(StatusCode::NotFound.as_u16(), 404);
        assert_eq!(StatusCode::InternalServerError.as_u16(), 500);
    }
    
    #[test]
    fn test_reason_phrases() {
        assert_eq!(StatusCode::Ok.reason_phrase(), "OK");
        assert_eq!(StatusCode::BadRequest.reason_phrase(), "Bad Request");
        assert_eq!(StatusCode::ServiceUnavailable.reason_phrase(), "Service Unavailable");
    }
    
    #[test]
    fn test_is_success() {
        assert!(StatusCode::Ok.is_success());
        assert!(!StatusCode::BadRequest.is_success());
        assert!(!StatusCode::InternalServerError.is_success());
    }
    
    #[test]
    fn test_is_client_error() {
        assert!(!StatusCode::Ok.is_client_error());
        assert!(StatusCode::BadRequest.is_client_error());
        assert!(StatusCode::NotFound.is_client_error());
        assert!(!StatusCode::InternalServerError.is_client_error());
    }
    
    #[test]
    fn test_is_server_error() {
        assert!(!StatusCode::Ok.is_server_error());
        assert!(!StatusCode::BadRequest.is_server_error());
        assert!(StatusCode::InternalServerError.is_server_error());
        assert!(StatusCode::ServiceUnavailable.is_server_error());
    }
    
    #[test]
    fn test_display() {
        assert_eq!(StatusCode::Ok.to_string(), "200 OK");
        assert_eq!(StatusCode::NotFound.to_string(), "404 Not Found");
        assert_eq!(StatusCode::InternalServerError.to_string(), "500 Internal Server Error");
    }
}