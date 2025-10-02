mod product;
mod factory;

use factory::Factory;
use std::thread;
use std::time::Duration;

fn main() {
    let factory = Factory::new();

    for id in 1..=5 {
        println!("📦 Product {} arrived", id);
        factory.send_product(id);
        thread::sleep(Duration::from_millis(1000)); // Simular llegada de productos cada segundo
    }

    thread::sleep(Duration::from_secs(20)); // esperar a que todos terminen
}