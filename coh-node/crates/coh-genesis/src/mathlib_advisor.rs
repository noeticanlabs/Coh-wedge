//! Mathlib Advisor Module
//!
//! Acts as a lemma source oracle inside the NPE-Lean loop.
//! Provides search suggestions for mathlib lemmas and imports, but does NOT have
//! authority - Lean compilation is still the truth gate.
//!
//! Authority stack: mathlib suggests -> NPE assembles -> Lean verifies -> Coh admits -> PhaseLoom learns
//!
//! ## Lake Integration
//!
//! This module can optionally query the Lean lake environment for authoritative
//! lemma and instance information. If lake is unavailable, it falls back to
//! heuristic strategy suggestions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Result from a lake lemma search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LemmaMatch {
    /// Lemma name
    pub name: String,
    /// File where it's defined
    pub file: String,
    /// Whether it's in mathlib
    pub in_mathlib: bool,
}

/// Result from a type class instance search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMatch {
    /// Instance name
    pub name: String,
    /// Type class it implements
    pub class: String,
}

/// Lake environment query interface
#[derive(Debug, Clone)]
pub struct MathlibLakeQuery {
    /// Path to the Lean project (coh-t-stack)
    pub project_path: PathBuf,
    /// Whether lake is available
    pub available: bool,
    /// Last query latency in ms
    pub last_latency_ms: u64,
}

impl MathlibLakeQuery {
    /// Create a new lake query interface
    pub fn new(project_path: PathBuf) -> Self {
        // Check if lake is available by running `lake --version`
        let available = Self::check_lake_available(&project_path);

        Self {
            project_path,
            available,
            last_latency_ms: 0,
        }
    }

    /// Check if lake is available
    fn check_lake_available(project_path: &PathBuf) -> bool {
        if !project_path.exists() {
            return false;
        }

        Command::new("lake")
            .arg("--version")
            .current_dir(project_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Search for lemmas matching a term pattern
    /// Returns empty if lake unavailable (fallback to heuristics)
    pub fn search_lemmas(&mut self, query: &str) -> Vec<LemmaMatch> {
        if !self.available {
            return Vec::new();
        }

        // Use `#find` tactic via lean if available
        // For now, return empty - real implementation would parse lean output
        // This is a placeholder that returns empty to indicate "fallback to heuristics"
        Vec::new()
    }

    /// Search for type class instances
    /// Returns empty if lake unavailable
    pub fn search_instances(&mut self, class: &str) -> Vec<InstanceMatch> {
        if !self.available {
            return Vec::new();
        }

        // Placeholder - real implementation would query lean
        Vec::new()
    }

    /// Verify a lemma exists in mathlib
    pub fn lemma_exists(&mut self, lemma: &str) -> bool {
        if !self.available {
            return false;
        }

        // Placeholder for real implementation
        false
    }
}

/// Report from the Mathlib Advisor containing lemma suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathlibAdvisorReport {
    /// The target theorem being worked on
    pub target_theorem: String,
    /// Query terms extracted from the target
    pub query_terms: Vec<String>,
    /// Suggested imports from mathlib
    pub suggested_imports: Vec<String>,
    /// Suggested lemmas to try
    pub suggested_lemmas: Vec<String>,
    /// The strategy category
    pub candidate_strategy: MathlibStrategy,
    /// Confidence (0.0 - 1.0) that this strategy will help
    pub confidence: f64,
    /// Risk tier of suggested imports
    pub import_risk: ImportRisk,
    // ==== Lake integration fields ====
    /// Whether lake environment was queried
    pub lake_queried: bool,
    /// Lemmas validated by lake (authoritative)
    pub lake_validated_lemmas: Vec<String>,
    /// Instances verified by lake
    pub lake_verified_instances: Vec<String>,
    /// Lake query latency in ms (0 if not available)
    pub lake_latency_ms: u64,
    /// Lake unavailable - using heuristics
    pub using_heuristics: bool,
}

/// Strategy categories for mathlib-assisted proof
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathlibStrategy {
    /// Order theory lemmas (IsGLB, IsLUB, etc.)
    OrderTheory,
    /// Greatest lower bound specific
    IsGLB,
    /// Supremum/infimum lemmas
    SInf,
    /// Set image operations
    SetImage,
    /// Monotonicity of addition
    MonotoneAdd,
    /// Proof by contradiction
    Contradiction,
    /// Approximation arguments
    Approximation,
    /// Hybrid: combining multiple strategies
    Hybrid,
}

impl MathlibStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            MathlibStrategy::OrderTheory => "OrderTheory",
            MathlibStrategy::IsGLB => "IsGLB",
            MathlibStrategy::SInf => "SInf",
            MathlibStrategy::SetImage => "SetImage",
            MathlibStrategy::MonotoneAdd => "MonotoneAdd",
            MathlibStrategy::Contradiction => "Contradiction",
            MathlibStrategy::Approximation => "Approximation",
            MathlibStrategy::Hybrid => "Hybrid",
        }
    }
}

