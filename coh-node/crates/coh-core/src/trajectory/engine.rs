//! Trajectory Engine - V3 Distance Computation
//!
//! Implements Dijkstra's algorithm to find the minimum defect distance d(x, y).

use super::types::{Transition, Trajectory};
use crate::types::Hash32;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    hash: Hash32,
    cost: u128,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TrajectoryEngine {
    pub adjacency: HashMap<Hash32, Vec<Transition>>,
}

impl Default for TrajectoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TrajectoryEngine {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.adjacency.entry(t.from.hash).or_default().push(t);
    }

    /// Compute d_hat(x, y) = inf { delta_hat(tau) | tau : x -> y }
    pub fn compute_distance(&self, start: Hash32, target: Hash32) -> Option<u128> {
        let mut distances = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start, 0);
        heap.push(Node {
            hash: start,
            cost: 0,
        });

        while let Some(Node { hash, cost }) = heap.pop() {
            if hash == target {
                return Some(cost);
            }

            if let Some(&current_dist) = distances.get(&hash) {
                if cost > current_dist {
                    continue;
                }
            }

            if let Some(transitions) = self.adjacency.get(&hash) {
                for t in transitions {
                    let next_cost = cost.saturating_add(t.delta_hat);

                    let is_better = match distances.get(&t.to.hash) {
                        Some(&d) => next_cost < d,
                        None => true,
                    };

                    if is_better {
                        distances.insert(t.to.hash, next_cost);
                        heap.push(Node {
                            hash: t.to.hash,
                            cost: next_cost,
                        });
                    }
                }
            }
        }

        None
    }

    /// Compute the optimal geodesic trajectory between start and target
    pub fn compute_geodesic(&self, start: Hash32, target: Hash32) -> Option<Trajectory> {
        let mut distances = HashMap::new();
        let mut parents: HashMap<Hash32, (Hash32, Transition)> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start, 0);
        heap.push(Node {
            hash: start,
            cost: 0,
        });

        while let Some(Node { hash, cost }) = heap.pop() {
            if hash == target {
                // Reconstruct path
                let mut steps = Vec::new();
                let mut current = target;
                while let Some((parent, transition)) = parents.remove(&current) {
                    steps.push(transition);
                    current = parent;
                    if current == start {
                        break;
                    }
                }
                steps.reverse();
                return Some(Trajectory { steps });
            }

            if let Some(&current_dist) = distances.get(&hash) {
                if cost > current_dist {
                    continue;
                }
            }

            if let Some(transitions) = self.adjacency.get(&hash) {
                for t in transitions {
                    let next_cost = cost.saturating_add(t.delta_hat);

                    let is_better = match distances.get(&t.to.hash) {
                        Some(&d) => next_cost < d,
                        None => true,
                    };

                    if is_better {
                        distances.insert(t.to.hash, next_cost);
                        parents.insert(t.to.hash, (hash, t.clone()));
                        heap.push(Node {
                            hash: t.to.hash,
                            cost: next_cost,
                        });
                    }
                }
            }
        }

        None
    }
}
