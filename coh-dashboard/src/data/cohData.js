const DEFAULT_COH_VERSION = '0.1.0';

export const DEFAULT_SIDECAR_BASE_URL =
    import.meta.env.VITE_COH_SIDECAR_URL ?? 'http://127.0.0.1:3030';

const SCENARIOS = {
    valid: {
        key: 'valid',
        label: 'Financial: Valid Path',
        domain: 'financial',
        description: 'Happy-path invoice processing with full safety margin.',
        chainPath: '/demo/ai_workflow_chain_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    ops_risky: {
        key: 'ops_risky',
        label: 'Ops: Stall Risk',
        domain: 'ops',
        description: 'Resource bottleneck detected. Safety first selection required.',
        chainPath: '/demo/ai_workflow_chain_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    agent_review: {
        key: 'agent_review',
        label: 'Agent: Policy Setback',
        domain: 'agent',
        description: 'Policy review triggered. Alignment setback during safety sweep.',
        chainPath: '/demo/ai_workflow_chain_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_policy_violation: {
        key: 'reject_policy_violation',
        label: 'Policy Violation (T5)',
        domain: 'financial',
        description: 'Accounting law v_post + spend <= v_pre + defect was violated.',
        chainPath: '/demo/reject_policy_violation.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    invalid_state_link: {
        key: 'invalid_state_link',
        label: 'Broken State Link (T1)',
        domain: 'financial',
        description: 'Receipt linkage fails on state continuity at a deterministic break point.',
        chainPath: '/demo/ai_workflow_chain_invalid_state_link.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
};

export const SCENARIO_OPTIONS = Object.values(SCENARIOS).map(({ key, label, description }) => ({
    key,
    label,
    description,
}));

function parseJsonLines(text) {
    return text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)
        .map((line) => JSON.parse(line));
}

function toBigIntSafe(value) {
    try {
        return BigInt(value);
    } catch {
        return null;
    }
}

function derivePolicyCheck(metrics) {
    const vPre = toBigIntSafe(metrics.vPre ?? metrics.v_pre);
    const vPost = toBigIntSafe(metrics.vPost ?? metrics.v_post);
    const spend = toBigIntSafe(metrics.spend);
    const defect = toBigIntSafe(metrics.defect);

    if ([vPre, vPost, spend, defect].some((value) => value === null)) {
        return { isValid: false, margin: 0n };
    }

    const leftSide = vPost + spend;
    const rightSide = vPre + defect;
    return {
        isValid: leftSide <= rightSide,
        margin: rightSide - leftSide,
    };
}

function deriveChainBreak(receipts) {
    for (let index = 1; index < receipts.length; index += 1) {
        const previous = receipts[index - 1];
        const current = receipts[index];
        if (current.step_index !== previous.step_index + 1) return { breakIndex: index, type: 'index' };
        if (current.state_hash_prev !== previous.state_hash_next) return { breakIndex: index, type: 'state' };
        if (current.chain_digest_prev !== previous.chain_digest_next) return { breakIndex: index, type: 'digest' };
    }
    return null;
}

/**
 * Graded Safety Margin calculation per Domain State
 */
function deriveSafetyMargin(metrics, domain = 'financial') {
    const vPre = Number(metrics.vPre ?? metrics.v_pre ?? 0);
    const vPost = Number(metrics.vPost ?? metrics.v_post ?? 0);
    const spend = Number(metrics.spend ?? 0);
    const defect = Number(metrics.defect ?? 0);

    if (domain === 'ops') {
        const stallRisk = 0.2; // Simulated property
        return Math.max(0, 1.0 - stallRisk);
    }
    if (domain === 'agent') {
        return vPost > 50 ? 0.9 : 0.4; // Simulated graded safety
    }

    const margin = vPre + defect - (vPost + spend);
    return vPre === 0 ? 1.0 : Math.max(0, Math.min(margin / vPre, 1.0));
}

function deriveAlignmentIndex(metrics, domain = 'financial', stepIndex = 0) {
    const progress = Math.min(stepIndex / 5, 1.0);
    if (domain === 'agent' && stepIndex === 2) return 0.2;
    return progress;
}

function normalizeStep(receipt, index, receipts, breakInfo) {
    const policy = derivePolicyCheck(receipt.metrics);
    return {
        id: `step-${receipt.step_index}`,
        index,
        stepIndex: receipt.step_index,
        objectId: receipt.object_id,
        stateHashPrev: receipt.state_hash_prev,
        stateHashNext: receipt.state_hash_next,
        status: (breakInfo && index >= breakInfo.breakIndex) ? 'TAMPERED' : 'TRUSTED',
        metrics: {
            vPre: receipt.metrics.v_pre,
            vPost: receipt.metrics.v_post,
            spend: receipt.metrics.spend,
            defect: receipt.metrics.defect,
            isAdmissible: policy.isValid,
        },
        raw: receipt,
    };
}

