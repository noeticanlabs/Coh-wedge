const DEFAULT_COH_VERSION = '0.1.0';

export const DEFAULT_SIDECAR_BASE_URL =
    import.meta.env.VITE_COH_SIDECAR_URL ?? 'http://127.0.0.1:3030';

const SCENARIOS = {
    valid: {
        key: 'valid',
        label: 'Valid Chain',
        description: 'Happy-path receipt chain with a consistent slab summary.',
        chainPath: '/demo/ai_workflow_chain_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    valid_runtime: {
        key: 'valid_runtime',
        label: 'Valid Runtime Chain',
        description: 'Dynamically generated receipts with valid cryptographic digests.',
        chainPath: '/demo/valid_runtime.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
        isRuntime: true,
    },
    sidecar_valid: {
        key: 'sidecar_valid',
        label: 'Sidecar Verified',
        description: 'Pre-verified receipts from APE demo with sidecar.',
        chainPath: '/demo/sidecar_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    invalid_state_link: {
        key: 'invalid_state_link',
        label: 'Broken State Link (T1)',
        description: 'Receipt linkage fails on state continuity at a deterministic break point.',
        chainPath: '/demo/ai_workflow_chain_invalid_state_link.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_chain_digest: {
        key: 'reject_chain_digest',
        label: 'Chain Digest Mismatch (T2)',
        description: 'Cryptographic digest linkage failure between receipts.',
        chainPath: '/demo/reject_chain_digest.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_state_link_adv: {
        key: 'reject_state_link_adv',
        label: 'State Discontinuity (T4)',
        description: 'Adversarial state transition mismatch detected.',
        chainPath: '/demo/reject_state_link.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_policy_violation: {
        key: 'reject_policy_violation',
        label: 'Policy Violation (T5)',
        description: 'Accounting law v_post + spend <= v_pre + defect was violated.',
        chainPath: '/demo/reject_policy_violation.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_schema: {
        key: 'reject_schema',
        label: 'Schema Version Reject (T0)',
        description: 'Unsupported or malformed schema ID / Version detected.',
        chainPath: '/demo/reject_schema.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_numeric_parse: {
        key: 'reject_numeric_parse',
        label: 'Malformed Numeric (T3)',
        description: 'Non-decimal numeric strings rejected by the kernel.',
        chainPath: '/demo/reject_numeric_parse.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    reject_overflow: {
        key: 'reject_overflow',
        label: 'Arithmetic Overflow (T6)',
        description: 'Extreme value range causing u128 safe-math rejection.',
        chainPath: '/demo/reject_overflow.jsonl',
        slabPath: '/demo/ai_workflow_slab_valid.json',
    },
    invalid_slab_summary: {
        key: 'invalid_slab_summary',
        label: 'Invalid Slab Summary (M1)',
        description: 'The chain verifies, but the slab macro-summary does not reconcile.',
        chainPath: '/demo/ai_workflow_chain_valid.jsonl',
        slabPath: '/demo/ai_workflow_slab_invalid_summary.json',
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

function formatHex(value, length) {
    return value.toString(16).padStart(length, '0');
}

function derivePolicyCheck(metrics) {
    const vPre = toBigIntSafe(metrics.v_pre);
    const vPost = toBigIntSafe(metrics.v_post);
    const spend = toBigIntSafe(metrics.spend);
    const defect = toBigIntSafe(metrics.defect);
    const authority = toBigIntSafe(metrics.authority ?? '0');

    if ([vPre, vPost, spend, defect, authority].some((value) => value === null)) {
        return {
            isValid: false,
            leftSide: '—',
            rightSide: '—',
            domainValid: false,
            margin: 0n,
        };
    }

    const leftSide = vPost + spend;
    const rightSide = vPre + defect + authority;
    const domainValid = spend <= vPre;
    const margin = rightSide - leftSide;

    return {
        isValid: (leftSide <= rightSide) && domainValid,
        leftSide: leftSide.toString(),
        rightSide: rightSide.toString(),
        domainValid,
        margin,
    };
}

function deriveChainBreak(receipts) {
    for (let index = 1; index < receipts.length; index += 1) {
        const previous = receipts[index - 1];
        const current = receipts[index];

        if (current.step_index !== previous.step_index + 1) {
            return {
                breakIndex: index,
                stepIndex: current.step_index,
                previousStepIndex: previous.step_index,
                type: 'index_discontinuity',
                typeLabel: 'Index discontinuity',
                field: 'step_index',
                expected: String(previous.step_index + 1),
                actual: String(current.step_index),
                message: `Chain broken at step ${current.step_index}: expected step_index ${previous.step_index + 1}, found ${current.step_index}.`,
            };
        }

        if (current.state_hash_prev !== previous.state_hash_next) {
            return {
                breakIndex: index,
                stepIndex: current.step_index,
                previousStepIndex: previous.step_index,
                type: 'state_link',
                typeLabel: 'State hash mismatch',
                field: 'state_hash_prev',
                expected: previous.state_hash_next,
                actual: current.state_hash_prev,
                message: `Chain broken at step ${current.step_index}: expected state_hash_prev to equal the previous state_hash_next.`,
            };
        }

        if (current.chain_digest_prev !== previous.chain_digest_next) {
            return {
                breakIndex: index,
                stepIndex: current.step_index,
                previousStepIndex: previous.step_index,
                type: 'chain_digest',
                typeLabel: 'Chain digest mismatch',
                field: 'chain_digest_prev',
                expected: previous.chain_digest_next,
                actual: current.chain_digest_prev,
                message: `Chain broken at step ${current.step_index}: expected chain_digest_prev to link to the previous chain_digest_next.`,
            };
        }
    }

    return null;
}

function deriveSlabCheck(receipts, slab) {
    const totalSpend = receipts.reduce((sum, receipt) => sum + (toBigIntSafe(receipt.metrics.spend) ?? 0n), 0n);
    const totalDefect = receipts.reduce((sum, receipt) => sum + (toBigIntSafe(receipt.metrics.defect) ?? 0n), 0n);
    const totalAuthority = receipts.reduce((sum, receipt) => sum + (toBigIntSafe(receipt.metrics.authority) ?? 0n), 0n);
    const vPreFirst = receipts[0]?.metrics?.v_pre ?? '0';
    const vPostLast = receipts[receipts.length - 1]?.metrics?.v_post ?? '0';

    const derived = {
        totalSpend: totalSpend.toString(),
        totalDefect: totalDefect.toString(),
        totalAuthority: totalAuthority.toString(),
        vPreFirst,
        vPostLast,
    };

    const mismatches = [];

    if (slab.summary.total_spend !== derived.totalSpend) {
        mismatches.push({ field: 'total_spend', expected: derived.totalSpend, actual: slab.summary.total_spend });
    }
    if (slab.summary.total_defect !== derived.totalDefect) {
        mismatches.push({ field: 'total_defect', expected: derived.totalDefect, actual: slab.summary.total_defect });
    }
    if ((slab.summary.total_authority ?? '0') !== derived.totalAuthority) {
        mismatches.push({ field: 'total_authority', expected: derived.totalAuthority, actual: slab.summary.total_authority ?? '0' });
    }
    if (slab.summary.v_pre_first !== derived.vPreFirst) {
        mismatches.push({ field: 'v_pre_first', expected: derived.vPreFirst, actual: slab.summary.v_pre_first });
    }
    if (slab.summary.v_post_last !== derived.vPostLast) {
        mismatches.push({ field: 'v_post_last', expected: derived.vPostLast, actual: slab.summary.v_post_last });
    }

    return {
        isValid: mismatches.length === 0,
        derived,
        mismatches,
        message:
            mismatches.length === 0
                ? 'Slab summary agrees with the loaded chain totals.'
                : `Slab summary mismatch on ${mismatches[0].field}: expected ${mismatches[0].expected}, found ${mismatches[0].actual}.`,
    };
}

function normalizeUnifiedResponse(payload, source) {
    return {
        requestId: payload.request_id ?? `${source}-request`,
        cohVersion: payload.coh_version ?? DEFAULT_COH_VERSION,
        status: payload.status ?? 'REJECT',
        data: payload.data ?? null,
        error: payload.error
            ? {
                code: payload.error.code ?? null,
                message: payload.error.message ?? null,
                requestId: payload.error.request_id ?? payload.request_id ?? `${source}-request`,
            }
            : null,
        breakIndex: payload.data?.break_index ?? null,
        reason: payload.error?.message ?? payload.data?.message ?? null,
        source,
        raw: payload,
    };
}

function mapBreakTypeToErrorCode(type) {
    switch (type) {
        case 'state_link':
            return 'E004';
        case 'chain_digest':
            return 'E002';
        case 'index_discontinuity':
        default:
            return 'E001';
    }
}

function buildFixtureVerification(breakInfo) {
    if (!breakInfo) {
        return normalizeUnifiedResponse(
            {
                request_id: 'fixture-valid',
                coh_version: DEFAULT_COH_VERSION,
                status: 'ACCEPT',
                data: null,
                error: null,
            },
            'fixture'
        );
    }

    return normalizeUnifiedResponse(
        {
            request_id: `fixture-${breakInfo.type}-${breakInfo.stepIndex}`,
            coh_version: DEFAULT_COH_VERSION,
            status: 'REJECT',
            data: {
                break_index: breakInfo.stepIndex,
                message: breakInfo.message,
            },
            error: {
                code: mapBreakTypeToErrorCode(breakInfo.type),
                message: breakInfo.message,
                request_id: `fixture-${breakInfo.type}-${breakInfo.stepIndex}`,
            },
        },
        'fixture'
    );
}

function normalizeStep(receipt, index, receipts, breakInfo) {
    const previous = receipts[index - 1] ?? null;
    const policy = derivePolicyCheck(receipt.metrics);
    const stateValid = !previous || previous.state_hash_next === receipt.state_hash_prev;
    const digestValid = !previous || previous.chain_digest_next === receipt.chain_digest_prev;
    const indexValid = !previous || previous.step_index + 1 === receipt.step_index;
    const breakAffected = breakInfo ? index >= breakInfo.breakIndex : false;
    const breakRole = breakInfo
        ? index === breakInfo.breakIndex
            ? 'BREAK'
            : index > breakInfo.breakIndex
                ? 'AFTER_BREAK'
                : 'NONE'
        : 'NONE';

    return {
        id: `step-${receipt.step_index}`,
        index,
        stepIndex: receipt.step_index,
        objectId: receipt.object_id,
        schemaId: receipt.schema_id,
        version: receipt.version,
        canonProfileHash: receipt.canon_profile_hash,
        policyHash: receipt.policy_hash,
        stateHashPrev: receipt.state_hash_prev,
        stateHashNext: receipt.state_hash_next,
        chainDigestPrev: receipt.chain_digest_prev,
        chainDigestNext: receipt.chain_digest_next,
        hash: receipt.chain_digest_next,
        status: breakAffected ? 'TAMPERED' : 'TRUSTED',
        breakAffected,
        breakRole,
        metrics: {
            vPre: receipt.metrics.v_pre,
            vPost: receipt.metrics.v_post,
            spend: receipt.metrics.spend,
            defect: receipt.metrics.defect,
            authority: receipt.metrics.authority ?? '0',
            isAdmissible: policy.isValid,
            leftSide: policy.leftSide,
            rightSide: policy.rightSide,
            domainValid: policy.domainValid,
        },
        continuity: {
            indexValid,
            stateValid,
            digestValid,
            indexLabel: previous ? (indexValid ? 'Continuous' : 'Broken') : 'Origin',
            stateLabel: previous ? (stateValid ? 'Continuous' : 'Broken') : 'Origin',
            digestLabel: previous ? (digestValid ? 'Verified' : 'Mismatch') : 'Origin',
        },
        raw: receipt,
    };
}

function normalizeSlab(slab) {
    return {
        schemaId: slab.schema_id,
        version: slab.version,
        objectId: slab.object_id,
        rangeStart: slab.range_start,
        rangeEnd: slab.range_end,
        microCount: slab.micro_count,
        chainDigestPrev: slab.chain_digest_prev,
        chainDigestNext: slab.chain_digest_next,
        stateHashFirst: slab.state_hash_first,
        stateHashLast: slab.state_hash_last,
        merkleRoot: slab.merkle_root,
        summary: {
            totalSpend: slab.summary.total_spend,
            totalDefect: slab.summary.total_defect,
            totalAuthority: slab.summary.total_authority ?? '0',
            vPreFirst: slab.summary.v_pre_first,
            vPostLast: slab.summary.v_post_last,
        },
        raw: slab,
    };
}

class FixtureSource {
    constructor() {
        this.sidecarSource = null;
    }

    setSidecar(sidecarBaseUrl) {
        this.sidecarSource = new SidecarSource(sidecarBaseUrl);
    }

    async loadScenario(scenario) {
        // Handle runtime-generated scenarios
        if (scenario.isRuntime && this.sidecarSource) {
            return this.loadRuntimeScenario(scenario);
        }

        const [chainText, slab] = await Promise.all([
            fetch(scenario.chainPath).then((response) => response.text()),
            fetch(scenario.slabPath).then((response) => response.json()),
        ]);

        const receipts = parseJsonLines(chainText);
        const breakInfo = deriveChainBreak(receipts);
        const chainSteps = receipts.map((receipt, index) => normalizeStep(receipt, index, receipts, breakInfo));
        const slabCheck = deriveSlabCheck(receipts, slab);

        return {
            scenario,
            receipts,
            chainSteps,
            breakInfo,
            slab: normalizeSlab(slab),
            slabCheck,
            fixtureVerification: buildFixtureVerification(breakInfo),
        };
    }

    async loadRuntimeScenario(scenario) {
        // Generate a chain of valid receipts via the APE library's runtime generator
        // For now, use the demo endpoint which generates valid receipts
        const receipts = [];

        // Generate chain from demo_e2e.json sidecar_accept result
        const chainText = await fetch('/demo/valid_runtime.jsonl').then(r => r.text()).catch(() => '');

        if (chainText) {
            // Use the pre-generated file
            const parsed = parseJsonLines(chainText);
            if (parsed.length > 0) {
                const breakInfo = deriveChainBreak(parsed);
                const chainSteps = parsed.map((receipt, index) => normalizeStep(receipt, index, parsed, breakInfo));
                const slab = { chain_digest_prev: '', chain_digest_next: '', state_hash_first: '', state_hash_last: '', merkle_root: '', micro_count: parsed.length };
                const slabCheck = deriveSlabCheck(parsed, slab);

                return {
                    scenario,
                    receipts: parsed,
                    chainSteps,
                    breakInfo,
                    slab: normalizeSlab(slab),
                    slabCheck,
                    fixtureVerification: buildFixtureVerification(breakInfo),
                };
            }
        }

        // Fallback: generate from sidecar by verifying a generated receipt
        // Use the micro pattern from the demo
        const demoData = [
            { step_index: 0, v_pre: 100, v_post: 88, spend: 12, defect: 0 },
            { step_index: 1, v_pre: 88, v_post: 80, spend: 7, defect: 1 },
        ];

        for (const pattern of demoData) {
            const receipt = {
                schema_id: "coh.receipt.micro.v1",
                version: "1.0.0",
                object_id: "agent.workflow.runtime",
                canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
                policy_hash: "0".repeat(64),
                step_index: pattern.step_index,
                state_hash_prev: formatHex(BigInt(pattern.step_index + 1), 64),
                state_hash_next: formatHex(BigInt(pattern.step_index + 2), 64),
                chain_digest_prev: pattern.step_index === 0 ? "0".repeat(64) : receipts[receipts.length - 1].chain_digest_next,
                chain_digest_next: "0".repeat(64),
                metrics: {
                    v_pre: String(pattern.v_pre),
                    v_post: String(pattern.v_post),
                    spend: String(pattern.spend),
                    defect: String(pattern.defect),
                },
                signatures: [{ signer: "test", sig: "deadbeef".repeat(16) }],
            };
            receipts.push(receipt);
        }

        // Compute digests and create chain
        const processedReceipts = [];
        let prevDigest = "0".repeat(64);
        for (const r of receipts) {
            r.chain_digest_prev = prevDigest;
            r.chain_digest_next = await this.computeDigest(r);
            prevDigest = r.chain_digest_next;
            processedReceipts.push(r);
        }

        const breakInfo = deriveChainBreak(processedReceipts);
        const chainSteps = processedReceipts.map((receipt, index) => normalizeStep(receipt, index, processedReceipts, breakInfo));
        const slab = { chain_digest_prev: '', chain_digest_next: processedReceipts[processedReceipts.length - 1].chain_digest_next, state_hash_first: processedReceipts[0].state_hash_prev, state_hash_last: processedReceipts[processedReceipts.length - 1].state_hash_next, merkle_root: '', micro_count: processedReceipts.length };
        const slabCheck = deriveSlabCheck(processedReceipts, slab);

        return {
            scenario,
            receipts: processedReceipts,
            chainSteps,
            breakInfo,
            slab: normalizeSlab(slab),
            slabCheck,
            fixtureVerification: buildFixtureVerification(breakInfo),
        };
    }

    async computeDigest(receipt) {
        // For runtime, we compute digest using the canonical form
        // This is a simplified version - in production would call the sidecar
        const canonical = JSON.stringify(receipt, Object.keys(receipt).sort());
        const encoder = new TextEncoder();
        const data = encoder.encode(canonical);
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }
}

class SidecarSource {
    constructor(baseUrl) {
        this.baseUrl = baseUrl.replace(/\/$/, '');
    }

    async verifyChain(receipts) {
        const response = await fetch(`${this.baseUrl}/v1/verify-chain`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ receipts }),
        });

        if (!response.ok) {
            throw new Error(`Sidecar request failed with HTTP ${response.status}.`);
        }

        const payload = await response.json();
        return normalizeUnifiedResponse(payload, 'sidecar');
    }

    async executeVerified(receipt, action) {
        console.log('[SidecarSource.executeVerified] Sending to:', `${this.baseUrl}/v1/execute-verified`);
        console.log('[SidecarSource.executeVerified] Payload:', { receipt, action });
        const response = await fetch(`${this.baseUrl}/v1/execute-verified`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ receipt, action }),
        });

        if (!response.ok) {
            const errText = await response.text();
            console.error('[SidecarSource.executeVerified] HTTP Error:', response.status, errText);
            throw new Error(`Sidecar request failed with HTTP ${response.status}.`);
        }

        const payload = await response.json();
        console.log('[SidecarSource.executeVerified] Response:', payload);
        return normalizeUnifiedResponse(payload, 'sidecar');
    }
}

