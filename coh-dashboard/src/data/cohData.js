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

function derivePolicyCheck(metrics) {
    const vPre = toBigIntSafe(metrics.v_pre);
    const vPost = toBigIntSafe(metrics.v_post);
    const spend = toBigIntSafe(metrics.spend);
    const defect = toBigIntSafe(metrics.defect);

    if ([vPre, vPost, spend, defect].some((value) => value === null)) {
        return {
            isValid: false,
            leftSide: '—',
            rightSide: '—',
        };
    }

    const leftSide = vPost + spend;
    const rightSide = vPre + defect;

    return {
        isValid: leftSide <= rightSide,
        leftSide: leftSide.toString(),
        rightSide: rightSide.toString(),
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
    const vPreFirst = receipts[0]?.metrics?.v_pre ?? '0';
    const vPostLast = receipts[receipts.length - 1]?.metrics?.v_post ?? '0';

    const derived = {
        totalSpend: totalSpend.toString(),
        totalDefect: totalDefect.toString(),
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
            isAdmissible: policy.isValid,
            leftSide: policy.leftSide,
            rightSide: policy.rightSide,
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
