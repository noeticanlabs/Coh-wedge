use coh_npe::failure_taxonomy::{
    AnalyticalFailure, DyadicBKMFailure, FailureKind, FailureLayer, FailureReport, FailureSeverity,
    MathematicalFailure, MetricAnalyticFailure, ProbabilityLimitFailure, RepairStrategy,
    SpectralFluxFailure,
};

/// High-level function to classify mathematical or analytical gaps in a proof attempt.
/// 
/// In a real system, this would involve LLM-based analysis of the proof state,
/// but here we provide a structural classifier based on keywords and target theorems.
pub fn classify_math_analytic_gap(
    candidate_id: &str,
    target: &str,
    proof_text: &str,
    raw_error: &str,
) -> Option<FailureReport> {
    // 1. Identify Domain
    if target.contains("dyadic_bkm_bridge") || target.contains("BKM") {
        return Some(classify_dyadic_bkm(candidate_id, target, proof_text, raw_error));
    }

    if target.contains("spectral_flux") || target.contains("Pi_bound") {
        return Some(classify_spectral_flux(candidate_id, target, proof_text, raw_error));
    }

    if target.contains("prob_law") || target.contains("diffusion_limit") {
        return Some(classify_prob_limit(candidate_id, target, proof_text, raw_error));
    }

    if target.contains("V3") || target.contains("metric") || target.contains("distance") {
        return Some(classify_metric_analytic(candidate_id, target, proof_text, raw_error));
    }

    // 2. Fallback to Generic Pure Math
    if raw_error.contains("MissingHypothesis") || raw_error.contains("unsolved goals") {
        return Some(FailureReport {
            candidate_id: candidate_id.to_string(),
            target: target.to_string(),
            layer: FailureLayer::Mathematical,
            kind: FailureKind::Mathematical(MathematicalFailure::MissingHypothesis),
            raw_error: raw_error.to_string(),
            normalized_message: "Missing mathematical hypothesis or assumption".to_string(),
            retryable: true,
            severity: FailureSeverity::AssumptionMissing,
            suggested_repairs: vec![RepairStrategy::AddMissingHypothesis],
            blocks_publication: false,
        });
    }

    None
}

fn classify_dyadic_bkm(
    id: &str,
    target: &str,
    proof: &str,
    err: &str,
) -> FailureReport {
    let (kind, msg, repairs, severity, blocks) = if !proof.contains("Bernstein") && err.contains("unsolved goals") {
        (
            AnalyticalFailure::BernsteinInequalityMissing,
            "Bernstein inequality missing from analytic bridge",
            vec![RepairStrategy::AddMissingHypothesis, RepairStrategy::MoveToPDEAssumptionLayer],
            FailureSeverity::AxiomNeeded,
            false,
        )
    } else if proof.contains("global_regularity") && !proof.contains("conditional") {
        (
            AnalyticalFailure::BKMCriterionMisused,
            "Claimed unconditional global regularity without BKM continuation justification",
            vec![RepairStrategy::ReplaceGlobalClaimWithConditionalClaim],
            FailureSeverity::PublicationBlocker,
            true,
        )
    } else {
        (
            AnalyticalFailure::DyadicBKM(DyadicBKMFailure::DyadicDecayTooWeak),
            "Dyadic decay bound too weak for Bernstein summation",
            vec![RepairStrategy::WeakenConclusion],
            FailureSeverity::HardInvalid,
            false,
        )
    };

    FailureReport {
        candidate_id: id.to_string(),
        target: target.to_string(),
        layer: FailureLayer::Analytical,
        kind: FailureKind::Analytical(kind),
        raw_error: err.to_string(),
        normalized_message: msg.to_string(),
        retryable: true,
        severity,
        suggested_repairs: repairs,
        blocks_publication: blocks,
    }
}

fn classify_spectral_flux(
    id: &str,
    target: &str,
    #[allow(unused_variables)] proof: &str,
    err: &str,
) -> FailureReport {
    FailureReport {
        candidate_id: id.to_string(),
        target: target.to_string(),
        layer: FailureLayer::Analytical,
        kind: FailureKind::Analytical(AnalyticalFailure::SpectralFlux(SpectralFluxFailure::FluxControlEstimateMissing)),
        raw_error: err.to_string(),
        normalized_message: "Spectral flux control estimate (PDE input) is missing".to_string(),
        retryable: true,
        severity: FailureSeverity::AxiomNeeded,
        suggested_repairs: vec![RepairStrategy::MoveToPDEAssumptionLayer],
        blocks_publication: false,
    }
}

fn classify_prob_limit(
    id: &str,
    target: &str,
    #[allow(unused_variables)] proof: &str,
    err: &str,
) -> FailureReport {
    FailureReport {
        candidate_id: id.to_string(),
        target: target.to_string(),
        layer: FailureLayer::Analytical,
        kind: FailureKind::Analytical(AnalyticalFailure::ProbabilityLimit(ProbabilityLimitFailure::GeneratorExpansionInvalid)),
        raw_error: err.to_string(),
        normalized_message: "Invalid generator expansion in diffusive limit passage".to_string(),
        retryable: true,
        severity: FailureSeverity::HardInvalid,
        suggested_repairs: vec![RepairStrategy::ProvideCounterexample],
        blocks_publication: false,
    }
}

fn classify_metric_analytic(
    id: &str,
    target: &str,
    #[allow(unused_variables)] proof: &str,
    err: &str,
) -> FailureReport {
    FailureReport {
        candidate_id: id.to_string(),
        target: target.to_string(),
        layer: FailureLayer::Mathematical,
        kind: FailureKind::Analytical(AnalyticalFailure::MetricAnalytic(MetricAnalyticFailure::GeodesicExistenceOverclaimed)),
        raw_error: err.to_string(),
        normalized_message: "Claimed geodesic existence without compactness or finite-domain assumption".to_string(),
        retryable: true,
        severity: FailureSeverity::PublicationBlocker,
        suggested_repairs: vec![RepairStrategy::AddCompactnessAssumption],
        blocks_publication: true,
    }
}