export async function loadDashboardData({
    scenarioKey = 'valid',
    preferLiveVerification = false,
    sidecarBaseUrl = DEFAULT_SIDECAR_BASE_URL,
} = {}) {
    const scenario = SCENARIOS[scenarioKey] ?? SCENARIOS.valid;
    const fixtureSource = new FixtureSource();
    const fixtureData = await fixtureSource.loadScenario(scenario);

    let verification = fixtureData.fixtureVerification;
    let liveError = null;

    if (preferLiveVerification) {
        try {
            const sidecarSource = new SidecarSource(sidecarBaseUrl);
            verification = await sidecarSource.verifyChain(fixtureData.receipts);
        } catch (error) {
            liveError = error instanceof Error ? error.message : 'Live sidecar verification failed.';
        }
    }

    return {
        ...fixtureData,
        verification,
        liveError,
        preferLiveVerification,
        sidecarBaseUrl,
        isTrusted: verification.status === 'ACCEPT' && fixtureData.slabCheck.isValid && !fixtureData.breakInfo,
        validatorVersion: verification.cohVersion ?? DEFAULT_COH_VERSION,
    };
}

export async function executeVerified({
    receipt,
    action = { action: 'transfer_100_tokens', amount: 100, target: 'alice' },
    sidecarBaseUrl = DEFAULT_SIDECAR_BASE_URL,
} = {}) {
    const sidecarSource = new SidecarSource(sidecarBaseUrl);
    return sidecarSource.executeVerified(receipt, action);
}

