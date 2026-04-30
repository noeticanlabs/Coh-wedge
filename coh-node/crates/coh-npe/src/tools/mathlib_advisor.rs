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
use crate::failure_taxonomy::{
    FailureKind, FailureLayer, FailureReport, FailureSeverity, LeanElabFailure, LeanProofFailure,
    LeanSyntaxFailure, RepairStrategy,
};
// use crate::lean_json_export::execute_lean_json_search;

/// Component that synthesizes missing lemmas based on algebraic patterns
#[derive(Debug, Clone, Default)]
pub struct LemmaSynthesizer {
    pub history: Vec<String>,
}

impl LemmaSynthesizer {
    /// Synthesize a lemma name and body based on a requested pattern
    pub fn synthesize(&mut self, target: &str, pattern: &str) -> Option<(String, String)> {
        // Pattern: "sub_eq_of_add_eq" for ENNRat
        if pattern.contains("sub_eq_of_add_eq") && target.contains("ENNRat") {
            let name = "ENNRat.sub_eq_of_add_eq".to_string();
            let body = "theorem sub_eq_of_add_eq {a b c : ENNRat} (h1 : a = b + c) (h2 : b < 1) : a - c = b := by sorry".to_string();
            self.history.push(name.clone());
            return Some((name, body));
        }

        None
    }
}

/// Generate a rich failure report from Lean output
pub fn generate_failure_report(
    candidate_id: &str,
    target: &str,
    output: &str,
) -> Option<FailureReport> {
    if output.is_empty() {
        return None;
    }

    for line in output.lines() {
        let line = line.trim();

        // 1. Lean Syntax Failures
        if line.contains("unexpected token") {
            return Some(FailureReport {
                candidate_id: candidate_id.to_string(),
                target: target.to_string(),
                layer: FailureLayer::LeanSyntax,
                kind: FailureKind::LeanSyntax(LeanSyntaxFailure::UnexpectedToken(line.to_string())),
                raw_error: output.to_string(),
                normalized_message: "Unexpected token in Lean syntax".to_string(),
                retryable: true,
                severity: FailureSeverity::Repairable,
                suggested_repairs: vec![RepairStrategy::SyntaxRepair],
                blocks_publication: false,
            });
        }

        // 2. Lean Elaboration Failures
        if line.contains("unknown identifier") {
            let id = if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    line[start + 1..start + 1 + end].to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };
            return Some(FailureReport {
                candidate_id: candidate_id.to_string(),
                target: target.to_string(),
                layer: FailureLayer::LeanElaboration,
                kind: FailureKind::LeanElab(LeanElabFailure::UnknownIdentifier(id)),
                raw_error: output.to_string(),
                normalized_message: "Unknown identifier".to_string(),
                retryable: true,
                severity: FailureSeverity::Repairable,
                suggested_repairs: vec![RepairStrategy::MathlibLemmaSearch],
                blocks_publication: false,
            });
        }

        if line.contains("failed to synthesize instance") {
            return Some(FailureReport {
                candidate_id: candidate_id.to_string(),
                target: target.to_string(),
                layer: FailureLayer::LeanElaboration,
                kind: FailureKind::LeanElab(LeanElabFailure::FailedToSynthesizeInstance(
                    line.to_string(),
                )),
                raw_error: output.to_string(),
                normalized_message: "Missing typeclass instance".to_string(),
                retryable: true,
                severity: FailureSeverity::Repairable,
                suggested_repairs: vec![RepairStrategy::CoercionRepair],
                blocks_publication: false,
            });
        }

        // 3. Lean Proof Failures
        if line.contains("unsolved goals") {
            return Some(FailureReport {
                candidate_id: candidate_id.to_string(),
                target: target.to_string(),
                layer: FailureLayer::LeanProof,
                kind: FailureKind::LeanProof(LeanProofFailure::UnsolvedGoals),
                raw_error: output.to_string(),
                normalized_message: "Goals remaining".to_string(),
                retryable: true,
                severity: FailureSeverity::UsefulNearMiss,
                suggested_repairs: vec![RepairStrategy::HelperLemmaCreation],
                blocks_publication: false,
            });
        }
    }

    // 4. Mathematical / Analytical Failures (Deep Audit)
    // if let Some(math_report) = crate::math_analytic_failure::classify_math_analytic_gap(
    //     candidate_id,
    //     target,
    //     "", // proof_text not available here, would need to be passed in
    //     output,
    // ) {
    //     return Some(math_report);
    // }

    Some(FailureReport {
        candidate_id: candidate_id.to_string(),
        target: target.to_string(),
        layer: FailureLayer::LeanProof,
        kind: FailureKind::Other(output.to_string()),
        raw_error: output.to_string(),
        normalized_message: "Unclassified Lean error".to_string(),
        retryable: true,
        severity: FailureSeverity::Advisory,
        suggested_repairs: vec![],
        blocks_publication: false,
    })
}

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
    /// Local synthesizer for missing lemmas
    pub synthesizer: LemmaSynthesizer,
    /// Path to lake executable if not in system PATH
    pub lake_cmd: String,
    /// Whether lake is available
    pub available: bool,
    /// Last query latency in ms
    pub last_latency_ms: u64,
    /// Timeout for lake queries in seconds
    pub timeout_secs: u64,
}

