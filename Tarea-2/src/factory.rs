use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::product::Product;

pub struct Factory {
    pub tx_cut: Sender<Product>,
    start: Instant,
}

impl Factory {
    pub fn new() -> Self {
        let (tx_cut, rx_cut) = mpsc::channel::<Product>();
        let (tx_asm, rx_asm) = mpsc::channel::<Product>();
        let (tx_pack, rx_pack) = mpsc::channel::<Product>();

        let start = Instant::now();

        // Hilo de Corte
        let tx_asm_clone = tx_asm.clone();
        let start_clone = start.clone();
        thread::spawn(move || {
            while let Ok(mut p) = rx_cut.recv() {
                p.entry_cutting = Some(start_clone.elapsed());
                thread::sleep(Duration::from_secs(2));
                p.exit_cutting = Some(start_clone.elapsed());
                tx_asm_clone.send(p).unwrap();
            }
        });

        // Hilo de Ensamblaje
        let tx_pack_clone = tx_pack.clone();
        let start_clone = start.clone();
        thread::spawn(move || {
            while let Ok(mut p) = rx_asm.recv() {
                p.entry_assembly = Some(start_clone.elapsed());
                thread::sleep(Duration::from_secs(3));
                p.exit_assembly = Some(start_clone.elapsed());
                tx_pack_clone.send(p).unwrap();
            }
        });

        // Hilo de Empaque
        let start_clone = start.clone();
        thread::spawn(move || {
            while let Ok(mut p) = rx_pack.recv() {
                p.entry_packaging = Some(start_clone.elapsed());
                thread::sleep(Duration::from_secs(1));
                p.exit_packaging = Some(start_clone.elapsed());

                println!("✅ Product {:?} finished", p);
                if let Some(turnaround) = p.turnaround_time() {
                    println!("   Turnaround: {:?}", turnaround);
                }
            }
        });

        Factory { tx_cut, start }
    }

    pub fn send_product(&self, id: u32) {
        let p = Product::new(id, self.start.elapsed());
        self.tx_cut.send(p).unwrap();
    }
}