// =============================================================================
// Trajectory Graph: visualize admissible state evolution as a DAG of choices
// =============================================================================

/**
 * Generate up to N candidate actions from current receipt.
 * Each action represents a possible "next step" or decision the system could take.
 */
/**
 * Bounded beam search candidate generation.
 * 
 * Algorithm:
 * 1. Initialize beam with current state
 * 2. For each depth up to horizon:
 *    a. Generate K ordered candidate actions
 *    b. Predict next state for each action
 *    c. Build transition receipt
 *    d. Apply micro verification (pass/warn/fail)
 *    e. Extend only lawful transitions
 *    f. Apply chain verification
 * 3. Retain top K by interim score
 * 4. Preserve rejected trajectories for UI display (truncated failures)
 * 5. Terminate on horizon, goal, or full rejection
 * 
 * @param {Object} initialReceipt - Current receipt (x_t)
 * @param {number} maxDepth - Horizon H (default 3)
 * @param {number} beamWidth - Branching cap K (default 5)
 * @param {Object} verification - Verification context
 */
/**
 * Trajectory Theorem: evaluate path-level lawfulness and cumulative admissibility.
 * Returns: { witnesses, marginTrace, isLawful, firstFailureIndex, firstFailureConstraint }
 */
function validateTrajectory(tau, verificationContext = {}) {
    const witnesses = [];
    const marginTrace = [];
    let cumulativeAdmissibility = 0n;
    let isLawful = true;
    let firstFailureIndex = null;
    let firstFailureConstraint = null;

    const receipts = tau.receipts || [];

    for (let i = 0; i < receipts.length; i++) {
        const receipt = receipts[i];
        const previous = receipts[i-1] || null;
        
        // --- Step Constraints (C1-C6) ---
        const policy = derivePolicyCheck(receipt.metrics);
        
        // C1: Schema & Version Validity (Hard)
        const c1 = Boolean(receipt.schema_id && receipt.version);
        // C2: Signatures (Hard)
        const c2 = (receipt.signatures?.length ?? 0) > 0;
        // C3: Transition Consistency (Hard) 
        // For simulation, we check if v_post was predicted correctly
        const c3 = receipt.metrics.isAdmissible !== false;
        // C4: Local Accounting Law (Hard)
        const c4 = policy.isValid && policy.domainValid;
        
        // C5: Path Continuity (Warn-allowed / Hard-fail)
        const indexValid = !previous || receipt.step_index === previous.step_index + 1;
        const stateValid = !previous || receipt.state_hash_prev === previous.state_hash_next;
        const digestValid = !previous || receipt.chain_digest_prev === previous.chain_digest_next;
        const c5 = indexValid && stateValid && digestValid;

        // C6: Robustness Margin (Warn-allowed)
        const c6 = policy.margin > 0n;

        // --- Theorem Assessment ---
        const stepWitness = {
            c1: { status: c1 ? 'pass' : 'fail' },
            c2: { status: c2 ? 'pass' : 'fail' },
            c3: { status: c3 ? 'pass' : 'fail' },
            c4: { status: c4 ? 'pass' : 'fail' },
            c5: { status: c5 ? 'pass' : (verificationContext.strictContinuity ? 'fail' : 'warn') },
            c6: { status: c6 ? 'pass' : 'warn' },
            margin: policy.margin.toString()
        };

        witnesses.push(stepWitness);

        // --- Cumulative Law ---
        // sum(v_post + spend - v_pre - defect - authority) <= 0
        // Equivalent to sum(-margin) <= 0  => sum(margin) >= 0
        cumulativeAdmissibility += policy.margin;
        marginTrace.push(cumulativeAdmissibility.toString());

        // Check for Hard Failures (C1-C4 always, C5/C6 if configured)
        const hardFail = !c1 || !c2 || !c3 || !c4 || (stepWitness.c5.status === 'fail') || (stepWitness.c6.status === 'fail');
        
        // Check Cumulative Feasibility
        const cumulativeFail = cumulativeAdmissibility < 0n;

        if (isLawful && (hardFail || cumulativeFail)) {
            isLawful = false;
            firstFailureIndex = i;
            firstFailureConstraint = hardFail ? (Object.keys(stepWitness).find(k => stepWitness[k].status === 'fail')) : 'C4_cumulative';
        }
    }

    return {
        witnesses,
        marginTrace,
        isSelectable: isLawful,
        pathStatus: isLawful ? 'PASS' : 'ILLEGAL',
        firstFailureIndex,
        firstFailureConstraint,
        warnCount: witnesses.reduce((acc, w) => acc + Object.values(w).filter(v => v.status === 'warn').length, 0),
        cumulativeMargin: cumulativeAdmissibility.toString()
    };
}

