//! Code-Patch Domain Adapter for NPE
//!
//! This adapter allows the NPE to generate code patches for Coh verifier modules,
//! then verifies them through the actual Rust compiler and test suite.
//!
//! First target: `semantic.rs`

use serde::{Deserialize, Serialize};

/// First failure classification for code patches
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodePatchFirstFailure {
    /// No failure - patch was accepted
    Accepted,
    /// Genesis margin was negative
    Genesis,
    /// cargo check failed
    CargoCheck,
    /// cargo test failed
    CargoTest,
    /// rustfmt failed
    Format,
    /// clippy lint failed
    Lint,
    /// Touched forbidden file
    ForbiddenFile,
    /// Schema compatibility check failed
    SchemaCompat,
    /// Coherence margin was negative
    Coherence,
}

impl CodePatchFirstFailure {
    /// Classify the first failure for a code patch
    pub fn classify(
        genesis_margin: i128,
        report: &CodePatchReport,
        coherence_margin: i128,
    ) -> Self {
        // First check Genesis
        if genesis_margin < 0 {
            return CodePatchFirstFailure::Genesis;
        }
        // Then check verification gates in order
        if !report.cargo_check_pass {
            return CodePatchFirstFailure::CargoCheck;
        }
        if !report.cargo_test_pass {
            return CodePatchFirstFailure::CargoTest;
        }
        if !report.fmt_pass {
            return CodePatchFirstFailure::Format;
        }
        if !report.lint_pass {
            return CodePatchFirstFailure::Lint;
        }
        if report.forbidden_files_touched {
            return CodePatchFirstFailure::ForbiddenFile;
        }
        if !report.schema_compat_pass {
            return CodePatchFirstFailure::SchemaCompat;
        }
        // Finally check Coherence
        if coherence_margin < 0 {
            return CodePatchFirstFailure::Coherence;
        }
        // All checks passed
        CodePatchFirstFailure::Accepted
    }

    /// Returns true if the patch was accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self, CodePatchFirstFailure::Accepted)
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CodePatchFirstFailure::Accepted => "accepted",
            CodePatchFirstFailure::Genesis => "genesis margin negative",
            CodePatchFirstFailure::CargoCheck => "cargo check failed",
            CodePatchFirstFailure::CargoTest => "cargo test failed",
            CodePatchFirstFailure::Format => "format check failed",
            CodePatchFirstFailure::Lint => "lint check failed",
            CodePatchFirstFailure::ForbiddenFile => "forbidden file touched",
            CodePatchFirstFailure::SchemaCompat => "schema compatibility failed",
            CodePatchFirstFailure::Coherence => "coherence margin negative",
        }
    }
}

/// Hard policy gates that can reject patches
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatchHardGate {
    /// Touched crypto/digest code
    CryptoTouched,
    /// Changed schema without version bump
    SchemaChangedWithoutVersionBump,
    /// Introduced unsafe code
    UnsafeIntroduced,
    /// Introduced floating point in verifier logic
    FloatArithmeticIntroduced,
    /// Changed canonical serialization
    CanonicalizationTouched,
    /// Changed admission profile defaults
    AdmissionProfileDefaultChanged,
    /// Too many files changed
    TooManyFiles,
    /// Too many lines changed
    TooManyLines,
    /// Patch affects reject code paths - keyword detection
    RejectPathModified,
    /// Patch weakens reject logic (structural)
    RejectPathWeakened,
}

/// Reject path impact classification
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RejectPathImpact {
    /// No reject-related content detected
    None,
    /// Patch mentions reject-related terms (warning/review)
    Mentioned,
    /// Patch adds stricter reject condition
    Strengthened,
    /// Patch removes/bypasses/relaxes reject logic
    Weakened,
    /// Patch changes reject code enum/serialization
    SchemaChanged,
}

