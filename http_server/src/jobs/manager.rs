//! # Job Manager
//!
//! Coordina la cola de jobs, su ejecución y almacenamiento.

use crate::jobs::job::{Job, JobTask, JobPriority, JobStatus};
use crate::commands;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Manager que coordina todos los jobs
pub struct JobManager {
    /// Almacenamiento de todos los jobs (por ID)
    jobs: Arc<Mutex<HashMap<String, Job>>>,
    
    /// Cola de jobs pendientes (por prioridad y FIFO)
    queue: Arc<Mutex<VecDeque<String>>>,
    
    /// Número de workers activos procesando jobs
    active_workers: Arc<Mutex<usize>>,
    
    /// Máximo de workers concurrentes
    max_workers: usize,
    
    /// Flag para shutdown graceful
    shutdown: Arc<Mutex<bool>>,
}

impl JobManager {
    /// Crea un nuevo Job Manager
    /// 
    /// # Argumentos
    /// - `max_workers`: Máximo de jobs ejecutándose simultáneamente
    pub fn new(max_workers: usize) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active_workers: Arc::new(Mutex::new(0)),
            max_workers,
            shutdown: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Inicia el procesamiento de jobs en background
    pub fn start(&self) {
        let jobs = Arc::clone(&self.jobs);
        let queue = Arc::clone(&self.queue);
        let active_workers = Arc::clone(&self.active_workers);
        let max_workers = self.max_workers;
        let shutdown = Arc::clone(&self.shutdown);
        
        thread::spawn(move || {
            loop {
                // Verificar si debemos apagar
                {
                    let should_shutdown = *shutdown.lock().unwrap();
                    if should_shutdown {
                        println!("🛑 Job Manager shutting down...");
                        break;
                    }
                }
                
                // Verificar si hay espacio para más workers
                let current_workers = *active_workers.lock().unwrap();
                if current_workers >= max_workers {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                
                // Obtener siguiente job de la cola
                let job_id = {
                    let mut q = queue.lock().unwrap();
                    q.pop_front()
                };
                
                if let Some(job_id) = job_id {
                    // Incrementar contador de workers
                    {
                        let mut workers = active_workers.lock().unwrap();
                        *workers += 1;
                    }
                    
                    // Clonar referencias para el thread
                    let jobs_clone = Arc::clone(&jobs);
                    let active_workers_clone = Arc::clone(&active_workers);
                    
                    // Spawn thread para ejecutar el job
                    thread::spawn(move || {
                        Self::execute_job(&job_id, &jobs_clone);
                        
                        // Decrementar contador al terminar
                        let mut workers = active_workers_clone.lock().unwrap();
                        *workers -= 1;
                    });
                } else {
                    // No hay jobs, esperar un poco
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }
    
    /// Encola un nuevo job
    /// 
    /// # Retorna
    /// El ID del job creado
    pub fn submit(&self, task: JobTask, priority: JobPriority) -> String {
        let job = Job::new(task, priority);
        let job_id = job.id().to_string();
        
        // Guardar job
        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(job_id.clone(), job);
        }
        
        // Agregar a la cola
        {
            let mut queue = self.queue.lock().unwrap();
            
            // Insertar según prioridad
            // High: al frente
            // Normal: en medio
            // Low: al final
            match priority {
                JobPriority::High => queue.push_front(job_id.clone()),
                JobPriority::Low => queue.push_back(job_id.clone()),
                JobPriority::Normal => {
                    // Insertar después de los High pero antes de los Low
                    let pos = queue.iter().position(|id| {
                        if let Some(j) = self.jobs.lock().unwrap().get(id) {
                            j.priority() < JobPriority::Normal
                        } else {
                            false
                        }
                    }).unwrap_or(queue.len());
                    
                    queue.insert(pos, job_id.clone());
                }
            }
        }
        
        println!("📝 Job {} enqueued (priority: {})", &job_id[..8], priority.as_str());
        
        job_id
    }
    
    /// Obtiene el estado de un job
    pub fn get_status(&self, job_id: &str) -> Option<String> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).map(|job| job.to_status_json())
    }
    
    /// Obtiene el resultado de un job
    pub fn get_result(&self, job_id: &str) -> Option<String> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).map(|job| job.to_result_json())
    }
    
    /// Cancela un job
    pub fn cancel(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        
        if let Some(job) = jobs.get_mut(job_id) {
            let status = job.status();
            
            match status {
                JobStatus::Queued => {
                    // Remover de la cola
                    let mut queue = self.queue.lock().unwrap();
                    if let Some(pos) = queue.iter().position(|id| id == job_id) {
                        queue.remove(pos);
                    }
                    job.mark_canceled();
                    println!("❌ Job {} canceled (was queued)", &job_id[..8]);
                    true
                }
                JobStatus::Running => {
                    // No podemos cancelar jobs en ejecución sin soporte de cancelación
                    println!("⚠️  Job {} cannot be canceled (already running)", &job_id[..8]);
                    false
                }
                _ => {
                    println!("⚠️  Job {} cannot be canceled (status: {})", &job_id[..8], status.as_str());
                    false
                }
            }
        } else {
            false
        }
    }
    
