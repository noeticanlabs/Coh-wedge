//! APE Strategy Engine
//!
//! Dispatches strategy implementations to generate proposals.

use crate::proposal::{Input, Proposal, Strategy};
use crate::seed::SeededRng;
use crate::strategies;

/// Generate a proposal using the specified strategy
pub fn generate(strategy: Strategy, input: &Input, seed: u64) -> Proposal {
    let mut rng = SeededRng::new(seed);

    let candidate = match strategy {
        Strategy::Mutation => strategies::mutation::run(input, &mut rng),
        Strategy::Recombination => strategies::recombination::run(input, &mut rng),
        Strategy::Violation => strategies::violation::run(input, &mut rng),
        Strategy::Overflow => strategies::overflow::run(input, &mut rng),
        Strategy::Contradiction => strategies::contradiction::run(input, &mut rng),
        Strategy::SpecificationGaming => {
            strategies::ai_failure_modes::specification_gaming(input, &mut rng)
        }
        Strategy::DistributionShift => {
            strategies::ai_failure_modes::distribution_shift(input, &mut rng)
        }
        Strategy::TemporalDrift => strategies::ai_failure_modes::temporal_drift(input, &mut rng),
        Strategy::AmbiguityExploitation => {
            strategies::ai_failure_modes::ambiguity_exploitation(input, &mut rng)
        }
        Strategy::AdversarialAlignment => {
            strategies::ai_failure_modes::adversarial_alignment(input, &mut rng)
        }
        Strategy::NonTermination => strategies::runtime::non_termination(input, &mut rng),
        Strategy::Livelock => strategies::runtime::livelock(input, &mut rng),
        Strategy::StateExplosion => strategies::runtime::state_explosion(input, &mut rng),
        Strategy::ResourceExhaustion => strategies::runtime::resource_exhaustion(input, &mut rng),
        Strategy::ParserPathology => strategies::runtime::parser_pathology(input, &mut rng),
        Strategy::ShadowChain => strategies::advanced::shadow_chain(input, &mut rng),
        Strategy::GradientDescent => strategies::advanced::gradient_descent(input, &mut rng),
        Strategy::OracleManipulation => strategies::advanced::oracle_manipulation(input, &mut rng),
        Strategy::TypeConfusion => strategies::advanced::type_confusion(input, &mut rng),
        Strategy::ReflexiveAttack => strategies::advanced::reflexive_attack(input, &mut rng),
    };

    Proposal::new(strategy, seed, candidate)
}

/// List all available strategies
pub fn strategies() -> Vec<Strategy> {
    vec![
        Strategy::Mutation,
        Strategy::Recombination,
        Strategy::Violation,
        Strategy::Overflow,
        Strategy::Contradiction,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposal::Input;
    use coh_core::types::MetricsWire;

    fn sample_micro() -> coh_core::types::MicroReceiptWire {
        coh_core::types::MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            canon_profile_hash: "0".repeat(64),
            policy_hash: "0".repeat(64),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: "0".repeat(64),
            state_hash_next: "0".repeat(64),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "100".to_string(),
                spend: "0".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        }
    }

    #[test]
    fn test_generate_mutation() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let proposal = generate(Strategy::Mutation, &input, 42);

        assert_eq!(proposal.strategy, Strategy::Mutation);
        assert!(proposal.candidate.as_micro().is_some());
    }

    #[test]
    fn test_determinism() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let p1 = generate(Strategy::Mutation, &input, 42);
        let p2 = generate(Strategy::Mutation, &input, 42);

        assert_eq!(p1.proposal_id, p2.proposal_id);
    }

    #[test]
    fn test_all_strategies() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        for strategy in strategies() {
            let proposal = generate(strategy, &input, 42);
            assert_eq!(proposal.strategy, strategy);
        }
    }
}