/// Policy for mathlib import usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MathlibPolicy {
    /// Maximum number of new imports allowed
    pub max_new_imports: usize,
    /// Whether to allow heavy imports
    pub allow_heavy_imports: bool,
}

impl Default for MathlibPolicy {
    fn default() -> Self {
        Self {
            max_new_imports: 2,
            allow_heavy_imports: false,
        }
    }
}

/// Import risk tier for scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportRisk {
    /// No imports needed - using existing namespace lemmas
    None,
    /// Lightweight import from standard mathlib
    Light,
    /// Moderate import from extended mathlib
    Moderate,
    /// Heavy import from large modules (Analysis, Geometry, etc.)
    Heavy,
    /// Forbidden namespace
    Forbidden,
}

impl ImportRisk {
    /// Convert to numeric score for candidate ranking (lower is better)
    pub fn to_score(&self) -> f64 {
        match self {
            ImportRisk::None => 0.0,
            ImportRisk::Light => 0.1,
            ImportRisk::Moderate => 0.3,
            ImportRisk::Heavy => 0.7,
            ImportRisk::Forbidden => 1.0,
        }
    }
}

/// Lemma families to search for specific theorem targets
pub struct MathlibLemmaFamilies;

impl MathlibLemmaFamilies {
    /// Lemma families relevant to isRationalInf_exists_lt_of_lt
    pub fn for_exists_lt_of_lt() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            (
                "OrderTheory",
                vec![
                    "not_lt_of_ge",
                    "le_of_not_gt",
                    "lt_of_not_le",
                    "exists_lt_of_lt",
                ],
            ),
            (
                "IsGLB",
                vec![
                    "isGLB_def",
                    "isGLB_of_forall_le",
                    "IsGLB.of_iInf",
                    "IsGLB.of_forall_le",
                ],
            ),
            (
                "SInf",
                vec!["iInf_le", "le_iInf", "sInf_le", "csInf_le", "iInf_mono"],
            ),
        ]
    }

    /// Lemma families relevant to isRationalInf_pairwise_add
    pub fn for_pairwise_add() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            (
                "OrderTheory",
                vec![
                    "add_le_add",
                    "le_add_left",
                    "le_add_right",
                    "add_le_add_left",
                ],
            ),
            (
                "IsGLB",
                vec!["IsGLB.add", "isGLB_pairwise", "isGLB_image", "IsGLB.of_set"],
            ),
            (
                "SetImage",
                vec!["Set.image", "Set.image_univ", "image_preimage"],
            ),
            (
                "SInf",
                vec![
                    "sInf_add",
                    "iInf_add",
                    "csInf_add",
                    "sInf_le_iInf",
                    "iInf_mono",
                ],
            ),
        ]
    }

    /// Imports typically needed for order/infimum proofs
    pub fn standard_imports() -> Vec<(&'static str, &'static str)> {
        vec![
            ("OrderTheory", "Mathlib.Order.LiminfLimsup"),
            ("OrderTheory", "Mathlib.Order.Basic"),
            ("SInf", "Mathlib.Topology.Algebra.Order"),
            ("IsGLB", "Mathlib.Data.Set.Basic"),
            ("IsGLB", "Mathlib.Order.Directed"),
        ]
    }
}

