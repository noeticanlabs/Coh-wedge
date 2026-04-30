use serde::{Deserialize, Serialize};

/// Layer of the NPE pipeline where the failure occurred
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureLayer {
    /// Failure at Coh pre-gate (before any engine runs)
    CohPre,
    /// Failure in Rust syntax or parse
    RustSyntax,
    /// Failure in Rust compilation (rustc)
    RustCompile,
    /// Failure in Rust execution/tests
    RustTest,
    /// Failure in Lean syntax parsing
    LeanSyntax,
    /// Failure in Lean type elaboration
    LeanElaboration,
    /// Failure in Lean logical proof state
    LeanProof,
    /// Failure in Mathlib advisor/search
    MathlibAdvisor,
    /// Pure mathematical/structural failure
    Mathematical,
    /// Analytical/PDE/Estimate failure
    Analytical,
    /// Failure at Coh post-gate (admission policy)
    CohPost,
}

/// Specific Rust coding failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RustFailure {
    ParseError,
    MissingSemicolon,
    UnclosedDelimiter,
    UnknownIdentifier(String),
    UnresolvedImport(String),
    TypeMismatch,
    TraitBoundUnsatisfied(String),
    BorrowAfterMove,
    MutableBorrowConflict,
    LifetimeError,
    PatternNonExhaustive,
    UnusedImportWarning,
    DeadCodeWarning,
    TestFailure(String),
    ClippyFailure,
    SerializationBreak,
    PublicApiBreak,
}

/// Lean syntax-level failures (the code doesn't parse)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanSyntaxFailure {
    ParseError,
    InvalidBinderSyntax,
    InvalidExistsSyntax,
    MissingBy,
    MissingComma,
    InvalidTacticBlock,
    UnexpectedToken(String),
    UnknownCommand(String),
}

/// Lean elaboration failures (the code parses but doesn't type-check)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanElabFailure {
    UnknownIdentifier(String),
    UnknownField(String),
    TypeMismatch,
    ApplicationTypeMismatch,
    FailedToSynthesizeInstance(String),
    AmbiguousCoercion,
    CannotInferImplicit,
    InvalidProjection,
    UniverseMismatch,
}

/// Lean proof-state failures (logical gaps)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeanProofFailure {
    UnsolvedGoals,
    GoalShapeMismatch,
    TacticFailed(String),
    // Domain-specific hints for the loop to learn from
    NeedLowerBoundHalf,
    NeedGreatestLowerBoundHalf,
    NeedApproximationLemma,
    NeedOrderContradiction,
    NeedWitnessConstruction,
    NeedSetExtensionality,
    NeedRewrite,
}

/// Pure mathematical failures (structural/logic)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MathematicalFailure {
    DefinitionIncomplete,
    DomainCodomainMismatch,
    MissingHypothesis,
    HypothesisTooWeak,
    HypothesisTooStrong,
    CircularDependency,
    VacuousTheorem,
    TheoremStatementWeakened,
    NonConstructiveGap,
    InvalidEquivalence,
    InvalidCancellation,
    InvalidOrderInference,
    MissingMonotonicity,
    MissingAssociativity,
    MissingIdentityLaw,
    MissingClosureLaw,
    MissingNonemptiness,
    MissingBoundedness,
    MissingSupremumExistence,
    MissingInfimumExistence,
    WrongCodomain,
    InvalidFiniteToInfiniteTransfer,
    FalseConverse,
    ProjectionGap,
    OplaxLawMissing,
    ResourceChannelCollapse,
}

/// Pure analytical failures (PDE/Estimates/Limits)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalyticalFailure {
    EstimateMissing,
    EstimateDirectionWrong,
    ConstantUncontrolled,
    NormMismatch,
    SpaceMismatch,
    RegularityInsufficient,
    CompactnessMissing,
    ConvergenceNotJustified,
    LimitInterchangeInvalid,
    SumIntegralSwapInvalid,
    DifferentiationUnderIntegralInvalid,
    BoundaryConditionMissing,
    DimensionThresholdWrong,
    ScalingMismatch,
    ContinuationCriterionMissing,
    PDEAssumptionMissing,
    WeakToStrongUpgradeInvalid,
    DiscreteToContinuousGap,
    ApproximationNotUniform,
    SupNormBoundMissing,
    BernsteinInequalityMissing,
    BKMCriterionMisused,
    FluxControlMissing,
    GradientSpendMismatch,
    UnboundedDefectBudget,
    VanishingSpendCoefficient,
    // Sub-domain domain-specific failures
    DyadicBKM(DyadicBKMFailure),
    SpectralFlux(SpectralFluxFailure),
    ProbabilityLimit(ProbabilityLimitFailure),
    MetricAnalytic(MetricAnalyticFailure),
}

