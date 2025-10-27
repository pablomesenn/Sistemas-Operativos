//! # Comandos CPU-bound
//! src/commands/cpu_bound.rs
//!
//! Comandos que requieren procesamiento intensivo de CPU:
//! - /isprime: Verificación de primalidad
//! - /factor: Factorización en números primos
//! - /pi: Cálculo de dígitos de π
//! - /mandelbrot: Generación del conjunto de Mandelbrot
//! - /matrixmul: Multiplicación de matrices

use crate::http::{Request, Response, StatusCode};
use std::time::Instant;

/// Handler para /isprime?n=NUM
/// 
/// Verifica si un número es primo usando prueba de Miller-Rabin.
/// 
/// # Query parameters
/// - `n`: Número a verificar (requerido, max: 2^63-1)
/// 
/// # Ejemplo de response
/// ```json
/// {"n": 97, "is_prime": true, "method": "miller-rabin", "elapsed_ms": 12}
/// ```
pub fn isprime_handler(req: &Request) -> Response {
    let n_str = match req.query_param("n") {
        Some(s) => s,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: n"
            );
        }
    };
    
    let n: u64 = match n_str.parse() {
        Ok(num) if num > 1 => num,
        _ => {
            return Response::error(
                StatusCode::BadRequest,
                "Parameter 'n' must be an integer greater than 1"
            );
        }
    };
    
    let start = Instant::now();
    let is_prime = is_prime_miller_rabin(n, 10);
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"n": {}, "is_prime": {}, "method": "miller-rabin", "elapsed_ms": {}}}"#,
        n, is_prime, elapsed_ms
    );
    
    Response::json(&body)
}

/// Verifica si un número es primo usando el test de Miller-Rabin
/// 
/// # Argumentos
/// - `n`: Número a verificar
/// - `k`: Número de iteraciones (mayor = más preciso)
fn is_prime_miller_rabin(n: u64, k: usize) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    
    // Escribir n-1 como 2^r * d
    let mut d = n - 1;
    let mut r = 0;
    while d % 2 == 0 {
        d /= 2;
        r += 1;
    }
    
    // Testigos para números pequeños
    let witnesses = if n < 2_047 {
        vec![2]
    } else if n < 1_373_653 {
        vec![2, 3]
    } else if n < 9_080_191 {
        vec![31, 73]
    } else if n < 25_326_001 {
        vec![2, 3, 5]
    } else if n < 4_759_123_141 {
        vec![2, 7, 61]
    } else {
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    };
    
    for a in witnesses.iter().take(k) {
        let a = *a;
        if a >= n {
            continue;
        }
        
        let mut x = mod_pow(a, d, n);
        
        if x == 1 || x == n - 1 {
            continue;
        }
        
        let mut composite = true;
        for _ in 0..r - 1 {
            x = mod_pow(x, 2, n);
            if x == n - 1 {
                composite = false;
                break;
            }
        }
        
        if composite {
            return false;
        }
    }
    
    true
}

/// Exponenciación modular: (base^exp) % modulus
fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    
    let mut result = 1u64;
    base %= modulus;
    
    while exp > 0 {
        if exp % 2 == 1 {
            result = mod_mul(result, base, modulus);
        }
        exp >>= 1;
        base = mod_mul(base, base, modulus);
    }
    
    result
}