/// Generate advisor report for a target theorem
pub fn generate_report(target: &str) -> MathlibAdvisorReport {
    let (query_terms, strategy, lemmas) = match target {
        "isRationalInf_exists_lt_of_lt" => (
            vec![
                "exists_lt_of_lt".to_string(),
                "IsGLB".to_string(),
                "sInf".to_string(),
                "iInf".to_string(),
                "csInf".to_string(),
                "lower_bound".to_string(),
            ],
            MathlibStrategy::OrderTheory,
            vec![
                "not_lt_of_ge".to_string(),
                "le_of_not_gt".to_string(),
                "lt_of_not_le".to_string(),
                "exists_lt_of_lt".to_string(),
                "iInf_le".to_string(),
            ],
        ),
        "isRationalInf_pairwise_add" => (
            vec![
                "pairwise_add".to_string(),
                "IsGLB".to_string(),
                "greatest_lower_bound".to_string(),
                "sInf".to_string(),
                "iInf".to_string(),
                "add_le_add".to_string(),
            ],
            MathlibStrategy::IsGLB,
            vec![
                "add_le_add".to_string(),
                "IsGLB.add".to_string(),
                "isGLB_image".to_string(),
                "csInf_add".to_string(),
                "sInf_add".to_string(),
                "iInf_add".to_string(),
                // Hybrid: Combined lemma families
                "sInf_eq_iInf".to_string(),
                "iInf_add_le_add".to_string(),
                "isGLB_of_isLUB".to_string(),
                "csInf_of_sInf".to_string(),
            ],
        ),
        _ => (
            vec![target.to_string()],
            MathlibStrategy::Approximation,
            vec![],
        ),
    };

    let suggested_imports: Vec<String> = MathlibLemmaFamilies::standard_imports()
        .into_iter()
        .map(|(_, imp)| imp.to_string())
        .collect();

    // Compute import risk
    let import_risk = assess_import_risk(&suggested_imports);

    let mut confidence = match strategy {
        MathlibStrategy::Hybrid => 0.85, // Higher confidence for combined strategies
        MathlibStrategy::IsGLB => 0.75,
        MathlibStrategy::SInf => 0.70,
        MathlibStrategy::OrderTheory => 0.65,
        MathlibStrategy::Approximation => 0.50,
        MathlibStrategy::MonotoneAdd => 0.60,
        MathlibStrategy::Contradiction => 0.55,
        MathlibStrategy::SetImage => 0.40,
    };

    // Penalize high risk imports
    confidence -= import_risk.to_score() * 0.2;

    MathlibAdvisorReport {
        target_theorem: target.to_string(),
        query_terms,
        suggested_imports,
        suggested_lemmas: lemmas,
        candidate_strategy: strategy,
        confidence: confidence.max(0.1),
        import_risk,
        // Lake integration fields - default to heuristics mode
        lake_queried: false,
        lake_validated_lemmas: Vec::new(),
        lake_verified_instances: Vec::new(),
        lake_latency_ms: 0,
        using_heuristics: true,
    }
}

/// Categorize the risk tier for a list of imports
pub fn assess_import_risk(imports: &[String]) -> ImportRisk {
    if imports.is_empty() {
        return ImportRisk::None;
    }

    let heavy = [
        "Mathlib.Analysis",
        "Mathlib.Geometry",
        "Mathlib.Representation",
        "Mathlib.CategoryTheory",
    ];
    let moderate = ["Mathlib.Data", "Mathlib.Algebra", "Mathlib.Topology"];

    // Check for forbidden/heavy
    for imp in imports {
        for h in heavy {
            if imp.starts_with(h) {
                return ImportRisk::Forbidden;
            }
        }
    }

    // Check for moderate
    for imp in imports {
        for m in moderate {
            if imp.starts_with(m) {
                return ImportRisk::Moderate;
            }
        }
    }

    // Default to light for valid mathlib imports
    ImportRisk::Light
}

/// Check if imports comply with policy
pub fn check_policy(imports: &[String], policy: MathlibPolicy) -> bool {
    if imports.len() > policy.max_new_imports {
        return false;
    }
    let heavy = [
        "Mathlib.Analysis",
        "Mathlib.Geometry",
        "Mathlib.Representation",
    ];
    if !policy.allow_heavy_imports {
        for imp in imports {
            for h in heavy {
                if imp.starts_with(h) {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_report_exists_lt() {
        let report = generate_report("isRationalInf_exists_lt_of_lt");
        assert_eq!(report.target_theorem, "isRationalInf_exists_lt_of_lt");
        assert!(!report.suggested_lemmas.is_empty());
        assert!(report.confidence > 0.0);
    }

    #[test]
    fn test_generate_report_pairwise_add() {
        let report = generate_report("isRationalInf_pairwise_add");
        assert_eq!(report.target_theorem, "isRationalInf_pairwise_add");
        assert!(report.candidate_strategy == MathlibStrategy::IsGLB);
    }

    #[test]
    fn test_policy_check_pass() {
        let policy = MathlibPolicy::default();
        let imports = vec![
            "Mathlib.Order.LiminfLimsup".to_string(),
            "Mathlib.Order.Basic".to_string(),
        ];
        assert!(check_policy(&imports, policy));
    }

    #[test]
    fn test_policy_check_fail_too_many() {
        let policy = MathlibPolicy::default();
        let imports = vec![
            "Mathlib.Order.LiminfLimsup".to_string(),
            "Mathlib.Order.Basic".to_string(),
            "Mathlib.Data.Set.Basic".to_string(),
        ];
        assert!(!check_policy(&imports, policy));
    }
}
