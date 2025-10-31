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
    
    // ==================== JobStatus Tests ====================
    
    #[test]
    fn test_job_status_serialization() {
        let status = JobStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");
    }
    
    #[test]
    fn test_job_status_serialization_all() {
        assert_eq!(serde_json::to_string(&JobStatus::Queued).unwrap(), "\"queued\"");
        assert_eq!(serde_json::to_string(&JobStatus::Running).unwrap(), "\"running\"");
        assert_eq!(serde_json::to_string(&JobStatus::Done).unwrap(), "\"done\"");
        assert_eq!(serde_json::to_string(&JobStatus::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&JobStatus::Canceled).unwrap(), "\"canceled\"");
        assert_eq!(serde_json::to_string(&JobStatus::Timeout).unwrap(), "\"timeout\"");
    }
    
    #[test]
    fn test_job_status_display() {
        assert_eq!(format!("{:?}", JobStatus::Queued), "Queued");
        assert_eq!(format!("{:?}", JobStatus::Running), "Running");
        assert_eq!(format!("{:?}", JobStatus::Done), "Done");
        assert_eq!(format!("{:?}", JobStatus::Error), "Error");
        assert_eq!(format!("{:?}", JobStatus::Canceled), "Canceled");
        assert_eq!(format!("{:?}", JobStatus::Timeout), "Timeout");
    }
    
    // ==================== JobPriority Tests ====================
    
    #[test]
    fn test_job_priority_ordering() {
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
    }
    
    #[test]
    fn test_job_priority_comparison() {
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
        assert!(JobPriority::High > JobPriority::Low);
    }
    
    #[test]
    fn test_job_priority_default() {
        let priority: JobPriority = Default::default();
        assert_eq!(priority, JobPriority::Normal);
    }
    
    #[test]
    fn test_job_priority_from_str() {
        assert_eq!(JobPriority::from_str("low"), Some(JobPriority::Low));
        assert_eq!(JobPriority::from_str("normal"), Some(JobPriority::Normal));
        assert_eq!(JobPriority::from_str("high"), Some(JobPriority::High));
        assert_eq!(JobPriority::from_str("LOW"), Some(JobPriority::Low));
        assert_eq!(JobPriority::from_str("NORMAL"), Some(JobPriority::Normal));
        assert_eq!(JobPriority::from_str("HIGH"), Some(JobPriority::High));
        assert_eq!(JobPriority::from_str("invalid"), None);
        assert_eq!(JobPriority::from_str(""), None);
    }
    
    // ==================== JobType Tests ====================
    
    #[test]
    fn test_job_type_categorization() {
        assert!(JobType::IsPrime.is_cpu_bound());
        assert!(!JobType::IsPrime.is_io_bound());
        assert!(JobType::SortFile.is_io_bound());
        assert!(!JobType::SortFile.is_cpu_bound());
    }
    
    #[test]
    fn test_job_type_from_task_name() {
        // CPU-bound
        assert_eq!(JobType::from_task_name("isprime"), Some(JobType::IsPrime));
        assert_eq!(JobType::from_task_name("factor"), Some(JobType::Factor));
        assert_eq!(JobType::from_task_name("pi"), Some(JobType::Pi));
        assert_eq!(JobType::from_task_name("mandelbrot"), Some(JobType::Mandelbrot));
        assert_eq!(JobType::from_task_name("matrixmul"), Some(JobType::MatrixMul));
        
        // IO-bound
        assert_eq!(JobType::from_task_name("sortfile"), Some(JobType::SortFile));
        assert_eq!(JobType::from_task_name("wordcount"), Some(JobType::WordCount));
        assert_eq!(JobType::from_task_name("grep"), Some(JobType::Grep));
        assert_eq!(JobType::from_task_name("compress"), Some(JobType::Compress));
        assert_eq!(JobType::from_task_name("hashfile"), Some(JobType::HashFile));
        
        // Basic
        assert_eq!(JobType::from_task_name("fibonacci"), Some(JobType::Fibonacci));
        assert_eq!(JobType::from_task_name("simulate"), Some(JobType::Simulate));
        
        // Invalid
        assert_eq!(JobType::from_task_name("invalid"), None);
    }
    
    #[test]
    fn test_job_type_from_task_name_case_insensitive() {
        assert_eq!(JobType::from_task_name("ISPRIME"), Some(JobType::IsPrime));
        assert_eq!(JobType::from_task_name("IsPrime"), Some(JobType::IsPrime));
        assert_eq!(JobType::from_task_name("IsPrImE"), Some(JobType::IsPrime));
        assert_eq!(JobType::from_task_name("FACTOR"), Some(JobType::Factor));
        assert_eq!(JobType::from_task_name("SortFile"), Some(JobType::SortFile));
    }
    
    #[test]
    fn test_job_type_is_cpu_bound() {
        assert!(JobType::IsPrime.is_cpu_bound());
        assert!(JobType::Factor.is_cpu_bound());
        assert!(JobType::Pi.is_cpu_bound());
        assert!(JobType::Mandelbrot.is_cpu_bound());
        assert!(JobType::MatrixMul.is_cpu_bound());
        assert!(!JobType::Fibonacci.is_cpu_bound());
        assert!(!JobType::SortFile.is_cpu_bound());
    }
    
    #[test]
    fn test_job_type_is_io_bound() {
        assert!(JobType::SortFile.is_io_bound());
        assert!(JobType::WordCount.is_io_bound());
        assert!(JobType::Grep.is_io_bound());
        assert!(JobType::Compress.is_io_bound());
        assert!(JobType::HashFile.is_io_bound());
        assert!(!JobType::IsPrime.is_io_bound());
        assert!(!JobType::Fibonacci.is_io_bound());
    }
    
    #[test]
    fn test_job_type_basic_tasks() {
        assert!(!JobType::Fibonacci.is_cpu_bound());
        assert!(!JobType::Fibonacci.is_io_bound());
        assert!(!JobType::Simulate.is_cpu_bound());
        assert!(!JobType::Simulate.is_io_bound());
    }
    
    // ==================== JobMetadata Tests ====================
    
    #[test]
    fn test_job_metadata_new() {
        let metadata = JobMetadata::new(
            "test-123".to_string(),
            JobType::IsPrime,
            r#"{"num": 17}"#.to_string(),
            JobPriority::High,
        );
        
        assert_eq!(metadata.id, "test-123");
        assert_eq!(metadata.status, JobStatus::Queued);
        assert_eq!(metadata.priority, JobPriority::High);
        assert_eq!(metadata.progress, 0);
        assert!(metadata.result.is_none());
        assert!(metadata.error.is_none());
        assert!(metadata.started_at.is_none());
        assert!(metadata.finished_at.is_none());
        assert_eq!(metadata.eta_ms, None);
    }
    
    #[test]
    fn test_job_metadata_timestamps() {
        let job = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        assert!(job.created_at > 0);
        assert!(job.started_at.is_none());
        assert!(job.finished_at.is_none());
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
    
    #[test]
    fn test_job_metadata_mark_running() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.mark_running();
        assert_eq!(metadata.status, JobStatus::Running);
        assert!(metadata.started_at.is_some());
    }
    
    #[test]
    fn test_job_metadata_mark_done() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.mark_done(r#"{"result": true}"#.to_string());
        assert_eq!(metadata.status, JobStatus::Done);
        assert!(metadata.finished_at.is_some());
        assert!(metadata.result.is_some());
        assert_eq!(metadata.progress, 100);
    }
    
    #[test]
    fn test_job_metadata_mark_error() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.mark_error("Test error".to_string());
        assert_eq!(metadata.status, JobStatus::Error);
        assert!(metadata.finished_at.is_some());
        assert!(metadata.error.is_some());
        assert_eq!(metadata.error.unwrap(), "Test error");
    }
    
    #[test]
    fn test_job_metadata_mark_canceled() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.mark_canceled();
        assert_eq!(metadata.status, JobStatus::Canceled);
        assert!(metadata.finished_at.is_some());
    }
    
    #[test]
    fn test_job_metadata_mark_timeout() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.mark_timeout();
        assert_eq!(metadata.status, JobStatus::Timeout);
        assert!(metadata.finished_at.is_some());
        assert!(metadata.error.is_some());
        let error_msg = metadata.error.unwrap();
        assert!(error_msg.to_lowercase().contains("timeout") || error_msg.contains("exceeded"));
    }
    
    #[test]
    fn test_job_metadata_update_progress() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.update_progress(50, None);
        assert_eq!(metadata.progress, 50);
        assert_eq!(metadata.eta_ms, None);
        
        metadata.update_progress(75, Some(5000));
        assert_eq!(metadata.progress, 75);
        assert_eq!(metadata.eta_ms, Some(5000));
    }
    
    #[test]
    fn test_job_metadata_update_progress_clamping() {
        let mut metadata = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        metadata.update_progress(150, None);
        assert_eq!(metadata.progress, 100);
    }
    
    #[test]
    fn test_job_metadata_is_terminal_all_states() {
        let mut job = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        assert!(!job.is_terminal()); // Queued
        
        job.mark_running();
        assert!(!job.is_terminal()); // Running
        
        let mut job_done = job.clone();
        job_done.mark_done("result".to_string());
        assert!(job_done.is_terminal()); // Done
        
        let mut job_error = job.clone();
        job_error.mark_error("error".to_string());
        assert!(job_error.is_terminal()); // Error
        
        let mut job_canceled = job.clone();
        job_canceled.mark_canceled();
        assert!(job_canceled.is_terminal()); // Canceled
        
        let mut job_timeout = job.clone();
        job_timeout.mark_timeout();
        assert!(job_timeout.is_terminal()); // Timeout
    }
    
    #[test]
    fn test_job_metadata_is_cancelable() {
        let mut job = JobMetadata::new(
            "test".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        assert!(job.is_cancelable()); // Queued
        
        job.mark_running();
        assert!(job.is_cancelable()); // Running
        
        job.mark_done("result".to_string());
        assert!(!job.is_cancelable()); // Done
    }
    
    #[test]
    fn test_job_metadata_full_lifecycle() {
        let mut job = JobMetadata::new(
            "test".to_string(),
            JobType::Factor,
            r#"{"n": 360}"#.to_string(),
            JobPriority::High,
        );
        
        // Initial state
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.progress, 0);
        
        // Start execution
        job.mark_running();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());
        
        // Update progress
        job.update_progress(25, Some(3000));
        assert_eq!(job.progress, 25);
        assert_eq!(job.eta_ms, Some(3000));
        
        job.update_progress(50, Some(1500));
        assert_eq!(job.progress, 50);
        
        job.update_progress(75, Some(750));
        assert_eq!(job.progress, 75);
        
        // Complete
        job.mark_done(r#"{"factors": [2,2,2,3,3,5]}"#.to_string());
        assert_eq!(job.status, JobStatus::Done);
        assert_eq!(job.progress, 100);
        assert!(job.finished_at.is_some());
        assert!(job.result.is_some());
        assert!(job.is_terminal());
        assert!(!job.is_cancelable());
    }
}