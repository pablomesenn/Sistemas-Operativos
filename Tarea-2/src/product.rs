use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Product {
    pub id: u32,
    pub arrival_time: Duration,
    pub entry_cutting: Option<Duration>,
    pub exit_cutting: Option<Duration>,
    pub entry_assembly: Option<Duration>,
    pub exit_assembly: Option<Duration>,
    pub entry_packaging: Option<Duration>,
    pub exit_packaging: Option<Duration>,
    // Tracking de tiempo acumulado en cada estación
    pub accumulated_cutting_ms: u64,
    pub accumulated_assembly_ms: u64,
    pub accumulated_packaging_ms: u64,
}

impl Product {
    pub fn new(id: u32, now: Duration) -> Self {
        Product {
            id,
            arrival_time: now,
            entry_cutting: None,
            exit_cutting: None,
            entry_assembly: None,
            exit_assembly: None,
            entry_packaging: None,
            exit_packaging: None,
            accumulated_cutting_ms: 0,
            accumulated_assembly_ms: 0,
            accumulated_packaging_ms: 0,
        }
    }
    
    /// Tiempo total desde llegada hasta salida final
    pub fn turnaround_time(&self) -> Option<Duration> {
        self.exit_packaging.map(|exit| exit - self.arrival_time)
    }
    
    /// Tiempo total de espera en colas (no procesando)
    pub fn waiting_time(&self) -> Option<Duration> {
        if let (Some(entry_cut), Some(exit_cut), 
                Some(entry_asm), Some(exit_asm),
                Some(entry_pack), Some(_exit_pack)) = 
            (self.entry_cutting, self.exit_cutting,
             self.entry_assembly, self.exit_assembly,
             self.entry_packaging, self.exit_packaging) {
            
            // Tiempo esperando antes de cada etapa
            let wait_before_cutting = entry_cut - self.arrival_time;
            let wait_before_assembly = entry_asm.saturating_sub(exit_cut);
            let wait_before_packaging = entry_pack.saturating_sub(exit_asm);
            
            Some(wait_before_cutting + wait_before_assembly + wait_before_packaging)
        } else {
            None
        }
    }
    
    /// Tiempo de procesamiento real (suma de todas las etapas)
    pub fn processing_time(&self) -> Option<Duration> {
        if let (Some(entry_cut), Some(exit_cut), 
                Some(entry_asm), Some(exit_asm),
                Some(entry_pack), Some(exit_pack)) = 
            (self.entry_cutting, self.exit_cutting,
             self.entry_assembly, self.exit_assembly,
             self.entry_packaging, self.exit_packaging) {
            
            let cutting_time = exit_cut - entry_cut;
            let assembly_time = exit_asm - entry_asm;
            let packaging_time = exit_pack - entry_pack;
            
            Some(cutting_time + assembly_time + packaging_time)
        } else {
            None
        }
    }
}