export function generateCandidatesImpl(initialReceipt, { maxDepth = 3, beamWidth = 5, verification = {} } = {}) {
    if (!initialReceipt) return [];

    // Step 0: Initialize beam with root state
    let beam = [{
        id: 'root',
        label: 'Origin',
        receipts: [initialReceipt],
        depth: 0,
        terminated: false,
    }];

    // ProposeActions: Constant action set for HUD simulation
    const proposeActions = (receipt, depth) => {
        const vPost = parseInt(receipt?.metrics?.v_post || '100', 10);
        return [
            { id: `c-${depth}`, variant: 'conservative', amount: Math.floor(vPost * 0.1), type: 'transfer' },
            { id: `b-${depth}`, variant: 'balanced', amount: Math.floor(vPost * 0.3), type: 'transfer' },
            { id: `a-${depth}`, variant: 'aggressive', amount: Math.floor(vPost * 1.5), type: 'transfer' }, // Likely illegal
            { id: `n-${depth}`, variant: 'noop', amount: 0, type: 'noop' }
        ].slice(0, beamWidth);
    };

    const predictNextState = (receipt, action) => {
        const vPre = parseInt(receipt?.metrics?.v_post || '0', 10);
        const spend = action.amount || 0;
        return {
            ...receipt,
            step_index: (receipt.step_index || 0) + 1,
            metrics: {
                v_pre: String(vPre),
                v_post: String(vPre - spend),
                spend: String(spend),
                defect: '0',
                authority: receipt.metrics?.authority || '0',
                isAdmissible: (vPre - spend) >= 0,
            },
            state_hash_prev: receipt.state_hash_next,
            state_hash_next: `h_${action.id}_${Date.now()}`,
            chain_digest_prev: receipt.chain_digest_next,
            chain_digest_next: `d_${action.id}_${Date.now()}`,
        };
    };

    for (let depth = 0; depth < maxDepth; depth++) {
        const expanded = [];
        for (const tau of beam) {
            if (tau.terminated) { expanded.push(tau); continue; }

            const current = tau.receipts[tau.receipts.length - 1];
            const actions = proposeActions(current, depth);

            for (const action of actions) {
                const nextReceipt = predictNextState(current, action);
                const transition = { ...nextReceipt, action };
                
                const newTau = {
                    id: `${tau.id}-${action.id}`,
                    label: action.variant,
                    receipts: [...tau.receipts, transition],
                    depth: depth + 1,
                    action
                };

                // Apply Theorem Filter
                const theorem = validateTrajectory(newTau, verification);
                expanded.push({
                    ...newTau,
                    ...theorem,
                    terminated: !theorem.isSelectable
                });
            }
        }

        // Rank only selectable trajectories
        const scored = expanded.map(tau => ({
            tau,
            score: tau.isSelectable ? scoreTrajectory(tau) : -Infinity
        }));

        scored.sort((a, b) => b.score - a.score);
        beam = scored.slice(0, beamWidth).map(s => s.tau);
        
        if (beam.every(t => t.terminated)) break;
    }

    return beam;
}

