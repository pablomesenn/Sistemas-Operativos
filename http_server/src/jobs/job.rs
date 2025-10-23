//! # Estructura de Job
//!
//! Representa un trabajo asíncrono con estado, progreso y resultado.

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Estados posibles de un job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Job está en la cola esperando ser procesado
    Queued,
    
    /// Job está siendo ejecutado
    Running,
    
    /// Job completado exitosamente
    Done,
    
    /// Job falló con error
    Error,
    
    /// Job fue cancelado
    Canceled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Running => "running",
            JobStatus::Done => "done",
            JobStatus::Error => "error",
            JobStatus::Canceled => "canceled",
        }
    }
}

/// Tipo de tarea que ejecutará el job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobTask {
    // CPU-bound
    IsPrime { n: u64 },
    Factor { n: u64 },
    Pi { digits: usize },
    Mandelbrot { width: usize, height: usize, max_iter: u32 },
    MatrixMul { size: usize, seed: u64 },
    
    // IO-bound
    SortFile { name: String, algo: String },
    WordCount { name: String },
    Grep { name: String, pattern: String },
    Compress { name: String, codec: String },
    HashFile { name: String, algo: String },
}

impl JobTask {
    /// Retorna el nombre del tipo de tarea
    pub fn task_type(&self) -> &'static str {
        match self {
            JobTask::IsPrime { .. } => "isprime",
            JobTask::Factor { .. } => "factor",
            JobTask::Pi { .. } => "pi",
            JobTask::Mandelbrot { .. } => "mandelbrot",
            JobTask::MatrixMul { .. } => "matrixmul",
            JobTask::SortFile { .. } => "sortfile",
            JobTask::WordCount { .. } => "wordcount",
            JobTask::Grep { .. } => "grep",
            JobTask::Compress { .. } => "compress",
            JobTask::HashFile { .. } => "hashfile",
        }
    }
}

/// Prioridad del job
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

impl JobPriority {
    pub fn from_str(s: &str) -> Self {
        match s {
            "low" => JobPriority::Low,
            "high" => JobPriority::High,
            _ => JobPriority::Normal,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            JobPriority::Low => "low",
            JobPriority::Normal => "normal",
            JobPriority::High => "high",
        }
    }
}

/// Datos internos mutables del job
#[derive(Debug, Clone)]
struct JobData {
    status: JobStatus,
    progress: u8,        // 0-100
    eta_ms: Option<u64>, // Tiempo estimado de finalización
    result: Option<String>, // Resultado JSON cuando está done
    error: Option<String>,  // Mensaje de error si falló
}

/// Representa un job individual
#[derive(Debug, Clone)]
pub struct Job {
    /// ID único del job (UUID)
    id: String,
    
    /// Tarea a ejecutar
    task: JobTask,
    
    /// Prioridad
    priority: JobPriority,
    
    /// Timestamp de creación
    created_at: Instant,
    
    /// Timestamp de inicio de ejecución
    started_at: Option<Instant>,
    
    /// Timestamp de finalización
    finished_at: Option<Instant>,
    
    /// Datos mutables (protegidos por Mutex)
    data: Arc<Mutex<JobData>>,
}

impl Job {
    /// Crea un nuevo job
    pub fn new(task: JobTask, priority: JobPriority) -> Self {
        // Generar ID único
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        let now = Instant::now();
        now.elapsed().as_nanos().hash(&mut hasher);
        task.task_type().hash(&mut hasher);
        
        let id = format!("{:016x}", hasher.finish());
        
        Self {
            id,
            task,
            priority,
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
            data: Arc::new(Mutex::new(JobData {
                status: JobStatus::Queued,
                progress: 0,
                eta_ms: None,
                result: None,
                error: None,
            })),
        }
    }
    
    /// ID del job
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Tarea del job
    pub fn task(&self) -> &JobTask {
        &self.task
    }
    
    /// Prioridad
    pub fn priority(&self) -> JobPriority {
        self.priority
    }
    
    /// Obtiene el estado actual
    pub fn status(&self) -> JobStatus {
        let data = self.data.lock().unwrap();
        data.status.clone()
    }
    
    /// Actualiza el estado
    pub fn set_status(&self, status: JobStatus) {
        let mut data = self.data.lock().unwrap();
        data.status = status;
    }
    
    /// Obtiene el progreso (0-100)
    pub fn progress(&self) -> u8 {
        let data = self.data.lock().unwrap();
        data.progress
    }
    
