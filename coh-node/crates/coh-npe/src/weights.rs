//! NPE Strategy and Template Weights
//!
//! Adaptive probability distributions over strategy and template space.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Adaptive weights for strategy or template selection
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StrategyWeights(pub HashMap<String, f64>);

impl StrategyWeights {
    /// Normalize weights to sum = 1.0 (probability distribution)
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.values().sum();
        if sum > 0.0 {
            for value in self.0.values_mut() {
                *value /= sum;
            }
        }
    }

    /// Get normalized weight for a class
    pub fn get(&self, class: &str) -> f64 {
        self.0.get(class).copied().unwrap_or(0.0)
    }

    /// Increment weight for a class by delta
    pub fn increment(&mut self, class: &str, delta: f64) {
        self.0
            .entry(class.to_string())
            .and_modify(|w| *w = (*w + delta).max(0.0))
            .or_insert(delta.max(0.0));
    }

    /// Get all strategy classes
    pub fn classes(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
    
    /// Prune weights below a maintenance threshold (metabolic forgetting)
    pub fn prune(&mut self, threshold: f64) {
        self.0.retain(|_, &mut w| w >= threshold);
    }
}
