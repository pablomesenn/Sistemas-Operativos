//! # Comandos Básicos
//! src/commands/basic.rs
//!
//! Implementación de los comandos básicos del servidor:
//! - /status: Estado del servidor
//! - /fibonacci: Cálculo de Fibonacci
//! - /reverse: Invertir texto
//! - /toupper: Convertir a mayúsculas
//! - /timestamp: Timestamp actual
//! - /help: Ayuda sobre comandos disponibles
//! - /random: Generar números aleatorios
//! - /hash: Hash SHA256 de texto
//! - /createfile: Crear archivo con contenido
//! - /deletefile: Eliminar archivo
//! - /simulate: Simular tarea con trabajo real
//! - /sleep: Dormir N segundos
//! - /loadtest: Generar carga de prueba

use crate::http::{Request, Response, StatusCode};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fs;
use std::path::Path;

/// Handler para /status
/// 
/// Retorna información sobre el estado del servidor.
/// Por ahora retorna información básica, luego agregaremos métricas.
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "status": "running",
///   "uptime_seconds": 123,
///   "connections_served": 42
/// }
/// ```
pub fn status_handler(_req: &Request) -> Response {
    // TODO: Agregar métricas reales cuando implementemos el sistema de métricas
    let body = r#"{
  "status": "running",
  "version": "0.1.0",
  "server": "RedUnix HTTP/1.0"
}"#;
    
    Response::json(body)
}

/// Handler para /fibonacci?num=N
/// 
/// Calcula el N-ésimo número de Fibonacci.
/// 
/// # Query parameters
/// - `num`: Número entero positivo (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "num": 10,
///   "result": 55
/// }
/// ```
pub fn fibonacci_handler(req: &Request) -> Response {
    // Obtener parámetro 'num'
    let num_str = match req.query_param("num") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: num"
            );
        }
    };
    
    // Parsear a número
    let num: u64 = match num_str.parse() {
        Ok(n) => n,
        Err(_) => {
            return Response::error(
                StatusCode::BadRequest,
                "Parameter 'num' must be a valid positive integer"
            );
        }
    };
    
    // Validar rango (evitar números muy grandes que tomen mucho tiempo)
    if num > 90 {
        return Response::error(
            StatusCode::BadRequest,
            "Parameter 'num' must be <= 90 (to avoid overflow)"
        );
    }
    
    // Calcular Fibonacci
    let result = calculate_fibonacci(num);
    
    // Construir response
    let body = format!(
        r#"{{"num": {}, "result": {}}}"#,
        num, result
    );
    
    Response::json(&body)
}

/// Calcula el N-ésimo número de Fibonacci
/// 
/// Usa algoritmo iterativo para eficiencia.
fn calculate_fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    
    let mut a = 0u64;
    let mut b = 1u64;
    
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

/// Handler para /reverse?text=TEXT
/// 
/// Invierte el texto proporcionado.
/// 
/// # Query parameters
/// - `text`: Texto a invertir (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "original": "hello",
///   "reversed": "olleh"
/// }
/// ```
pub fn reverse_handler(req: &Request) -> Response {
    let text = match req.query_param("text") {
        Some(t) => t,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: text"
            );
        }
    };
    
    // Invertir el texto (manejando correctamente caracteres UTF-8)
    let reversed: String = text.chars().rev().collect();
    
    let body = format!(
        r#"{{"original": "{}", "reversed": "{}"}}"#,
        text, reversed
    );
    
    Response::json(&body)
}

/// Handler para /toupper?text=TEXT
/// 
/// Convierte el texto a mayúsculas.
/// 
/// # Query parameters
/// - `text`: Texto a convertir (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "original": "hello",
///   "upper": "HELLO"
/// }
/// ```
pub fn toupper_handler(req: &Request) -> Response {
    let text = match req.query_param("text") {
        Some(t) => t,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: text"
            );
        }
    };
    
    let upper = text.to_uppercase();
    
    let body = format!(
        r#"{{"original": "{}", "upper": "{}"}}"#,
        text, upper
    );
    
    Response::json(&body)
}

