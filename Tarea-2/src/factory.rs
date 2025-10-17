use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use crate::product::Product;
use crate::scheduler::{Scheduler, SchedulingAlgorithm};

pub struct Factory {
    tx_input: mpsc::SyncSender<Product>,       // Canal para enviar productos a la primera estación
    start: Instant,                            // Marca de tiempo del inicio de la simulación
    handles: Vec<JoinHandle<()>>,              // Manejadores de los hilos de las estaciones
    stats_collector: Arc<Mutex<StatsCollector>>, // Recolector de estadísticas compartido entre hilos
}

pub struct FactoryStats {
    pub algorithm: SchedulingAlgorithm,
    pub total_products: usize,
    pub avg_waiting_time: f64,
    pub avg_turnaround_time: f64,
    pub completion_order: Vec<u32>,
    pub product_stats: Vec<ProductStats>,
}

pub struct ProductStats {
    pub id: u32,
    pub waiting_time: f64,
    pub turnaround_time: f64,
}

pub struct StationTimes {
    pub cutting_ms: u64,
    pub assembly_ms: u64,
    pub packaging_ms: u64,
}

impl StationTimes {
    pub fn default() -> Self {
        StationTimes {
            cutting_ms: 2000,
            assembly_ms: 3000,
            packaging_ms: 1000,
        }
    }
}

struct StatsCollector {
    completed_products: Vec<Product>,   // Productos completamente procesados
    completion_order: Vec<u32>,         // Orden en que se completaron
    algorithm: SchedulingAlgorithm,
}

impl StatsCollector {
    fn new(algorithm: SchedulingAlgorithm) -> Self {
        StatsCollector {
            completed_products: Vec::new(),
            completion_order: Vec::new(),
            algorithm,
        }
    }
    
    fn add_completed(&mut self, product: Product) {
        self.completion_order.push(product.id);
        self.completed_products.push(product);
    }
    
    // Calcula tiempos promedio de espera y turnaround a partir de los productos completados
    fn compute_stats(&self) -> FactoryStats {
        let total = self.completed_products.len();
        
        let mut total_waiting = 0.0;
        let mut total_turnaround = 0.0;
        let mut product_stats = Vec::new();
        
        for product in &self.completed_products {
            let waiting = product.waiting_time()
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            let turnaround = product.turnaround_time()
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            
            total_waiting += waiting;
            total_turnaround += turnaround;
            
            product_stats.push(ProductStats {
                id: product.id,
                waiting_time: waiting,
                turnaround_time: turnaround,
            });
        }
        
        let avg_waiting = if total > 0 { total_waiting / total as f64 } else { 0.0 };
        let avg_turnaround = if total > 0 { total_turnaround / total as f64 } else { 0.0 };
        
        FactoryStats {
            algorithm: self.algorithm.clone(),
            total_products: total,
            avg_waiting_time: avg_waiting,
            avg_turnaround_time: avg_turnaround,
            completion_order: self.completion_order.clone(),
            product_stats,
        }
    }
}

impl Factory {
    pub fn new(capacity: usize, algorithm: SchedulingAlgorithm) -> Self {
        Self::new_with_times(capacity, algorithm, StationTimes::default())
    }
    
