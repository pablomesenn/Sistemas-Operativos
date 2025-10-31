//! # Gestor Central de Jobs
//! src/jobs/manager.rs
//!
//! Coordina la ejecución de jobs: encolado, workers, timeouts, cancelación.

use crate::jobs::types::{JobMetadata, JobPriority, JobType};
use crate::jobs::queue::JobQueue;
use crate::jobs::storage::JobStorage;
use crate::http::{Request, Response};
use crate::commands;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Configuración del Job Manager
#[derive(Clone)]
pub struct JobManagerConfig {
    /// Capacidad máxima de la cola CPU
    pub cpu_queue_capacity: usize,
    
    /// Capacidad máxima de la cola IO
    pub io_queue_capacity: usize,
    
    /// Capacidad máxima de la cola básica
    pub basic_queue_capacity: usize,
    
    /// Timeout para jobs CPU-bound (milisegundos)
    pub cpu_timeout_ms: u64,
    
    /// Timeout para jobs IO-bound (milisegundos)
    pub io_timeout_ms: u64,
    
    /// Timeout para jobs básicos (milisegundos)
    pub basic_timeout_ms: u64,
    
    /// Número de workers para CPU-bound
    pub cpu_workers: usize,
    
    /// Número de workers para IO-bound
    pub io_workers: usize,
    
    /// Número de workers para básicos
    pub basic_workers: usize,
    
    /// Ruta del archivo de persistencia
    pub storage_path: String,
}

impl Default for JobManagerConfig {
    fn default() -> Self {
        Self {
            cpu_queue_capacity: 1000,
            io_queue_capacity: 1000,
            basic_queue_capacity: 500,
            cpu_timeout_ms: 60_000,
            io_timeout_ms: 120_000,
            basic_timeout_ms: 30_000,
            cpu_workers: 4,
            io_workers: 4,
            basic_workers: 2,
            storage_path: "./data/jobs.json".to_string(),
        }
    }
}

impl JobManagerConfig {
    /// Crea una configuración desde el Config principal
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            cpu_queue_capacity: config.cpu_queue_capacity,
            io_queue_capacity: config.io_queue_capacity,
            basic_queue_capacity: config.basic_queue_capacity,
            cpu_timeout_ms: config.cpu_timeout_ms,
            io_timeout_ms: config.io_timeout_ms,
            basic_timeout_ms: config.basic_timeout_ms,
            cpu_workers: config.cpu_workers,
            io_workers: config.io_workers,
            basic_workers: config.basic_workers,
            storage_path: config.jobs_storage_path.clone(),
        }
    }
}

/// Gestor central de jobs
pub struct JobManager {
    /// Configuración
    config: JobManagerConfig,
    
    /// Colas por tipo de job
    cpu_queue: JobQueue,
    io_queue: JobQueue,
    basic_queue: JobQueue,
    
    /// Storage persistente
    storage: JobStorage,
    
    /// Jobs actualmente en ejecución (job_id -> thread_handle)
    running_jobs: Arc<Mutex<HashMap<String, ()>>>,
}

