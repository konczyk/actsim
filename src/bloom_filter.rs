use std::hash::{DefaultHasher, Hash, Hasher};

pub struct ScalableBloomFilter {
    filters: Vec<BloomFilter>,
    target_fpr: f64,
    growth_factor: usize,
    tightening_ratio: f64
}

impl ScalableBloomFilter {
    pub fn new() -> Self {
        let target_fpr = 0.01f64;
        Self {
            filters: vec![BloomFilter::new(1024, -target_fpr.log2().ceil() as usize, 1)],
            target_fpr,
            growth_factor: 2,
            tightening_ratio: 0.8
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
        if !self.contains(input) {
            if let Some(filter) = self.filters.last() {
                if filter.bits.iter().map(|b| b.count_ones()).sum::<u32>() as f64/filter.size as f64 > 0.5 {
                    self.target_fpr *= self.tightening_ratio;
                    self.filters.push(BloomFilter::new(filter.size * self.growth_factor, -self.target_fpr.log2().ceil() as usize, self.filters.len()+1))
                }
            }
            let last = self.filters.len() - 1;
            self.filters[last].insert(input);
        }
    }

}

pub struct BloomFilter {
    bits: Vec<u8>,
    size: usize,
    hashes: usize,
    layer: usize
}

impl BloomFilter {
    pub fn new(size: usize, hashes: usize, layer: usize) -> Self {
        Self {
            bits: vec![0; (size + 7) >> 3],
            size,
            hashes,
            layer
        }
    }

    fn hash<T: Hash>(&self, input: &T, seed: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        seed.hash(&mut hasher);
        self.layer.hash(&mut hasher);
        hasher.finish() as usize % self.size
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
        let mut bf = BloomFilter::new(1024, 16, 1);
        let input = gen_input(64);
        input.iter().for_each(|i| bf.insert(i));

        for i in input {
            assert_eq!(true, bf.contains(&i), "input {i}");
        }
    }

    #[test]
    fn test_bf_negative() {
        let mut bf = BloomFilter::new(4096, 16, 1);
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
        let input = gen_input(512);
        input.iter().for_each(|i| bf.insert(i));

        for i in input {
            assert_eq!(true, bf.contains(&i), "input {i}");
        }
        assert_eq!(true, bf.filters.len() > 1)
    }

    #[test]
    fn test_sbf_negative() {
        let mut bf = ScalableBloomFilter::new();
        let input = gen_input(256);
        input.iter().for_each(|i| bf.insert(i));

        let neg_input = gen_input(1);
        for i in neg_input {
            assert_eq!(false, bf.contains(&i), "input {i}");
        }
        assert_eq!(true, bf.filters.len() > 1)
    }
}