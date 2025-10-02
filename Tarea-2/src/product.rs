use std::time::Duration;

#[derive(Debug)]
pub struct Product {
    pub id: u32,
    pub arrival_time: Duration,
    pub entry_cutting: Option<Duration>,
    pub exit_cutting: Option<Duration>,
    pub entry_assembly: Option<Duration>,
    pub exit_assembly: Option<Duration>,
    pub entry_packaging: Option<Duration>,
    pub exit_packaging: Option<Duration>,
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
        }
    }

    pub fn turnaround_time(&self) -> Option<Duration> {
        self.exit_packaging.map(|exit| exit - self.arrival_time)
    }
}
