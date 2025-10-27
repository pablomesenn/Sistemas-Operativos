//! # Cola de Prioridad para Jobs
//! src/jobs/queue.rs
//!
//! Implementa una cola thread-safe que ordena jobs por prioridad.

use crate::jobs::types::JobMetadata;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, Condvar};
use std::cmp::Ordering;

/// Wrapper para ordenar jobs en el heap
#[derive(Clone)]
struct QueuedJob {
    metadata: JobMetadata,
}

impl QueuedJob {
    fn new(metadata: JobMetadata) -> Self {
        Self { metadata }
    }
}

// Implementar ordenamiento: mayor prioridad primero
impl PartialEq for QueuedJob {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.priority == other.metadata.priority
            && self.metadata.created_at == other.metadata.created_at
    }
}

impl Eq for QueuedJob {}

impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primero comparar por prioridad (mayor prioridad primero)
        match self.metadata.priority.cmp(&other.metadata.priority) {
            Ordering::Equal => {
                // Si misma prioridad, FIFO (menor timestamp primero)
                // Invertimos para que BinaryHeap nos dé el menor
                other.metadata.created_at.cmp(&self.metadata.created_at)
            }
            ordering => ordering,
        }
    }
}

/// Cola de prioridad thread-safe
pub struct JobQueue {
    /// Heap interno
    heap: Arc<Mutex<BinaryHeap<QueuedJob>>>,
    
    /// Condvar para notificar cuando hay nuevos jobs
    condvar: Arc<Condvar>,
    
    /// Capacidad máxima de la cola
    max_capacity: usize,
}

impl JobQueue {
    /// Crea una nueva cola con capacidad máxima
    pub fn new(max_capacity: usize) -> Self {
        Self {
            heap: Arc::new(Mutex::new(BinaryHeap::new())),
            condvar: Arc::new(Condvar::new()),
            max_capacity,
        }
    }
    
    /// Encola un job
    /// 
    /// Retorna Ok(()) si se encoló exitosamente,
    /// Err si la cola está llena
    pub fn enqueue(&self, metadata: JobMetadata) -> Result<(), String> {
        let mut heap = self.heap.lock().unwrap();
        
        // Verificar capacidad
        if heap.len() >= self.max_capacity {
            return Err(format!(
                "Queue is full (max capacity: {})",
                self.max_capacity
            ));
        }
        
        heap.push(QueuedJob::new(metadata));
        
        // Notificar a workers esperando
        self.condvar.notify_one();
        
        Ok(())
    }
    
    /// Desencola el job de mayor prioridad
    /// 
    /// Bloquea hasta que haya un job disponible
    pub fn dequeue(&self) -> JobMetadata {
        let mut heap = self.heap.lock().unwrap();
        
        loop {
            if let Some(job) = heap.pop() {
                return job.metadata;
            }
            
            // Esperar a que haya jobs
            heap = self.condvar.wait(heap).unwrap();
        }
    }
    
    /// Intenta desencolar sin bloquear
    /// 
    /// Retorna Some(metadata) si hay un job, None si la cola está vacía
    pub fn try_dequeue(&self) -> Option<JobMetadata> {
        let mut heap = self.heap.lock().unwrap();
        heap.pop().map(|job| job.metadata)
    }
    
    /// Retorna el tamaño actual de la cola
    pub fn len(&self) -> usize {
        let heap = self.heap.lock().unwrap();
        heap.len()
    }
    
    /// Verifica si la cola está vacía
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Retorna la capacidad máxima
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }
    
    /// Verifica si la cola está llena
    pub fn is_full(&self) -> bool {
        self.len() >= self.max_capacity
    }
    
    /// Busca un job por ID (sin removerlo)
    pub fn find_by_id(&self, job_id: &str) -> Option<JobMetadata> {
        let heap = self.heap.lock().unwrap();
        heap.iter()
            .find(|job| job.metadata.id == job_id)
            .map(|job| job.metadata.clone())
    }
    
    /// Remueve un job específico por ID (para cancelación)
    pub fn remove_by_id(&self, job_id: &str) -> Option<JobMetadata> {
        let mut heap = self.heap.lock().unwrap();
        
        // Convertir heap a vec, filtrar, y reconstruir
        let mut jobs: Vec<QueuedJob> = heap.drain().collect();
        
        let removed = jobs.iter()
            .position(|job| job.metadata.id == job_id)
            .map(|idx| jobs.remove(idx).metadata);
        
        // Reconstruir heap con los jobs restantes
        *heap = jobs.into_iter().collect();
        
        removed
    }
    
    /// Obtiene estadísticas de la cola
    pub fn stats(&self) -> QueueStats {
        let heap = self.heap.lock().unwrap();
        
        let mut by_priority = [0usize; 3]; // Low, Normal, High
        
        for job in heap.iter() {
            let idx = job.metadata.priority as usize;
            by_priority[idx] += 1;
        }
        
        QueueStats {
            total: heap.len(),
            capacity: self.max_capacity,
            low_priority: by_priority[0],
            normal_priority: by_priority[1],
            high_priority: by_priority[2],
        }
    }
}

/// Estadísticas de una cola
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: usize,
    pub capacity: usize,
    pub low_priority: usize,
    pub normal_priority: usize,
    pub high_priority: usize,
}

impl Clone for JobQueue {
    fn clone(&self) -> Self {
        Self {
            heap: Arc::clone(&self.heap),
            condvar: Arc::clone(&self.condvar),
            max_capacity: self.max_capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::JobType;
    use crate::jobs::types::JobPriority;
    
    #[test]
    fn test_queue_ordering() {
        let queue = JobQueue::new(100);
        
        // Encolar con diferentes prioridades
        let job1 = JobMetadata::new(
            "1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Low,
        );
        
        let job2 = JobMetadata::new(
            "2".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::High,
        );
        
        let job3 = JobMetadata::new(
            "3".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        queue.enqueue(job1).unwrap();
        queue.enqueue(job2).unwrap();
        queue.enqueue(job3).unwrap();
        
        // Debe salir en orden: High, Normal, Low
        let out1 = queue.try_dequeue().unwrap();
        assert_eq!(out1.priority, JobPriority::High);
        
        let out2 = queue.try_dequeue().unwrap();
        assert_eq!(out2.priority, JobPriority::Normal);
        
        let out3 = queue.try_dequeue().unwrap();
        assert_eq!(out3.priority, JobPriority::Low);
    }
    
    #[test]
    fn test_queue_capacity() {
        let queue = JobQueue::new(2);
        
        let job1 = JobMetadata::new(
            "1".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        let job2 = JobMetadata::new(
            "2".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        let job3 = JobMetadata::new(
            "3".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        assert!(queue.enqueue(job1).is_ok());
        assert!(queue.enqueue(job2).is_ok());
        assert!(queue.enqueue(job3).is_err()); // Cola llena
    }
    
    #[test]
    fn test_remove_by_id() {
        let queue = JobQueue::new(100);
        
        let job = JobMetadata::new(
            "test-123".to_string(),
            JobType::IsPrime,
            "{}".to_string(),
            JobPriority::Normal,
        );
        
        queue.enqueue(job).unwrap();
        assert_eq!(queue.len(), 1);
        
        let removed = queue.remove_by_id("test-123");
        assert!(removed.is_some());
        assert_eq!(queue.len(), 0);
    }
}