class FixtureSource {
    async loadScenario(scenario) {
        const [chainText, slab] = await Promise.all([
            fetch(scenario.chainPath).then((response) => response.text()),
            fetch(scenario.slabPath).then((response) => response.json()),
        ]);
        const receipts = parseJsonLines(chainText);
        const breakInfo = deriveChainBreak(receipts);
        const chainSteps = receipts.map((r, i) => normalizeStep(r, i, receipts, breakInfo));
        return { scenario, receipts, chainSteps, breakInfo, slab: { raw: slab } };
    }
}

class SidecarSource {
    constructor(baseUrl) { this.baseUrl = baseUrl; }
    // ... Simplified sidecar wrapper ...
}

export async function loadDashboardData({ scenarioKey = 'valid' } = {}) {
    const scenario = SCENARIOS[scenarioKey] ?? SCENARIOS.valid;
    const fixtureSource = new FixtureSource();
    return fixtureSource.loadScenario(scenario);
}

// =============================================================================
// HARDENED TRAJECTORY ENGINE (SIDECAR BOUND)
// =============================================================================

const SIDECAR_URL = DEFAULT_SIDECAR_BASE_URL;
const COH_PRECISION = 1000000000;

export async function generateCandidatesImpl(initialReceipt, options = {}) {
    const { maxDepth = 4, beamWidth = 3, domain = 'financial' } = options;
    if (!initialReceipt) return [];

    // Map initialReceipt back to DomainState
    const initial_state = {
        domain: domain.charAt(0).toUpperCase() + domain.slice(1),
        data: domain === 'financial' ? {
            balance: Number(initialReceipt.metrics.v_post),
            initial_balance: 1000 * COH_PRECISION,
            status: 'Idle',
            current_invoice_amount: 0
        } : domain === 'ops' ? {
            status: 'Open',
            materials_logged: false,
            stall_risk: Math.round(0.1 * COH_PRECISION),
            resource_readiness: Math.round(0.9 * COH_PRECISION)
        } : {
            complexity_index: 0,
            complexity_budget: 100,
            authority_level: 1,
            status: 'Observing'
        }
    };

    const target_state = { ...initial_state }; // Simplified target for demo

    const body = {
        context: {
            initial_state,
            target_state,
            max_depth: maxDepth,
            beam_width: beamWidth,
            weights: {
                goal: COH_PRECISION,
                risk: COH_PRECISION,
                cost: COH_PRECISION / 10
            }
        }
    };

    try {
        const response = await fetch(`${SIDECAR_URL}/trajectory/search`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });

        const unified = await response.json();
        if (unified.status !== 'ACCEPT' || !unified.data) {
            console.error('[Engine] Search failed:', unified.error);
            return [];
        }

        const engineResult = unified.data;
        
        // Map Admissible paths
        const admissible = (engineResult.admissible || []).map((tau, idx) => ({
            id: `traj-a-${idx}`,
            isSelectable: true,
            pathStatus: 'ADMISSIBLE',
            evaluation: {
                safetyBottleneck: tau.evaluation.safety_bottleneck / COH_PRECISION,
                alignment: tau.evaluation.progress / COH_PRECISION,
                normalizedCost: tau.evaluation.normalized_cost / COH_PRECISION
            },
            receipts: tau.steps.map(s => ({
                step_index: 0, // Placeholder
                object_id: 'traj.step',
                metrics: {
                    v_pre: s.state_prev.data?.balance || 0,
                    v_post: s.state_next.data?.balance || 0
                }
            })),
            witnesses: tau.steps.map(_ => ({
                c1: { status: 'pass' }, c2: { status: 'pass' }, c3: { status: 'pass' },
                c4: { status: 'pass' }, c5: { status: 'pass' }, c6: { status: 'pass' }
            }))
        }));

        // Map Rejected paths
        const rejected = (engineResult.rejected || []).map((edge, idx) => ({
            id: `traj-r-${idx}`,
            isSelectable: false,
            pathStatus: 'ILLEGAL',
            evaluation: {
                safetyBottleneck: 0,
                alignment: 0,
                normalizedCost: 1.0
            },
            receipts: Array(edge.receipt.step_index).fill({}).concat([edge.receipt]),
            firstFailureIndex: edge.receipt.step_index,
            violationDelta: edge.verification.violation_delta,
            rejectCode: edge.verification.code,
            witnesses: edge.witnesses.map(w => ({
                id: w[0],
                status: w[1].toLowerCase()
            }))
        }));

        return [...admissible, ...rejected];
    } catch (err) {
        console.error('[Engine] Sidecar connection failed:', err);
        return [];
    }
}
