use crate::product::Product;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum SchedulingAlgorithm {
    FCFS,
    RoundRobin { quantum_ms: u64 },
}

pub struct Scheduler {
    algorithm: SchedulingAlgorithm,
    queue: VecDeque<WorkUnit>,
}

#[derive(Clone)]
pub struct WorkUnit {
    pub product: Product,
    pub remaining_time_ms: u64,
    pub total_time_ms: u64,
}

impl Scheduler {
    pub fn new(algorithm: SchedulingAlgorithm) -> Self {
        Scheduler {
            algorithm,
            queue: VecDeque::new(),
        }
    }
    
    pub fn add_product(&mut self, product: Product, processing_time_ms: u64) {
        self.queue.push_back(WorkUnit {
            product,
            remaining_time_ms: processing_time_ms,
            total_time_ms: processing_time_ms,
        });
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Obtiene el siguiente trabajo a procesar
    /// Retorna (producto, tiempo_a_procesar_ms)
    pub fn get_next(&mut self) -> Option<(Product, u64)> {
        if self.queue.is_empty() {
            return None;
        }
        
        match self.algorithm {
            SchedulingAlgorithm::FCFS => {
                // FCFS: procesar completamente
                let work_unit = self.queue.pop_front()?;
                Some((work_unit.product, work_unit.remaining_time_ms))
            }
            SchedulingAlgorithm::RoundRobin { quantum_ms } => {
                // Round Robin: procesar hasta quantum
                let work_unit = self.queue.pop_front()?;
                let time_to_process = work_unit.remaining_time_ms.min(quantum_ms);
                Some((work_unit.product, time_to_process))
            }
        }
    }
    
    /// Devuelve un producto a la cola si no terminó
    pub fn return_incomplete(&mut self, product: Product, time_processed: u64, total_time: u64) {
        let remaining = total_time.saturating_sub(time_processed);
        if remaining > 0 {
            self.queue.push_back(WorkUnit {
                product,
                remaining_time_ms: remaining,
                total_time_ms: total_time,
            });
        }
    }
}