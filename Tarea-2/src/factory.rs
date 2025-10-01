use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};

use crate::product::Product;

pub struct Factory {
    pub cutting_queue: VecDeque<Product>,
    pub assembly_queue: VecDeque<Product>,
    pub packaging_queue: VecDeque<Product>,
    pub current_time: Instant,
}

impl Factory {
    pub fn new() -> Self {
        Factory {
            cutting_queue: VecDeque::new(),
            assembly_queue: VecDeque::new(),
            packaging_queue: VecDeque::new(),
            current_time: Instant::now(),
        }
    }

    pub fn cutting_stage(&mut self) {
        if let Some(mut p) = self.cutting_queue.pop_front() {
            p.entry_cutting = Some(self.current_time.elapsed());
            thread::sleep(Duration::from_secs(2));
            p.exit_cutting = Some(self.current_time.elapsed());
            self.assembly_queue.push_back(p);
        }
    }

    pub fn assembly_stage(&mut self) {
        if let Some(mut p) = self.assembly_queue.pop_front() {
            p.entry_assembly = Some(self.current_time.elapsed());
            thread::sleep(Duration::from_secs(3));
            p.exit_assembly = Some(self.current_time.elapsed());
            self.packaging_queue.push_back(p);
        }
    }

    pub fn packaging_stage(&mut self) {
        if let Some(mut p) = self.packaging_queue.pop_front() {
            p.entry_packaging = Some(self.current_time.elapsed());
            thread::sleep(Duration::from_secs(1));
            p.exit_packaging = Some(self.current_time.elapsed());
            println!("✅ Product {:?} finished", p);
            if let Some(turnaround) = p.turnaround_time() {
                println!("   Turnaround time: {:?}", turnaround);
            }
        }
    }
}