impl MathlibLakeQuery {
    /// Create a new lake query interface
    pub fn new(project_path: PathBuf) -> Self {
        // Check if lake is available and find its command
        let (available, lake_cmd) = Self::discover_lake(&project_path);

        Self {
            project_path,
            lake_cmd,
            available,
            last_latency_ms: 0,
            timeout_secs: 60, // Default to 1 minute
            synthesizer: LemmaSynthesizer::default(),
        }
    }

    /// Discover lake command and availability
    fn discover_lake(project_path: &PathBuf) -> (bool, String) {
        if !project_path.exists() {
            return (false, "lake".to_string());
        }

        // 1. Try system PATH
        if Command::new("lake")
            .arg("--version")
            .current_dir(project_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return (true, "lake".to_string());
        }

        // 2. Try common elan path on Windows
        if let Ok(home) = std::env::var("USERPROFILE") {
            let elan_lake = PathBuf::from(home).join(".elan").join("bin").join("lake.exe");
            if elan_lake.exists() {
                let available = Command::new(&elan_lake)
                    .arg("--version")
                    .current_dir(project_path)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if available {
                    return (true, elan_lake.to_string_lossy().to_string());
                }
            }
        }

        (false, "lake".to_string())
    }

    /// Search for lemmas matching a term pattern
    /// Returns empty if lake unavailable (fallback to heuristics)
    pub fn search_lemmas(&mut self, query: &str) -> Vec<LemmaMatch> {
        if !self.available {
            return Vec::new();
        }

        let start = std::time::Instant::now();
        // let results = execute_lean_json_search(&self.project_path, &self.lake_cmd, query, Some(self.timeout_secs));
        
        self.last_latency_ms = start.elapsed().as_millis() as u64;

        // Mock results
        vec![]
    }

    /// Parse Lean search output into LemmaMatch objects
    #[allow(dead_code)]
    fn parse_search_output(&self, output: &str) -> Vec<LemmaMatch> {
        let mut results = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("import") || line.contains("Checking") {
                continue;
            }

            // Look for patterns like "lemma_name : type"
            if let Some(colon_idx) = line.find(':') {
                let name = line[..colon_idx].trim().to_string();
                if !name.is_empty() && !name.contains(' ') {
                    results.push(LemmaMatch {
                        name,
                        file: "Mathlib".to_string(), // Simplified
                        in_mathlib: true,
                    });
                }
            }
        }
        results
    }

    /// Search for type class instances
    /// Returns empty if lake unavailable
    pub fn search_instances(&mut self, _class: &str) -> Vec<InstanceMatch> {
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

        let matches = self.search_lemmas(lemma);
        matches.iter().any(|m| m.name == lemma)
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
    /// Detected Coh template pattern
    pub coh_template: Option<crate::templates::CohTemplateKind>,
    /// Suggested tactics from the Coh template
    pub template_tactics: Vec<String>,
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
    /// Timeout for searches in seconds
    pub timeout_secs: u64,
}

impl Default for MathlibPolicy {
    fn default() -> Self {
        Self {
            max_new_imports: 2,
            allow_heavy_imports: false,
            timeout_secs: 60,
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
        coh_template: crate::templates::classify_coh_template(target),
        template_tactics: crate::templates::classify_coh_template(target)
            .map(|t| t.preferred_tactics().into_iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
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
    #[test]
    fn test_lake_search() {
        let project_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent().unwrap().parent().unwrap().parent().unwrap()
            .join("coh-t-stack");
        
        if !project_path.exists() {
            println!("Skipping lake search test: coh-t-stack not found at {:?}", project_path);
            return;
        }

        let mut query = MathlibLakeQuery::new(project_path);
        if !query.available {
            println!("Skipping lake search test: lake not available");
            return;
        }

        let results = query.search_lemmas("NNRat → ENNRat");
        // We expect at least some results or at least no crash
        println!("Lake search found {} results in {}ms", results.len(), query.last_latency_ms);
    }
}
