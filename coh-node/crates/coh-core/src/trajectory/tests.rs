#[cfg(test)]
mod trajectory_tests {
    use crate::trajectory::domain::{
        AgentState, AgentStatus, FinancialAction, FinancialState, FinancialStatus, OpsAction,
        OpsState, OpsStatus,
    };
    use crate::trajectory::engine::{search, SearchContext};
    use crate::trajectory::scoring::{evaluate_path, ScoringWeights};
    use crate::trajectory::types::{
        AcceptWitness, Action, AdmissibleTrajectory, DomainState, VerifiedStep,
    };
    use crate::types::Hash32;

    #[test]
    fn test_lexicographic_safety_priority() {
        // Path A: Lower progress, Higher safety
        let mut traj_a = AdmissibleTrajectory::new();
        traj_a.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState {
                balance: 10000,
                initial_balance: 10000,
                status: FinancialStatus::Idle,
                current_invoice_amount: 0,
            }),
            Action::Financial(FinancialAction::CreateInvoice { amount: 1000 }),
            DomainState::Financial(FinancialState {
                balance: 10000,
                initial_balance: 10000,
                status: FinancialStatus::Invoiced,
                current_invoice_amount: 1000,
            }),
            Hash32::default(),
            Hash32::default(),
            AcceptWitness,
        ));
        let eval_a = evaluate_path(&traj_a, 10); // Safety: 1.0, Progress: 0.3

        // Path B: Higher progress, Lower safety
        let mut traj_b = AdmissibleTrajectory::new();
        traj_b.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState {
                balance: 5000,
                initial_balance: 10000,
                status: FinancialStatus::Idle,
                current_invoice_amount: 0,
            }),
            Action::Financial(FinancialAction::CreateInvoice { amount: 1000 }),
            DomainState::Financial(FinancialState {
                balance: 5000,
                initial_balance: 10000,
                status: FinancialStatus::Invoiced,
                current_invoice_amount: 1000,
            }),
            Hash32::default(),
            Hash32::default(),
            AcceptWitness,
        ));
        traj_b.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState {
                balance: 5000,
                initial_balance: 10000,
                status: FinancialStatus::Invoiced,
                current_invoice_amount: 1000,
            }),
            Action::Financial(FinancialAction::VerifyVendor),
            DomainState::Financial(FinancialState {
                balance: 5000,
                initial_balance: 10000,
                status: FinancialStatus::ReadyToPay,
                current_invoice_amount: 1000,
            }),
            Hash32::default(),
            Hash32::default(),
            AcceptWitness,
        ));
        let eval_b = evaluate_path(&traj_b, 10); // Safety: 0.5, Progress: 0.7

        // Safety Bottleneck Rule: Path A (1.0 safety) MUST beat Path B (0.5 safety) despite inferior progress
        assert!(eval_a > eval_b);
    }

    #[test]
    fn test_ops_semantic_guard() {
        // Construct context where CloseTicket is the only next action
        let ctx = SearchContext {
            initial_state: DomainState::Ops(OpsState {
                status: OpsStatus::Open,
                materials_logged: false,
                stall_risk: 0,
                resource_readiness: 100,
            }),
            target_state: DomainState::Ops(OpsState {
                status: OpsStatus::Closed,
                materials_logged: true,
                stall_risk: 0,
                resource_readiness: 100,
            }),
            max_depth: 2,
            beam_width: 5,
            weights: ScoringWeights::default(),
        };

        let result = search(&ctx);

        // The search should NOT find an admissible path to Closed if materials weren't logged,
        // because CloseTicket requires materials_logged=true in derive_state,
        // OR is_transition_valid_semantic rejects it.
        for traj in result.admissible {
            for step in traj.steps {
                if matches!(step.action, Action::Ops(OpsAction::CloseTicket)) {
                    panic!("CloseTicket found before materials logged!");
                }
            }
        }
    }

    #[test]
    fn test_hash_grounding() {
        let s1 = DomainState::Financial(FinancialState {
            balance: 1000,
            initial_balance: 1000,
            status: FinancialStatus::Idle,
            current_invoice_amount: 0,
        });
        let s2 = DomainState::Financial(FinancialState {
            balance: 1000,
            initial_balance: 1000,
            status: FinancialStatus::Invoiced,
            current_invoice_amount: 1000,
        });

        let h1 = crate::hash::sha256(&serde_json::to_vec(&s1).unwrap());
        let h2 = crate::hash::sha256(&serde_json::to_vec(&s2).unwrap());

        assert_ne!(
            h1, h2,
            "State hashes must be distinct for distinct semantic states"
        );
    }

    #[test]
    fn test_bottleneck_vs_shortcut() {
        // Path A: 3 steps, all margin 1.0
        // Path B: 1 step, margin 0.4

        let mut traj_a = AdmissibleTrajectory::new();
        for _ in 0..3 {
            traj_a.steps.push(VerifiedStep::new(
                DomainState::Financial(FinancialState {
                    balance: 1000,
                    initial_balance: 1000,
                    status: FinancialStatus::Idle,
                    current_invoice_amount: 0,
                }),
                Action::Financial(FinancialAction::VerifyVendor),
                DomainState::Financial(FinancialState {
                    balance: 1000,
                    initial_balance: 1000,
                    status: FinancialStatus::Idle,
                    current_invoice_amount: 0,
                }),
                Hash32::default(),
                Hash32::default(),
                AcceptWitness,
            ));
        }

        let mut traj_b = AdmissibleTrajectory::new();
        traj_b.steps.push(VerifiedStep::new(
            DomainState::Financial(FinancialState {
                balance: 400,
                initial_balance: 1000,
                status: FinancialStatus::Idle,
                current_invoice_amount: 0,
            }),
            Action::Financial(FinancialAction::VerifyVendor),
            DomainState::Financial(FinancialState {
                balance: 400,
                initial_balance: 1000,
                status: FinancialStatus::Idle,
                current_invoice_amount: 0,
            }),
            Hash32::default(),
            Hash32::default(),
            AcceptWitness,
        ));

        let eval_a = evaluate_path(&traj_a, 10); // Safety: 1.0
        let eval_b = evaluate_path(&traj_b, 10); // Safety: 0.4

        assert!(eval_a > eval_b, "Safe long path must beat risky short path");
    }

    #[test]
    fn test_alignment_index_setback() {
        let s_acting = DomainState::Agent(AgentState {
            complexity_index: 10,
            complexity_budget: 100,
            authority_level: 1,
            status: AgentStatus::Acting,
        });
        let s_review = DomainState::Agent(AgentState {
            complexity_index: 10,
            complexity_budget: 100,
            authority_level: 2,
            status: AgentStatus::PolicyReview,
        });

        assert!(
            s_acting.alignment_index() > s_review.alignment_index(),
            "PolicyReview must be a setback"
        );
    }
}
