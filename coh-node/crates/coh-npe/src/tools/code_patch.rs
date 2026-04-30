//! Code-Patch Domain Adapter for NPE
//!
//! This adapter allows the NPE to generate code patches for Coh verifier modules,
//! then verifies them through the actual Rust compiler and test suite.
//!
//! First target: `semantic.rs`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    if policy.forbid_float_arithmetic
        && (candidate.target_file.contains("verify")
            || candidate.target_file.contains("coh")
            || candidate.target_file.contains("auth"))
        && (patch_lower.contains("f32")
            || patch_lower.contains("f64")
            || patch_lower.contains("float"))
    {
        gates.push(PatchHardGate::FloatArithmeticIntroduced);
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

// === DEPENDENCY UPGRADE TYPES ===

/// Target type for NPE generation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpgradeTarget {
    /// Code patch (existing .rs file modifications)
    CodePatch,
    /// Dependency upgrade (Cargo.toml / toolchain updates)
    Dependency,
    /// Mixed mode - alternate between both
    Mixed,
}

/// Classification for dependency upgrades
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpgradeClass {
    /// Update existing crate to new version
    VersionBump,
    /// Add a new dependency
    NewCrate,
    /// Remove an unused dependency
    RemoveCrate,
    /// Update Rust toolchain
    Toolchain,
    /// Security patch (patch version bump)
    PatchVersion,
}

impl UpgradeClass {
    /// Get description for this upgrade class
    pub fn description(&self) -> &'static str {
        match self {
            UpgradeClass::VersionBump => "Update crate to newer version",
            UpgradeClass::NewCrate => "Add new dependency",
            UpgradeClass::RemoveCrate => "Remove unused dependency",
            UpgradeClass::Toolchain => "Update Rust toolchain version",
            UpgradeClass::PatchVersion => "Security patch update",
        }
    }

    /// Get risk level (0 = low, 10 = high)
    pub fn risk_level(&self) -> u8 {
        match self {
            UpgradeClass::VersionBump => 3,
            UpgradeClass::NewCrate => 5,
            UpgradeClass::RemoveCrate => 4,
            UpgradeClass::Toolchain => 7,
            UpgradeClass::PatchVersion => 1,
        }
    }
}

/// Dependency upgrade candidate generated by NPE
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyUpgradeCandidate {
    pub id: String,
    pub wildness: f64,
    pub upgrade_class: UpgradeClass,
    pub cargo_toml_path: String,
    pub upgrade_text: String,
    pub old_version: String,
    pub new_version: String,
    pub crate_name: String,
    pub novelty: f64,
}

/// Dependency upgrade verification report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyUpgradeReport {
    pub cargo_update_dry_run_pass: bool,
    pub cargo_check_pass: bool,
    pub outdated_check_pass: bool,
    pub breaking_change: bool,
    pub new_version_available: bool,
    pub update_size_kb: u64,
    pub upgrade_margin: i128,
}

/// Parsed Cargo.toml dependency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedDependency {
    pub name: String,
    pub version: String,
    pub is_workspace: bool,
}

/// Crate update information from cargo outdated
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateUpdate {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
    pub kind: String, // "Normal" or "Dev"
}

/// Parse Cargo.toml and extract dependencies
pub fn parse_cargo_toml(path: &str) -> Result<HashMap<String, ParsedDependency>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let mut deps = HashMap::new();
    let mut in_dependencies = false;
    let mut in_dev_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[dependencies]" {
            in_dependencies = true;
            in_dev_dependencies = false;
            continue;
        }
        if trimmed == "[dev-dependencies]" {
            in_dependencies = false;
            in_dev_dependencies = true;
            continue;
        }
        if trimmed.starts_with('[') && !trimmed.starts_with("[dependencies") {
            in_dependencies = false;
            in_dev_dependencies = false;
            continue;
        }

        if in_dependencies || in_dev_dependencies {
            if let Some((name, version)) = parse_toml_dep_line(trimmed) {
                deps.insert(
                    name.clone(),
                    ParsedDependency {
                        name,
                        version,
                        is_workspace: false,
                    },
                );
            }
        }
    }

    Ok(deps)
}

