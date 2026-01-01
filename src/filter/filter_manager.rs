use crate::filter::bloom_filter::ScalableBloomFilter;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

pub struct FilterStats {
    pub layer_count: usize,
    pub total_bits: usize,
    pub fill_ratio: f64,
    pub est_fpr: f64,
}

pub struct FilterManager<T: Hash> {
    sbf: ScalableBloomFilter,
    pub pending: HashMap<T, u8>,
    threshold: u8,
}

impl<T: Clone + Eq + Hash> FilterManager<T> {
    pub fn new() -> Self {
        Self {
            sbf: ScalableBloomFilter::new(),
            pending: HashMap::new(),
            threshold: 3
        }
    }

    pub fn fpr(&self) -> f64 {
        self.sbf.fpr()
    }

    // return true if the input is new
    pub fn insert(&mut self, input: &T) -> bool {
        // input is in the bloom filter
        if self.sbf.contains(input) {
            return false;
        }

        let count = self.pending.entry(input.clone()).or_insert(0);
        *count += 1;

        if *count >= self.threshold {
            self.pending.remove(input);
            self.sbf.insert(input);
            return false;
        }

        true
    }

    pub fn prune(&mut self, max_age: Duration) {
        self.sbf.prune(max_age);
        self.pending.clear();
    }

    pub fn stats(&self) -> FilterStats {
        let total_bits = self.sbf.filters.iter().map(|l| l.size * 8).sum();
        let set_bits = self.sbf.filters.iter().map(|l| l.bits.iter().map(|v| v.count_ones()).sum::<u32>()).sum::<u32>() as usize;
        FilterStats {
            layer_count: self.sbf.filters.len(),
            total_bits,
            fill_ratio: set_bits as f64 / total_bits as f64,
            est_fpr: self.fpr(),
        }
    }
}