/// Handler para /timestamp
/// 
/// Retorna el timestamp actual en formato Unix (segundos desde epoch).
/// 
/// # Ejemplo de response
/// ```json
/// {
///   "timestamp": 1234567890,
///   "iso": "2024-01-01T00:00:00Z"
/// }
/// ```
pub fn timestamp_handler(_req: &Request) -> Response {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // TODO: Agregar formato ISO cuando implementemos manejo de fechas
    let body = format!(
        r#"{{"timestamp": {}}}"#,
        now
    );
    
    Response::json(&body)
}

/// Handler para /help
/// 
/// Retorna la lista de comandos disponibles con su descripción.
pub fn help_handler(_req: &Request) -> Response {
    let body = r#"{
  "commands": [
    {
      "path": "/status",
      "description": "Server status and metrics",
      "parameters": []
    },
    {
      "path": "/fibonacci",
      "description": "Calculate Fibonacci number",
      "parameters": ["num (required): integer <= 90"]
    },
    {
      "path": "/reverse",
      "description": "Reverse a text string",
      "parameters": ["text (required): string to reverse"]
    },
    {
      "path": "/toupper",
      "description": "Convert text to uppercase",
      "parameters": ["text (required): string to convert"]
    },
    {
      "path": "/timestamp",
      "description": "Get current Unix timestamp",
      "parameters": []
    },
    {
      "path": "/random",
      "description": "Generate random numbers",
      "parameters": ["count (optional): number of values", "min (optional): minimum value", "max (optional): maximum value"]
    },
    {
      "path": "/hash",
      "description": "Calculate SHA256 hash of text",
      "parameters": ["text (required): text to hash"]
    },
    {
      "path": "/createfile",
      "description": "Create a file with content",
      "parameters": ["name (required): filename", "content (required): text content", "repeat (optional): repetitions"]
    },
    {
      "path": "/deletefile",
      "description": "Delete a file",
      "parameters": ["name (required): filename"]
    },
    {
      "path": "/simulate",
      "description": "Simulate a task with real work",
      "parameters": ["seconds (required): duration", "task (optional): task name"]
    },
    {
      "path": "/sleep",
      "description": "Sleep for N seconds",
      "parameters": ["seconds (required): duration"]
    },
    {
      "path": "/loadtest",
      "description": "Generate test load",
      "parameters": ["tasks (optional): number of tasks", "sleep (optional): sleep per task in ms"]
    },
    {
      "path": "/help",
      "description": "Show this help message",
      "parameters": []
    }
  ]
}"#;
    
    Response::json(body)
}

/// Handler para /random?count=N&min=A&max=B
/// 
/// Genera números aleatorios en el rango especificado.
/// 
/// # Query parameters
/// - `count`: Cantidad de números (default: 1, max: 1000)
/// - `min`: Valor mínimo (default: 0)
/// - `max`: Valor máximo (default: 100)
pub fn random_handler(req: &Request) -> Response {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Parsear parámetros
    let count: usize = req.query_param("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(1000); // Máximo 1000 números
    
    let min: i32 = req.query_param("min")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let max: i32 = req.query_param("max")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    
    if min >= max {
        return Response::error(
            StatusCode::BadRequest,
            "Parameter 'min' must be less than 'max'"
        );
    }
    
    // Generar números pseudo-aleatorios usando el timestamp como seed
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let mut seed = hasher.finish();
    
    let range = (max - min) as u64;
    let mut numbers = Vec::new();
    
    for _ in 0..count {
        // Linear congruential generator simple
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let num = min + ((seed % range) as i32);
        numbers.push(num);
    }
    
    let body = format!(
        r#"{{"count": {}, "min": {}, "max": {}, "values": {:?}}}"#,
        count, min, max, numbers
    );
    
    Response::json(&body)
}

/// Handler para /hash?text=TEXT
/// 
/// Calcula el hash SHA256 del texto.
/// 
/// # Query parameters
/// - `text`: Texto a hashear (requerido)
pub fn hash_handler(req: &Request) -> Response {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let text = match req.query_param("text") {
        Some(t) => t,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: text"
            );
        }
    };
    
    // Usar un hash simple (DefaultHasher) por ahora
    // En producción usaríamos SHA256 real
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash_value = hasher.finish();
    
    let body = format!(
        r#"{{"text": "{}", "hash": "{:016x}", "algorithm": "simple-hash"}}"#,
        text, hash_value
    );
    
    Response::json(&body)
}

