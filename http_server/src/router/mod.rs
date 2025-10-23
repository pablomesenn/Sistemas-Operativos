//! # Sistema de Routing
//! src/router/mod.rs
//!
//! Este módulo implementa el router que mapea paths HTTP a handlers específicos.
//!
//! ## Arquitectura
//!
//! ```text
//! Request → Router → Handler → Response
//! ```
//!
//! El router examina el path del request y lo dirige al handler apropiado.
//! Si no hay handler para ese path, retorna 404 Not Found.

use crate::http::{Request, Response, StatusCode};

/// Tipo de función handler
/// 
/// Un handler recibe un Request y retorna una Response
pub type Handler = fn(&Request) -> Response;

/// Router que mapea paths a handlers
pub struct Router {
    /// Mapa de path → handler
    routes: Vec<(String, Handler)>,
}

impl Router {
    /// Crea un nuevo router vacío
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }
    
    /// Registra una ruta con su handler
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::router::Router;
    /// use http_server::http::{Request, Response, StatusCode};
    /// 
    /// fn hello_handler(req: &Request) -> Response {
    ///     Response::json(r#"{"message": "Hello"}"#)
    /// }
    /// 
    /// let mut router = Router::new();
    /// router.register("/hello", hello_handler);
    /// ```
    pub fn register(&mut self, path: &str, handler: Handler) {
        self.routes.push((path.to_string(), handler));
    }
    
    /// Encuentra y ejecuta el handler apropiado para un request
    /// 
    /// Si no encuentra un handler para el path, retorna 404 Not Found.
    /// 
    /// # Ejemplo
    /// ```
    /// use http_server::router::Router;
    /// use http_server::http::{Request, Response};
    /// 
    /// let mut router = Router::new();
    /// // ... registrar handlers ...
    /// 
    /// let raw = b"GET /test HTTP/1.0\r\n\r\n";
    /// let request = Request::parse(raw).unwrap();
    /// let response = router.route(&request);
    /// ```
    pub fn route(&self, request: &Request) -> Response {
        let path = request.path();
        
        // Buscar handler para este path
        for (route_path, handler) in &self.routes {
            if route_path == path {
                // Encontramos el handler, ejecutarlo
                let mut response = handler(request);
                // Agregar headers comunes a todas las respuestas
                self.add_common_headers(&mut response);
                return response;
            }
        }
        
        // No se encontró handler para este path
        let mut response = Response::error(
            StatusCode::NotFound,
            &format!("Route not found: {}", path)
        );
        self.add_common_headers(&mut response);
        response
    }
    
    /// Agrega headers comunes a todas las respuestas
    fn add_common_headers(&self, response: &mut Response) {
        response.add_header("Server", "RedUnix-HTTP/1.0");
        response.add_header("Connection", "close");
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_handler(_req: &Request) -> Response {
        Response::json(r#"{"test": "ok"}"#)
    }
    
    fn hello_handler(_req: &Request) -> Response {
        Response::json(r#"{"message": "hello"}"#)
    }
    
    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.routes.len(), 0);
    }
    
    #[test]
    fn test_register_route() {
        let mut router = Router::new();
        router.register("/test", test_handler);
        
        assert_eq!(router.routes.len(), 1);
    }
    
    #[test]
    fn test_route_found() {
        let mut router = Router::new();
        router.register("/test", test_handler);
        
        let raw = b"GET /test HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = router.route(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
    }
    
    #[test]
    fn test_route_not_found() {
        let router = Router::new();
        
        let raw = b"GET /nonexistent HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = router.route(&request);
        
        assert_eq!(response.status(), StatusCode::NotFound);
    }
    
    #[test]
    fn test_multiple_routes() {
        let mut router = Router::new();
        router.register("/test", test_handler);
        router.register("/hello", hello_handler);
        
        let raw1 = b"GET /test HTTP/1.0\r\n\r\n";
        let request1 = Request::parse(raw1).unwrap();
        let response1 = router.route(&request1);
        assert_eq!(response1.status(), StatusCode::Ok);
        
        let raw2 = b"GET /hello HTTP/1.0\r\n\r\n";
        let request2 = Request::parse(raw2).unwrap();
        let response2 = router.route(&request2);
        assert_eq!(response2.status(), StatusCode::Ok);
    }
}