/// Domain-specific failures for the Dyadic/BKM bridge
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DyadicBKMFailure {
    BernsteinAxiomMissing,
    LPReconstructionMissing,
    DyadicDecayTooWeak,
    GeometricSeriesDiverges,
    BKMQuantityNotControlled,
    VorticityGradientMismatch,
    TimeIntegrabilityMissing,
}

/// Domain-specific failures for Spectral Flux
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpectralFluxFailure {
    GradientSpendNotCharged,
    DefectRateUnbounded,
    FluxControlEstimateMissing,
    FluxControlConstantUnknown,
    IntegratedBoundOnlyButPointwiseClaimed,
}

/// Domain-specific failures for Probability Limits
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProbabilityLimitFailure {
    KernelNotNormalized,
    PositivityFailure,
    DiffusiveScalingMissing,
    GeneratorExpansionInvalid,
    RemainderNotVanishing,
    TightnessMissing,
}

/// Domain-specific failures for Metric Analytic/V3
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricAnalyticFailure {
    InfimumOverEmptySet,
    IdentityTightnessMissing,
    TriangleProofMissingEpsilon,
    InfiniteDistanceCaseIgnored,
    GeodesicExistenceOverclaimed,
}

/// Coh governance and admission policy failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceFailure {
    SorryIntroduced,
    AdmitIntroduced,
    UnauthorizedAxiomIntroduced,
    TheoremStatementChanged,
    DefinitionWeakened,
    ForbiddenImport,
    TooManyImports,
    ProofCostTooHigh,
    EpistemicViolation,
}

/// Mathlib advisor failures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MathlibFailure {
    LemmaNotFound(String),
    LemmaNameMismatch,
    NamespaceMismatch,
    SuggestedLemmaWrongShape,
}

/// Severity of the failure
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureSeverity {
    /// Fatal error, claim is false or theorem was weakened
    HardInvalid,
    /// Theorem may be true with added hypothesis
    AssumptionMissing,
    /// Claim plausible but proof incomplete
    ProofGap,
    /// External theorem required
    AxiomNeeded,
    /// Not proven, should be labeled
    Conjectural,
    /// Cannot publish as theorem
    PublicationBlocker,
    /// Wording/clarity issue
    Advisory,
    /// Generic repairable tweak
    Repairable,
    /// Almost worked
    UsefulNearMiss,
}

/// Math/Analytic specific layers for specialized reporting
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MathAnalyticLayer {
    PureMath,
    OrderTheory,
    CategoryTheory,
    MetricGeometry,
    FunctionalAnalysis,
    PDE,
    ProbabilityLimit,
    NumericalAnalysis,
}

/// Suggested repair strategies
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepairStrategy {
    SyntaxRepair,
    FieldNameRepair,
    CoercionRepair,
    TypeAnnotationRepair,
    ImportMinimization,
    MathlibLemmaSearch,
    HelperLemmaCreation,
    // New Math/Analytic strategies
    AddMissingHypothesis,
    WeakenConclusion,
    SplitLemma,
    MarkAsAxiom,
    MarkAsConjecture,
    ProvideCounterexample,
    AddBoundednessAssumption,
    AddNonemptinessAssumption,
    AddContinuityAssumption,
    AddCompactnessAssumption,
    AddIntegrabilityAssumption,
    ReplaceEqualityWithInequality,
    ReplaceGlobalClaimWithConditionalClaim,
    MoveToPDEAssumptionLayer,
    MoveToProofDebtLedger,
}

/// Combined failure kind
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureKind {
    Rust(RustFailure),
    LeanSyntax(LeanSyntaxFailure),
    LeanElab(LeanElabFailure),
    LeanProof(LeanProofFailure),
    Mathlib(MathlibFailure),
    Mathematical(MathematicalFailure),
    Analytical(AnalyticalFailure),
    Governance(GovernanceFailure),
    Other(String),
}

/// Unified Failure Report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailureReport {
    pub candidate_id: String,
    pub target: String,
    pub layer: FailureLayer,
    pub kind: FailureKind,
    pub raw_error: String,
    pub normalized_message: String,
    pub retryable: bool,
    pub severity: FailureSeverity,
    pub suggested_repairs: Vec<RepairStrategy>,
    pub blocks_publication: bool,
}

impl FailureSeverity {
    pub fn reward_signal(&self) -> f64 {
        match self {
            FailureSeverity::HardInvalid => -1.5,
            FailureSeverity::PublicationBlocker => -1.0,
            FailureSeverity::AssumptionMissing => -0.5,
            FailureSeverity::AxiomNeeded => -0.2,
            FailureSeverity::ProofGap => 0.1,
            FailureSeverity::Conjectural => 0.0,
            FailureSeverity::Advisory => 0.05,
            FailureSeverity::Repairable => -0.1,
            FailureSeverity::UsefulNearMiss => 0.3,
        }
    }
}