/// Handler para /createfile?name=FILE&content=TEXT&repeat=N
/// 
/// Crea un archivo con el contenido especificado.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo (requerido)
/// - `content`: Contenido del archivo (requerido)
/// - `repeat`: Número de repeticiones del contenido (default: 1, max: 10000)
pub fn createfile_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    let content = match req.query_param("content") {
        Some(c) => c,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: content"
            );
        }
    };
    
    let repeat: usize = req.query_param("repeat")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(10000); // Máximo 10000 repeticiones
    
    // Validar nombre de archivo (seguridad básica)
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename: cannot contain path separators or .."
        );
    }
    
    // Crear en el directorio data/
    let filepath = format!("./data/{}", name);
    
    // Crear directorio data/ si no existe
    if let Err(e) = fs::create_dir_all("./data") {
        return Response::error(
            StatusCode::InternalServerError,
            &format!("Failed to create data directory: {}", e)
        );
    }
    
    // Construir contenido repetido
    let full_content = content.repeat(repeat);
    
    // Escribir archivo
    match fs::write(&filepath, &full_content) {
        Ok(_) => {
            let body = format!(
                r#"{{"filename": "{}", "size": {}, "repeat": {}}}"#,
                name, full_content.len(), repeat
            );
            Response::json(&body)
        }
        Err(e) => {
            Response::error(
                StatusCode::InternalServerError,
                &format!("Failed to write file: {}", e)
            )
        }
    }
}

/// Handler para /deletefile?name=FILE
/// 
/// Elimina un archivo del directorio data/.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo (requerido)
pub fn deletefile_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    // Validar nombre de archivo
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename: cannot contain path separators or .."
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    // Verificar que el archivo existe
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    // Eliminar archivo
    match fs::remove_file(&filepath) {
        Ok(_) => {
            let body = format!(r#"{{"filename": "{}", "deleted": true}}"#, name);
            Response::json(&body)
        }
        Err(e) => {
            Response::error(
                StatusCode::InternalServerError,
                &format!("Failed to delete file: {}", e)
            )
        }
    }
}

/// Handler para /simulate?seconds=S&task=NAME
/// 
/// Simula una tarea con trabajo real (no solo sleep).
/// Realiza cálculos para consumir CPU durante el tiempo especificado.
/// 
/// # Query parameters
/// - `seconds`: Duración en segundos (requerido, max: 30)
/// - `task`: Nombre de la tarea (opcional)
pub fn simulate_handler(req: &Request) -> Response {
    let seconds: u64 = match req.query_param("seconds") {
        Some(s) => match s.parse() {
            Ok(n) if n > 0 && n <= 30 => n,
            _ => {
                return Response::error(
                    StatusCode::BadRequest,
                    "Parameter 'seconds' must be between 1 and 30"
                );
            }
        },
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: seconds"
            );
        }
    };
    
    let task_name = req.query_param("task").unwrap_or("simulation");
    
    let start = SystemTime::now();
    let target_duration = Duration::from_secs(seconds);
    
    // Hacer trabajo real (cálculos) en lugar de solo sleep
    let mut counter: u64 = 0;
    let mut result: u64 = 1;
    
    loop {
        // Calcular algo para consumir CPU
        for _ in 0..10000 {
            result = result.wrapping_mul(997).wrapping_add(counter);
            counter = counter.wrapping_add(1);
        }
        
        // Verificar si ya pasó el tiempo
        if start.elapsed().unwrap() >= target_duration {
            break;
        }
    }
    
    let elapsed = start.elapsed().unwrap().as_secs_f64();
    
    let body = format!(
        r#"{{"task": "{}", "seconds": {}, "elapsed": {:.3}, "iterations": {}}}"#,
        task_name, seconds, elapsed, counter
    );
    
    Response::json(&body)
}

