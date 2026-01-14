use std::sync::atomic::AtomicU64;

pub struct SimulationMetrics {
    pub pairs_checked: AtomicU64
}

impl SimulationMetrics {
    pub fn new() -> SimulationMetrics {
        SimulationMetrics {
            pairs_checked: AtomicU64::new(0)
        }
    }
}