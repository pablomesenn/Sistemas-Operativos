//! # Collector de Métricas
//! src/metrics/collector.rs
//!
//! Recolecta y agrega métricas del servidor en tiempo real.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Collector de métricas thread-safe
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<Mutex<MetricsData>>,
    start_time: Instant,
}

/// Datos internos de métricas
struct MetricsData {
    /// Contador total de requests
    total_requests: u64,
    
    /// Requests por código de estado
    status_codes: HashMap<u16, u64>,
    
    /// Latencias registradas (en microsegundos)
    latencies: Vec<u64>,
    
    /// Máximo de latencias a guardar (para calcular percentiles)
    max_latencies: usize,
    
    /// Requests por ruta
    requests_per_path: HashMap<String, u64>,
    
    /// Threads activos actualmente
    active_threads: u64,
}

impl MetricsCollector {
    /// Crea un nuevo collector de métricas
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MetricsData {
                total_requests: 0,
                status_codes: HashMap::new(),
                latencies: Vec::with_capacity(10000),
                max_latencies: 10000, // Guardar últimas 10k latencias
                requests_per_path: HashMap::new(),
                active_threads: 0,
            })),
            start_time: Instant::now(),
        }
    }
    
    /// Registra un nuevo request
    pub fn record_request(&self, path: &str, status_code: u16, latency: Duration) {
        let mut data = self.inner.lock().unwrap();
        
        // Incrementar contador total
        data.total_requests += 1;
        
        // Registrar código de estado
        *data.status_codes.entry(status_code).or_insert(0) += 1;
        
        // Registrar latencia (en microsegundos)
        let latency_us = latency.as_micros() as u64;
        
        // Si tenemos demasiadas latencias, eliminar las más antiguas
        if data.latencies.len() >= data.max_latencies {
            data.latencies.remove(0);
        }
        data.latencies.push(latency_us);
        
        // Registrar request por ruta
        *data.requests_per_path.entry(path.to_string()).or_insert(0) += 1;
    }
    
    /// Incrementa el contador de threads activos
    pub fn increment_active_threads(&self) {
        let mut data = self.inner.lock().unwrap();
        data.active_threads += 1;
    }
    
    /// Decrementa el contador de threads activos
    pub fn decrement_active_threads(&self) {
        let mut data = self.inner.lock().unwrap();
        if data.active_threads > 0 {
            data.active_threads -= 1;
        }
    }
    
    /// Obtiene el número de threads activos
    pub fn active_threads(&self) -> u64 {
        let data = self.inner.lock().unwrap();
        data.active_threads
    }
    
    /// Obtiene las métricas actuales en formato JSON
    pub fn get_metrics_json(&self) -> String {
        let data = self.inner.lock().unwrap();
        
        // Calcular uptime
        let uptime_secs = self.start_time.elapsed().as_secs();
        
        // Calcular percentiles de latencia
        let (p50, p95, p99, avg) = self.calculate_percentiles(&data.latencies);
        let stddev = self.calculate_stddev(&data.latencies, avg);
        
        // Formatear status codes
        let status_codes_json = data.status_codes.iter()
            .map(|(code, count)| format!(r#""{}": {}"#, code, count))
            .collect::<Vec<_>>()
            .join(", ");
        
        // Top 10 rutas más accedidas
        let mut paths: Vec<_> = data.requests_per_path.iter().collect();
        paths.sort_by(|a, b| b.1.cmp(a.1));
        let top_paths_json = paths.iter()
            .take(10)
            .map(|(path, count)| format!(r#"{{"path": "{}", "count": {}}}"#, path, count))
            .collect::<Vec<_>>()
            .join(", ");
        
        format!(
            r#"{{
  "server": {{
    "uptime_seconds": {},
    "start_time": "{:?}"
  }},
  "requests": {{
    "total": {},
    "active_threads": {},
    "status_codes": {{{}}},
    "top_paths": [{}]
  }},
  "latency_us": {{
    "p50": {},
    "p95": {},
    "p99": {},
    "avg": {},
    "stddev": {:.2},
    "samples": {}
  }}
}}"#,
            uptime_secs,
            self.start_time,
            data.total_requests,
            data.active_threads,
            status_codes_json,
            top_paths_json,
            p50, p95, p99, avg,
            stddev,
            data.latencies.len()
        )
    }
    
    /// Calcula percentiles de latencia
    fn calculate_percentiles(&self, latencies: &[u64]) -> (u64, u64, u64, u64) {
        if latencies.is_empty() {
            return (0, 0, 0, 0);
        }
        
        let mut sorted = latencies.to_vec();
        sorted.sort_unstable();
        
        let len = sorted.len();
        let p50 = sorted[len * 50 / 100];
        let p95 = sorted[len * 95 / 100];
        let p99 = sorted[len * 99 / 100];
        
        let sum: u64 = sorted.iter().sum();
        let avg = sum / len as u64;
        
        (p50, p95, p99, avg)
    }

    // NUEVO: Calcular desviación estándar
    fn calculate_stddev(&self, latencies: &[u64], avg: u64) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }
        
        let variance: f64 = latencies.iter()
            .map(|&x| {
                let diff = x as f64 - avg as f64;
                diff * diff
            })
            .sum::<f64>() / latencies.len() as f64;
        
        variance.sqrt()
    }
    
    /// Obtiene un snapshot de las métricas
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let data = self.inner.lock().unwrap();
        let (p50, p95, p99, avg) = self.calculate_percentiles(&data.latencies);
        
        MetricsSnapshot {
            total_requests: data.total_requests,
            active_threads: data.active_threads,
            uptime_secs: self.start_time.elapsed().as_secs(),
            latency_p50_us: p50,
            latency_p95_us: p95,
            latency_p99_us: p99,
            latency_avg_us: avg,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot de métricas (para uso externo)
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub active_threads: u64,
    pub uptime_secs: u64,
    pub latency_p50_us: u64,
    pub latency_p95_us: u64,
    pub latency_p99_us: u64,
    pub latency_avg_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // Registrar algunos requests
        collector.record_request("/test", 200, Duration::from_millis(10));
        collector.record_request("/test", 200, Duration::from_millis(20));
        collector.record_request("/test", 404, Duration::from_millis(5));
        
        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.total_requests, 3);
    }
    
    #[test]
    fn test_percentiles() {
        let collector = MetricsCollector::new();
        
        // Registrar latencias conocidas
        for i in 1..=100 {
            collector.record_request("/test", 200, Duration::from_micros(i));
        }
        
        let snapshot = collector.get_snapshot();
        assert!(snapshot.latency_p50_us > 0);
        assert!(snapshot.latency_p95_us > snapshot.latency_p50_us);
        assert!(snapshot.latency_p99_us > snapshot.latency_p95_us);
    }

    #[test]
    fn test_multiple_status_codes() {
        let collector = MetricsCollector::new();
        
        collector.record_request("/test1", 200, Duration::from_millis(10));
        collector.record_request("/test2", 200, Duration::from_millis(20));
        collector.record_request("/test3", 404, Duration::from_millis(5));
        collector.record_request("/test4", 500, Duration::from_millis(15));
        
        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.total_requests, 4);
    }
    
    #[test]
    fn test_active_threads_tracking() {
        let collector = MetricsCollector::new();
        
        assert_eq!(collector.active_threads(), 0);
        
        collector.increment_active_threads();
        assert_eq!(collector.active_threads(), 1);
        
        collector.increment_active_threads();
        assert_eq!(collector.active_threads(), 2);
        
        collector.decrement_active_threads();
        assert_eq!(collector.active_threads(), 1);
        
        collector.decrement_active_threads();
        assert_eq!(collector.active_threads(), 0);
    }
    
    #[test]
    fn test_active_threads_no_negative() {
        let collector = MetricsCollector::new();
        
        collector.decrement_active_threads();
        collector.decrement_active_threads();
        
        assert_eq!(collector.active_threads(), 0);
    }
    
    #[test]
    fn test_latency_calculations() {
        let collector = MetricsCollector::new();
        
        collector.record_request("/test", 200, Duration::from_millis(100));
        collector.record_request("/test", 200, Duration::from_millis(200));
        collector.record_request("/test", 200, Duration::from_millis(150));
        
        let snapshot = collector.get_snapshot();
        assert!(snapshot.latency_avg_us > 0);
        assert!(snapshot.latency_p50_us > 0);
        assert!(snapshot.latency_p99_us > 0);
        assert!(snapshot.latency_p99_us >= snapshot.latency_p50_us);
    }
    
    #[test]
    fn test_json_format() {
        let collector = MetricsCollector::new();
        collector.record_request("/test", 200, Duration::from_millis(50));
        
        let json = collector.get_metrics_json();
        // Verificar que sea JSON válido y contenga datos
        assert!(json.len() > 10);
        assert!(json.contains("{"));
        assert!(json.contains("}"));
        // Verificar campos de latencia
        assert!(json.contains("latency") || json.contains("p50") || json.contains("requests"));
    }
    
    #[test]
    fn test_uptime_increases() {
        let collector = MetricsCollector::new();
        
        let snapshot1 = collector.get_snapshot();
        std::thread::sleep(Duration::from_millis(100));
        let snapshot2 = collector.get_snapshot();
        
        assert!(snapshot2.uptime_secs >= snapshot1.uptime_secs);
    }
    
    #[test]
    fn test_requests_per_path() {
        let collector = MetricsCollector::new();
        
        collector.record_request("/fibonacci", 200, Duration::from_millis(10));
        collector.record_request("/fibonacci", 200, Duration::from_millis(15));
        collector.record_request("/status", 200, Duration::from_millis(5));
        
        let json = collector.get_metrics_json();
        assert!(json.contains("fibonacci"));
        assert!(json.contains("status"));
    }
    
    #[test]
    fn test_latency_window_management() {
        let collector = MetricsCollector::new();
        
        // Agregar muchas latencias
        for i in 0..15000 {
            collector.record_request("/test", 200, Duration::from_micros(i));
        }
        
        let snapshot = collector.get_snapshot();
        assert!(snapshot.total_requests == 15000);
    }
}