use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Product {
    pub id: u32,
    pub arrival_time: Option<Duration>,
    pub entry_cutting: Option<Duration>,
    pub exit_cutting: Option<Duration>,
    pub entry_assembly: Option<Duration>,
    pub exit_assembly: Option<Duration>,
    pub entry_packaging: Option<Duration>,
    pub exit_packaging: Option<Duration>,
}

impl Product {
    pub fn new(id: u32) -> Self {
        Product {
            id,
            arrival_time: None,
            entry_cutting: None,
            exit_cutting: None,
            entry_assembly: None,
            exit_assembly: None,
            entry_packaging: None,
            exit_packaging: None,
        }
    }

    pub fn turnaround_time(&self) -> Option<Duration> {
        if let (Some(entry), Some(exit)) = (self.arrival_time, self.exit_packaging) {
            Some(exit - entry)
        } else {
            None
        }
    }
}