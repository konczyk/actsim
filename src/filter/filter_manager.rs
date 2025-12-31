use crate::filter::bloom_filter::ScalableBloomFilter;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use std::time::Duration;

pub struct FilterManager<T: Hash> {
    sbf: ScalableBloomFilter,
    cache: HashSet<T>,
    queue: VecDeque<T>,
    cache_max_size: usize,
}

impl<T: Clone + Eq + Hash> FilterManager<T> {
    pub fn new() -> Self {
        let cache_max_size = 128;
        Self {
            sbf: ScalableBloomFilter::new(),
            cache: HashSet::new(),
            queue: VecDeque::with_capacity(cache_max_size),
            cache_max_size
        }
    }

    pub fn fpr(&self) -> f64 {
        self.sbf.fpr()
    }

    pub fn insert(&mut self, input: &T) -> bool {
        if self.cache.contains(input) {
            false
        } else {
            if !self.sbf.contains(input) {
                self.sbf.insert(input);
                self.cache.insert(input.clone());
                self.queue.push_front(input.clone());
                if self.queue.len() > self.cache_max_size {
                    self.queue.pop_back().iter().for_each(|popped| {
                        self.cache.remove(popped);
                        ()
                    });
                }
                return true
            }
            false
        }
    }

    pub fn prune(&mut self, max_age: Duration) {
        self.sbf.prune(max_age);
        self.cache.clear();
        self.queue.clear();
    }
}