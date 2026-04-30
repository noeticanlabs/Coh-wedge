//! Synthetic NPE Generator
//!
//! A deterministic synthetic proposal generator that produces candidates
//! where increasing wildness (λ) causes:
//! - higher novelty
//! - higher generation cost
//! - higher risk of Genesis violation
//! - higher risk of RV rejection
//! - higher risk of Coherence violation
//!
//! This simulates the boundary behavior of a real NPE without needing
//! an actual LLM or intelligent system.

use crate::candidate::{GenesisCandidate, ProjectedCohClaim, WildnessLevel};

/// Seeded random number generator with mulberry32
struct Mulberry32(u32);

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self(seed)
    }

    /// Generate next random f64 in range [0, 1)
    fn next_f64(&mut self) -> f64 {
        let mut t = self.0.wrapping_add(0x6D2B79F5);
        self.0 = t;
        t = t ^ (t >> 15);
        t = t.wrapping_mul(1 | t >> 1);
        t ^= t << 7;
        t ^= t >> 3;
        t ^= t << 10;
        t ^= t >> 15;
        // Skew toward [0, 1) using mantissa bits
        let result = (t as f64) / u32::MAX as f64;
        result % 1.0
    }

    /// Generate random u128 in range [min, max]
    fn next_u128(&mut self, min: u128, max: u128) -> u128 {
        let range = max.saturating_sub(min) + 1;
        let r = (self.next_f64() * range as f64) as u128;
        min.saturating_add(r)
    }
}

/// Synthetic NPE Generator
///
/// Generates candidate transitions with controlled wildness behavior.
///
/// Wildness levels:
/// - λ = 0: conservative proposals (minimal variation)
/// - λ = 1: normal variation
/// - λ = 2: aggressive recombination
/// - λ = 5: unconventional proposals
/// - λ = 10: near-chaotic generation
pub struct SyntheticNpeGenerator {
    seed: u32,
    /// Base complexity before generation
    base_m_before: u128,
    /// Baseline generation cost (low wildness)
    base_cost: u128,
    /// Baseline defect budget
    base_defect: u128,
    /// Base valuation before
    base_v_before: u128,
}

impl SyntheticNpeGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            base_m_before: 1000,
            base_cost: 50,
            base_defect: 200,
            base_v_before: 1000,
        }
    }

    /// Generate candidates at the given wildness level
    pub fn generate(&self, wildness: f64, count: usize) -> Vec<GenesisCandidate> {
        let mut rng = Mulberry32::new(self.seed);
        let lambda = wildness.max(0.0);

        (0..count)
            .map(|i| {
                let id = format!("{:}-{:05}", self.seed, i);
                self.generate_one(&mut rng, lambda, &id)
            })
            .collect()
    }

    /// Generate one candidate at the given wildness level
    fn generate_one(&self, rng: &mut Mulberry32, lambda: f64, id: &str) -> GenesisCandidate {
        // 1. Generate base metrics with wildness-dependent variation
        let m_before = self.base_m_before;

        // M(g'): complexity after generation
        // As λ increases, output complexity tends to increase
        // (more complex, less predictable outputs)
        let _m_after_base = self.base_m_before;
        let m_after_variation = (lambda * 50.0) as u128;
        let m_after_noise = rng.next_u128(0, m_after_variation * 2);
        let m_after = m_before.saturating_add(m_after_noise.saturating_sub(m_after_variation));

        // C(p): generation cost increases with wildness
        let cost_base = self.base_cost;
        let cost_increase = (lambda * 20.0) as u128;
        let cost_variation = rng.next_u128(0, cost_increase);
        let generation_cost = cost_base + cost_variation;

        // D(p): generation defect budget
        // Higher wildness = less predictable, potentially more defects
        let defect_base = self.base_defect;
        let defect_increase = (lambda * 10.0) as u128;
        let defect_variation = rng.next_u128(0, defect_increase);
        let generation_defect = defect_base.saturating_sub(defect_variation.min(defect_base / 2));

        // 2. Novelty increases with wildness
        // At λ=0: ~1-2 (very similar to baseline)
        // At λ=10: ~20-40 (highly novel, unusual)
        let novelty_base = 1.0 + lambda;
        let novelty_variation = rng.next_f64() * lambda * 2.0;
        let novelty = novelty_base + novelty_variation;

        // 3. Projected Coherence metrics
        let v_before = self.base_v_before;

        // V(y): valuation after
        // Higher wildness tends to produce less coherent outputs
        let v_after_variation = (lambda * 30.0) as u128;
        let v_after_noise = rng.next_u128(0, v_after_variation * 2);
        let v_after = if rng.next_f64() > 0.5 {
            // Sometimes increases, sometimes decreases
            v_before.saturating_add(v_after_noise.saturating_sub(v_after_variation))
        } else {
            v_before.saturating_sub(v_after_noise.min(v_before / 2))
        };

        // Spend: execution cost increases with wildness
        let spend_base = 50u128;
        let spend_increase = (lambda * 15.0) as u128;
        let spend = spend_base + rng.next_u128(0, spend_increase);

        // Defect: allowed defect during execution
        let defect_exe_base = 100u128;
        let defect_exe_increase = (lambda * 5.0) as u128;
        let defect = defect_exe_base + rng.next_u128(0, defect_exe_increase);

        // 4. RV accept probability decreases with wildness
        // At λ=0: ~95% accept
        // At λ=1: ~80% accept
        // At λ=2: ~60% accept
        // At λ=5: ~25% accept
        // At λ=10: ~5% accept
        let rv_accept_prob = (1.0 - lambda * 0.095).max(0.05);
        let rv_accept = rng.next_f64() < rv_accept_prob;

        let projection = ProjectedCohClaim::new(v_before, v_after, spend, defect, rv_accept);

        GenesisCandidate::new(
            id.to_string(),
            lambda,
            m_before,
            m_after,
            generation_cost,
            generation_defect,
            novelty,
            projection,
        )
    }
}