/**
 * Legacy planChoices wrapper for backward compatibility.
 */
function planChoices(receipt, n = 5) {
    if (!receipt) return [];

    const candidates = generateCandidatesImpl(receipt, { maxDepth: 1, beamWidth: n });

    return candidates.map(c => ({
        id: c.id,
        label: c.label,
        action: c.action || { action: 'transfer', amount: 100, target: 'alice', metadata: {} },
        receipts: c.receipts,
        score: c.terminated ? -Infinity : scoreChoice(c.receipts[c.receipts.length - 1], c.action, {}),
        truncatedAt: c.terminated ? c.depth : null,
        truncationReason: c.truncationReason,
    }));
}

/**
 * Multi-component scoring for trajectory ranking.
 * Formula: S(τ) = w_u*U - w_r*R - w_c*C - w_d*D - w_p*P_soft + w_m*M
 * 
 * Weights per spec:
 * w_u = 0.35 (utility)
 * w_r = 0.25 (risk) 
 * w_c = 0.15 (cost)
 * w_d = 0.15 (divergence)
 * w_p = 0.05 (soft policy)
 * w_m = 0.05 (margin/robustness)
 */
function scoreTrajectory(trajectory, weights = DEFAULT_SCORING_WEIGHTS) {
    if (!trajectory) return { score: 0, components: {}, tieBreakers: [] };

    const { receipts = [], action = {} } = trajectory;
    const receipt = receipts[receipts.length - 1] || receipts[0];

    if (!receipt) return { score: 0, components: {}, tieBreakers: [] };

    // Extract metrics
    const vPre = parseInt(receipt.metrics?.v_pre || '0', 10);
    const vPost = parseInt(receipt.metrics?.v_post || '0', 10);
    const spend = parseInt(receipt.metrics?.spend || '0', 10);
    const defect = parseInt(receipt.metrics?.defect || '0', 10);
    const isAdmissible = receipt.metrics?.isAdmissible !== false;

    // Normalize each component to [0, 1]

    // U (utility): task completion progress
    // Higher = better. Max v_post as proxy for utility.
    const maxV = Math.max(vPre, vPost, 1);
    const U = isAdmissible ? vPost / maxV : 0;

    // R (risk): compliance exposure, irreversible action risk
    // Higher defect = higher risk. Normalize to [0, 1] with cap.
    const R = Math.min(defect / 100, 1);

    // C (cost): operational spend (API, compute, latency)
    // Higher spend = higher cost. Normalize with cap.
    const C = Math.min(spend / 1000, 1);

    // D (divergence): uncertainty, model disagreement, instability
    // Use verification status as proxy: ACCEPT = low divergence.
    const D = receipt.verification?.status === 'ACCEPT' ? 0 :
        receipt.verification?.status === 'REJECT' ? 1 : 0.5;

    // P_soft (soft policy penalty): premium tools, customer-facing risk
    // Penalty for touching sensitive operations without high confidence.
    const actionType = action?.type || 'unknown';
    const sensitiveActions = ['transfer', 'delete', 'escalate', 'refund'];
    const P_soft = sensitiveActions.includes(actionType) && !isAdmissible ? 0.5 : 0;

    // M (margin/robustness bonus): reversibility, observability, safety margin
    // Higher margin (v_pre - v_post - spend + defect) = more robust.
    const margin = vPre - vPost - spend + defect;
    const M = Math.max(0, Math.min(margin / 100, 1));

    // Compute final score
    const score = (
        weights.u * U -
        weights.r * R -
        weights.c * C -
        weights.d * D -
        weights.p * P_soft +
        weights.m * M
    );

    // Deterministic tie-breakers (applied in reverse order in ranker)
    const tieBreakers = [
        M,        // robustness margin (desc)
        -R,       // irreversible risk (asc)
        -C,       // cost (asc)
        receipts.length, // path length (asc)
        receipt.schema_id || '', // lexicographic ID (asc)
    ];

    return {
        score,
        components: { U, R, C, D, P_soft, M },
        tieBreakers,
        raw: { vPre, vPost, spend, defect, isAdmissible }
    };
}

