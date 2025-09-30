// Factory module, main module of the project managing the overall workflow
use std::thread;
use std::time::Duration;

pub struct Factory {
    name: String,
}

impl Factory {
    pub fn new(name: String) -> Self {
        Factory { name }
    }

    pub fn operate(&self) {
        let handle1 = thread::spawn(self.cut_stage());
        let handle2 = thread::spawn(self.assemble_stage());
        let handle3 = thread::spawn(self.pack_stage());    

        // Esperar a que todos los hilos terminen
        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();
    }

    fn cut_stage(&self) {
        for _i in 1..=5 {
            println!("Cutting stage in factory: {}", self.name);
        }
    }

    fn assemble_stage(&self) {
        for _i in 1..=5 {
            thread::sleep(Duration::from_millis(500));
            println!("Assembling stage in factory: {}", self.name);
        }
    }

    fn pack_stage(&self) {
        for _i in 1..=5 {
            thread::sleep(Duration::from_millis(1500));
            println!("Packing stage in factory: {}", self.name);
        }
    }
}