impl RejectPathImpact {
    /// Analyze patch text for reject-path impact
    pub fn classify(patch_text: &str, changed_files: &[String]) -> Self {
        let text_lower = patch_text.to_lowercase();

        // Check for keywords
        let has_reject = text_lower.contains("reject");
        let has_deny = text_lower.contains("deny");
        let has_invalid = text_lower.contains("invalid");

        if !has_reject && !has_deny && !has_invalid {
            return RejectPathImpact::None;
        }

        // Simple structural heuristics
        // Strengthening: adding assert/require/stricter condition
        let strengthens = text_lower.contains("assert")
            || text_lower.contains("require")
            || text_lower.contains("must")
            || text_lower.contains("should")
            || (text_lower.contains("reject") && text_lower.contains("+"));

        // Weakening: removing checks, defaulting, unwrap_or
        let weakens = text_lower.contains("unwrap_or_default")
            || text_lower.contains("unwrap_or")
            || text_lower.contains("default()")
            || text_lower.contains(".unwrap()")
            || (text_lower.contains("reject") && text_lower.contains("-"))
            || text_lower.contains("allow(");

        // Schema change: enum variant changes
        let schema_change = changed_files
            .iter()
            .any(|f| f.contains("types") || f.contains("reject"));

        if schema_change && (has_reject || has_deny) {
            RejectPathImpact::SchemaChanged
        } else if weakens {
            RejectPathImpact::Weakened
        } else if strengthens {
            RejectPathImpact::Strengthened
        } else {
            RejectPathImpact::Mentioned
        }
    }

    /// Returns true if this impact level is allowed under strict policy
    pub fn allowed_strict(&self) -> bool {
        matches!(self, RejectPathImpact::None)
    }

    /// Returns true if this impact level is allowed under audited policy
    pub fn allowed_audited(&self) -> bool {
        matches!(
            self,
            RejectPathImpact::None | RejectPathImpact::Mentioned | RejectPathImpact::Strengthened
        )
    }
}

/// Patch policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchPolicy {
    pub allow_crypto_touch: bool,
    pub allow_schema_change: bool,
    pub allow_cross_file: bool,
    pub max_changed_files: usize,
    pub max_changed_lines: usize,
    pub forbid_unsafe: bool,
    pub forbid_float_arithmetic: bool,
    pub forbid_reject_path_change: bool,
    pub reject_policy: RejectPolicyMode,
}

impl Default for PatchPolicy {
    fn default() -> Self {
        PatchPolicy {
            allow_crypto_touch: false,
            allow_schema_change: false,
            allow_cross_file: false,
            max_changed_files: 5,
            max_changed_lines: 500,
            forbid_unsafe: true,
            forbid_float_arithmetic: true,
            forbid_reject_path_change: true,
            reject_policy: RejectPolicyMode::Strict,
        }
    }
}

/// Reject path policy mode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RejectPolicyMode {
    /// No reject-path modifications allowed
    #[default]
    Strict,
    /// Allow strengthening modifications, audit strengthned and schema changes
    Audited,
}

/// Patch selection mode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatchSelectorMode {
    /// Maximizes novelty with safety margin
    SafeNovel,
    /// Maximizes novelty near boundary
    Edge,
    /// Finds near-boundary still-admissible
    NearBoundary,
}

/// Formation result for code patches
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodePatchFormationResult {
    pub candidate_id: String,
    pub formation_accept: bool,
    pub first_failure: CodePatchFirstFailure,

    pub genesis_margin: i128,
    pub coherence_margin: i128,
    pub boundary_margin: i128,

    pub novelty: f64,
    pub safe_score: f64,
    pub edge_score: f64,

    pub hard_gates: Vec<PatchHardGate>,
    pub report: CodePatchReport,
}

/// Code patch candidate generated by the NPE
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodePatchCandidate {
    pub id: String,
    pub wildness: f64,
    pub target_file: String,
    pub patch_text: String,
    pub changed_files: Vec<String>,
    pub changed_lines: usize,
    pub generated_tokens: usize,
    pub novelty: f64,
}

/// Code patch verification report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodePatchReport {
    pub cargo_check_pass: bool,
    pub cargo_test_pass: bool,
    pub fmt_pass: bool,
    pub lint_pass: bool,
    pub forbidden_files_touched: bool,
    pub schema_compat_pass: bool,

    pub compile_time_ms: u64,
    pub test_time_ms: u64,
    pub warnings: usize,
    pub failed_tests: usize,

    pub genesis_margin: i128,
    pub coherence_margin: i128,
    pub formation_accept: bool,
}