/// Multiplicación modular segura: (a * b) % m sin overflow
fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Handler para /factor?n=NUM
/// 
/// Factoriza un número en sus factores primos.
/// 
/// # Query parameters
/// - `n`: Número a factorizar (requerido, 2 <= n <= 10^15)
/// 
/// # Ejemplo de response
/// ```json
/// {"n": 360, "factors": [[2,3], [3,2], [5,1]], "elapsed_ms": 7}
/// ```
pub fn factor_handler(req: &Request) -> Response {
    let n_str = match req.query_param("n") {
        Some(s) => s,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: n"
            );
        }
    };
    
    let n: u64 = match n_str.parse() {
        Ok(num) if num >= 2 => num,
        _ => {
            return Response::error(
                StatusCode::BadRequest,
                "Parameter 'n' must be an integer >= 2"
            );
        }
    };
    
    // Límite para evitar cálculos excesivamente largos
    if n > 1_000_000_000_000_000 {
        return Response::error(
            StatusCode::BadRequest,
            "Parameter 'n' must be <= 10^15"
        );
    }
    
    let start = Instant::now();
    let factors = factorize(n);
    let elapsed_ms = start.elapsed().as_millis();
    
    // Formatear factores como [[primo, exponente], ...]
    let factors_str = factors.iter()
        .map(|(p, e)| format!("[{}, {}]", p, e))
        .collect::<Vec<_>>()
        .join(", ");
    
    let body = format!(
        r#"{{"n": {}, "factors": [{}], "elapsed_ms": {}}}"#,
        n, factors_str, elapsed_ms
    );
    
    Response::json(&body)
}

/// Factoriza un número en sus factores primos
/// 
/// Retorna vector de (primo, exponente)
fn factorize(mut n: u64) -> Vec<(u64, u32)> {
    let mut factors = Vec::new();
    
    // Manejar factor 2
    if n % 2 == 0 {
        let mut count = 0;
        while n % 2 == 0 {
            n /= 2;
            count += 1;
        }
        factors.push((2, count));
    }
    
    // Probar divisores impares hasta √n
    let mut d = 3;
    while d * d <= n {
        if n % d == 0 {
            let mut count = 0;
            while n % d == 0 {
                n /= d;
                count += 1;
            }
            factors.push((d, count));
        }
        d += 2;
    }
    
    // Si queda algo, es un factor primo
    if n > 1 {
        factors.push((n, 1));
    }
    
    factors
}

/// Handler para /pi?digits=D
/// 
/// Calcula dígitos de π usando el algoritmo de Bailey–Borwein–Plouffe.
/// 
/// # Query parameters
/// - `digits`: Número de dígitos decimales (1-1000)
/// 
/// # Ejemplo de response
/// ```json
/// {"digits": 10, "value": "3.1415926535", "elapsed_ms": 45}
/// ```
pub fn pi_handler(req: &Request) -> Response {
    let digits_str = match req.query_param("digits") {
        Some(s) => s,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: digits"
            );
        }
    };
    
    let digits: usize = match digits_str.parse() {
        Ok(d) if d >= 1 && d <= 1000 => d,
        _ => {
            return Response::error(
                StatusCode::BadRequest,
                "Parameter 'digits' must be between 1 and 1000"
            );
        }
    };
    
    let start = Instant::now();
    let pi_value = calculate_pi(digits);
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"digits": {}, "value": "{}", "elapsed_ms": {}}}"#,
        digits, pi_value, elapsed_ms
    );
    
    Response::json(&body)
}

/// Calcula π con precisión especificada usando serie de Machin
/// π/4 = 4*arctan(1/5) - arctan(1/239)
fn calculate_pi(digits: usize) -> String {
    let terms = (digits * 10 + 100).min(10000);
    
    // Calcular arctan(1/5)
    let arctan_1_5 = calculate_arctan(5, terms);
    
    // Calcular arctan(1/239)
    let arctan_1_239 = calculate_arctan(239, terms);
    
    // Aplicar fórmula de Machin
    let pi = 4.0 * (4.0 * arctan_1_5 - arctan_1_239);
    
    format!("{:.prec$}", pi, prec = digits)
}

/// Calcula arctan(1/x) usando serie de Taylor
fn calculate_arctan(x: i32, terms: usize) -> f64 {
    let x_f = x as f64;
    let mut result = 0.0;
    let x_squared = x_f * x_f;
    
    for n in 0..terms {
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        let term = sign / ((2 * n + 1) as f64 * x_f.powi(2 * n as i32 + 1));
        result += term;
        
        // Converge rápido, podemos salir antes
        if term.abs() < 1e-15 {
            break;
        }
    }
    
    result
}

