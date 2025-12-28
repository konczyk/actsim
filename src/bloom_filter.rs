use std::hash::{DefaultHasher, Hash, Hasher};

struct BloomFilter {
    bits: Vec<u8>,
    size: usize,
    hashes: u8,
}

impl BloomFilter {
    fn new(size: usize, hashes: u8) -> Self {
        Self {
            bits: vec![0; (size + 7) >> 3],
            size,
            hashes,
        }
    }

    fn hash<T: Hash>(&self, input: &T, seed: u8) -> usize {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() as usize % self.size
    }

    fn insert<T: Hash>(&mut self, input: &T) {
        for i in 0..self.hashes {
            let idx = self.hash(input, i);
            self.bits[idx >> 3] |= 1 << (idx & 7);
        }

    }

    fn contains<T: Hash>(&self, input: &T) -> bool {
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
    fn test_positive() {
        let mut bf = BloomFilter::new(1024, 16);
        let input = gen_input(64);
        input.iter().for_each(|i| bf.insert(i));

        for i in input {
            assert_eq!(true, bf.contains(&i), "input {i}");
        }
    }

    #[test]
    fn test_negative() {
        let mut bf = BloomFilter::new(4096, 16);
        let input = gen_input(1);
        input.iter().for_each(|i| bf.insert(i));

        let neg_input = gen_input(16);
        for i in neg_input {
            assert_eq!(false, bf.contains(&i), "input {i}");
        }
    }
}