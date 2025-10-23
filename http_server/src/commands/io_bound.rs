//! # Comandos IO-bound
//!
//! Comandos que requieren operaciones intensivas de I/O:
//! - /sortfile: Ordenar archivos con números
//! - /wordcount: Contar líneas, palabras y bytes
//! - /grep: Buscar patrones en archivos
//! - /compress: Comprimir archivos (gzip)
//! - /hashfile: Calcular hash SHA256 de archivos

use crate::http::{Request, Response, StatusCode};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write, Read};
use std::path::Path;
use std::time::Instant;

/// Handler para /sortfile?name=FILE&algo=merge|quick
/// 
/// Ordena un archivo que contiene números (uno por línea).
/// 
/// # Query parameters
/// - `name`: Nombre del archivo en data/ (requerido)
/// - `algo`: Algoritmo (merge o quick, default: merge)
/// 
/// # Ejemplo de response
/// ```json
/// {"file": "numbers.txt", "algo": "merge", "sorted_file": "numbers.sorted", "elapsed_ms": 234}
/// ```
pub fn sortfile_handler(req: &Request) -> Response {
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
            "Invalid filename"
        );
    }
    
    let algo = req.query_param("algo").unwrap_or("merge");
    if algo != "merge" && algo != "quick" {
        return Response::error(
            StatusCode::BadRequest,
            "Parameter 'algo' must be 'merge' or 'quick'"
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    // Verificar que existe
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    let start = Instant::now();
    
    // Leer números del archivo
    let numbers = match read_numbers_from_file(&filepath) {
        Ok(nums) => nums,
        Err(e) => {
            return Response::error(
                StatusCode::InternalServerError,
                &format!("Failed to read file: {}", e)
            );
        }
    };
    
    // Ordenar según algoritmo
    let mut sorted = numbers.clone();
    match algo {
        "merge" => merge_sort(&mut sorted),
        "quick" => sorted.sort(), // Rust usa quicksort por defecto
        _ => unreachable!(),
    }
    
    // Escribir archivo ordenado
    let output_name = format!("{}.sorted", name);
    let output_path = format!("./data/{}", output_name);
    
    if let Err(e) = write_numbers_to_file(&output_path, &sorted) {
        return Response::error(
            StatusCode::InternalServerError,
            &format!("Failed to write sorted file: {}", e)
        );
    }
    
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"file": "{}", "algo": "{}", "sorted_file": "{}", "lines": {}, "elapsed_ms": {}}}"#,
        name, algo, output_name, sorted.len(), elapsed_ms
    );
    
    Response::json(&body)
}

/// Lee números de un archivo (un número por línea)
fn read_numbers_from_file(path: &str) -> std::io::Result<Vec<i64>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut numbers = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if let Ok(num) = line.trim().parse::<i64>() {
            numbers.push(num);
        }
    }
    
    Ok(numbers)
}

/// Escribe números a un archivo (uno por línea)
fn write_numbers_to_file(path: &str, numbers: &[i64]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    
    for num in numbers {
        writeln!(writer, "{}", num)?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Merge sort implementation
fn merge_sort(arr: &mut [i64]) {
    let len = arr.len();
    if len <= 1 {
        return;
    }
    
    let mid = len / 2;
    merge_sort(&mut arr[..mid]);
    merge_sort(&mut arr[mid..]);
    
    let mut temp = arr.to_vec();
    merge(&arr[..mid], &arr[mid..], &mut temp);
    arr.copy_from_slice(&temp);
}

fn merge(left: &[i64], right: &[i64], result: &mut [i64]) {
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result[k] = left[i];
            i += 1;
        } else {
            result[k] = right[j];
            j += 1;
        }
        k += 1;
    }
    
    while i < left.len() {
        result[k] = left[i];
        i += 1;
        k += 1;
    }
    
    while j < right.len() {
        result[k] = right[j];
        j += 1;
        k += 1;
    }
}

/// Handler para /wordcount?name=FILE
/// 
/// Cuenta líneas, palabras y bytes de un archivo.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo en data/ (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {"file": "text.txt", "lines": 100, "words": 543, "bytes": 3421, "elapsed_ms": 12}
/// ```
pub fn wordcount_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    // Validar nombre
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename"
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    let start = Instant::now();
    
    let (lines, words, bytes) = match count_file_stats(&filepath) {
        Ok(stats) => stats,
        Err(e) => {
            return Response::error(
                StatusCode::InternalServerError,
                &format!("Failed to count: {}", e)
            );
        }
    };
    
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"file": "{}", "lines": {}, "words": {}, "bytes": {}, "elapsed_ms": {}}}"#,
        name, lines, words, bytes, elapsed_ms
    );
    
    Response::json(&body)
}

/// Cuenta estadísticas de un archivo
fn count_file_stats(path: &str) -> std::io::Result<(usize, usize, usize)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    
    for line in reader.lines() {
        let line = line?;
        lines += 1;
        bytes += line.len() + 1; // +1 para el newline
        words += line.split_whitespace().count();
    }
    
    Ok((lines, words, bytes))
}