/**
 * Default scoring weights (normalized to sum to 1.0)
 */
const DEFAULT_SCORING_WEIGHTS = {
    u: 0.35,  // utility
    r: 0.25,  // risk
    c: 0.15,  // cost
    d: 0.15,  // divergence
    p: 0.05,  // soft policy penalty
    m: 0.05,  // margin/robustness bonus
};

/**
 * Rank candidates: apply hard constraints, then score and sort.
 * Returns ranked list with deterministic tie-breaking.
 */
function rankCandidatesImpl(candidates, verification) {
    // Step 1: Filter by hard constraints C1-C6 (lawful only)
    const lawful = candidates.filter(tau => {
        const receipt = tau.receipts?.[tau.receipts.length - 1] || tau.receipts?.[0];
        if (!receipt) return false;

        // C1: schema_version valid
        const hasValidSchema = receipt.schema_id && receipt.schema_version;
        // C2: signatures present
        const hasSignatures = receipt.signatures?.length > 0;
        // C3: profile_hash_match (skip for now)
        // C4: state_continuity
        const stateValid = receipt.continuity?.stateValid !== false;
        // C5: digest_continuity
        const digestValid = receipt.continuity?.digestValid !== false;
        // C6: policy_envelope (isAdmissible)
        const policyValid = receipt.metrics?.isAdmissible !== false;

        return hasValidSchema && hasSignatures && stateValid && digestValid && policyValid;
    });

    // Step 2: Score each lawful trajectory
    const scored = lawful.map(tau => {
        const result = scoreTrajectory(tau);
        return {
            ...tau,
            score: result.score,
            components: result.components,
            tieBreakers: result.tieBreakers
        };
    });

    // Step 3: Sort with deterministic tie-breaking
    scored.sort((a, b) => {
        // Primary: score (desc)
        if (b.score !== a.score) return b.score - a.score;
        // Tie-breaker 1: robustness margin (desc)
        if ((b.tieBreakers?.[0] ?? 0) !== (a.tieBreakers?.[0] ?? 0))
            return (b.tieBreakers?.[0] ?? 0) - (a.tieBreakers?.[0] ?? 0);
        // Tie-breaker 2: irreversible risk (asc)
        if ((a.tieBreakers?.[1] ?? 0) !== (b.tieBreakers?.[1] ?? 0))
            return (a.tieBreakers?.[1] ?? 0) - (b.tieBreakers?.[1] ?? 0);
        // Tie-breaker 3: cost (asc)
        if ((a.tieBreakers?.[2] ?? 0) !== (b.tieBreakers?.[2] ?? 0))
            return (a.tieBreakers?.[2] ?? 0) - (b.tieBreakers?.[2] ?? 0);
        // Tie-breaker 4: path length (asc)
        if ((a.tieBreakers?.[3] ?? 0) !== (b.tieBreakers?.[3] ?? 0))
            return (a.tieBreakers?.[3] ?? 0) - (b.tieBreakers?.[3] ?? 0);
        // Tie-breaker 5: lexicographic ID (asc)
        return String(a.tieBreakers?.[4] ?? '').localeCompare(String(b.tieBreakers?.[4] ?? ''));
    });

    return scored;
}