/// Compute Genesis metrics for a code patch
pub fn compute_genesis_metrics(candidate: &CodePatchCandidate) -> (u128, u128, u128) {
    // M(t'): post-patch complexity
    let m_after = compute_complexity(&candidate.target_file, candidate.changed_lines);

    // C(p): generation cost
    let cost = candidate.generated_tokens as u128 / 10 + (candidate.changed_files.len() as u128);

    // D(p): defect budget based on wildness
    let defect = 100 + (candidate.wildness * 20.0) as u128;

    (m_after, cost, defect)
}

/// Compute patch complexity
fn compute_complexity(file: &str, changed_lines: usize) -> u128 {
    let mut complexity = changed_lines as u128 * 3;

    // Add weight for verifier core files
    if file.contains("verify") {
        complexity += 50;
    }
    if file.contains("semantic") {
        complexity += 30;
    }
    if file.contains("canon") {
        complexity += 30;
    }
    if file.contains("types") {
        complexity += 20;
    }

    complexity
}

/// Compute Coherence metrics for a code patch
pub fn compute_coherence_metrics(report: &CodePatchReport) -> (u128, u128, u128) {
    // V(y): post-patch risk
    let mut risk = 0u128;
    if !report.cargo_check_pass {
        risk += 100;
    }
    if !report.cargo_test_pass {
        risk += 80;
    }
    if !report.schema_compat_pass {
        risk += 50;
    }
    if report.forbidden_files_touched {
        risk += 40;
    }
    risk += report.warnings as u128 * 2;
    risk += report.failed_tests as u128 * 10;

    // Spend(R): execution cost
    let spend = (report.compile_time_ms / 100) as u128 + (report.test_time_ms / 100) as u128;

    // Defect(R): allowed budget
    let defect = 50; // base budget

    (risk, spend, defect)
}

/// Check if candidate is formation-admissible
pub fn is_formation_admissible(
    candidate: &CodePatchCandidate,
    base_complexity: u128,
    report: &CodePatchReport,
) -> (bool, i128, i128) {
    let (m_after, cost, defect) = compute_genesis_metrics(candidate);
    let genesis_margin = base_complexity as i128 + defect as i128 - m_after as i128 - cost as i128;

    let (risk, spend, def) = compute_coherence_metrics(report);
    let coherence_margin = risk as i128 + def as i128 - spend as i128;

    let formation_accept = genesis_margin >= 0
        && report.cargo_check_pass
        && report.cargo_test_pass
        && coherence_margin >= 0;

    (formation_accept, genesis_margin, coherence_margin)
}

/// Check hard policy gates for a patch
pub fn check_hard_gates(
    candidate: &CodePatchCandidate,
    policy: &PatchPolicy,
) -> Vec<PatchHardGate> {
    let mut gates = Vec::new();

    // Check file/directory limits
    if candidate.changed_files.len() > policy.max_changed_files {
        gates.push(PatchHardGate::TooManyFiles);
    }
    if candidate.changed_lines > policy.max_changed_lines {
        gates.push(PatchHardGate::TooManyLines);
    }

    let patch_lower = candidate.patch_text.to_lowercase();

    // Check for crypto/digest file touches
    if candidate
        .changed_files
        .iter()
        .any(|f| f.contains("hash") || f.contains("digest") || f.contains("crypto"))
        && !policy.allow_crypto_touch
    {
        gates.push(PatchHardGate::CryptoTouched);
    }

    // Check for unsafe keyword
    if policy.forbid_unsafe && patch_lower.contains("unsafe") {
        gates.push(PatchHardGate::UnsafeIntroduced);
    }

    // Check for floating point in verifier context
    if policy.forbid_float_arithmetic {
        if candidate.target_file.contains("verify")
            || candidate.target_file.contains("coh")
            || candidate.target_file.contains("auth")
        {
            if patch_lower.contains("f32")
                || patch_lower.contains("f64")
                || patch_lower.contains("float")
            {
                gates.push(PatchHardGate::FloatArithmeticIntroduced);
            }
        }
    }

    // Check for reject path modifications - use structural analysis
    if policy.forbid_reject_path_change {
        let impact = RejectPathImpact::classify(&candidate.patch_text, &candidate.changed_files);

        match policy.reject_policy {
            RejectPolicyMode::Strict => {
                if !impact.allowed_strict() {
                    if impact == RejectPathImpact::Weakened {
                        gates.push(PatchHardGate::RejectPathWeakened);
                    } else {
                        gates.push(PatchHardGate::RejectPathModified);
                    }
                }
            }
            RejectPolicyMode::Audited => {
                if !impact.allowed_audited() {
                    if impact == RejectPathImpact::Weakened
                        || impact == RejectPathImpact::SchemaChanged
                    {
                        gates.push(PatchHardGate::RejectPathWeakened);
                    } else {
                        gates.push(PatchHardGate::RejectPathModified);
                    }
                }
            }
        }
    }

    // Check for cross-file patches
    if candidate.changed_files.len() > 1 && !policy.allow_cross_file {
        gates.push(PatchHardGate::TooManyFiles);
    }

    gates
}