/// Handler para /sleep?seconds=S
/// 
/// Duerme durante N segundos.
/// 
/// # Query parameters
/// - `seconds`: Duración en segundos (requerido, max: 10)
pub fn sleep_handler(req: &Request) -> Response {
    let seconds: u64 = match req.query_param("seconds") {
        Some(s) => match s.parse() {
            Ok(n) if n > 0 && n <= 10 => n,
            _ => {
                return Response::error(
                    StatusCode::BadRequest,
                    "Parameter 'seconds' must be between 1 and 10"
                );
            }
        },
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: seconds"
            );
        }
    };
    
    std::thread::sleep(Duration::from_secs(seconds));
    
    let body = format!(r#"{{"slept": {}}}"#, seconds);
    Response::json(&body)
}

/// Handler para /loadtest?tasks=N&sleep=X
/// 
/// Genera carga de prueba ejecutando múltiples tareas.
/// 
/// # Query parameters
/// - `tasks`: Número de tareas (default: 10, max: 100)
/// - `sleep`: Sleep por tarea en ms (default: 10, max: 1000)
pub fn loadtest_handler(req: &Request) -> Response {
    let tasks: usize = req.query_param("tasks")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
        .min(100);
    
    let sleep_ms: u64 = req.query_param("sleep")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
        .min(1000);
    
    let start = SystemTime::now();
    
    // Ejecutar tareas
    for i in 0..tasks {
        // Trabajo simple: calcular fibonacci
        let _ = calculate_fibonacci((i % 20) as u64);
        
        // Sleep pequeño
        std::thread::sleep(Duration::from_millis(sleep_ms));
    }
    
    let elapsed = start.elapsed().unwrap().as_millis();
    
    let body = format!(
        r#"{{"tasks": {}, "sleep_ms": {}, "total_time_ms": {}}}"#,
        tasks, sleep_ms, elapsed
    );
    
    Response::json(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Request, StatusCode};
    
    // Helper para crear requests de prueba
    fn make_request(path: &str) -> Request {
        let raw = format!("GET {} HTTP/1.0\r\n\r\n", path);
        Request::parse(raw.as_bytes()).unwrap()
    }
    
    // ==================== FIBONACCI ====================
    
    #[test]
    fn test_fibonacci_calculation() {
        assert_eq!(calculate_fibonacci(0), 0);
        assert_eq!(calculate_fibonacci(1), 1);
        assert_eq!(calculate_fibonacci(2), 1);
        assert_eq!(calculate_fibonacci(3), 2);
        assert_eq!(calculate_fibonacci(4), 3);
        assert_eq!(calculate_fibonacci(5), 5);
        assert_eq!(calculate_fibonacci(10), 55);
        assert_eq!(calculate_fibonacci(20), 6765);
    }
    
    #[test]
    fn test_fibonacci_handler_success() {
        let request = make_request("/fibonacci?num=10");
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("55"));
        assert!(body.contains("\"num\": 10"));
    }
    
    #[test]
    fn test_fibonacci_handler_missing_param() {
        let request = make_request("/fibonacci");
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("Missing required parameter"));
    }
    
    #[test]
    fn test_fibonacci_handler_invalid_param() {
        let request = make_request("/fibonacci?num=abc");
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_fibonacci_handler_too_large() {
        let request = make_request("/fibonacci?num=100");
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("must be <= 90"));
    }
    
    #[test]
    fn test_fibonacci_edge_cases() {
        let req0 = make_request("/fibonacci?num=0");
        let resp0 = fibonacci_handler(&req0);
        let body0 = String::from_utf8(resp0.body().to_vec()).unwrap();
        assert!(body0.contains("\"result\": 0"));
        
        let req1 = make_request("/fibonacci?num=1");
        let resp1 = fibonacci_handler(&req1);
        let body1 = String::from_utf8(resp1.body().to_vec()).unwrap();
        assert!(body1.contains("\"result\": 1"));
    }
    
    // ==================== REVERSE ====================
    
    #[test]
    fn test_reverse_handler_success() {
        let request = make_request("/reverse?text=hello");
        let response = reverse_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("olleh"));
    }
    
    #[test]
    fn test_reverse_handler_empty_string() {
        let request = make_request("/reverse?text=");
        let response = reverse_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("\"reversed\": \"\""));
    }
    
    #[test]
    fn test_reverse_handler_unicode() {
        let request = make_request("/reverse?text=🔥rust");
        let response = reverse_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        // Unicode debería manejarse correctamente
    }
    
    #[test]
    fn test_reverse_handler_missing_param() {
        let request = make_request("/reverse");
        let response = reverse_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    // ==================== TOUPPER ====================
    
    #[test]
    fn test_toupper_handler_success() {
        let request = make_request("/toupper?text=hello");
        let response = toupper_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("HELLO"));
    }
    
    #[test]
    fn test_toupper_handler_already_upper() {
        let request = make_request("/toupper?text=HELLO");
        let response = toupper_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("\"upper\": \"HELLO\""));
    }
    
    #[test]
    fn test_toupper_handler_mixed_case() {
        let request = make_request("/toupper?text=HeLLo");
        let response = toupper_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("HELLO"));
    }
    
    #[test]
    fn test_toupper_handler_missing_param() {
        let request = make_request("/toupper");
        let response = toupper_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    // ==================== STATUS ====================
    
    #[test]
    fn test_status_handler() {
        let request = make_request("/status");
        let response = status_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("running"));
        assert!(body.contains("version"));
    }
    
    // ==================== TIMESTAMP ====================
    
    #[test]
    fn test_timestamp_handler() {
        let request = make_request("/timestamp");
        let response = timestamp_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("timestamp"));
        
        // Verificar que el timestamp es un número válido
        let timestamp_str = body.split("timestamp\":").nth(1).unwrap().split('}').next().unwrap();
        let _timestamp: u64 = timestamp_str.trim().parse().expect("Should be valid number");
    }
    
    // ==================== HELP ====================
    
    #[test]
    fn test_help_handler() {
        let request = make_request("/help");
        let response = help_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("commands"));
        assert!(body.contains("fibonacci"));
        assert!(body.contains("reverse"));
    }
    
    // ==================== RANDOM ====================
    
    #[test]
    fn test_random_handler_default() {
        let request = make_request("/random");
        let response = random_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("values"));
        assert!(body.contains("count"));
    }
    
    #[test]
    fn test_random_handler_with_params() {
        let request = make_request("/random?count=5&min=10&max=20");
        let response = random_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("\"count\": 5"));
        assert!(body.contains("\"min\": 10"));
        assert!(body.contains("\"max\": 20"));
    }
    
    #[test]
    fn test_random_handler_invalid_range() {
        let request = make_request("/random?min=100&max=10");
        let response = random_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_random_handler_large_count() {
        let request = make_request("/random?count=2000");
        let response = random_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        // Debe limitar a 1000
        assert!(body.contains("\"count\": 1000"));
    }
    
    // ==================== HASH ====================
    
    #[test]
    fn test_hash_handler_success() {
        let request = make_request("/hash?text=hello");
        let response = hash_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("hash"));
        assert!(body.contains("algorithm"));
    }
    
    #[test]
    fn test_hash_handler_same_input_same_hash() {
        let req1 = make_request("/hash?text=test123");
        let resp1 = hash_handler(&req1);
        let body1 = String::from_utf8(resp1.body().to_vec()).unwrap();
        
        let req2 = make_request("/hash?text=test123");
        let resp2 = hash_handler(&req2);
        let body2 = String::from_utf8(resp2.body().to_vec()).unwrap();
        
        // Mismo input debe dar mismo hash
        assert_eq!(body1, body2);
    }
    
    #[test]
    fn test_hash_handler_missing_param() {
        let request = make_request("/hash");
        let response = hash_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    // ==================== SIMULATE ====================
    
    #[test]
    fn test_simulate_handler_success() {
        let request = make_request("/simulate?seconds=1");
        let start = std::time::Instant::now();
        let response = simulate_handler(&request);
        let elapsed = start.elapsed();
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("iterations"));
        
        // Debe tomar aproximadamente 1 segundo
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_secs() <= 2);
    }
    
    #[test]
    fn test_simulate_handler_with_task_name() {
        let request = make_request("/simulate?seconds=1&task=test_task");
        let response = simulate_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("test_task"));
    }
    
    #[test]
    fn test_simulate_handler_invalid_seconds() {
        let request = make_request("/simulate?seconds=100");
        let response = simulate_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_simulate_handler_missing_param() {
        let request = make_request("/simulate");
        let response = simulate_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    // ==================== SLEEP ====================
    
    #[test]
    fn test_sleep_handler_success() {
        let request = make_request("/sleep?seconds=1");
        let start = std::time::Instant::now();
        let response = sleep_handler(&request);
        let elapsed = start.elapsed();
        
        assert_eq!(response.status(), StatusCode::Ok);
        assert!(elapsed.as_secs() >= 1);
    }
    
    #[test]
    fn test_sleep_handler_invalid_seconds() {
        let request = make_request("/sleep?seconds=20");
        let response = sleep_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    // ==================== LOADTEST ====================
    
    #[test]
    fn test_loadtest_handler_default() {
        let request = make_request("/loadtest");
        let response = loadtest_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("tasks"));
        assert!(body.contains("total_time_ms"));
    }
    
    #[test]
    fn test_loadtest_handler_with_params() {
        let request = make_request("/loadtest?tasks=5&sleep=1");
        let response = loadtest_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("\"tasks\": 5"));
    }
    
    // ==================== FILE OPERATIONS ====================
    
    #[test]
    fn test_createfile_handler_success() {
        let request = make_request("/createfile?name=test.txt&content=hello");
        let response = createfile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        
        // Limpiar
        let _ = std::fs::remove_file("./data/test.txt");
    }
    
    #[test]
    fn test_createfile_handler_with_repeat() {
        let request = make_request("/createfile?name=test_repeat.txt&content=x&repeat=100");
        let response = createfile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("\"size\": 100"));
        
        // Limpiar
        let _ = std::fs::remove_file("./data/test_repeat.txt");
    }
    
    #[test]
    fn test_createfile_handler_invalid_name() {
        let request = make_request("/createfile?name=../etc/passwd&content=hack");
        let response = createfile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("Invalid filename"));
    }
    
    #[test]
    fn test_createfile_handler_missing_params() {
        let request = make_request("/createfile?name=test.txt");
        let response = createfile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_deletefile_handler_success() {
        // Crear archivo primero
        std::fs::create_dir_all("./data").ok();
        std::fs::write("./data/test_delete.txt", "test").unwrap();
        
        let request = make_request("/deletefile?name=test_delete.txt");
        let response = deletefile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        
        // Verificar que se eliminó
        assert!(!std::path::Path::new("./data/test_delete.txt").exists());
    }
    
    #[test]
    fn test_deletefile_handler_not_found() {
        let request = make_request("/deletefile?name=nonexistent.txt");
        let response = deletefile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::NotFound);
    }
    
    #[test]
    fn test_deletefile_handler_invalid_name() {
        let request = make_request("/deletefile?name=../etc/passwd");
        let response = deletefile_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
}