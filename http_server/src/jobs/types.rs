//! # Tipos y Estructuras para el Sistema de Jobs
//! src/jobs/types.rs
//!
//! Define los tipos fundamentales para el manejo de trabajos asíncronos.

use serde::{Serialize, Deserialize};

/// Estado de un job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job en cola esperando ejecución
    Queued,
    
    /// Job ejecutándose actualmente
    Running,
    
    /// Job completado exitosamente
    Done,
    
    /// Job falló con error
    Error,
    
    /// Job cancelado por el usuario
    Canceled,
    
    /// Job excedió el timeout
    Timeout,
}

/// Prioridad de un job
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

impl JobPriority {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(JobPriority::Low),
            "normal" => Some(JobPriority::Normal),
            "high" => Some(JobPriority::High),
            _ => None,
        }
    }
}

/// Tipo de comando que ejecuta el job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobType {
    // CPU-bound
    IsPrime,
    Factor,
    Pi,
    Mandelbrot,
    MatrixMul,
    
    // IO-bound
    SortFile,
    WordCount,
    Grep,
    Compress,
    HashFile,
    
    // Básicos que pueden ser largos
    Fibonacci,
    Simulate,
}

impl JobType {
    pub fn from_task_name(task: &str) -> Option<Self> {
        match task.to_lowercase().as_str() {
            "isprime" => Some(JobType::IsPrime),
            "factor" => Some(JobType::Factor),
            "pi" => Some(JobType::Pi),
            "mandelbrot" => Some(JobType::Mandelbrot),
            "matrixmul" => Some(JobType::MatrixMul),
            "sortfile" => Some(JobType::SortFile),
            "wordcount" => Some(JobType::WordCount),
            "grep" => Some(JobType::Grep),
            "compress" => Some(JobType::Compress),
            "hashfile" => Some(JobType::HashFile),
            "fibonacci" => Some(JobType::Fibonacci),
            "simulate" => Some(JobType::Simulate),
            _ => None,
        }
    }
    
    pub fn is_cpu_bound(&self) -> bool {
        matches!(
            self,
            JobType::IsPrime
                | JobType::Factor
                | JobType::Pi
                | JobType::Mandelbrot
                | JobType::MatrixMul
        )
    }
    
    pub fn is_io_bound(&self) -> bool {
        matches!(
            self,
            JobType::SortFile
                | JobType::WordCount
                | JobType::Grep
                | JobType::Compress
                | JobType::HashFile
        )
    }
}

/// Metadatos de un job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// ID único del job
    pub id: String,
    
    /// Tipo de job
    pub job_type: JobType,
    
    /// Estado actual
    pub status: JobStatus,
    
    /// Prioridad
    pub priority: JobPriority,
    
    /// Parámetros del job (JSON serializado)
    pub params: String,
    
    /// Timestamp de creación
    pub created_at: u64,
    
    /// Timestamp de inicio (si ya comenzó)
    pub started_at: Option<u64>,
    
    /// Timestamp de finalización (si ya terminó)
    pub finished_at: Option<u64>,
    
    /// Progreso (0-100)
    pub progress: u8,
    
    /// ETA estimado en milisegundos
    pub eta_ms: Option<u64>,
    
    /// Resultado del job (JSON serializado)
    pub result: Option<String>,
    
    /// Mensaje de error (si falló)
    pub error: Option<String>,
}

impl JobMetadata {
    /// Crea un nuevo job metadata
    pub fn new(id: String, job_type: JobType, params: String, priority: JobPriority) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id,
            job_type,
            status: JobStatus::Queued,
            priority,
            params,
            created_at: now,
            started_at: None,
            finished_at: None,
            progress: 0,
            eta_ms: None,
            result: None,
            error: None,
        }
    }
    
    /// Marca el job como iniciado
    pub fn mark_running(&mut self) {
        self.status = JobStatus::Running;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.started_at = Some(now);
    }
    
    /// Marca el job como completado
    pub fn mark_done(&mut self, result: String) {
        self.status = JobStatus::Done;
        self.progress = 100;
        self.result = Some(result);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.finished_at = Some(now);
    }
    
    /// Marca el job como fallido
    pub fn mark_error(&mut self, error: String) {
        self.status = JobStatus::Error;
        self.error = Some(error);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.finished_at = Some(now);
    }
    
    /// Marca el job como cancelado
    pub fn mark_canceled(&mut self) {
        self.status = JobStatus::Canceled;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.finished_at = Some(now);
    }
    
    /// Marca el job como timeout
    pub fn mark_timeout(&mut self) {
        self.status = JobStatus::Timeout;
        self.error = Some("Job exceeded maximum execution time".to_string());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.finished_at = Some(now);
    }
    
    /// Actualiza el progreso
    pub fn update_progress(&mut self, progress: u8, eta_ms: Option<u64>) {
        self.progress = progress.min(100);
        self.eta_ms = eta_ms;
    }
    
    /// Verifica si el job está en estado terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Done | JobStatus::Error | JobStatus::Canceled | JobStatus::Timeout
        )
    }
    
    /// Verifica si el job puede ser cancelado
    pub fn is_cancelable(&self) -> bool {
        matches!(self.status, JobStatus::Queued | JobStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_job_status_serialization() {
        let status = JobStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");
    }
    
    #[test]
    fn test_job_priority_ordering() {
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
    }
    
    #[test]
    fn test_job_type_categorization() {
        assert!(JobType::IsPrime.is_cpu_bound());
        assert!(!JobType::IsPrime.is_io_bound());
        assert!(JobType::SortFile.is_io_bound());
        assert!(!JobType::SortFile.is_cpu_bound());
    }
    
    #[test]
    fn test_job_metadata_lifecycle() {
        let mut job = JobMetadata::new(
            "test-123".to_string(),
            JobType::IsPrime,
            r#"{"n":97}"#.to_string(),
            JobPriority::Normal,
        );
        
        assert_eq!(job.status, JobStatus::Queued);
        assert!(!job.is_terminal());
        assert!(job.is_cancelable());
        
        job.mark_running();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());
        
        job.mark_done(r#"{"result":true}"#.to_string());
        assert_eq!(job.status, JobStatus::Done);
        assert!(job.is_terminal());
        assert!(!job.is_cancelable());
    }
}