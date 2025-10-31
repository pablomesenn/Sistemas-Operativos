//! # Persistencia Efímera de Jobs
//! src/jobs/storage.rs
//!
//! Permite que los metadatos de jobs sobrevivan a un graceful restart.
//! Usa un archivo JSON simple en disco.

use crate::jobs::types::JobMetadata;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Storage para persistir metadatos de jobs
pub struct JobStorage {
    /// Ruta al archivo de persistencia
    path: String,
    
    /// Cache en memoria de los jobs
    jobs: Arc<Mutex<HashMap<String, JobMetadata>>>,
}

impl JobStorage {
    /// Crea un nuevo storage y carga datos existentes
    pub fn new(path: &str) -> std::io::Result<Self> {
        let jobs = if Path::new(path).exists() {
            Self::load_from_file(path)?
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            path: path.to_string(),
            jobs: Arc::new(Mutex::new(jobs)),
        })
    }
    
    /// Carga jobs desde el archivo
    fn load_from_file(path: &str) -> std::io::Result<HashMap<String, JobMetadata>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        match serde_json::from_reader(reader) {
            Ok(jobs) => Ok(jobs),
            Err(_) => {
                // Si el archivo está corrupto, empezar limpio
                Ok(HashMap::new())
            }
        }
    }
    
    /// Guarda todos los jobs al archivo
    fn save_to_file(&self) -> std::io::Result<()> {
        let jobs = self.jobs.lock().unwrap();
        
        // Crear archivo temporal primero (atomic write)
        let temp_path = format!("{}.tmp", self.path);
        let file = File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(&mut writer, &*jobs)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        writer.flush()?;
        
        // Renombrar (atómico en sistemas Unix)
        fs::rename(&temp_path, &self.path)?;
        
        Ok(())
    }
    
    /// Guarda o actualiza un job
    pub fn save(&self, metadata: &JobMetadata) -> std::io::Result<()> {
        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(metadata.id.clone(), metadata.clone());
        }
        
        self.save_to_file()
    }
    
    /// Obtiene un job por ID
    pub fn get(&self, job_id: &str) -> Option<JobMetadata> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).cloned()
    }
    
    /// Obtiene todos los jobs
    pub fn get_all(&self) -> Vec<JobMetadata> {
        let jobs = self.jobs.lock().unwrap();
        jobs.values().cloned().collect()
    }
    
    /// Elimina un job
    pub fn remove(&self, job_id: &str) -> std::io::Result<Option<JobMetadata>> {
        let removed = {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.remove(job_id)
        };
        
        if removed.is_some() {
            self.save_to_file()?;
        }
        
        Ok(removed)
    }
    
    /// Obtiene el número de jobs almacenados
    pub fn count(&self) -> usize {
        let jobs = self.jobs.lock().unwrap();
        jobs.len()
    }
    
    /// Limpia jobs terminados antiguos (más de N segundos)
    pub fn cleanup_old(&self, max_age_secs: u64) -> std::io::Result<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let removed_count = {
            let mut jobs = self.jobs.lock().unwrap();
            let before = jobs.len();
            
            jobs.retain(|_, job| {
                if job.is_terminal() {
                    if let Some(finished_at) = job.finished_at {
                        // Mantener si aún no es muy viejo
                        now - finished_at < max_age_secs
                    } else {
                        true
                    }
                } else {
                    true // Mantener jobs no terminados
                }
            });
            
            before - jobs.len()
        };
        
        if removed_count > 0 {
            self.save_to_file()?;
        }
        
        Ok(removed_count)
    }
}