    // Inicializa la fábrica con tiempos personalizados para cada estación
    pub fn new_with_times(capacity: usize, algorithm: SchedulingAlgorithm, times: StationTimes) -> Self {
        let (tx_input, rx_input) = mpsc::sync_channel::<Product>(capacity);
        let start = Instant::now();
        let mut handles = Vec::new();
        
        let stats_collector = Arc::new(Mutex::new(StatsCollector::new(algorithm.clone())));
        
        let (tx_complete, rx_complete) = mpsc::sync_channel::<Product>(capacity);
        
        // === ESTACIÓN DE CORTE ===
        {
            let algorithm_cut = algorithm.clone();
            let start_clone = start.clone();
            let tx_next = tx_complete.clone();
            let cutting_time = times.cutting_ms;
            
            // Hilo que simula la estación de corte
            let h = thread::spawn(move || {
                let mut scheduler = Scheduler::new(algorithm_cut.clone());
                
                loop {
                    match rx_input.try_recv() {
                        Ok(product) => {
                            scheduler.add_product(product, cutting_time);
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            if scheduler.is_empty() {
                                break; // termina cuando no hay más productos ni conexiones
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                    
                    if let Some((mut product, time_to_process)) = scheduler.get_next() {
                        if product.entry_cutting.is_none() {
                            product.entry_cutting = Some(start_clone.elapsed());
                        }
                        
                        println!("▶️  Product {} procesando en Corte ({}ms, acumulado: {}ms)", 
                                 product.id, time_to_process, product.accumulated_cutting_ms);
                        
                        // Simula el tiempo de procesamiento
                        thread::sleep(Duration::from_millis(time_to_process));
                        product.accumulated_cutting_ms += time_to_process;
                        
                        if product.accumulated_cutting_ms >= cutting_time {
                            // Producto completó la estación
                            product.exit_cutting = Some(start_clone.elapsed());
                            println!("✂️  Product {} completó Corte (total: {}ms)", product.id, product.accumulated_cutting_ms);
                            
                            if let Err(e) = tx_next.send(product) {
                                eprintln!("❌ Error enviando de Corte: {:?}", e);
                                break;
                            }
                        } else {
                            // Producto interrumpido, se reprograma
                            let remaining = cutting_time - product.accumulated_cutting_ms;
                            let accumulated = product.accumulated_cutting_ms;
                            println!("🔄 Product {} interrumpido en Corte (quedan {}ms)", product.id, remaining);
                            scheduler.return_incomplete(product, accumulated, cutting_time);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(50)); // espera breve antes de volver a intentar
                    }
                }
                
                drop(tx_next);
            });
            handles.push(h);
        }
        
        // === ESTACIÓN DE ENSAMBLAJE ===
        {
            let algorithm_asm = algorithm.clone();
            let start_clone = start.clone();
            let rx_from_cutting = rx_complete;
            let (tx_to_packaging, rx_to_packaging) = mpsc::sync_channel::<Product>(capacity);
            let assembly_time = times.assembly_ms;
            
            // Hilo que simula la estación de ensamblaje
            let h = thread::spawn(move || {
                let mut scheduler = Scheduler::new(algorithm_asm.clone());
                
                loop {
                    match rx_from_cutting.try_recv() {
                        Ok(product) => {
                            scheduler.add_product(product, assembly_time);
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            if scheduler.is_empty() {
                                break;
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                    
                    if let Some((mut product, time_to_process)) = scheduler.get_next() {
                        if product.entry_assembly.is_none() {
                            product.entry_assembly = Some(start_clone.elapsed());
                        }
                        
                        println!("▶️  Product {} procesando en Ensamblaje ({}ms, acumulado: {}ms)", 
                                 product.id, time_to_process, product.accumulated_assembly_ms);
                        
                        thread::sleep(Duration::from_millis(time_to_process));
                        product.accumulated_assembly_ms += time_to_process;
                        
                        if product.accumulated_assembly_ms >= assembly_time {
                            product.exit_assembly = Some(start_clone.elapsed());
                            println!("🔧 Product {} completó Ensamblaje (total: {}ms)", product.id, product.accumulated_assembly_ms);
                            
                            if let Err(e) = tx_to_packaging.send(product) {
                                eprintln!("❌ Error enviando de Ensamblaje: {:?}", e);
                                break;
                            }
                        } else {
                            // Si no termina, vuelve al scheduler con el progreso guardado
                            let remaining = assembly_time - product.accumulated_assembly_ms;
                            let accumulated = product.accumulated_assembly_ms;
                            println!("🔄 Product {} interrumpido en Ensamblaje (quedan {}ms)", product.id, remaining);
                            scheduler.return_incomplete(product, accumulated, assembly_time);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                
                drop(tx_to_packaging);
            });
            handles.push(h);
            
            // === ESTACIÓN DE EMPAQUE ===
            let algorithm_pack = algorithm.clone();
            let stats_clone = Arc::clone(&stats_collector);
            let start_clone = start.clone();
            let packaging_time = times.packaging_ms;
            
            // Hilo que simula la estación de empaque final
            let h = thread::spawn(move || {
                let mut scheduler = Scheduler::new(algorithm_pack.clone());
                
                loop {
                    match rx_to_packaging.try_recv() {
                        Ok(product) => {
                            scheduler.add_product(product, packaging_time);
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            if scheduler.is_empty() {
                                break;
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                    
                    if let Some((mut product, time_to_process)) = scheduler.get_next() {
                        if product.entry_packaging.is_none() {
                            product.entry_packaging = Some(start_clone.elapsed());
                        }
                        
                        println!("▶️  Product {} procesando en Empaque ({}ms, acumulado: {}ms)", 
                                 product.id, time_to_process, product.accumulated_packaging_ms);
                        
                        thread::sleep(Duration::from_millis(time_to_process));
                        product.accumulated_packaging_ms += time_to_process;
                        
                        if product.accumulated_packaging_ms >= packaging_time {
                            // Producto finalizado completamente
                            product.exit_packaging = Some(start_clone.elapsed());
                            println!("📦 Product {} completó Empaque (total: {}ms)", product.id, product.accumulated_packaging_ms);
                            println!("✅ Product {} TERMINADO", product.id);
                            
                            // Se guarda en el recolector de estadísticas
                            if let Ok(mut collector) = stats_clone.lock() {
                                collector.add_completed(product);
                            }
                        } else {
                            let remaining = packaging_time - product.accumulated_packaging_ms;
                            let accumulated = product.accumulated_packaging_ms;
                            println!("🔄 Product {} interrumpido en Empaque (quedan {}ms)", product.id, remaining);
                            scheduler.return_incomplete(product, accumulated, packaging_time);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            });
            handles.push(h);
        }
        
        Factory {
            tx_input,
            start,
            handles,
            stats_collector,
        }
    }
    
    // Envía un nuevo producto al canal de entrada
    pub fn send_product(&self, id: u32) -> Result<(), mpsc::SendError<Product>> {
        let p = Product::new(id, self.start.elapsed());
        self.tx_input.send(p)
    }
    
    // Finaliza la ejecución de la fábrica y devuelve las estadísticas globales
    pub fn shutdown(self) -> FactoryStats {
        drop(self.tx_input);
        
        for handle in self.handles {
            if let Err(e) = handle.join() {
                eprintln!("⚠️ Error al unir hilo: {:?}", e);
            }
        }
        
        self.stats_collector.lock().unwrap().compute_stats()
    }
}