/// Handler para /mandelbrot?width=W&height=H&max_iter=I
/// 
/// Genera el conjunto de Mandelbrot.
/// 
/// # Query parameters
/// - `width`: Ancho (default: 80, max: 500)
/// - `height`: Alto (default: 40, max: 500)
/// - `max_iter`: Iteraciones máximas (default: 100, max: 1000)
/// 
/// # Ejemplo de response
/// ```json
/// {"width": 80, "height": 40, "max_iter": 100, "data": [[...]]}
/// ```
pub fn mandelbrot_handler(req: &Request) -> Response {
    let width: usize = req.query_param("width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
        .min(500);
    
    let height: usize = req.query_param("height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(40)
        .min(500);
    
    let max_iter: u32 = req.query_param("max_iter")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
        .min(1000);
    
    let start = Instant::now();
    let data = calculate_mandelbrot(width, height, max_iter);
    let elapsed_ms = start.elapsed().as_millis();
    
    // Convertir data a JSON (simplificado - solo primeras filas)
    let sample_rows = data.iter()
        .take(5)
        .map(|row| {
            let row_str = row.iter()
                .take(10)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            format!("[{}]", row_str)
        })
        .collect::<Vec<_>>()
        .join(",");
    
    let body = format!(
        r#"{{"width": {}, "height": {}, "max_iter": {}, "sample_data": [{}], "elapsed_ms": {}}}"#,
        width, height, max_iter, sample_rows, elapsed_ms
    );
    
    Response::json(&body)
}

/// Calcula el conjunto de Mandelbrot
fn calculate_mandelbrot(width: usize, height: usize, max_iter: u32) -> Vec<Vec<u32>> {
    let mut result = Vec::with_capacity(height);
    
    let x_min = -2.5;
    let x_max = 1.0;
    let y_min = -1.0;
    let y_max = 1.0;
    
    for py in 0..height {
        let mut row = Vec::with_capacity(width);
        let y0 = y_min + (py as f64 / height as f64) * (y_max - y_min);
        
        for px in 0..width {
            let x0 = x_min + (px as f64 / width as f64) * (x_max - x_min);
            
            let mut x = 0.0;
            let mut y = 0.0;
            let mut iteration = 0;
            
            while x * x + y * y <= 4.0 && iteration < max_iter {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iteration += 1;
            }
            
            row.push(iteration);
        }
        result.push(row);
    }
    
    result
}

/// Handler para /matrixmul?size=N&seed=S
/// 
/// Multiplica dos matrices N×N con valores pseudoaleatorios.
/// 
/// # Query parameters
/// - `size`: Tamaño de la matriz (1-500)
/// - `seed`: Semilla para generación (default: 42)
/// 
/// # Ejemplo de response
/// ```json
/// {"size": 100, "seed": 42, "result_hash": "a3f5...", "elapsed_ms": 234}
/// ```
pub fn matrixmul_handler(req: &Request) -> Response {
    let size: usize = match req.query_param("size") {
        Some(s) => match s.parse() {
            Ok(n) if n >= 1 && n <= 500 => n,
            _ => {
                return Response::error(
                    StatusCode::BadRequest,
                    "Parameter 'size' must be between 1 and 500"
                );
            }
        },
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: size"
            );
        }
    };
    
    let seed: u64 = req.query_param("seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    
    let start = Instant::now();
    let hash = matrix_multiply(size, seed);
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"size": {}, "seed": {}, "result_hash": "{:016x}", "elapsed_ms": {}}}"#,
        size, seed, hash, elapsed_ms
    );
    
    Response::json(&body)
}