impl Clone for JobStorage {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            jobs: Arc::clone(&self.jobs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::{JobType, JobPriority, JobStatus};
    
    // ==================== Basic Operations ====================
    
    #[test]
    fn test_storage_save_and_get() {
        let temp_file = "/tmp/test_jobs.json";
        let _ = fs::remove_file(temp_file); // Limpiar
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let job = JobMetadata::new(
            "test-123".to_string(),
            JobType::IsPrime,
            r#"{"n":97}"#.to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job).unwrap();
        
        let retrieved = storage.get("test-123").unwrap();
        assert_eq!(retrieved.id, "test-123");
        
        // Cleanup
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_persistence() {
        let temp_file = "/tmp/test_jobs_persist.json";
        let _ = fs::remove_file(temp_file);
        
        // Primera instancia: guardar job
        {
            let storage = JobStorage::new(temp_file).unwrap();
            let job = JobMetadata::new(
                "persist-123".to_string(),
                JobType::Factor,
                r#"{"n":360}"#.to_string(),
                JobPriority::High,
            );
            storage.save(&job).unwrap();
        }
        
        // Segunda instancia: debe cargar el job guardado
        {
            let storage = JobStorage::new(temp_file).unwrap();
            let retrieved = storage.get("persist-123").unwrap();
            assert_eq!(retrieved.id, "persist-123");
            assert_eq!(retrieved.priority, JobPriority::High);
        }
        
        // Cleanup
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Get Operations ====================
    
    #[test]
    fn test_storage_get_nonexistent() {
        let temp_file = "/tmp/test_jobs_get_none.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        let result = storage.get("nonexistent-id");
        
        assert!(result.is_none());
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_get_all() {
        let temp_file = "/tmp/test_jobs_getall.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let job1 = JobMetadata::new(
            "job1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        let job2 = JobMetadata::new(
            "job2".to_string(),
            JobType::Factor,
            "{}".to_string(),
            JobPriority::High,
        );
        
        let job3 = JobMetadata::new(
            "job3".to_string(),
            JobType::Pi,
            "{}".to_string(),
            JobPriority::Low,
        );
        
        storage.save(&job1).unwrap();
        storage.save(&job2).unwrap();
        storage.save(&job3).unwrap();
        
        let all_jobs = storage.get_all();
        assert_eq!(all_jobs.len(), 3);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_get_all_empty() {
        let temp_file = "/tmp/test_jobs_getall_empty.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        let all_jobs = storage.get_all();
        
        assert_eq!(all_jobs.len(), 0);
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Count Operations ====================
    
    #[test]
    fn test_storage_count() {
        let temp_file = "/tmp/test_jobs_count.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        assert_eq!(storage.count(), 0);
        
        let job = JobMetadata::new(
            "job1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job).unwrap();
        assert_eq!(storage.count(), 1);
        
        let job2 = JobMetadata::new(
            "job2".to_string(),
            JobType::Factor,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job2).unwrap();
        assert_eq!(storage.count(), 2);
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Remove Operations ====================
    
    #[test]
    fn test_storage_remove() {
        let temp_file = "/tmp/test_jobs_remove.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let job = JobMetadata::new(
            "job1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job).unwrap();
        assert_eq!(storage.count(), 1);
        
        let removed = storage.remove("job1").unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "job1");
        assert_eq!(storage.count(), 0);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_remove_nonexistent() {
        let temp_file = "/tmp/test_jobs_remove_nonexistent.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let removed = storage.remove("nonexistent").unwrap();
        assert!(removed.is_none());
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Cleanup Operations ====================
    
    #[test]
    fn test_storage_cleanup_old() {
        let temp_file = "/tmp/test_jobs_cleanup.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let mut job = JobMetadata::new(
            "old-job".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        // Marcar como completado hace mucho tiempo
        job.mark_done("result".to_string());
        job.finished_at = Some(1); // Timestamp muy viejo (1970)
        
        storage.save(&job).unwrap();
        assert_eq!(storage.count(), 1);
        
        // Limpiar jobs más viejos de 1 segundo
        let removed = storage.cleanup_old(1).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(storage.count(), 0);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_cleanup_keeps_recent() {
        let temp_file = "/tmp/test_jobs_cleanup_recent.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let mut job = JobMetadata::new(
            "recent-job".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        job.mark_done("result".to_string());
        storage.save(&job).unwrap();
        
        // Limpiar jobs más viejos de 1 hora
        let removed = storage.cleanup_old(3600).unwrap();
        assert_eq!(removed, 0); // No debe remover el job reciente
        assert_eq!(storage.count(), 1);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_cleanup_keeps_non_terminal() {
        let temp_file = "/tmp/test_jobs_cleanup_running.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let mut job = JobMetadata::new(
            "running-job".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        job.mark_running();
        storage.save(&job).unwrap();
        
        // Intentar limpiar
        let removed = storage.cleanup_old(1).unwrap();
        assert_eq!(removed, 0); // No debe remover jobs no terminados
        assert_eq!(storage.count(), 1);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_cleanup_multiple_jobs() {
        let temp_file = "/tmp/test_jobs_cleanup_multiple.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        // Job viejo terminado
        let mut old_job1 = JobMetadata::new(
            "old-1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        old_job1.mark_done("result".to_string());
        old_job1.finished_at = Some(1);
        
        // Otro job viejo terminado
        let mut old_job2 = JobMetadata::new(
            "old-2".to_string(),
            JobType::Factor,
            "{}".to_string(),
            JobPriority::Normal,
        );
        old_job2.mark_error("error".to_string());
        old_job2.finished_at = Some(1);
        
        // Job reciente
        let mut recent_job = JobMetadata::new(
            "recent".to_string(),
            JobType::Pi,
            "{}".to_string(),
            JobPriority::Normal,
        );
        recent_job.mark_done("result".to_string());
        
        // Job en ejecución
        let mut running_job = JobMetadata::new(
            "running".to_string(),
            JobType::Mandelbrot,
            "{}".to_string(),
            JobPriority::Normal,
        );
        running_job.mark_running();
        
        storage.save(&old_job1).unwrap();
        storage.save(&old_job2).unwrap();
        storage.save(&recent_job).unwrap();
        storage.save(&running_job).unwrap();
        
        assert_eq!(storage.count(), 4);
        
        // Limpiar jobs de más de 1 hora
        let removed = storage.cleanup_old(3600).unwrap();
        assert_eq!(removed, 2); // Solo los dos viejos
        assert_eq!(storage.count(), 2); // Quedan el reciente y el running
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Corrupted File Handling ====================
    
    #[test]
    fn test_storage_corrupted_file() {
        use std::io::Write;
        
        let temp_file = "/tmp/test_jobs_corrupted.json";
        let _ = fs::remove_file(temp_file);
        
        // Crear archivo corrupto
        let mut file = File::create(temp_file).unwrap();
        file.write_all(b"{ this is not valid json }").unwrap();
        drop(file);
        
        // Debe poder crear storage sin panic (empieza limpio)
        let storage = JobStorage::new(temp_file);
        assert!(storage.is_ok());
        
        let storage = storage.unwrap();
        assert_eq!(storage.count(), 0);
        
        let _ = fs::remove_file(temp_file);
    }
    
    #[test]
    fn test_storage_empty_corrupted_file() {
        use std::io::Write;
        
        let temp_file = "/tmp/test_jobs_empty_corrupted.json";
        let _ = fs::remove_file(temp_file);
        
        // Crear archivo vacío
        let mut file = File::create(temp_file).unwrap();
        file.write_all(b"").unwrap();
        drop(file);
        
        // Debe poder crear storage sin panic
        let storage = JobStorage::new(temp_file);
        assert!(storage.is_ok());
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Clone Operations ====================
    
    #[test]
    fn test_storage_clone() {
        let temp_file = "/tmp/test_jobs_clone.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let job = JobMetadata::new(
            "job1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job).unwrap();
        
        let storage_clone = storage.clone();
        assert_eq!(storage_clone.count(), 1);
        
        // Ambos storages deben compartir el mismo estado
        let job2 = JobMetadata::new(
            "job2".to_string(),
            JobType::Factor,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage_clone.save(&job2).unwrap();
        assert_eq!(storage.count(), 2);
        assert_eq!(storage_clone.count(), 2);
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== Update Job ====================
    
    #[test]
    fn test_storage_update_job() {
        let temp_file = "/tmp/test_jobs_update.json";
        let _ = fs::remove_file(temp_file);
        
        let storage = JobStorage::new(temp_file).unwrap();
        
        let mut job = JobMetadata::new(
            "job1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        storage.save(&job).unwrap();
        
        // Actualizar el job
        job.mark_running();
        storage.save(&job).unwrap();
        
        let retrieved = storage.get("job1").unwrap();
        assert_eq!(retrieved.status, JobStatus::Running);
        
        // Actualizar nuevamente
        job.mark_done("result".to_string());
        storage.save(&job).unwrap();
        
        let retrieved = storage.get("job1").unwrap();
        assert_eq!(retrieved.status, JobStatus::Done);
        
        let _ = fs::remove_file(temp_file);
    }
    
    // ==================== New Storage with Nonexistent File ====================
    
    #[test]
    fn test_storage_new_nonexistent_file() {
        let temp_file = "/tmp/test_jobs_new_nonexistent.json";
        let _ = fs::remove_file(temp_file);
        
        // Crear storage con archivo que no existe
        let storage = JobStorage::new(temp_file);
        assert!(storage.is_ok());
        
        let storage = storage.unwrap();
        assert_eq!(storage.count(), 0);
        
        let _ = fs::remove_file(temp_file);
    }
}