impl JobManager {
    /// Crea un nuevo Job Manager
    pub fn new(config: JobManagerConfig) -> Self {
        // Crear directorio data/ si no existe
        let _ = std::fs::create_dir_all("./data");
        
        let storage = JobStorage::new(&config.storage_path)
            .expect("Failed to initialize job storage");
        
        let manager = Self {
            config: config.clone(),
            cpu_queue: JobQueue::new(config.cpu_queue_capacity),
            io_queue: JobQueue::new(config.io_queue_capacity),
            basic_queue: JobQueue::new(config.basic_queue_capacity),
            storage,
            running_jobs: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Iniciar workers
        manager.spawn_workers();
        
        manager
    }
    
    /// Inicia los workers para procesar jobs
    fn spawn_workers(&self) {
        // Workers CPU-bound
        for i in 0..self.config.cpu_workers {
            let queue = self.cpu_queue.clone();
            let storage = self.storage.clone();
            let running = Arc::clone(&self.running_jobs);
            let timeout_ms = self.config.cpu_timeout_ms;
            
            thread::spawn(move || {
                Self::worker_loop(
                    format!("CPU-{}", i),
                    queue,
                    storage,
                    running,
                    timeout_ms,
                )
            });
        }
        
        // Workers IO-bound
        for i in 0..self.config.io_workers {
            let queue = self.io_queue.clone();
            let storage = self.storage.clone();
            let running = Arc::clone(&self.running_jobs);
            let timeout_ms = self.config.io_timeout_ms;
            
            thread::spawn(move || {
                Self::worker_loop(
                    format!("IO-{}", i),
                    queue,
                    storage,
                    running,
                    timeout_ms,
                )
            });
        }
        
        // Workers básicos
        for i in 0..self.config.basic_workers {
            let queue = self.basic_queue.clone();
            let storage = self.storage.clone();
            let running = Arc::clone(&self.running_jobs);
            let timeout_ms = self.config.basic_timeout_ms;
            
            thread::spawn(move || {
                Self::worker_loop(
                    format!("Basic-{}", i),
                    queue,
                    storage,
                    running,
                    timeout_ms,
                )
            });
        }
    }
    
    /// Loop principal del worker
    fn worker_loop(
        name: String,
        queue: JobQueue,
        storage: JobStorage,
        running_jobs: Arc<Mutex<HashMap<String, ()>>>,
        timeout_ms: u64,
    ) {
        println!("🔧 Worker {} started", name);
        
        loop {
            // Esperar por un job
            let mut job = queue.dequeue();
            
            println!("🔨 Worker {} picked up job: {}", name, job.id);
            
            // Marcar como running
            job.mark_running();
            {
                let mut running = running_jobs.lock().unwrap();
                running.insert(job.id.clone(), ());
            }
            let _ = storage.save(&job);
            
            // Ejecutar el job
            let result = Self::execute_job(&job, timeout_ms);
            
            // Actualizar con el resultado
            match result {
                Ok(response_body) => {
                    job.mark_done(response_body);
                    println!("✅ Worker {} completed job: {}", name, job.id);
                }
                Err(error) => {
                    if error.contains("timeout") {
                        job.mark_timeout();
                        println!("⏱️  Worker {} timeout job: {}", name, job.id);
                    } else {
                        job.mark_error(error.clone());
                        println!("❌ Worker {} failed job: {} - {}", name, job.id, error);
                    }
                }
            }
            
            // Remover de running
            {
                let mut running = running_jobs.lock().unwrap();
                running.remove(&job.id);
            }
            
            // Guardar estado final
            let _ = storage.save(&job);
        }
    }
    
    /// Ejecuta un job específico
    fn execute_job(job: &JobMetadata, timeout_ms: u64) -> Result<String, String> {
        // Parsear los parámetros
        let params_json: serde_json::Value = serde_json::from_str(&job.params)
            .map_err(|e| format!("Invalid params JSON: {}", e))?;
        
        // Construir un Request simulado con los parámetros
        let query_string = Self::json_to_query_string(&params_json);
        let request_str = format!(
            "GET /{}?{} HTTP/1.0\r\n\r\n",
            Self::job_type_to_path(&job.job_type),
            query_string
        );
        
        let request = Request::parse(request_str.as_bytes())
            .map_err(|e| format!("Failed to parse request: {}", e))?;
        
        // Clonar job_type para moverlo al thread
        let job_type = job.job_type.clone();
        
        // Ejecutar con timeout
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);
        
        let handle = thread::spawn(move || {
            let response = Self::dispatch_command(&job_type, &request);
            let body = String::from_utf8_lossy(response.body()).to_string();
            let mut res = result_clone.lock().unwrap();
            *res = Some(body);
        });
        
        // Esperar con timeout
        let timeout_duration = Duration::from_millis(timeout_ms);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout_duration {
            if handle.is_finished() {
                let _ = handle.join();
                let res = result.lock().unwrap();
                return res.clone().ok_or_else(|| "No result".to_string());
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        Err("Job exceeded timeout".to_string())
    }
    
    /// Convierte JSON params a query string
    fn json_to_query_string(json: &serde_json::Value) -> String {
        if let Some(obj) = json.as_object() {
            obj.iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        _ => v.to_string(),
                    };
                    format!("{}={}", k, val)
                })
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        }
    }
    
    /// Convierte JobType a path
    fn job_type_to_path(job_type: &JobType) -> &'static str {
        match job_type {
            JobType::IsPrime => "isprime",
            JobType::Factor => "factor",
            JobType::Pi => "pi",
            JobType::Mandelbrot => "mandelbrot",
            JobType::MatrixMul => "matrixmul",
            JobType::SortFile => "sortfile",
            JobType::WordCount => "wordcount",
            JobType::Grep => "grep",
            JobType::Compress => "compress",
            JobType::HashFile => "hashfile",
            JobType::Fibonacci => "fibonacci",
            JobType::Simulate => "simulate",
        }
    }
    
    /// Despacha a la función handler correcta
    fn dispatch_command(job_type: &JobType, request: &Request) -> Response {
        match job_type {
            JobType::IsPrime => commands::isprime_handler(request),
            JobType::Factor => commands::factor_handler(request),
            JobType::Pi => commands::pi_handler(request),
            JobType::Mandelbrot => commands::mandelbrot_handler(request),
            JobType::MatrixMul => commands::matrixmul_handler(request),
            JobType::SortFile => commands::sortfile_handler(request),
            JobType::WordCount => commands::wordcount_handler(request),
            JobType::Grep => commands::grep_handler(request),
            JobType::Compress => commands::compress_handler(request),
            JobType::HashFile => commands::hashfile_handler(request),
            JobType::Fibonacci => commands::fibonacci_handler(request),
            JobType::Simulate => commands::simulate_handler(request),
        }
    }
    
    /// Encola un nuevo job
    pub fn submit_job(
        &self,
        job_type: JobType,
        params: String,
        priority: JobPriority,
    ) -> Result<String, String> {
        // Generar ID único
        let job_id = self.generate_job_id();
        
        // Crear metadata
        let metadata = JobMetadata::new(job_id.clone(), job_type, params, priority);
        
        // Seleccionar cola
        let queue = if job_type.is_cpu_bound() {
            &self.cpu_queue
        } else if job_type.is_io_bound() {
            &self.io_queue
        } else {
            &self.basic_queue
        };
        
        // Encolar
        queue.enqueue(metadata.clone())?;
        
        // Guardar en storage
        self.storage.save(&metadata)
            .map_err(|e| format!("Storage error: {}", e))?;
        
        Ok(job_id)
    }
    
    /// Obtiene el estado de un job
    pub fn get_job_status(&self, job_id: &str) -> Option<JobMetadata> {
        self.storage.get(job_id)
    }
    
    /// Cancela un job
    pub fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        // Buscar en las colas primero
        let removed = self.cpu_queue.remove_by_id(job_id)
            .or_else(|| self.io_queue.remove_by_id(job_id))
            .or_else(|| self.basic_queue.remove_by_id(job_id));
        
        if let Some(mut job) = removed {
            // Estaba en cola, marcarlo cancelado
            job.mark_canceled();
            self.storage.save(&job)
                .map_err(|e| format!("Storage error: {}", e))?;
            return Ok(());
        }
        
        // Si no está en cola, verificar si está running
        let is_running = {
            let running = self.running_jobs.lock().unwrap();
            running.contains_key(job_id)
        };
        
        if is_running {
            return Err("Job is currently running and cannot be canceled".to_string());
        }
        
        // Si no está ni en cola ni running, verificar si ya terminó
        if let Some(job) = self.storage.get(job_id) {
            if job.is_terminal() {
                return Err("Job already finished".to_string());
            }
        }
        
        Err("Job not found".to_string())
    }
    
    /// Genera un ID único para el job
    fn generate_job_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut hasher = DefaultHasher::new();
        now.hash(&mut hasher);
        thread::current().id().hash(&mut hasher);
        
        format!("job-{:016x}", hasher.finish())
    }
    
    /// Obtiene estadísticas de las colas
    pub fn get_queue_stats(&self) -> serde_json::Value {
        let cpu_stats = self.cpu_queue.stats();
        let io_stats = self.io_queue.stats();
        let basic_stats = self.basic_queue.stats();
        
        let running_count = {
            let running = self.running_jobs.lock().unwrap();
            running.len()
        };
        
        serde_json::json!({
            "cpu_queue": {
                "total": cpu_stats.total,
                "capacity": cpu_stats.capacity,
                "low": cpu_stats.low_priority,
                "normal": cpu_stats.normal_priority,
                "high": cpu_stats.high_priority,
            },
            "io_queue": {
                "total": io_stats.total,
                "capacity": io_stats.capacity,
                "low": io_stats.low_priority,
                "normal": io_stats.normal_priority,
                "high": io_stats.high_priority,
            },
            "basic_queue": {
                "total": basic_stats.total,
                "capacity": basic_stats.capacity,
            },
            "running_jobs": running_count,
        })
    }
}

