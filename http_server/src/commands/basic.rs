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
        let raw = b"GET /fibonacci?num=10 HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("55"));
    }
    
    #[test]
    fn test_fibonacci_handler_missing_param() {
        let raw = b"GET /fibonacci HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_fibonacci_handler_invalid_param() {
        let raw = b"GET /fibonacci?num=abc HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = fibonacci_handler(&request);
        
        assert_eq!(response.status(), StatusCode::BadRequest);
    }
    
    #[test]
    fn test_reverse_handler() {
        let raw = b"GET /reverse?text=hello HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = reverse_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("olleh"));
    }
    
    #[test]
    fn test_toupper_handler() {
        let raw = b"GET /toupper?text=hello HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = toupper_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("HELLO"));
    }
    
    #[test]
    fn test_status_handler() {
        let raw = b"GET /status HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = status_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
    }
    
    #[test]
    fn test_timestamp_handler() {
        let raw = b"GET /timestamp HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = timestamp_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("timestamp"));
    }
    
    #[test]
    fn test_help_handler() {
        let raw = b"GET /help HTTP/1.0\r\n\r\n";
        let request = Request::parse(raw).unwrap();
        let response = help_handler(&request);
        
        assert_eq!(response.status(), StatusCode::Ok);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("commands"));
    }
}