/// Handler para /grep?name=FILE&pattern=REGEX
/// 
/// Busca líneas que coincidan con un patrón.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo en data/ (requerido)
/// - `pattern`: Expresión regular (requerido)
/// 
/// # Ejemplo de response
/// ```json
/// {"file": "text.txt", "pattern": "error", "matches": 5, "lines": ["line 1...", "line 2..."], "elapsed_ms": 45}
/// ```
pub fn grep_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    let pattern = match req.query_param("pattern") {
        Some(p) => p,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: pattern"
            );
        }
    };
    
    // Validar nombre
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename"
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    let start = Instant::now();
    
    let (count, lines) = match grep_file(&filepath, pattern) {
        Ok(result) => result,
        Err(e) => {
            return Response::error(
                StatusCode::InternalServerError,
                &format!("Grep failed: {}", e)
            );
        }
    };
    
    let elapsed_ms = start.elapsed().as_millis();
    
    // Formatear primeras 10 líneas para JSON
    let lines_json = lines.iter()
        .take(10)
        .map(|l| format!(r#""{}""#, l.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    
    let body = format!(
        r#"{{"file": "{}", "pattern": "{}", "matches": {}, "sample_lines": [{}], "elapsed_ms": {}}}"#,
        name, pattern, count, lines_json, elapsed_ms
    );
    
    Response::json(&body)
}

/// Busca líneas que coincidan con un patrón
fn grep_file(path: &str, pattern: &str) -> Result<(usize, Vec<String>), Box<dyn std::error::Error>> {
    use regex::Regex;
    
    let re = Regex::new(pattern)?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut matching_lines = Vec::new();
    let mut count = 0;
    
    for line in reader.lines() {
        let line = line?;
        if re.is_match(&line) {
            count += 1;
            if matching_lines.len() < 10 {
                matching_lines.push(line);
            }
        }
    }
    
    Ok((count, matching_lines))
}

/// Handler para /compress?name=FILE&codec=gzip
/// 
/// Comprime un archivo usando gzip.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo en data/ (requerido)
/// - `codec`: Codec de compresión (solo gzip por ahora)
/// 
/// # Ejemplo de response
/// ```json
/// {"file": "text.txt", "codec": "gzip", "output": "text.txt.gz", "original_size": 1024, "compressed_size": 512, "elapsed_ms": 78}
/// ```
pub fn compress_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    let codec = req.query_param("codec").unwrap_or("gzip");
    if codec != "gzip" {
        return Response::error(
            StatusCode::BadRequest,
            "Only 'gzip' codec is supported"
        );
    }
    
    // Validar nombre
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename"
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    let start = Instant::now();
    
    let output_name = format!("{}.gz", name);
    let output_path = format!("./data/{}", output_name);
    
    let (original_size, compressed_size) = match compress_file_gzip(&filepath, &output_path) {
        Ok(sizes) => sizes,
        Err(e) => {
            return Response::error(
                StatusCode::InternalServerError,
                &format!("Compression failed: {}", e)
            );
        }
    };
    
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"file": "{}", "codec": "gzip", "output": "{}", "original_size": {}, "compressed_size": {}, "ratio": {:.2}, "elapsed_ms": {}}}"#,
        name, output_name, original_size, compressed_size, 
        (compressed_size as f64 / original_size as f64), elapsed_ms
    );
    
    Response::json(&body)
}

/// Comprime un archivo con gzip
fn compress_file_gzip(input: &str, output: &str) -> std::io::Result<(u64, u64)> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    
    let mut input_file = File::open(input)?;
    let output_file = File::create(output)?;
    
    let original_size = input_file.metadata()?.len();
    
    let mut encoder = GzEncoder::new(output_file, Compression::default());
    std::io::copy(&mut input_file, &mut encoder)?;
    encoder.finish()?;
    
    let compressed_size = fs::metadata(output)?.len();
    
    Ok((original_size, compressed_size))
}

/// Handler para /hashfile?name=FILE&algo=sha256
/// 
/// Calcula el hash SHA256 de un archivo.
/// 
/// # Query parameters
/// - `name`: Nombre del archivo en data/ (requerido)
/// - `algo`: Algoritmo (solo sha256 por ahora)
/// 
/// # Ejemplo de response
/// ```json
/// {"file": "text.txt", "algo": "sha256", "hash": "a3f5...", "size": 1024, "elapsed_ms": 23}
/// ```
pub fn hashfile_handler(req: &Request) -> Response {
    let name = match req.query_param("name") {
        Some(n) => n,
        None => {
            return Response::error(
                StatusCode::BadRequest,
                "Missing required parameter: name"
            );
        }
    };
    
    let algo = req.query_param("algo").unwrap_or("sha256");
    if algo != "sha256" {
        return Response::error(
            StatusCode::BadRequest,
            "Only 'sha256' algorithm is supported"
        );
    }
    
    // Validar nombre
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Response::error(
            StatusCode::BadRequest,
            "Invalid filename"
        );
    }
    
    let filepath = format!("./data/{}", name);
    
    if !Path::new(&filepath).exists() {
        return Response::error(
            StatusCode::NotFound,
            &format!("File not found: {}", name)
        );
    }
    
    let start = Instant::now();
    
    let (hash, size) = match hash_file_sha256(&filepath) {
        Ok(result) => result,
        Err(e) => {
            return Response::error(
                StatusCode::InternalServerError,
                &format!("Hashing failed: {}", e)
            );
        }
    };
    
    let elapsed_ms = start.elapsed().as_millis();
    
    let body = format!(
        r#"{{"file": "{}", "algo": "sha256", "hash": "{}", "size": {}, "elapsed_ms": {}}}"#,
        name, hash, size, elapsed_ms
    );
    
    Response::json(&body)
}

/// Calcula el hash SHA256 de un archivo
fn hash_file_sha256(path: &str) -> std::io::Result<(String, u64)> {
    use sha2::{Sha256, Digest};
    
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    
    let result = hasher.finalize();
    let hash_string = format!("{:x}", result);
    
    Ok((hash_string, size))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_sort() {
        let mut arr = vec![5, 2, 8, 1, 9];
        merge_sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 5, 8, 9]);
    }
}