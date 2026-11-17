pub struct Rng {
    pub y: i32,
    pub z: i32,
}

impl Rng {
    pub fn new(seed: i32) -> Self {
        Self {
            y: seed % 32000,
            z: seed % 65535,
        }
    }

    pub fn next(&mut self) -> i32 {
        self.z = 36969i32
            .wrapping_mul(self.z & 0xffff)
            .wrapping_add(self.z >> 16);
        self.y = 18000i32
            .wrapping_mul(self.y & 0xffff)
            .wrapping_add(self.y >> 16);
        (self.z << 16).wrapping_add(self.y).wrapping_abs()
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        (self.next() % (max - min)) + min
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;
    #[test]
    fn rng_deterministic() {
        let mut a = Rng::new(123);
        let mut b = Rng::new(123);
        for _ in 0..100 {
            assert_eq!(a.next(), b.next());
        }
    }
}