/// Parse a single dependency line like "crate_name = \"1.0\""
fn parse_toml_dep_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    // Handle simple version: crate = "1.0"
    // Handle with features: crate = { version = "1.0", features = [...] }
    if let Some(eq_pos) = line.find('=') {
        let name = line[..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim();

        if let Some(stripped) = value.strip_prefix('\"') {
            // Simple string version
            if let Some(end_quote) = stripped.find('"') {
                let version = value[1..=end_quote].to_string();
                return Some((name, version));
            }
        } else if value.starts_with('{') {
            // Object form - try to extract version
            if let Some(v_start) = value.find("version") {
                let v_part = &value[v_start..];
                if let Some(eq) = v_part.find('=') {
                    let v_value = v_part[eq + 1..].trim();
                    if let Some(_stripped) = v_value.strip_prefix('"') {
                        if let Some(end_quote) = v_value[1..].find('"') {
                            let version = v_value[1..=end_quote].to_string();
                            return Some((name, version));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Query available crate updates using cargo outdated
pub fn check_cargo_outdated(cargo_toml_dir: &str) -> Result<Vec<CrateUpdate>, String> {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["outdated", "--format", "json"])
        .current_dir(cargo_toml_dir)
        .output()
        .map_err(|e| format!("Failed to run cargo outdated: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new()); // No updates or cargo-outdated not installed
    }

    let json_str = String::from_utf8_lossy(&output.stdout);

    // Try to parse as JSON array
    if let Ok(updates) = serde_json::from_str::<Vec<CrateUpdate>>(&json_str) {
        Ok(updates)
    } else {
        // Try to parse from toml-like output
        let mut results = Vec::new();
        for line in json_str.lines() {
            let line = line.trim();
            if line.is_empty() || !line.contains('=') {
                continue;
            }
            // Parse lines like "sha2  0.10  0.11  0.11  Normal"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[0] != "Name" {
                results.push(CrateUpdate {
                    name: parts[0].to_string(),
                    current: parts[1].to_string(),
                    wanted: parts[2].to_string(),
                    latest: parts[3].to_string(),
                    kind: parts[4].to_string(),
                });
            }
        }
        Ok(results)
    }
}

/// Generate TOML diff for dependency upgrade
pub fn generate_dep_upgrade_text(
    class: &UpgradeClass,
    crate_name: &str,
    old_version: &str,
    new_version: &str,
    cargo_toml_path: &str,
) -> String {
    match class {
        UpgradeClass::VersionBump | UpgradeClass::PatchVersion => {
            format!(
                "--- a/{}\n+++ b/{}\n@@ -1,1 +1,1 @@\n-{} = \"{}\"\n+{} = \"{}\"",
                cargo_toml_path, cargo_toml_path, crate_name, old_version, crate_name, new_version
            )
        }
        UpgradeClass::NewCrate => {
            format!(
                "--- a/{}\n+++ b/{}\n@@ -10,0 +10,1 @@\n+{} = \"{}\"",
                cargo_toml_path, cargo_toml_path, crate_name, new_version
            )
        }
        UpgradeClass::RemoveCrate => {
            format!(
                "--- a/{}\n+++ b/{}\n@@ -10,1 +10,0 @@\n-{} = \"{}\"",
                cargo_toml_path, cargo_toml_path, crate_name, old_version
            )
        }
        UpgradeClass::Toolchain => {
            // For toolchain, we'd modify rust-toolchain.toml
            format!("# Update toolchain from {} to {}", old_version, new_version)
        }
    }
}

/// Check if a dependency upgrade is admissible
pub fn is_upgrade_admissible(
    candidate: &DependencyUpgradeCandidate,
    report: &DependencyUpgradeReport,
    base_complexity: u128,
) -> (bool, i128) {
    // Genesis margin based on update characteristics
    let complexity_cost = report.update_size_kb as i128 * 10;
    let audit_cost = 20 + (candidate.upgrade_class.risk_level() as i128 * 5);
    let defect_budget = 100 + (candidate.wildness * 20.0) as i128;

    let genesis_margin = base_complexity as i128 + defect_budget - complexity_cost - audit_cost;

    // Coherence margin based on breaking change risk
    let risk_tolerance: i128 = 200;
    let breaking_risk = if report.breaking_change { 150 } else { 20 };
    let integration_cost = 10; // Base integration cost

    let coherence_margin = risk_tolerance + defect_budget - breaking_risk - integration_cost;

    // Must pass verification and have positive margins
    let upgrade_accept = genesis_margin >= 0
        && report.cargo_update_dry_run_pass
        && report.cargo_check_pass
        && coherence_margin >= 0;

    let combined_margin = genesis_margin + coherence_margin;

    (upgrade_accept, combined_margin)
}

/// Compute upgrade metrics for a dependency candidate
pub fn compute_upgrade_metrics(candidate: &DependencyUpgradeCandidate) -> (u128, u128, u128) {
    let update_size = candidate.upgrade_text.len() as u128 / 1024 + 1;
    let audit_cost = 20 + (candidate.upgrade_class.risk_level() as u128 * 5);
    let defect = 100 + (candidate.wildness * 20.0) as u128;

    (update_size, audit_cost, defect)
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

        let (m_after, cost, _defect) = compute_genesis_metrics(&candidate);
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