    /// Actualiza el progreso
    pub fn set_progress(&self, progress: u8) {
        let mut data = self.data.lock().unwrap();
        data.progress = progress.min(100);
    }
    
    /// Obtiene el ETA en milisegundos
    pub fn eta_ms(&self) -> Option<u64> {
        let data = self.data.lock().unwrap();
        data.eta_ms
    }
    
    /// Actualiza el ETA
    pub fn set_eta_ms(&self, eta_ms: Option<u64>) {
        let mut data = self.data.lock().unwrap();
        data.eta_ms = eta_ms;
    }
    
    /// Marca el job como iniciado
    pub fn mark_started(&mut self) {
        self.started_at = Some(Instant::now());
        self.set_status(JobStatus::Running);
    }
    
    /// Marca el job como completado con resultado
    pub fn mark_done(&mut self, result: String) {
        self.finished_at = Some(Instant::now());
        let mut data = self.data.lock().unwrap();
        data.status = JobStatus::Done;
        data.progress = 100;
        data.result = Some(result);
    }
    
    /// Marca el job como fallido con error
    pub fn mark_error(&mut self, error: String) {
        self.finished_at = Some(Instant::now());
        let mut data = self.data.lock().unwrap();
        data.status = JobStatus::Error;
        data.error = Some(error);
    }
    
    /// Marca el job como cancelado
    pub fn mark_canceled(&mut self) {
        self.finished_at = Some(Instant::now());
        let mut data = self.data.lock().unwrap();
        data.status = JobStatus::Canceled;
    }
    
    /// Obtiene el resultado (si está done)
    pub fn result(&self) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.result.clone()
    }
    
    /// Obtiene el error (si está error)
    pub fn error(&self) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.error.clone()
    }
    
    /// Tiempo transcurrido desde creación
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }
    
    /// Tiempo de ejecución (si ya empezó)
    pub fn execution_time(&self) -> Option<Duration> {
        self.started_at.map(|start| {
            if let Some(end) = self.finished_at {
                end.duration_since(start)
            } else {
                start.elapsed()
            }
        })
    }
    
    /// Convierte el job a JSON para /jobs/status
    pub fn to_status_json(&self) -> String {
        let data = self.data.lock().unwrap();
        
        let eta_str = if let Some(eta) = data.eta_ms {
            format!(r#", "eta_ms": {}"#, eta)
        } else {
            String::new()
        };
        
        format!(
            r#"{{"status": "{}", "progress": {}{}}}"#,
            data.status.as_str(),
            data.progress,
            eta_str
        )
    }
    
    /// Convierte el job a JSON para /jobs/result
    pub fn to_result_json(&self) -> String {
        let data = self.data.lock().unwrap();
        
        match data.status {
            JobStatus::Done => {
                if let Some(ref result) = data.result {
                    result.clone()
                } else {
                    r#"{"error": "No result available"}"#.to_string()
                }
            }
            JobStatus::Error => {
                let error_msg = data.error.as_deref().unwrap_or("Unknown error");
                format!(r#"{{"error": "{}"}}"#, error_msg)
            }
            JobStatus::Canceled => {
                r#"{"error": "Job was canceled"}"#.to_string()
            }
            _ => {
                format!(r#"{{"error": "Job not finished yet, status: {}"}}"#, data.status.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_job_creation() {
        let task = JobTask::IsPrime { n: 97 };
        let job = Job::new(task, JobPriority::Normal);
        
        assert_eq!(job.status(), JobStatus::Queued);
        assert_eq!(job.progress(), 0);
        assert!(job.id().len() > 0);
    }
    
    #[test]
    fn test_job_lifecycle() {
        let task = JobTask::Factor { n: 360 };
        let mut job = Job::new(task, JobPriority::High);
        
        assert_eq!(job.status(), JobStatus::Queued);
        
        job.mark_started();
        assert_eq!(job.status(), JobStatus::Running);
        
        job.set_progress(50);
        assert_eq!(job.progress(), 50);
        
        job.mark_done(r#"{"result": "success"}"#.to_string());
        assert_eq!(job.status(), JobStatus::Done);
        assert_eq!(job.progress(), 100);
        assert!(job.result().is_some());
    }
    
    #[test]
    fn test_job_error() {
        let task = JobTask::Pi { digits: 100 };
        let mut job = Job::new(task, JobPriority::Normal);
        
        job.mark_started();
        job.mark_error("Calculation failed".to_string());
        
        assert_eq!(job.status(), JobStatus::Error);
        assert!(job.error().is_some());
    }
    
    #[test]
    fn test_job_priority() {
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
    }
}