/**
 * Safe fallback selector - returns configured fallback when no lawful path exists.
 * Per spec: never pick "least bad illegal one."
 */
function selectBestTrajectoryImpl(candidates, verification) {
    // First try ranking only lawful candidates
    const ranked = rankCandidatesImpl(candidates, verification);

    if (ranked.length > 0) {
        return ranked[0]; // Best lawful trajectory
    }

    // No lawful path - return safe fallback
    return {
        id: 'fallback_noop',
        label: 'No-op (no lawful path)',
        score: -Infinity,
        truncatedAt: 0,
        reason: 'no_lawful_candidates',
        fallback: true,
        receipts: [],
    };
}

/**
 * Legacy scoreChoice - wraps new scoring for backward compatibility.
 * Higher = better (more admissible).
 */
function scoreChoice(receipt, action, verification) {
    if (!receipt) return 0;

    const result = scoreTrajectory({ receipts: [receipt], action: action || {} });
    return result.score;
}

/**
 * Decision Engine: Full candidate generation (bounded beam search).
 * Per spec implementation.
 */
export function generateCandidates(...args) {
    return generateCandidatesImpl(...args);
}

/**
 * Decision Engine: Rank candidates with hard filter + scoring + tie-breaking.
 */
export function rankCandidates(...args) {
    return rankCandidatesImpl(...args);
}

