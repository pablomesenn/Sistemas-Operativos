mod product;
mod factory;
mod scheduler;

use factory::Factory;
use scheduler::SchedulingAlgorithm;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Simulación con FCFS ===");
    run_simulation(SchedulingAlgorithm::FCFS);
    
    println!("\n\n=== Simulación con Round Robin ===");
    run_simulation(SchedulingAlgorithm::RoundRobin { quantum_ms: 750 });
}

fn run_simulation(algorithm: SchedulingAlgorithm) {
    let factory = Factory::new(5, algorithm);
    
    // Generar 10 productos con tiempos de llegada simulados
    let arrival_intervals = vec![0, 500, 800, 1200, 1500, 2000, 2300, 2800, 3200, 3500];
    
    for (idx, interval) in arrival_intervals.iter().enumerate() {
        let id = idx as u32 + 1;
        thread::sleep(Duration::from_millis(*interval));
        println!("📦 Product {} arrived at {}ms", id, interval);
        factory.send_product(id).expect("Failed to send product");
    }
    
    // Cierre ordenado y obtener estadísticas
    let stats = factory.shutdown();
    
    println!("\n📊 === RESUMEN DE ESTADÍSTICAS ===");
    println!("Algoritmo: {:?}", stats.algorithm);
    println!("Total de productos procesados: {}", stats.total_products);
    println!("⏱️  Tiempo promedio de espera: {:.2}s", stats.avg_waiting_time);
    println!("⏱️  Tiempo promedio de turnaround: {:.2}s", stats.avg_turnaround_time);
    
    println!("\n📋 Orden final de procesamiento:");
    for (idx, id) in stats.completion_order.iter().enumerate() {
        println!("  {}. Product {}", idx + 1, id);
    }
    
    println!("\n📈 Detalle por producto:");
    for product_stat in stats.product_stats {
        println!("  Product {}: Espera = {:.2}s, Turnaround = {:.2}s", 
                 product_stat.id, 
                 product_stat.waiting_time, 
                 product_stat.turnaround_time);
    }
}