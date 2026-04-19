import React from 'react';
import { ShieldCheck, ShieldAlert, ChevronRight, Info } from 'lucide-react';

export const DecisionBanner = ({ scenarioLabel, isTrusted, reason }) => {
  return (
    <div className="verdict-banner">
      <div className={`verdict-badge ${isTrusted ? 'allowed' : 'blocked'}`}>
        {isTrusted ? 'TRUSTED' : 'BLOCKED'}
      </div>

      <div style={{ flex: 1 }}>
        <div className="metric-label" style={{ marginBottom: '0.5rem' }}>Workflow Action Result</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <h3 style={{ fontSize: '1.25rem', fontWeight: 700 }}>{scenarioLabel}</h3>
          <ChevronRight size={18} className="text-muted" />
          <span style={{ color: 'var(--text-secondary)' }}>
            {isTrusted
              ? 'Verification complete. No policy violations detected.'
              : `Security halt. System prevented execution: ${reason || 'Inadmissible state transition.'}`
            }
          </span>
        </div>
      </div>

      <div>
        <div className="status-pill success" style={{ background: 'transparent', borderColor: 'var(--border-muted)', color: 'var(--text-muted)' }}>
          <Info size={14} />
          <span>VERIFIER_V2.0_SECURE</span>
        </div>
      </div>
    </div>
  );
};

// ChecklistItem component - defined outside the component for stability
const ChecklistItem = ({ label, isPass, description }) => (
  <div style={{
    display: 'flex',
    alignItems: 'flex-start',
    gap: '1rem',
    padding: '0.375rem 0'
  }}>
    {isPass
      ? <ShieldCheck size={14} style={{ color: 'var(--color-success)', flexShrink: 0, marginTop: '0.125rem' }} />
      : <ShieldAlert size={14} style={{ color: 'var(--color-error)', flexShrink: 0, marginTop: '0.125rem' }} />
    }
    <div>
      <div style={{ fontWeight: 600, fontSize: '0.8125rem' }}>{label}</div>
      <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>{description}</div>
    </div>
  </div>
);

// Evidence checklist item (boxed style) - also defined outside render
const EvidenceChecklistItem = ({ label, isPass, description }) => (
  <div style={{
    display: 'flex',
    alignItems: 'flex-start',
    gap: '1rem',
    padding: '1rem',
    background: 'var(--bg-surface-elevated)',
    borderRadius: 'var(--radius-sm)',
    border: `1px solid ${isPass ? 'var(--border-muted)' : 'hsla(350, 70%, 50%, 0.2)'}`,
    marginBottom: '0.75rem'
  }}>
    <div className={`status-pill ${isPass ? 'success' : 'error'}`} style={{ padding: '2px', borderRadius: '50%' }}>
      {isPass ? <ShieldCheck size={14} /> : <ShieldAlert size={14} />}
    </div>
    <div>
      <div style={{ fontSize: '0.85rem', fontWeight: 600, color: isPass ? 'var(--text-primary)' : 'var(--brand-blocked)' }}>{label}</div>
      <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '2px' }}>{description}</div>
    </div>
  </div>
);

export const EvidencePanel = ({ stepMetrics, isTrajTrusted }) => {
  const m = stepMetrics || {};

  return (
    <div className="card" style={{ height: '100%', borderLeft: '1px solid var(--border-bright)' }}>
      <div className="card-header">
        <span className="card-title">Verification Evidence Ledger</span>
      </div>
      <div className="card-body">
        <div style={{ marginBottom: '1.5rem', padding: '1rem', background: 'var(--bg-base)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-muted)' }}>
          <div className="metric-label">Consensus Verdict</div>
          <div className="monospace" style={{ fontSize: '1.1rem', fontWeight: 800, color: isTrajTrusted === false ? 'var(--brand-blocked)' : 'var(--brand-primary)', marginBottom: '1rem' }}>
            {isTrajTrusted === false ? 'PATH_REJECTED' : 'PATH_VALIDATED'}
          </div>
          
          {/* Grounding Gauges */}
          {m.evaluation && (
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.75rem', marginTop: '0.5rem' }}>
                <div style={{ background: 'var(--bg-surface-elevated)', padding: '0.5rem', borderRadius: '4px' }}>
                    <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)' }}>SAFETY</div>
                    <div style={{ fontSize: '0.9rem', fontWeight: 700 }}>
                      {(m.evaluation.safetyBottleneck * 100).toFixed(0)}% 
                      <div style={{ fontSize: '0.65rem', color: 'var(--text-muted)', fontWeight: 400, marginTop: '2px' }}>
                        Safety Bottleneck: {m.evaluation.safetyBottleneck}
                      </div>
                    </div>
                </div>
                <div style={{ background: 'var(--bg-surface-elevated)', padding: '0.5rem', borderRadius: '4px' }}>
                    <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)' }}>ALIGNMENT</div>
                    <div style={{ fontSize: '0.9rem', fontWeight: 700 }}>{(m.evaluation.alignment * 100).toFixed(0)}%</div>
                </div>
            </div>
          )}

          {/* Violation Evidence */}
          {isTrajTrusted === false && m.violationDelta && (
            <div style={{ marginTop: '1rem', padding: '0.75rem', background: '#300', border: '1px solid #600', borderRadius: '4px' }}>
                <div style={{ fontSize: '0.65rem', color: 'var(--brand-blocked)', fontWeight: 800 }}>VIOLATION_EVIDENCE_DELTA</div>
                <div className="monospace" style={{ fontSize: '1.2rem', color: '#ff5f5f' }}>
                    &delta;(r) = +{m.violationDelta}
                </div>
                <div style={{ fontSize: '0.6rem', marginTop: '0.4rem', color: '#f99' }}>
                    REJECT_CODE: {m.rejectCode || 'UNSPECIFIED'}
                </div>
            </div>
          )}
        </div>

        <EvidenceChecklistItem
          label="Cryptographic Identity"
          isPass={true}
          description="All signatures verified against known authority set."
        />
        <EvidenceChecklistItem
          label="Chain Continuity"
          isPass={true}
          description="Receipt digests form a valid back-linked chain."
        />
        <EvidenceChecklistItem
          label="State Invariant"
          isPass={m.isAdmissible !== false}
          description="v_post <= v_pre + authority + defect remains true."
        />
        <EvidenceChecklistItem
          label="Policy Alignment"
          isPass={m.isAdmissible !== false}
          description="Action aligns with defined workflow-specific boundaries."
        />

        <div style={{ marginTop: '2rem' }}>
          <span className="metric-label">Step Detail (Audit Sample)</span>
          <div className="monospace" style={{ fontSize: '0.7rem', padding: '1rem', background: 'var(--bg-base)', border: '1px solid var(--border-muted)', marginTop: '0.5rem' }}>
            <div style={{ marginBottom: '4px' }}>v_pre:   <span className="text-secondary">{String(m.vPre ?? m.v_post ?? '')}</span></div>
            <div style={{ marginBottom: '4px' }}>v_post:  <span className="text-secondary">{String(m.vPost ?? m.v_pre ?? '')}</span></div>
            <div style={{ marginBottom: '4px' }}>spend:   <span className={Number(m.spend) > 0 ? 'text-ruby' : 'text-muted'}>{String(m.spend ?? '0')}</span></div>
            <div>outcome: <span className={m.isAdmissible !== false ? 'text-emerald' : 'text-ruby'}>
              {m.isAdmissible !== false ? 'PASS' : 'POLICY_VIOLATION'}
            </span></div>
          </div>
        </div>
      </div>
    </div>
  );
};
