//! Lean Failure Taxonomy
//!
//! Provides a structured failure classification system for NPE-Lean proof attempts.
//! The taxonomy tells the NPE what kind of failure happened and how to respond.

use serde::{Deserialize, Serialize};

/// Top-level failure layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofFailureLayer {
    /// Failure at Coh pre-gate (before Lean)
    CohPre,
    /// Failure in mathlib advisor
    MathlibAdvisor,
    /// Lean syntax/parse failure
    LeanSyntax,
    /// Lean type elaboration failure
    LeanElaboration,
    /// Lean proof state failure
    LeanProofState,
    /// Failure after Lean compiles but before Coh admission
    CohPost,
}

/// Coh pre-gate failures (before Lean runs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CohPreFailure {
    /// Sorry introduced in proof
    SorryIntroduced,
    /// Admit introduced
    AdmitIntroduced,
    /// Axiom introduced
    AxiomIntroduced,
    /// Theorem statement changed
    TheoremStatementChanged,
    /// Theorem name changed
    TheoremNameChanged,
    /// Definition weakened
    DefinitionWeakened,
    /// Import outside policy
    ForbiddenImport,
    /// Too many imports
    TooManyImports,
    /// Too many helper lemmas
    TooManyHelperLemmas,
    /// Tactic budget exceeded
    TacticBudgetExceeded,
    /// Proof touches unrelated files
    TouchesUnrelatedFile,
    /// Proof too large
    ProofTooLarge,
    /// Generation cost too high (Genesis violation)
    GenesisViolation,
}

/// Mathlib advisor failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathlibFailure {
    /// Lemma not found in mathlib
    LemmaNotFound,
    /// Lemma name doesn't match
    LemmaNameMismatch,
    /// Namespace mismatch
    NamespaceMismatch,
    /// Import rejected by policy
    ImportPolicyRejected,
    /// Import too heavy (outside policy)
    ImportTooHeavy,
    /// Import creates dependency cycle
    ImportCausesCycle,
    /// Import doesn't expose intended lemma
    ImportDoesNotExposeLemma,
    /// Suggested lemma wrong shape
    SuggestedLemmaWrongShape,
    /// Suggested lemma too strong
    SuggestedLemmaTooStrong,
    /// Suggested lemma too weak
    SuggestedLemmaTooWeak,
    /// Lemma requires missing typeclass
    SuggestedLemmaNeedsTypeclass,
    /// Mathlib policy blocks; use advisory-only
    StrategyOnlyNoImportAllowed,
}

/// Lean syntax failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeanSyntaxFailure {
    /// Parse error
    ParseError,
    /// Unexpected token
    UnexpectedToken,
    /// Invalid binder syntax
    InvalidBinderSyntax,
    /// Invalid exists syntax
    InvalidExistsSyntax,
    /// Invalid unicode
    InvalidUnicode,
    /// Bad indentation
    BadIndentation,
    /// Missing `by` keyword
    MissingBy,
    /// Missing comma
    MissingComma,
    /// Invalid tactic block
    InvalidTacticBlock,
    /// Unknown command
    UnknownCommand,
}

/// Lean elaboration/type failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeanElabFailure {
    /// Undefined identifier
    UnknownIdentifier,
    /// Unknown field
    UnknownField,
    /// Type mismatch
    TypeMismatch,
    /// Function application type mismatch
    ApplicationTypeMismatch,
    /// Failed to synthesize typeclass instance
    FailedToSynthesizeInstance,
    /// Ambiguous coercion
    AmbiguousCoercion,
    /// Invalid field projection
    InvalidProjection,
    /// Invalid constructor
    InvalidConstructor,
    /// Invalid recursor
    InvalidRecursor,
    /// Universe mismatch
    UniverseMismatch,
    /// Cannot infer implicit argument
    CannotInferImplicit,
    /// Coercion failed
    CoercionFailed,
}

/// Lean proof-state failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeanProofFailure {
    /// Unsolved goals remaining
    UnsolvedGoals,
    /// Goal shape mismatch
    GoalShapeMismatch,
    /// Need lower bound half proof
    NeedLowerBoundHalf,
    /// Need greatest lower bound half proof
    NeedGreatestLowerBoundHalf,
    /// Need approximation lemma
    NeedApproximationLemma,
    /// Need contradiction proof
    NeedContradiction,
    /// Need witness construction
    NeedWitnessConstruction,
    /// Need set extensionality
    NeedSetExtensionality,
    /// Need monotonicity proof
    NeedMonotonicity,
    /// Need rewrite
    NeedRewrite,
    /// Need transitivity
    NeedTransitivity,
    /// Need order contradiction
    NeedOrderContradiction,
    /// Need induction
    NeedInduction,
    /// Need cases
    NeedCases,
    /// Generic tactic failed
    TacticFailed,
    /// Simp did nothing
    SimpDidNothing,
    /// Rewrite failed
    RewriteFailed,
    /// Exact failed
    ExactFailed,
    /// Apply failed
    ApplyFailed,
}

/// Coh post-verification failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CohPostFailure {
    /// Proof cost too high
    ProofCostTooHigh,
    /// Import cost too high
    ImportCostTooHigh,
    /// Dependency risk too high
    DependencyRiskTooHigh,
    /// Coherence margin negative
    CoherenceMarginNegative,
    /// Receipt hash mismatch
    ReceiptHashMismatch,
    /// Statement hash mismatch
    StatementHashMismatch,
    /// Proof hash mismatch
    ProofHashMismatch,
    /// Build receipt missing
    BuildReceiptMissing,
    /// Mathlib policy violation
    MathlibPolicyViolation,
}