    /// Obtiene métricas del Job Manager
    pub fn get_metrics(&self) -> String {
        let jobs = self.jobs.lock().unwrap();
        let queue = self.queue.lock().unwrap();
        let active_workers = *self.active_workers.lock().unwrap();
        
        // Contar jobs por estado
        let mut queued = 0;
        let mut running = 0;
        let mut done = 0;
        let mut error = 0;
        let mut canceled = 0;
        
        for job in jobs.values() {
            match job.status() {
                JobStatus::Queued => queued += 1,
                JobStatus::Running => running += 1,
                JobStatus::Done => done += 1,
                JobStatus::Error => error += 1,
                JobStatus::Canceled => canceled += 1,
            }
        }
        
        format!(
            r#"{{"total_jobs": {}, "queued": {}, "running": {}, "done": {}, "error": {}, "canceled": {}, "queue_size": {}, "active_workers": {}, "max_workers": {}}}"#,
            jobs.len(), queued, running, done, error, canceled,
            queue.len(), active_workers, self.max_workers
        )
    }
    
    /// Ejecuta un job (llamado desde un worker thread)
    fn execute_job(job_id: &str, jobs: &Arc<Mutex<HashMap<String, Job>>>) {
        println!("⚙️  Executing job {}", &job_id[..8]);
        
        // Obtener el job y marcarlo como running
        let task = {
            let mut jobs_guard = jobs.lock().unwrap();
            if let Some(job) = jobs_guard.get_mut(job_id) {
                job.mark_started();
                job.task().clone()
            } else {
                println!("❌ Job {} not found!", job_id);
                return;
            }
        };
        
        // Ejecutar la tarea según su tipo
        let result = Self::execute_task(&task);
        
        // Actualizar el job con el resultado
        {
            let mut jobs_guard = jobs.lock().unwrap();
            if let Some(job) = jobs_guard.get_mut(job_id) {
                match result {
                    Ok(json) => {
                        job.mark_done(json);
                        println!("✅ Job {} completed", &job_id[..8]);
                    }
                    Err(error) => {
                        job.mark_error(error);
                        println!("❌ Job {} failed", &job_id[..8]);
                    }
                }
            }
        }
    }
    
    /// Ejecuta una tarea específica
    fn execute_task(task: &JobTask) -> Result<String, String> {
        // Crear un Request fake para pasar a los handlers
        use crate::http::Request;
        
        let request_str = match task {
            JobTask::IsPrime { n } => format!("GET /isprime?n={} HTTP/1.0\r\n\r\n", n),
            JobTask::Factor { n } => format!("GET /factor?n={} HTTP/1.0\r\n\r\n", n),
            JobTask::Pi { digits } => format!("GET /pi?digits={} HTTP/1.0\r\n\r\n", digits),
            JobTask::Mandelbrot { width, height, max_iter } => {
                format!("GET /mandelbrot?width={}&height={}&max_iter={} HTTP/1.0\r\n\r\n", 
                       width, height, max_iter)
            }
            JobTask::MatrixMul { size, seed } => {
                format!("GET /matrixmul?size={}&seed={} HTTP/1.0\r\n\r\n", size, seed)
            }
            JobTask::SortFile { name, algo } => {
                format!("GET /sortfile?name={}&algo={} HTTP/1.0\r\n\r\n", name, algo)
            }
            JobTask::WordCount { name } => {
                format!("GET /wordcount?name={} HTTP/1.0\r\n\r\n", name)
            }
            JobTask::Grep { name, pattern } => {
                format!("GET /grep?name={}&pattern={} HTTP/1.0\r\n\r\n", name, pattern)
            }
            JobTask::Compress { name, codec } => {
                format!("GET /compress?name={}&codec={} HTTP/1.0\r\n\r\n", name, codec)
            }
            JobTask::HashFile { name, algo } => {
                format!("GET /hashfile?name={}&algo={} HTTP/1.0\r\n\r\n", name, algo)
            }
        };
        
        let request = Request::parse(request_str.as_bytes())
            .map_err(|e| format!("Failed to parse request: {}", e))?;
        
        // Llamar al handler apropiado
        let response = match task {
            JobTask::IsPrime { .. } => commands::isprime_handler(&request),
            JobTask::Factor { .. } => commands::factor_handler(&request),
            JobTask::Pi { .. } => commands::pi_handler(&request),
            JobTask::Mandelbrot { .. } => commands::mandelbrot_handler(&request),
            JobTask::MatrixMul { .. } => commands::matrixmul_handler(&request),
            JobTask::SortFile { .. } => commands::sortfile_handler(&request),
            JobTask::WordCount { .. } => commands::wordcount_handler(&request),
            JobTask::Grep { .. } => commands::grep_handler(&request),
            JobTask::Compress { .. } => commands::compress_handler(&request),
            JobTask::HashFile { .. } => commands::hashfile_handler(&request),
        };
        
        // Extraer el body del response
        let body = String::from_utf8(response.body().to_vec())
            .map_err(|e| format!("Failed to convert response: {}", e))?;
        
        // Verificar si fue exitoso
        if response.status().is_success() {
            Ok(body)
        } else {
            Err(body)
        }
    }
    
    /// Detiene el Job Manager
    pub fn shutdown(&self) {
        let mut shutdown = self.shutdown.lock().unwrap();
        *shutdown = true;
    }
}

impl Drop for JobManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_job_manager_creation() {
        let manager = JobManager::new(4);
        let metrics = manager.get_metrics();
        assert!(metrics.contains("\"total_jobs\": 0"));
    }
    
    #[test]
    fn test_job_submit() {
        let manager = JobManager::new(4);
        let task = JobTask::IsPrime { n: 97 };
        let job_id = manager.submit(task, JobPriority::Normal);
        
        assert!(job_id.len() > 0);
        
        // Verificar que el job existe
        let status = manager.get_status(&job_id);
        assert!(status.is_some());
    }
}