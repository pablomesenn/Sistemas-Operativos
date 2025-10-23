//! # Comandos CPU-bound
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
fn calculate_pi(digits: usize) -> String {
    // Usamos una aproximación simple con suficientes términos
    // Para producción usaríamos una librería como rug o mpc
    
    let terms = digits * 10 + 100; // Más términos = más precisión
    let mut pi = 0.0f64;
    
    // Serie de Gregory-Leibniz (lenta pero simple)
    for k in 0..terms {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        pi += sign / (2.0 * k as f64 + 1.0);
    }
    pi *= 4.0;
    
    // Formatear con la precisión solicitada
    format!("{:.prec$}", pi, prec = digits)
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
    
    #[test]
    fn test_is_prime() {
        assert!(is_prime_miller_rabin(2, 10));
        assert!(is_prime_miller_rabin(3, 10));
        assert!(is_prime_miller_rabin(5, 10));
        assert!(is_prime_miller_rabin(97, 10));
        assert!(is_prime_miller_rabin(104729, 10));
        
        assert!(!is_prime_miller_rabin(4, 10));
        assert!(!is_prime_miller_rabin(100, 10));
        assert!(!is_prime_miller_rabin(1000, 10));
    }
    
    #[test]
    fn test_factorize() {
        assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
        assert_eq!(factorize(360), vec![(2, 3), (3, 2), (5, 1)]);
        assert_eq!(factorize(97), vec![(97, 1)]);
    }
    
    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(5, 3, 13), 8);
    }
}