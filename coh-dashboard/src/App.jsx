import React, { useEffect, useMemo, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  ChevronRight,
  Code,
  Database,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  Waypoints,
  Workflow,
  Zap,
} from 'lucide-react';
import { motion } from 'framer-motion';
import AdapterItem from './components/AdapterItem';
import {
  DEFAULT_SIDECAR_BASE_URL,
  SCENARIO_OPTIONS,
  loadDashboardData,
  executeVerified,
} from './data/cohData';

function truncateHash(value, lead = 12, tail = 8) {
  if (!value) {
    return '—';
  }

  return `${value.slice(0, lead)}…${value.slice(-tail)}`;
}

function buildProofPayload(verification) {
  return {
    request_id: verification?.requestId ?? '—',
    coh_version: verification?.cohVersion ?? '—',
    status: verification?.status ?? 'REJECT',
    data: verification?.data ?? null,
    error: verification?.error ?? null,
  };
}

export default function App() {
  const [scenarioKey, setScenarioKey] = useState('valid');
  const [preferLiveVerification, setPreferLiveVerification] = useState(false);
  const [reloadTick, setReloadTick] = useState(0);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [dashboardData, setDashboardData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(null);
  const [actionName, setActionName] = useState('transfer_100_tokens');
  const [actionAmount, setActionAmount] = useState(100);
  const [actionTarget, setActionTarget] = useState('alice');
  const [executing, setExecuting] = useState(false);
  const [execution, setExecution] = useState(null);
  const [executionError, setExecutionError] = useState(null);

  useEffect(() => {
    let cancelled = false;

    async function hydrateDashboard() {
      setLoading(true);
      setLoadError(null);

      try {
        const data = await loadDashboardData({
          scenarioKey,
          preferLiveVerification,
          sidecarBaseUrl: DEFAULT_SIDECAR_BASE_URL,
        });

        if (cancelled) {
          return;
        }

        setDashboardData(data);
        setSelectedIdx((current) => {
          if (data.chainSteps.length === 0) {
            return 0;
          }

          return Math.min(current, data.chainSteps.length - 1);
        });
      } catch (error) {
        if (cancelled) {
          return;
        }

        setLoadError(error instanceof Error ? error.message : 'Failed to load AI demo data.');
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    hydrateDashboard();

    return () => {
      cancelled = true;
    };
  }, [scenarioKey, preferLiveVerification, reloadTick]);

  async function runExecute() {
    console.log('[Execute] Starting execution with sidecar:', DEFAULT_SIDECAR_BASE_URL);
    setExecuting(true);
    setExecution(null);
    setExecutionError(null);
    try {
      const receipt = (dashboardData?.chainSteps?.[selectedIdx]?.raw) ?? (dashboardData?.chainSteps?.[0]?.raw);
      console.log('[Execute] Receipt:', receipt);
      if (!receipt) {
        throw new Error('No receipt available to execute');
      }
      const actionPayload = { action: actionName, amount: Number(actionAmount), target: actionTarget };
      console.log('[Execute] Action:', actionPayload);
      const result = await executeVerified({
        receipt,
        action: actionPayload,
        sidecarBaseUrl: DEFAULT_SIDECAR_BASE_URL,
      });
      console.log('[Execute] Result:', result);
      setExecution(result);
    } catch (e) {
      console.error('[Execute] Error:', e);
      setExecutionError(e instanceof Error ? e.message : String(e));
    } finally {
      setExecuting(false);
    }
  }

  const scenario = dashboardData?.scenario;
  const chainSteps = dashboardData?.chainSteps ?? [];
  const selectedStep = chainSteps[selectedIdx] ?? chainSteps[0];
  const verification = dashboardData?.verification;
  const slab = dashboardData?.slab;
  const slabCheck = dashboardData?.slabCheck;
  const breakInfo = dashboardData?.breakInfo;
  const isSystemTrusted = dashboardData?.isTrusted ?? false;
  const statusTone = isSystemTrusted ? 'is-trusted' : 'is-tampered';
  const decision = selectedStep?.status === 'TAMPERED' || !selectedStep?.metrics?.isAdmissible ? 'REJECT' : 'ACCEPT';
  const policyStatus = selectedStep?.metrics?.isAdmissible ? 'Admissible' : 'Policy violated';
  const stateLink = selectedStep?.continuity?.stateLabel ?? 'Unknown';
  const chainDigestState = selectedStep?.continuity?.digestLabel ?? 'Unknown';
  const liveNote = dashboardData?.liveError
    ? `Live sidecar unavailable: ${dashboardData.liveError}`
    : verification?.source === 'sidecar'
      ? `Live verification active via ${dashboardData.sidecarBaseUrl}`
      : 'Using bundled AI demo fixtures';
  const proofPayload = buildProofPayload(verification);

  const overviewCards = useMemo(
    () => [
      {
        label: 'Chain Steps',
        value: String(chainSteps.length).padStart(2, '0'),
        meta: scenario?.label ?? 'Loaded scenario',
      },
      {
        label: 'Break Index',
        value: breakInfo ? `#${breakInfo.stepIndex}` : '—',
        meta: breakInfo ? breakInfo.typeLabel : 'No deterministic chain break detected',
      },
      {
        label: 'Slab Check',
        value: slabCheck?.isValid ? 'PASS' : 'FAIL',
        meta: slabCheck?.message ?? 'Awaiting slab verification',
      },
      {
        label: 'Validator',
        value: verification?.cohVersion ?? '—',
        meta: verification?.source === 'sidecar' ? 'UnifiedResponse from live sidecar' : 'Fixture-backed verification contract',
      },
    ],
    [breakInfo, chainSteps.length, scenario, slabCheck, verification]
  );

  const sourceCards = useMemo(
    () => [
      {
        name: 'Fixture Chain',
        subtitle: scenario?.description,
        statusLabel: `${chainSteps.length} receipts loaded`,
        isActive: true,
        primaryLabel: 'Steps',
        primaryValue: chainSteps.length,
        secondaryLabel: 'Digest',
        secondaryValue: truncateHash(chainSteps[chainSteps.length - 1]?.hash, 8, 6),
        note: 'JSONL receipt chain mirrored into the frontend boundary for deterministic demo playback.',
      },
      {
        name: 'Slab Summary',
        subtitle: slab?.objectId,
        statusLabel: slabCheck?.isValid ? 'Summary verified' : 'Summary mismatch',
        isActive: Boolean(slabCheck?.isValid),
        primaryLabel: 'Micro',
        primaryValue: slab?.microCount ?? '—',
        secondaryLabel: 'Merkle',
        secondaryValue: truncateHash(slab?.merkleRoot, 8, 6),
        note: slabCheck?.message ?? 'Slab summary unavailable.',
      },
      {
        name: 'Live Sidecar',
        subtitle: dashboardData?.sidecarBaseUrl,
        statusLabel:
          verification?.source === 'sidecar'
            ? 'Live verification active'
            : preferLiveVerification
              ? 'Fallback to fixture mode'
              : 'Live verification disabled',
        isActive: verification?.source === 'sidecar',
        primaryLabel: 'Decision',
        primaryValue: verification?.status ?? '—',
        secondaryLabel: 'Request',
        secondaryValue: verification?.requestId?.slice(0, 8) ?? '—',
        note: liveNote,
      },
    ],
    [chainSteps, dashboardData?.sidecarBaseUrl, liveNote, preferLiveVerification, scenario, slab, slabCheck, verification]
  );

  if (loading && !dashboardData) {
    return (
      <div className="app-shell">
        <div className="dashboard">
          <section className="panel hero-panel load-panel">
            <div className="eyebrow">COH AUDIT CONSOLE</div>
            <h1>Loading AI demo data…</h1>
            <p>Hydrating real receipt chains, slab summaries, and verification contracts.</p>
          </section>
        </div>
      </div>
    );
  }

  if (loadError && !dashboardData) {
    return (
      <div className="app-shell">
        <div className="dashboard">
          <section className="panel hero-panel load-panel">
            <div className="eyebrow">COH AUDIT CONSOLE</div>
            <h1>Unable to load demo data</h1>
            <p>{loadError}</p>
            <div className="command-actions">
              <button type="button" className="button button-secondary" onClick={() => setReloadTick((tick) => tick + 1)}>
                <RefreshCw size={14} /> Retry load
              </button>
            </div>
          </section>
        </div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <div className="dashboard">
        <header className="panel hero-panel">
          <div className="hero-copy">
            <div className="eyebrow">COH AUDIT CONSOLE</div>
            <div className="hero-title-row">
              <div>
                <h1>Integrity Operations Dashboard</h1>
                <p>
                  Deterministic receipt verification across chain continuity, accounting policy,
                  slab reconciliation, and live validator responses.
                </p>
              </div>
              <div className={`status-badge ${statusTone}`}>
                {isSystemTrusted ? <ShieldCheck size={18} /> : <ShieldAlert size={18} />}
                <div>
                  <span className="status-label">System status</span>
                  <strong>{isSystemTrusted ? 'Trusted' : 'Attention Required'}</strong>
                </div>
              </div>
            </div>

            <div className="command-bar">
              <div className="command-meta">
                <label className="command-field">
                  <span>Scenario</span>
                  <select
                    className="command-select"
                    value={scenarioKey}
                    onChange={(event) => {
                      setScenarioKey(event.target.value);
                      setSelectedIdx(0);
                    }}
                  >
                    {SCENARIO_OPTIONS.map((option) => (
                      <option key={option.key} value={option.key}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                <span><Workflow size={14} /> Source: {verification?.source === 'sidecar' ? 'Live sidecar' : 'Fixture mode'}</span>
                <span><Waypoints size={14} /> Validator: {verification?.cohVersion ?? '0.1.0'}</span>
                <span><Activity size={14} /> Profile: deterministic_accounting_v1</span>
              </div>
              <div className="command-actions">
                <button
                  type="button"
                  className={`button ${preferLiveVerification ? 'button-primary' : 'button-secondary'}`}
                  onClick={() => setPreferLiveVerification((current) => !current)}
                >
                  <Zap size={14} /> {preferLiveVerification ? 'Live verify enabled' : 'Enable live verify'}
                </button>
                <button type="button" className="button button-secondary" onClick={() => setReloadTick((tick) => tick + 1)}>
                  <RefreshCw size={14} /> Reload data
                </button>
                <div className="command-field" style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                  <span>Action</span>
                  <input className="command-select" style={{ width: 180 }} value={actionName} onChange={(e) => setActionName(e.target.value)} />
                  <input className="command-select" style={{ width: 100 }} type="number" value={actionAmount} onChange={(e) => setActionAmount(e.target.value)} />
                  <input className="command-select" style={{ width: 160 }} value={actionTarget} onChange={(e) => setActionTarget(e.target.value)} />
                  <button type="button" className="button button-primary" disabled={executing} onClick={() => { console.log('[Execute] Button clicked!'); runExecute(); }}>
                    <Code size={14} /> {executing ? 'Executing…' : execution ? 'Done - Click to retry' : 'Execute verified'}
                  </button>
                </div>
              </div>
            </div>

            <div className={`status-note ${dashboardData?.liveError ? 'is-tampered' : 'is-trusted'}`}>
              {loading ? 'Refreshing data…' : liveNote}
            </div>
          </div>

          <div className="overview-grid">
            {overviewCards.map((card) => (
              <div className="metric-card" key={card.label}>
                <span className="metric-label">{card.label}</span>
                <strong className="metric-value">{card.value}</strong>
                <span className="metric-meta">{card.meta}</span>
              </div>
            ))}
          </div>
        </header>

        <main className="dashboard-main">
          <section className="primary-grid">
            <div className="panel timeline-panel">
              <div className="panel-header">
                <div>
                  <span className="section-label">Chain continuity</span>
                  <h2>Receipt timeline</h2>
                </div>
                <div className={`inline-status ${statusTone}`}>
                  <span className="status-dot" />
                  {breakInfo ? `Break at step #${breakInfo.stepIndex}` : 'Continuity intact'}
                </div>
              </div>

              <div className="timeline-list" role="list" aria-label="Receipt timeline">
                {chainSteps.map((node, i) => {
                  const isSelected = selectedIdx === i;
                  const tone = node.status === 'TAMPERED' ? 'is-tampered' : 'is-trusted';
                  const timelineLabel =
                    node.breakRole === 'BREAK'
                      ? `Break: ${breakInfo?.typeLabel}`
                      : node.breakRole === 'AFTER_BREAK'
                        ? 'Downstream impact'
                        : 'Verified receipt';

                  return (
                    <React.Fragment key={node.id}>
                      <motion.button
                        type="button"
                        initial={{ opacity: 0, y: 12 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: i * 0.08 }}
                        onClick={() => setSelectedIdx(i)}
                        className={`timeline-item ${isSelected ? 'is-selected' : ''} ${tone}`}
                      >
                        <div className="timeline-index">#{node.stepIndex}</div>
                        <div className="timeline-body">
                          <strong>{timelineLabel}</strong>
                          <span>{truncateHash(node.hash)}</span>
                        </div>
                        <div className={`timeline-pill ${tone}`}>
                          {node.status === 'TAMPERED' ? 'Reject' : 'Accept'}
                        </div>
                      </motion.button>
                      {i < chainSteps.length - 1 && <ChevronRight className="timeline-arrow" size={16} />}
                    </React.Fragment>
                  );
                })}
              </div>

              <div className="timeline-summary-grid">
                <div className="summary-card">
                  <span className="summary-label">Final digest state</span>
                  <strong>{verification?.status === 'ACCEPT' ? 'Verified' : 'Rejected'}</strong>
                  <p>{verification?.reason ?? 'All chain links and state hashes align.'}</p>
                </div>
                <div className="summary-card">
                  <span className="summary-label">Slab integrity</span>
                  <strong>{slabCheck?.isValid ? 'Verified' : 'Mismatch'}</strong>
                  <p>{slabCheck?.message}</p>
                </div>
                <div className="summary-card break-card">
                  <span className="summary-label">Break analysis</span>
                  <strong>{breakInfo ? `Step #${breakInfo.stepIndex}` : 'No break detected'}</strong>
                  <p>
                    {breakInfo
                      ? breakInfo.message
                      : 'Deterministic linkage checks did not find a chain discontinuity in this scenario.'}
                  </p>
                  {breakInfo ? (
                    <div className="break-details">
                      <div>
                        <span>Expected</span>
                        <code>{truncateHash(breakInfo.expected)}</code>
                      </div>
                      <div>
                        <span>Actual</span>
                        <code>{truncateHash(breakInfo.actual)}</code>
                      </div>
                    </div>
                  ) : null}
                </div>
                <div className={`summary-card ${executionError ? 'break-card' : ''}`}>
                  <span className="summary-label">Execution result</span>
                  <strong>{execution?.status ?? (executionError ? 'Error' : '—')}</strong>
                  <p>{execution?.reason ?? executionError ?? 'Submit verified action to sidecar.'}</p>
                </div>
              </div>
            </div>

            <aside className="panel inspector-panel">
              <div className="panel-header">
                <div>
                  <span className="section-label">Selected receipt</span>
                  <h2>Audit inspector</h2>
                </div>
                <div className={`decision-chip ${decision === 'ACCEPT' ? 'is-trusted' : 'is-tampered'}`}>
                  <Zap size={14} /> {decision}
                </div>
              </div>

              <div className="inspector-topline">
                <div>
                  <span className="inspector-label">Object</span>
                  <strong>{selectedStep?.objectId}</strong>
                </div>
                <div>
                  <span className="inspector-label">Step</span>
                  <strong>{selectedStep?.stepIndex}</strong>
                </div>
                <div>
                  <span className="inspector-label">State link</span>
                  <strong>{stateLink}</strong>
                </div>
              </div>

              <div className="inspector-grid">
                <section className="data-card">
                  <div className="mini-heading">
                    <Database size={15} /> Receipt digest
                  </div>
                  <code className="digest-block">{selectedStep?.hash}</code>
                </section>

                <section className="data-card">
                  <div className="mini-heading">
                    <Code size={15} /> Accounting verification
                  </div>
                  <div className="kv-list monospace-grid">
                    <div><span>v_pre</span><strong>{selectedStep?.metrics?.vPre}</strong></div>
                    <div><span>v_post</span><strong>{selectedStep?.metrics?.vPost}</strong></div>
                    <div><span>spend</span><strong>{selectedStep?.metrics?.spend}</strong></div>
                    <div><span>defect</span><strong>{selectedStep?.metrics?.defect}</strong></div>
                    <div><span>Envelope</span><strong>{selectedStep?.metrics?.leftSide} ≤ {selectedStep?.metrics?.rightSide}</strong></div>
                    <div><span>Decision</span><strong className={selectedStep?.metrics?.isAdmissible ? 'text-success' : 'text-danger'}>{policyStatus}</strong></div>
                    <div><span>Digest</span><strong>{chainDigestState}</strong></div>
                  </div>
                </section>

                <section className="data-card span-full">
                  <div className="mini-heading">
                    <Activity size={15} /> Receipt linkage
                  </div>
                  <div className="checkpoint-list">
                    <div className="checkpoint-row">
                      <span>Schema / version</span>
                      <strong>{selectedStep?.schemaId} / {selectedStep?.version}</strong>
                    </div>
                    <div className="checkpoint-row">
                      <span>Canon profile</span>
                      <strong>{truncateHash(selectedStep?.canonProfileHash)}</strong>
                    </div>
                    <div className="checkpoint-row">
                      <span>State prev → next</span>
                      <strong>{truncateHash(selectedStep?.stateHashPrev)} → {truncateHash(selectedStep?.stateHashNext)}</strong>
                    </div>
                    <div className="checkpoint-row">
                      <span>Digest prev → next</span>
                      <strong>{truncateHash(selectedStep?.chainDigestPrev)} → {truncateHash(selectedStep?.chainDigestNext)}</strong>
                    </div>
                  </div>
                </section>
              </div>
            </aside>
          </section>

          <section className="secondary-grid">
            <div className="panel secondary-panel">
              <div className="panel-header compact">
                <div>
                  <span className="section-label">Verification sources</span>
                  <h2>Data source health</h2>
                </div>
                <div className="inline-count">{verification?.source === 'sidecar' ? 'live' : 'fixture'}</div>
              </div>
              <div className="adapter-grid">
                {sourceCards.map((adapter) => (
                  <AdapterItem
                    key={adapter.name}
                    {...adapter}
                  />
                ))}
              </div>
            </div>

            <div className="panel secondary-panel">
              <div className="panel-header compact">
                <div>
                  <span className="section-label">Operational evidence</span>
                  <h2>Proof payload</h2>
                </div>
              </div>

              <div className="evidence-stack">
                <div className="data-card">
                  <div className="mini-heading">
                    <Workflow size={15} /> Audit frame
                  </div>
                  <div className="kv-list">
                    <div><span>Protocol</span><strong>{selectedStep?.schemaId}</strong></div>
                    <div><span>Scenario</span><strong>{scenario?.label}</strong></div>
                    <div><span>Policy</span><strong>deterministic_accounting_v1</strong></div>
                    <div><span>Validator version</span><strong>{verification?.cohVersion}</strong></div>
                    <div><span>Verify source</span><strong>{verification?.source}</strong></div>
                    <div><span>Error code</span><strong>{verification?.error?.code ?? '—'}</strong></div>
                    <div><span>Decision</span><strong>{verification?.status}</strong></div>
                  </div>
                </div>

                <div className="data-card">
                  <div className="mini-heading">
                    <Database size={15} /> Slab summary
                  </div>
                  <div className="kv-list monospace-grid">
                    <div><span>Range</span><strong>{slab?.rangeStart} → {slab?.rangeEnd}</strong></div>
                    <div><span>Micro count</span><strong>{slab?.microCount}</strong></div>
                    <div><span>Total spend</span><strong>{slab?.summary?.totalSpend}</strong></div>
                    <div><span>Total defect</span><strong>{slab?.summary?.totalDefect}</strong></div>
                    <div><span>Slab verdict</span><strong className={slabCheck?.isValid ? 'text-success' : 'text-danger'}>{slabCheck?.isValid ? 'Verified' : 'Mismatch'}</strong></div>
                  </div>
                  <code className="digest-block compact">{slab?.merkleRoot}</code>
                </div>

                <div className="data-card">
                  <div className="mini-heading">
                    <Code size={15} /> Internal proof vector
                  </div>
                  <pre>{JSON.stringify(proofPayload, null, 2)}</pre>
                </div>
              </div>
            </div>
          </section>
        </main>

        <footer className="dashboard-footer">
          <span>COH</span>
          <span>2026</span>
          <span>Deterministic Verification Kernel</span>
          <span>Validator {verification?.cohVersion}</span>
          <span>No-Bluff System</span>
        </footer>
      </div>
    </div>
  );
}