/// Failure severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureSeverity {
    /// Hard reject - should never have been generated
    HardReject,
    /// Repairable with syntax/type fix
    Repairable,
    /// Useful near-miss
    UsefulNearMiss,
    /// Unknown classification
    Unknown,
}

/// Near-miss classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NearMissClass {
    /// Only one goal remaining
    OneGoalRemaining,
    /// Correct lemma, wrong field name
    CorrectLemmaWrongField,
    /// Correct strategy, wrong import
    CorrectStrategyWrongImport,
    /// Correct idea, type mismatch
    CorrectIdeaTypeMismatch,
    /// Correct bridge, missing helper lemma
    CorrectBridgeMissingHelper,
    /// Needs order contradiction
    NeedsOrderContradiction,
    /// Needs witness construction
    NeedsWitness,
    /// Needs set extensionality
    NeedsSetRewrite,
}

/// Combined proof failure wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofFailureKind {
    CohPre(CohPreFailure),
    Mathlib(MathlibFailure),
    Syntax(LeanSyntaxFailure),
    Elab(LeanElabFailure),
    Proof(LeanProofFailure),
    CohPost(CohPostFailure),
}

/// Failure report with structured metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanFailureReport {
    pub candidate_id: String,
    pub target_theorem: String,
    pub layer: ProofFailureLayer,
    pub kind: ProofFailureKind,
    pub raw_error: Option<String>,
    pub normalized_message: String,
    pub severity: FailureSeverity,
    pub near_miss_class: Option<NearMissClass>,
}

/// Reward signal based on failure classification
impl FailureSeverity {
    pub fn reward_signal(&self) -> f64 {
        match self {
            FailureSeverity::HardReject => -1.0,
            FailureSeverity::Repairable => -0.1,
            FailureSeverity::UsefulNearMiss => 0.3,
            FailureSeverity::Unknown => 0.0,
        }
    }
}

/// Map failure to suggested next strategies
pub fn suggest_next_strategies(report: &LeanFailureReport) -> Vec<&'static str> {
    match &report.kind {
        ProofFailureKind::CohPre(CohPreFailure::SorryIntroduced) => {
            vec!["avoid_sorry", "direct_proof"]
        }

        ProofFailureKind::CohPre(CohPreFailure::GenesisViolation) => {
            vec!["lower_wildness", "conservative_search"]
        }

        ProofFailureKind::Mathlib(MathlibFailure::ImportTooHeavy) => {
            vec!["advisory_only", "strategy_boost"]
        }

        ProofFailureKind::Mathlib(MathlibFailure::LemmaNotFound) => {
            vec!["lemma_variant_search", "namespace_browse"]
        }

        ProofFailureKind::Syntax(LeanSyntaxFailure::InvalidExistsSyntax) => {
            vec!["syntax_repair", "binder_restructure"]
        }

        ProofFailureKind::Elab(LeanElabFailure::UnknownField) => {
            vec!["field_name_repair", "projection_fix"]
        }

        ProofFailureKind::Elab(LeanElabFailure::TypeMismatch) => {
            vec!["coercion_repair", "type_bridge"]
        }

        ProofFailureKind::Elab(LeanElabFailure::FailedToSynthesizeInstance) => {
            vec!["typeclass_avoid", "instance_search"]
        }

        ProofFailureKind::Proof(LeanProofFailure::NeedGreatestLowerBoundHalf) => {
            vec!["glb_decomposition", "order_theory"]
        }

        ProofFailureKind::Proof(LeanProofFailure::NeedApproximationLemma) => {
            vec!["approx_lemma", "exists_lt_proof"]
        }

        ProofFailureKind::Proof(LeanProofFailure::NeedOrderContradiction) => {
            vec!["contradiction_proof", "not_gt_proof"]
        }

        ProofFailureKind::Proof(LeanProofFailure::UnsolvedGoals) => {
            vec!["goal_split", "tactic_decomposition"]
        }

        _ => vec!["retry", "next_strategy"],
    }
}

/// Specific failures for RationalInf chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RationalInfFailure {
    /// Missing pairwise add lemma
    MissingPairwiseAddLemma,
    /// Missing approximation lemma  
    MissingApproximationLemma,
    /// Lower bound half unclosed
    LowerBoundHalfUnclosed,
    /// Greatest lower bound half unclosed
    GreatestLowerBoundHalfUnclosed,
    /// Exists_lt contradiction unclosed
    ExistsLtContradictionUnclosed,
    /// Pairwise set destructure failed
    PairwiseSetDestructureFailed,
    /// RationalInf field access failed
    RationalInfFieldAccessFailed,
    /// ENNRat coercion ambiguous
    ENNRatCoercionAmbiguous,
    /// NNRat division unavailable
    NNRatDivisionUnavailable,
    /// Mathlib inf add lemma unavailable
    MathlibInfAddLemmaUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_reward() {
        assert_eq!(FailureSeverity::HardReject.reward_signal(), -1.0);
        assert_eq!(FailureSeverity::UsefulNearMiss.reward_signal(), 0.3);
    }

    #[test]
    fn test_fail_report() {
        let report = LeanFailureReport {
            candidate_id: "test_001".to_string(),
            target_theorem: "isRationalInf_pairwise_add".to_string(),
            layer: ProofFailureLayer::LeanProofState,
            kind: ProofFailureKind::Proof(LeanProofFailure::NeedGreatestLowerBoundHalf),
            raw_error: None,
            normalized_message: "goal unsynced".to_string(),
            severity: FailureSeverity::UsefulNearMiss,
            near_miss_class: Some(NearMissClass::CorrectBridgeMissingHelper),
        };

        let strategies = suggest_next_strategies(&report);
        assert!(strategies.contains(&"glb_decomposition"));
    }
}