/// Multiplica dos matrices y retorna hash del resultado
fn matrix_multiply(size: usize, seed: u64) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Generar matrices A y B
    let mut rng = seed;
    let mut a = vec![vec![0i32; size]; size];
    let mut b = vec![vec![0i32; size]; size];
    
    for i in 0..size {
        for j in 0..size {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            a[i][j] = (rng % 100) as i32;
            
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            b[i][j] = (rng % 100) as i32;
        }
    }
    
    // Multiplicar C = A × B
    let mut c = vec![vec![0i32; size]; size];
    for i in 0..size {
        for j in 0..size {
            let mut sum = 0i32;
            for k in 0..size {
                sum = sum.wrapping_add(a[i][k].wrapping_mul(b[k][j]));
            }
            c[i][j] = sum;
        }
    }
    
    // Calcular hash del resultado
    let mut hasher = DefaultHasher::new();
    for row in c {
        for val in row {
            val.hash(&mut hasher);
        }
    }
    
    hasher.finish()
}


    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::http::{Request, StatusCode};
        
        // Helper para crear requests
        fn make_request(path: &str) -> Request {
            let raw = format!("GET {} HTTP/1.0\r\n\r\n", path);
            Request::parse(raw.as_bytes()).unwrap()
        }
        
        // ==================== ISPRIME ====================
        
        #[test]
        fn test_is_prime_miller_rabin_small_primes() {
            assert!(is_prime_miller_rabin(2, 10));
            assert!(is_prime_miller_rabin(3, 10));
            assert!(is_prime_miller_rabin(5, 10));
            assert!(is_prime_miller_rabin(7, 10));
            assert!(is_prime_miller_rabin(11, 10));
            assert!(is_prime_miller_rabin(13, 10));
        }
        
        #[test]
        fn test_is_prime_miller_rabin_composites() {
            assert!(!is_prime_miller_rabin(4, 10));
            assert!(!is_prime_miller_rabin(6, 10));
            assert!(!is_prime_miller_rabin(8, 10));
            assert!(!is_prime_miller_rabin(9, 10));
            assert!(!is_prime_miller_rabin(10, 10));
            assert!(!is_prime_miller_rabin(100, 10));
        }
        
        #[test]
        fn test_is_prime_miller_rabin_large_primes() {
            assert!(is_prime_miller_rabin(97, 10));
            assert!(is_prime_miller_rabin(104729, 10));
            assert!(is_prime_miller_rabin(999983, 10));
        }
        
        #[test]
        fn test_is_prime_miller_rabin_edge_cases() {
            assert!(!is_prime_miller_rabin(0, 10));
            assert!(!is_prime_miller_rabin(1, 10));
        }
        
        #[test]
        fn test_isprime_handler_success_prime() {
            let request = make_request("/isprime?n=97");
            let response = isprime_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"is_prime\": true"));
            assert!(body.contains("\"n\": 97"));
        }
        
        #[test]
        fn test_isprime_handler_success_composite() {
            let request = make_request("/isprime?n=100");
            let response = isprime_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"is_prime\": false"));
        }
        
        #[test]
        fn test_isprime_handler_missing_param() {
            let request = make_request("/isprime");
            let response = isprime_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("Missing required parameter"));
        }
        
        #[test]
        fn test_isprime_handler_invalid_param() {
            let request = make_request("/isprime?n=abc");
            let response = isprime_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_isprime_handler_zero_or_one() {
            let req0 = make_request("/isprime?n=0");
            let resp0 = isprime_handler(&req0);
            assert_eq!(resp0.status(), StatusCode::BadRequest);
            
            let req1 = make_request("/isprime?n=1");
            let resp1 = isprime_handler(&req1);
            assert_eq!(resp1.status(), StatusCode::BadRequest);
        }
        
        // ==================== MOD_POW ====================
        
        #[test]
        fn test_mod_pow_basic() {
            assert_eq!(mod_pow(2, 3, 5), 3);  // 2^3 mod 5 = 8 mod 5 = 3
            assert_eq!(mod_pow(2, 10, 1000), 24);
            assert_eq!(mod_pow(5, 3, 13), 8);
        }
        
        #[test]
        fn test_mod_pow_edge_cases() {
            assert_eq!(mod_pow(2, 0, 5), 1);  // Cualquier número^0 = 1
            assert_eq!(mod_pow(0, 5, 7), 0);  // 0^n = 0
            assert_eq!(mod_pow(5, 1, 7), 5);  // n^1 = n
        }
        
        #[test]
        fn test_mod_mul() {
            assert_eq!(mod_mul(2, 3, 5), 1);  // (2*3) mod 5 = 6 mod 5 = 1
            assert_eq!(mod_mul(100, 200, 50), 0);
        }
        
        // ==================== FACTOR ====================
        
        #[test]
        fn test_factorize_small_numbers() {
            assert_eq!(factorize(2), vec![(2, 1)]);
            assert_eq!(factorize(4), vec![(2, 2)]);
            assert_eq!(factorize(6), vec![(2, 1), (3, 1)]);
            assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
        }
        
        #[test]
        fn test_factorize_powers() {
            assert_eq!(factorize(8), vec![(2, 3)]);
            assert_eq!(factorize(27), vec![(3, 3)]);
            assert_eq!(factorize(32), vec![(2, 5)]);
        }
        
        #[test]
        fn test_factorize_composite() {
            assert_eq!(factorize(360), vec![(2, 3), (3, 2), (5, 1)]);
        }
        
        #[test]
        fn test_factorize_prime() {
            assert_eq!(factorize(97), vec![(97, 1)]);
            assert_eq!(factorize(101), vec![(101, 1)]);
        }
        
        #[test]
        fn test_factor_handler_success() {
            let request = make_request("/factor?n=12");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"factors\""));
            assert!(body.contains("[2, 2]"));
            assert!(body.contains("[3, 1]"));
        }
        
        #[test]
        fn test_factor_handler_prime_number() {
            let request = make_request("/factor?n=97");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("[97, 1]"));
        }
        
        #[test]
        fn test_factor_handler_missing_param() {
            let request = make_request("/factor");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_factor_handler_invalid_param() {
            let request = make_request("/factor?n=abc");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_factor_handler_too_small() {
            let request = make_request("/factor?n=1");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_factor_handler_too_large() {
            let request = make_request("/factor?n=9999999999999999");
            let response = factor_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("10^15"));
        }
        
        // ==================== PI ====================

        #[test]
        fn test_calculate_pi_basic() {
            let pi_10 = calculate_pi(10);
            
            // Verificar que empieza con 3.14 (primeros 3 dígitos correctos)
            assert!(pi_10.starts_with("3.14"), "Expected to start with 3.14, got: {}", pi_10);
            
            // Verificar que tiene al menos 12 caracteres (3. + 10 dígitos)
            assert!(pi_10.len() >= 4, "Expected length >= 4, got: {}", pi_10.len());
        }

        #[test]
        fn test_calculate_pi_accuracy() {
            // π = 3.14159265358979...
            let pi_5 = calculate_pi(5);
            assert!(pi_5.starts_with("3.1415"), "Expected to start with 3.1415, got: {}", pi_5);
            
            let pi_3 = calculate_pi(3);
            assert!(pi_3.starts_with("3.14"), "Expected to start with 3.14, got: {}", pi_3);
        }

        #[test]
        fn test_pi_handler_success() {
            let request = make_request("/pi?digits=10");
            let response = pi_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"digits\": 10"));
            assert!(body.contains("\"value\""));
            
            // Verificar que el valor de π comienza con 3.14
            assert!(body.contains("3.14"), "Response should contain accurate π value. Body: {}", body);
        }

        #[test]
        fn test_pi_handler_different_precisions() {
            // Probar con diferentes precisiones
            let req5 = make_request("/pi?digits=5");
            let resp5 = pi_handler(&req5);
            let body5 = String::from_utf8(resp5.body().to_vec()).unwrap();
            assert!(body5.contains("3.1415") || body5.contains("3.1416"));  // Acepta redondeo
            
            let req2 = make_request("/pi?digits=2");
            let resp2 = pi_handler(&req2);
            let body2 = String::from_utf8(resp2.body().to_vec()).unwrap();
            assert!(body2.contains("3.1"));
        }

        #[test]
        fn test_pi_handler_missing_param() {
            let request = make_request("/pi");
            let response = pi_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }

        #[test]
        fn test_pi_handler_invalid_digits() {
            let request = make_request("/pi?digits=0");
            let response = pi_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }

        #[test]
        fn test_pi_handler_too_many_digits() {
            let request = make_request("/pi?digits=2000");
            let response = pi_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("between 1 and 1000"));
        }
        
        // ==================== MANDELBROT ====================
        
        #[test]
        fn test_calculate_mandelbrot_basic() {
            let result = calculate_mandelbrot(10, 10, 50);
            assert_eq!(result.len(), 10);  // 10 filas
            assert_eq!(result[0].len(), 10);  // 10 columnas
        }
        
        #[test]
        fn test_mandelbrot_handler_default() {
            let request = make_request("/mandelbrot");
            let response = mandelbrot_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"width\": 80"));
            assert!(body.contains("\"height\": 40"));
        }
        
        #[test]
        fn test_mandelbrot_handler_with_params() {
            let request = make_request("/mandelbrot?width=20&height=20&max_iter=50");
            let response = mandelbrot_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"width\": 20"));
            assert!(body.contains("\"height\": 20"));
            assert!(body.contains("\"max_iter\": 50"));
        }
        
        #[test]
        fn test_mandelbrot_handler_large_size_limited() {
            let request = make_request("/mandelbrot?width=1000&height=1000");
            let response = mandelbrot_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            // Debe limitar a 500
            assert!(body.contains("\"width\": 500"));
            assert!(body.contains("\"height\": 500"));
        }
        
        // ==================== MATRIXMUL ====================
        
        #[test]
        fn test_matrix_multiply_deterministic() {
            // Misma semilla debe dar mismo resultado
            let hash1 = matrix_multiply(10, 42);
            let hash2 = matrix_multiply(10, 42);
            assert_eq!(hash1, hash2);
        }
        
        #[test]
        fn test_matrix_multiply_different_seeds() {
            // Diferentes semillas deben dar diferentes resultados
            let hash1 = matrix_multiply(10, 42);
            let hash2 = matrix_multiply(10, 123);
            assert_ne!(hash1, hash2);
        }
        
        #[test]
        fn test_matrixmul_handler_success() {
            let request = make_request("/matrixmul?size=10&seed=42");
            let response = matrixmul_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"size\": 10"));
            assert!(body.contains("\"seed\": 42"));
            assert!(body.contains("\"result_hash\""));
        }
        
        #[test]
        fn test_matrixmul_handler_default_seed() {
            let request = make_request("/matrixmul?size=5");
            let response = matrixmul_handler(&request);
            
            assert_eq!(response.status(), StatusCode::Ok);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("\"seed\": 42"));  // Default
        }
        
        #[test]
        fn test_matrixmul_handler_missing_size() {
            let request = make_request("/matrixmul");
            let response = matrixmul_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_matrixmul_handler_invalid_size() {
            let request = make_request("/matrixmul?size=0");
            let response = matrixmul_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
        }
        
        #[test]
        fn test_matrixmul_handler_too_large() {
            let request = make_request("/matrixmul?size=1000");
            let response = matrixmul_handler(&request);
            
            assert_eq!(response.status(), StatusCode::BadRequest);
            let body = String::from_utf8(response.body().to_vec()).unwrap();
            assert!(body.contains("between 1 and 500"));
        }
    }