impl Clone for JobManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cpu_queue: self.cpu_queue.clone(),
            io_queue: self.io_queue.clone(),
            basic_queue: self.basic_queue.clone(),
            storage: self.storage.clone(),
            running_jobs: Arc::clone(&self.running_jobs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::JobStatus;
    use std::fs;
    use std::path::PathBuf;

    /// Crea un JobManager SIN workers y con storage en un path temporal,
    /// asegurando que el directorio padre exista para evitar "No such file or directory".
    fn manager_with_zero_workers() -> JobManager {
        let mut cfg = JobManagerConfig::default();
        cfg.cpu_workers = 0;
        cfg.io_workers = 0;
        cfg.basic_workers = 0;

        // Construir ruta: <tmp>/http_server_tests/<pid>/jobs.json
        let mut base = std::env::temp_dir();
        base.push("http_server_tests");
        base.push(format!("pid-{}", std::process::id()));
        fs::create_dir_all(&base).expect("create temp storage dir");

        let storage_path: PathBuf = base.join("jobs.json");
        cfg.storage_path = storage_path.to_string_lossy().to_string();

        JobManager::new(cfg)
    }

    #[test]
    fn test_json_to_query_string_basic() {
        let v = serde_json::json!({"n":97, "verbose": true, "label":"X"});
        let qs = JobManager::json_to_query_string(&v);
        // El orden puede variar; validemos presencia
        assert!(qs.contains("n=97"));
        assert!(qs.contains("verbose=true"));
        assert!(qs.contains("label=X"));
    }

    #[test]
    fn test_job_type_to_path_mapping() {
        assert_eq!(JobManager::job_type_to_path(&JobType::IsPrime), "isprime");
        assert_eq!(JobManager::job_type_to_path(&JobType::Factor), "factor");
        assert_eq!(JobManager::job_type_to_path(&JobType::Pi), "pi");
        assert_eq!(JobManager::job_type_to_path(&JobType::Mandelbrot), "mandelbrot");
        assert_eq!(JobManager::job_type_to_path(&JobType::MatrixMul), "matrixmul");
        assert_eq!(JobManager::job_type_to_path(&JobType::SortFile), "sortfile");
        assert_eq!(JobManager::job_type_to_path(&JobType::WordCount), "wordcount");
        assert_eq!(JobManager::job_type_to_path(&JobType::Grep), "grep");
        assert_eq!(JobManager::job_type_to_path(&JobType::Compress), "compress");
        assert_eq!(JobManager::job_type_to_path(&JobType::HashFile), "hashfile");
        assert_eq!(JobManager::job_type_to_path(&JobType::Fibonacci), "fibonacci");
        assert_eq!(JobManager::job_type_to_path(&JobType::Simulate), "simulate");
    }

    #[test]
    fn test_dispatch_command_basic_route() {
        let req = Request::parse(b"GET /isprime?n=97 HTTP/1.0\r\n\r\n").unwrap();
        let resp = JobManager::dispatch_command(&JobType::IsPrime, &req);
        // No asumimos contenido exacto, pero debe ser HTTP válido
        assert!(resp.status().as_u16() >= 200);
    }

    #[test]
    fn test_submit_and_get_status_flow() {
        let mgr = manager_with_zero_workers();

        // submit_job
        let params = serde_json::json!({"n":97}).to_string();
        let job_id = mgr.submit_job(JobType::IsPrime, params, JobPriority::Normal)
            .expect("submit should succeed");

        // get_job_status (persistido)
        let md = mgr.get_job_status(&job_id).expect("status exists after submit");
        assert_eq!(md.id, job_id);
        assert_eq!(md.status, JobStatus::Queued);
    }

    #[test]
    fn test_cancel_job_when_queued() {
        let mgr = manager_with_zero_workers();

        let params = serde_json::json!({"n":97}).to_string();
        let job_id = mgr.submit_job(JobType::IsPrime, params, JobPriority::Low)
            .expect("submit ok");

        // cancelar
        mgr.cancel_job(&job_id).expect("cancel queued ok");

        // verificar storage
        let md = mgr.get_job_status(&job_id).expect("exists");
        assert_eq!(md.status, JobStatus::Canceled);
    }

    #[test]
    fn test_cancel_job_not_found() {
        let mgr = manager_with_zero_workers();
        let err = mgr.cancel_job("job-no-such").unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_cancel_job_finished_conflict() {
        let mgr = manager_with_zero_workers();

        // Creamos un job terminado manualmente en storage para cubrir rama "already finished"
        let mut md = JobMetadata::new("job-finished".into(), JobType::IsPrime, "{}".into(), JobPriority::Normal);
        md.mark_done(r#"{"ok":true}"#.into());
        mgr.storage.save(&md).unwrap();

        let err = mgr.cancel_job("job-finished").unwrap_err();
        assert!(err.contains("already finished"));
    }

    #[test]
    fn test_cancel_job_running_conflict() {
        let mgr = manager_with_zero_workers();

        // Simular running metiéndolo en el mapa running_jobs
        let job_id = "job-running-sim".to_string();
        {
            let mut running = mgr.running_jobs.lock().unwrap();
            running.insert(job_id.clone(), ());
        }
        let err = mgr.cancel_job(&job_id).unwrap_err();
        assert!(err.contains("cannot be canceled") || err.contains("currently running"));
    }

    #[test]
    fn test_execute_job_ok_isprime() {
        // cubrir execute_job camino feliz
        let params = serde_json::json!({"n":97}).to_string();
        let md = JobMetadata::new("job-x".into(), JobType::IsPrime, params, JobPriority::Normal);

        let body = JobManager::execute_job(&md, 2_000).expect("should finish well");
        // No asumimos JSON exacto, pero debe contener algo
        assert!(!body.is_empty());
    }

    #[test]
    fn test_execute_job_timeout_simulate() {
        // cubrir timeout en execute_job usando Simulate con retardo
        let params = serde_json::json!({"ms":100}).to_string();
        let md = JobMetadata::new("job-slow".into(), JobType::Simulate, params, JobPriority::Normal);

        let err = JobManager::execute_job(&md, 1).unwrap_err();
        assert!(err.to_lowercase().contains("timeout"));
    }

    #[test]
    fn test_generate_job_id_is_uniqueish() {
        let mgr = manager_with_zero_workers();
        let a = mgr.generate_job_id();
        let b = mgr.generate_job_id();
        assert!(a.starts_with("job-") && b.starts_with("job-"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_get_queue_stats_json_shape() {
        let mgr = manager_with_zero_workers();
        let v = mgr.get_queue_stats();
        assert!(v.get("cpu_queue").is_some());
        assert!(v.get("io_queue").is_some());
        assert!(v.get("basic_queue").is_some());
        assert!(v.get("running_jobs").is_some());
    }
}