/// Create a proposal engine trait (for future extensibility)
pub trait ProposalEngine {
    fn generate(&self, wildness: WildnessLevel, count: usize, seed: u64) -> Vec<GenesisCandidate>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Generate produces correct count
    #[test]
    fn test_generate_count() {
        let generator = SyntheticNpeGenerator::new(42);
        let candidates = generator.generate(1.0, 100);
        assert_eq!(candidates.len(), 100);
    }

    /// Test: All candidates have correct wildness
    #[test]
    fn test_generate_wildness() {
        let generator = SyntheticNpeGenerator::new(42);
        let candidates = generator.generate(2.5, 50);
        for c in &candidates {
            assert!((c.wildness - 2.5).abs() < 0.001);
        }
    }

    /// Test: Generation is deterministic with same seed
    #[test]
    fn test_generate_deterministic() {
        let gen1 = SyntheticNpeGenerator::new(42);
        let gen2 = SyntheticNpeGenerator::new(42);

        let c1 = gen1.generate(1.0, 10);
        let c2 = gen2.generate(1.0, 10);

        for (a, b) in c1.iter().zip(c2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.m_before, b.m_before);
        }
    }

    /// Test: Higher wildness produces higher novelty on average
    #[test]
    fn test_wildness_novelty_correlation() {
        let generator = SyntheticNpeGenerator::new(42);

        let low = generator.generate(0.0, 100);
        let mid = generator.generate(2.0, 100);
        let high = generator.generate(5.0, 100);

        let avg_low: f64 = low.iter().map(|c| c.novelty).sum::<f64>() / 100.0;
        let avg_mid: f64 = mid.iter().map(|c| c.novelty).sum::<f64>() / 100.0;
        let avg_high: f64 = high.iter().map(|c| c.novelty).sum::<f64>() / 100.0;

        assert!(avg_mid > avg_low);
        assert!(avg_high > avg_mid);
    }

    /// Test: Higher wildness produces lower RV accept rate
    #[test]
    fn test_wildness_rv_accept_correlation() {
        let generator = SyntheticNpeGenerator::new(42);

        let low = generator.generate(0.0, 500);
        let mid = generator.generate(2.0, 500);
        let high = generator.generate(5.0, 500);

        let rv_low: f64 = low.iter().filter(|c| c.projection.rv_accept).count() as f64 / 500.0;
        let rv_mid: f64 = mid.iter().filter(|c| c.projection.rv_accept).count() as f64 / 500.0;
        let rv_high: f64 = high.iter().filter(|c| c.projection.rv_accept).count() as f64 / 500.0;

        assert!(rv_mid < rv_low);
        assert!(rv_high < rv_mid);
    }

    /// Test: Low wildness has high formation acceptance
    #[test]
    fn test_low_wildness_high_acceptance() {
        let generator = SyntheticNpeGenerator::new(42);
        let candidates = generator.generate(0.0, 200);

        let formation_ok = candidates
            .iter()
            .filter(|c| c.is_formation_admissible())
            .count();
        let rate = formation_ok as f64 / 200.0;

        // Should be > 80% at λ=0
        assert!(rate > 0.80, "Low wildness acceptance too low: {}", rate);
    }

    /// Test: High wildness has low formation acceptance
    #[test]
    fn test_high_wildness_low_acceptance() {
        let generator = SyntheticNpeGenerator::new(42);
        let candidates = generator.generate(10.0, 200);

        let formation_ok = candidates
            .iter()
            .filter(|c| c.is_formation_admissible())
            .count();
        let rate = formation_ok as f64 / 200.0;

        // Should be < 20% at λ=10
        assert!(rate < 0.20, "High wildness acceptance too high: {}", rate);
    }
}
