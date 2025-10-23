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
    use crate::jobs::types::{JobType, JobPriority};
    
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
}