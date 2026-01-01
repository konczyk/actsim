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

#[derive(Debug, PartialEq)]
pub enum FilterResult {
    Pending,
    Promoted,
    Trusted,
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

    pub fn insert(&mut self, input: &T) -> FilterResult {
        if self.sbf.contains(input) {
            return FilterResult::Trusted;
        }

        let count = self.pending.entry(input.clone()).or_insert(0);
        *count += 1;

        if *count >= self.threshold {
            self.pending.remove(input);
            self.sbf.insert(input);
            return FilterResult::Promoted;
        }

        FilterResult::Pending
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending() {
        let mut fm = FilterManager::<&str>::new();
        let plane = "ALPHA1";

        // 1st check
        assert_eq!(FilterResult::Pending, fm.insert(&plane));
        assert_eq!(1, fm.pending.len());
        assert_eq!(Some(&1u8), fm.pending.get(&plane));
        assert!(fm.sbf.filters.iter().all(|f| f.bits.iter().all(|x| x.count_ones() == 0)));

        // 2nd check
        assert_eq!(FilterResult::Pending, fm.insert(&plane));
        assert_eq!(1, fm.pending.len());
        assert_eq!(Some(&2u8), fm.pending.get(&plane));
        assert!(fm.sbf.filters.iter().all(|f| f.bits.iter().all(|x| x.count_ones() == 0)));

        // 3rd check
        assert_eq!(FilterResult::Promoted, fm.insert(&plane));
        assert!(fm.pending.is_empty());
        assert!(fm.sbf.filters.iter().all(|f| f.bits.iter().any(|x| x.count_ones() > 0)));

        // 4th check (bf)
        assert_eq!(FilterResult::Trusted, fm.insert(&plane));
        assert!(fm.pending.is_empty());
        assert!(fm.sbf.filters.iter().all(|f| f.bits.iter().any(|x| x.count_ones() > 0)));
    }
    #[test]
    fn test_prune() {
        let mut fm = FilterManager::<&str>::new();
        let plane = "ALPHA1";

        fm.insert(&plane);
        fm.insert(&plane);
        fm.insert(&plane);
        assert_eq!(FilterResult::Trusted, fm.insert(&plane));

        fm.prune(Duration::from_secs(10));

        assert_eq!(FilterResult::Trusted, fm.insert(&plane));
    }
}