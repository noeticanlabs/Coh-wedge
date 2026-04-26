//! Deterministic RNG for APE
//!
//! Uses PCG32 algorithm for reproducible results based on seed.

/// PCG32 Random Number Generator
///
/// Provides deterministic random numbers based on initial seed.
/// Same seed + same sequence → same output (critical for testing).
#[derive(Clone, Debug)]
pub struct SeededRng {
    state: u64,
    increment: u64,
}

impl SeededRng {
    /// Create new RNG from seed
    pub fn new(seed: u64) -> Self {
        // PCG32 initialization
        // Split seed into state and increment
        let increment = (seed.wrapping_mul(0x85ebca6b).wrapping_add(0x7c15e)) | 1;
        let mut state = seed.wrapping_mul(0x85ebca6b).wrapping_add(0x7c15e);
        state = state.wrapping_mul(0x85ebca6b).wrapping_add(0x7c15e);

        Self { state, increment }
    }

    /// Generate next random u32
    ///
    /// Uses PCG32 XSH RR (xorshift high, random rotation)
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        // PCG32 step
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.increment);

        // Xorshift
        let xorshifted: u32 = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;

        // Output with rotation (simple version)
        if rot == 0 {
            xorshifted
        } else {
            xorshifted.rotate_right(rot)
        }
    }

    /// Generate random u64 (two u32s)
    pub fn next_u64(&mut self) -> u64 {
        ((self.next() as u64) << 32) | (self.next() as u64)
    }

    /// Generate random u128 (for metrics)
    pub fn next_u128(&mut self) -> u128 {
        ((self.next_u64() as u128) << 64) | (self.next_u64() as u128)
    }

    /// Generate random f64 in [0, 1)
    pub fn next_f64(&mut self) -> f64 {
        let mantissa = self.next() as u64 & 0xFFFFFFFFFFFFF;
        mantissa as f64 / (1u64 << 52) as f64
    }

    /// Generate random bool
    pub fn next_bool(&mut self) -> bool {
        self.next() & 1 == 1
    }

    /// Generate random index in [0, len)
    pub fn next_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        (self.next() as usize) % len
    }

    /// Generate bytes of random data
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(4) {
            let val = self.next();
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte = (val >> (i * 8)) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism() {
        let seed = 42;
        let mut rng1 = SeededRng::new(seed);
        let mut rng2 = SeededRng::new(seed);

        for _ in 0..1000 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut rng1 = SeededRng::new(1);
        let mut rng2 = SeededRng::new(2);

        assert_ne!(rng1.next(), rng2.next());
    }

    #[test]
    fn test_boundary() {
        let mut rng0 = SeededRng::new(0);
        let mut rng_max = SeededRng::new(u64::MAX);

        // Both should produce valid output (no panic)
        rng0.next();
        rng_max.next();
    }
}