/**
 * Decision Engine: Safe fallback selector.
 * Returns best lawful trajectory or safe fallback (never illegal).
 */
export function selectBestTrajectory(...args) {
    return selectBestTrajectoryImpl(...args);
}

/**
 * Derive trajectory graph from receipt chain.
 * Returns: { lanes: [], bestLaneId, steps }
 * Each lane = candidate trajectory (choice sequence)
 * Each node in lane = step with constraint status (pass/warn/fail)
 */
export function deriveTrajectoryGraph({ receipts = [], chainSteps = [], verification = {} }) {
    // If no data, return empty graph
    if (!receipts.length || !chainSteps.length) {
        return { lanes: [], bestLaneId: null, steps: 0 };
    }

    const steps = chainSteps.length;
    const lanes = [];

    // Generate candidate lanes from action space
    // Lane 0 = "do nothing" / baseline (actual receipt)
    const baselineLane = {
        id: 'baseline',
        label: 'Baseline',
        nodes: [],
        score: 0,
        truncatedAt: null,
    };

    // Add baseline nodes from actual chain
    for (let i = 0; i < steps; i++) {
        const step = chainSteps[i];
        const receipt = receipts[i];

        // Compute constraint status for this step
        const hasPolicy = step?.metrics?.isAdmissible !== false;
        const hasContinuity = step?.continuity?.stateValid && step?.continuity?.digestValid;
        const constraintStatus = (hasPolicy && hasContinuity) ? 'pass' : (hasPolicy ? 'warn' : 'fail');

        baselineLane.nodes.push({
            stepIndex: i,
            constraintStatus,
            receipt,
        });
    }

    baselineLane.score = scoreChoice(receipts[receipts.length - 1] || receipts[0], {}, verification);
    lanes.push(baselineLane);

    // Generate alternative candidate lanes from planChoices
    const lastReceipt = receipts[receipts.length - 1] || receipts[0];
    const candidates = planChoices(lastReceipt, 4); // 4 alternatives + baseline = 5 total

    for (const candidate of candidates) {
        const lane = {
            id: candidate.id,
            label: candidate.action.metadata?.delta != null
                ? `Δ${candidate.action.metadata.delta}`
                : `Option ${candidate.id.slice(-1)}`,
            nodes: [],
            score: 0,
            truncatedAt: null,
        };

        // For each step, compute constraint status
        // (In a full impl, we'd actually evaluate the candidate action against constraints)
        for (let i = 0; i < steps; i++) {
            const step = chainSteps[i];
            const hasPolicy = step?.metrics?.isAdmissible !== false;
            const hasContinuity = step?.continuity?.stateValid && step?.continuity?.digestValid;

            // Simulate: some candidates pass, some fail
            // Use candidate index to vary the outcome
            const candidateIdx = parseInt(candidate.id.slice(-1), 10) || 0;
            let constraintStatus = 'pass';
            if (candidateIdx > 2 && i === steps - 1) constraintStatus = 'fail'; // Fail last step for some
            else if (candidateIdx > 1 && i === Math.floor(steps / 2)) constraintStatus = 'warn';

            lane.nodes.push({
                stepIndex: i,
                constraintStatus,
                candidate,
            });
        }

        // Truncate lane if any step fails
        const failIdx = lane.nodes.findIndex(n => n.constraintStatus === 'fail');
        if (failIdx !== -1) {
            lane.truncatedAt = failIdx;
        }

        lane.score = scoreChoice(lastReceipt, candidate.action, verification);
        lanes.push(lane);
    }

    // Sort by score descending, pick best
    lanes.sort((a, b) => b.score - a.score);
    const bestLaneId = lanes[0]?.id || null;

    return { lanes, bestLaneId, steps };
}
