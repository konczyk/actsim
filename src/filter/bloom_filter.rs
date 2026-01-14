use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

pub struct ScalableBloomFilter {
    pub filters: Vec<BloomFilter>,
    initial_size: usize,
    initial_hashes: usize,
    target_fpr: f64,
    growth_factor: usize,
    tightening_ratio: f64,
    partition_size: usize
}

impl ScalableBloomFilter {
    pub fn new() -> Self {
        let target_fpr = 0.001f64;
        let initial_hashes = -target_fpr.log2().ceil() as usize;
        let partition_size = 2048;
        let initial_size = partition_size * initial_hashes;
        Self {
            filters: vec![BloomFilter::new(initial_size, initial_hashes, 1, partition_size)],
            initial_size,
            target_fpr,
            initial_hashes,
            growth_factor: 2,
            tightening_ratio: 0.8,
            partition_size
        }
    }

    pub fn fpr(&self) -> f64 {
        1.0 - self.filters.iter()
            .map(|bf| 1.0 - 0.5f64.powi(bf.hashes as i32))
            .product::<f64>()
    }

    pub fn contains<T: Hash>(&self, input: &T) -> bool {
        self.filters.iter().any(|f| f.contains(input))
    }

    pub fn insert<T: Hash>(&mut self, input: &T) {
        if let Some(filter) = self.filters.last() {
            if filter.bits.iter().map(|b| b.count_ones()).sum::<u32>() as f64/filter.size as f64 > 0.5 {
                self.target_fpr *= self.tightening_ratio;
                let hashes = -self.target_fpr.log2().ceil() as usize;
                self.filters.push(BloomFilter::new(self.partition_size * hashes * self.growth_factor, hashes , self.filters.len() + 1, self.partition_size))
            }
        }
        let last = self.filters.len() - 1;
        self.filters[last].insert(input);
    }

    pub fn prune(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.filters.retain(|f| now.duration_since(f.timestamp) < max_age);
        if self.filters.is_empty() {
            self.filters.push(BloomFilter::new(self.initial_size, self.initial_hashes, 1, self.partition_size))
        }
    }

}

pub struct BloomFilter {
    pub bits: Vec<u8>,
    pub size: usize,
    hashes: usize,
    layer: usize,
    timestamp: Instant,
    partition_size: usize
}

impl BloomFilter {
    pub fn new(size: usize, hashes: usize, layer: usize, partition_size: usize) -> Self {
        Self {
            bits: vec![0; (size + 7) >> 3],
            size,
            hashes,
            layer,
            timestamp: Instant::now(),
            partition_size,
        }
    }

    fn hash<T: Hash>(&self, input: &T, partition: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        partition.hash(&mut hasher);
        self.layer.hash(&mut hasher);
        partition * self.partition_size +  hasher.finish() as usize % self.partition_size
    }

    pub fn insert<T: Hash>(&mut self, input: &T) {
        for i in 0..self.hashes {
            let idx = self.hash(input, i);
            self.bits[idx >> 3] |= 1 << (idx & 7);
        }

    }

    pub fn contains<T: Hash>(&self, input: &T) -> bool {
        (0..self.hashes).all(|i| {
            let idx = self.hash(input, i);
            self.bits[idx >> 3] & (1 << (idx & 7)) != 0
        })
    }

}

#[cfg(test)]
mod tests {
    use rand::distr::{Alphanumeric, SampleString};
    use super::*;

    fn gen_input(n: usize) -> Vec<String> {
        let mut rng = rand::rng();
        (0..n).map(|_| Alphanumeric.sample_string(&mut rng, 8)).collect()
    }

    #[test]
    fn test_bf_positive() {
        let mut bf = BloomFilter::new(1024, 4, 1, 256);
        let input = gen_input(64);
        input.iter().for_each(|i| bf.insert(i));

        for i in input {
            assert_eq!(true, bf.contains(&i), "input {i}");
        }
    }

    #[test]
    fn test_bf_negative() {
        let mut bf = BloomFilter::new(4096, 16, 1, 256);
        let input = gen_input(64);
        input.iter().for_each(|i| bf.insert(i));

        let neg_input = gen_input(16);
        for i in neg_input {
            assert_eq!(false, bf.contains(&i), "input {i}");
        }
    }

    #[test]
    fn test_sbf_positive() {
        let mut bf = ScalableBloomFilter::new();
        let input = gen_input(2048);
        input.iter().for_each(|i| bf.insert(i));

        for i in input {
            assert!(bf.contains(&i), "input {i}");
        }
        assert!(bf.filters.len() > 1)
    }

    #[test]
    fn test_sbf_negative() {
        let mut bf = ScalableBloomFilter::new();
        let input = gen_input(2048);
        input.iter().for_each(|i| bf.insert(i));

        let neg_input = gen_input(1);
        for i in neg_input {
            assert_eq!(false, bf.contains(&i), "input {i}");
        }
        assert!(bf.filters.len() > 1)
    }

    #[test]
    fn partitioning() {
        let input = gen_input(256);
        for i in input {
            let mut bf = BloomFilter::new(128, 4, 1, 32);
            bf.insert(&i);
            assert!(bf.contains(&i), "input {i}");
            assert_eq!(4, bf.bits.iter().map(|x| x.count_ones()).sum::<u32>())
        }
    }
}