/// Compute patch selector scores
pub fn compute_patch_scores(
    novelty: f64,
    genesis_margin: i128,
    coherence_margin: i128,
    mode: &PatchSelectorMode,
    alpha: f64,
) -> (f64, f64) {
    let boundary_margin = genesis_margin.min(coherence_margin);

    let safe_score = novelty + alpha * boundary_margin as f64;
    let edge_score = novelty - alpha * boundary_margin as f64;

    match mode {
        PatchSelectorMode::SafeNovel => (safe_score, edge_score),
        PatchSelectorMode::Edge => (safe_score, edge_score),
        PatchSelectorMode::NearBoundary => (safe_score, edge_score),
    }
}

/// Build formation result for a code patch
pub fn build_formation_result(
    candidate: &CodePatchCandidate,
    base_complexity: u128,
    report: &CodePatchReport,
    policy: &PatchPolicy,
    mode: &PatchSelectorMode,
    alpha: f64,
) -> CodePatchFormationResult {
    // Compute Genesis metrics
    let (m_after, cost, defect) = compute_genesis_metrics(candidate);
    let genesis_margin = base_complexity as i128 + defect as i128 - m_after as i128 - cost as i128;

    // Compute Coherence metrics
    let (risk, spend, def) = compute_coherence_metrics(report);
    let coherence_margin = risk as i128 + def as i128 - spend as i128;

    let boundary_margin = genesis_margin.min(coherence_margin);

    // Compute scores
    let (safe_score, edge_score) = compute_patch_scores(
        candidate.novelty,
        genesis_margin,
        coherence_margin,
        mode,
        alpha,
    );

    // Check hard gates
    let hard_gates = check_hard_gates(candidate, policy);

    // Determine formation acceptance
    let formation_accept = genesis_margin >= 0
        && report.cargo_check_pass
        && report.cargo_test_pass
        && coherence_margin >= 0
        && hard_gates.is_empty();

    // Classify first failure
    let first_failure = CodePatchFirstFailure::classify(genesis_margin, report, coherence_margin);

    CodePatchFormationResult {
        candidate_id: candidate.id.clone(),
        formation_accept,
        first_failure,
        genesis_margin,
        coherence_margin,
        boundary_margin,
        novelty: candidate.novelty,
        safe_score,
        edge_score,
        hard_gates,
        report: report.clone(),
    }
}

/// Wildness level to patch type mapping
pub fn patch_type_for_wildness(wildness: f64) -> &'static str {
    if wildness < 0.5 {
        "doc/comment/test-only patch"
    } else if wildness < 1.0 {
        "small helper refactor"
    } else if wildness < 1.5 {
        "add one semantic registry case"
    } else if wildness < 2.0 {
        "refactor local function plus tests"
    } else if wildness < 2.5 {
        "add new envelope source or reject path"
    } else if wildness < 3.0 {
        "multi-function patch with tests"
    } else if wildness < 5.0 {
        "cross-file patch"
    } else if wildness < 10.0 {
        "architecture-level change"
    } else {
        "extreme redesign"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_metrics() {
        let candidate = CodePatchCandidate {
            id: "test-1".to_string(),
            wildness: 2.0,
            target_file: "semantic.rs".to_string(),
            patch_text: "...".to_string(),
            changed_files: vec!["semantic.rs".to_string()],
            changed_lines: 50,
            generated_tokens: 500,
            novelty: 5.0,
        };

        let (m_after, cost, defect) = compute_genesis_metrics(&candidate);
        assert!(m_after > 0);
        assert!(cost > 0);
    }

    #[test]
    fn test_patch_type() {
        assert_eq!(patch_type_for_wildness(0.0), "doc/comment/test-only patch");
        assert_eq!(
            patch_type_for_wildness(2.0),
            "add new envelope source or reject path"
        );
    }
}
