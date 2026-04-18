import React, { useState, useEffect, useMemo } from 'react';
import { loadDashboardData, generateCandidatesImpl } from './data/cohData';
import TrajectoryGraph from './components/TrajectoryGraph';

const WITNESS_SHORT = ['C1', 'C2', 'C3', 'C4', 'C5', 'C6'];

const App = () => {
  const [data, setData] = useState(null);
  const [candidates, setCandidates] = useState([]);
  const [selectedScenario, setSelectedScenario] = useState('valid');
  const [selectedTrajectoryId, setSelectedTrajectoryId] = useState(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const init = async () => {
      try {
        setIsLoading(true);
        console.log('[HUD] Initializing with scenario:', selectedScenario);
        
        const dashboardData = await loadDashboardData({ scenarioKey: selectedScenario });
        console.log('[HUD] Data loaded:', dashboardData);
        setData(dashboardData);
        
        const steps = dashboardData.chainSteps || [];
        const rootReceipt = steps.length > 0 ? steps[steps.length - 1].raw : null;
        
        if (rootReceipt) {
          console.log('[HUD] Generating trajectories from:', rootReceipt.step_index);
          const proposed = generateCandidatesImpl(rootReceipt, { maxDepth: 4, beamWidth: 3 });
          setCandidates(proposed);
          
          const selectable = proposed.filter(t => t.isSelectable);
          if (selectable.length > 0) {
            setSelectedTrajectoryId(selectable[0].id);
          } else if (proposed.length > 0) {
            setSelectedTrajectoryId(proposed[0].id);
          }
        }
      } catch (err) {
        console.error('[HUD] Initialization failed:', err);
      } finally {
        setIsLoading(false);
      }
    };
    init();
  }, [selectedScenario]);

  const selectedTrajectory = useMemo(() => 
    candidates.find(t => t.id === selectedTrajectoryId),
    [candidates, selectedTrajectoryId]
  );

  if (isLoading) return <div className="app-shell monospace" style={{ padding: '2rem' }}>[ INITIALIZING HUD ... ]</div>;

  return (
    <div className="app-shell">
      <div className="hud-overlay" />
      
      <header className="dashboard-header">
        <div className="brand-section">
          <div style={{ color: 'var(--neon-cyan)', fontSize: '1.5rem', fontWeight: 900 }}>COH-WEDGE</div>
          <div className="section-label">Trajectory HUD v2.0</div>
        </div>

        <div className="guarantee-banner panel">
          <div className="section-label monospace">Selection Guarantee</div>
          The system only highlights and selects trajectories whose steps are locally lawful, 
          chain-consistent, and cumulatively within accounting and policy bounds.
        </div>

        <div className="brand-section">
          <div className="section-label">State</div>
          <div className={data?.isTrusted ? 'status-pass' : 'status-fail'}>
            {data?.isTrusted ? 'ADMISSIBLE' : 'POLICY_VIOLATION'}
          </div>
        </div>
      </header>

      <main className="primary-grid">
        <section className="trajectory-container">
          <div className="panel-header compact" style={{ padding: '1rem', borderBottom: '1px solid var(--border-subtle)' }}>
             <span className="section-label">Admissible Path Search (Beam Search)</span>
          </div>
          
          <TrajectoryGraph 
            candidates={candidates}
            selectedId={selectedTrajectoryId}
            onSelect={setSelectedTrajectoryId}
          />
        </section>

        <section className="inspector-panel panel">
          <div className="section-label monospace">Trajectory Inspector</div>
          
          <div className="metric-card">
            <span className="section-label">Path Rank Index S(τ)</span>
            <div className="metric-value status-pass">
              {selectedTrajectory?.score?.toFixed(4) || '0.0000'}
            </div>
          </div>

          <div className="metric-card">
            <span className="section-label">Cumulative Margin Trace</span>
            <div className={`metric-value ${selectedTrajectory?.isSelectable ? 'status-pass' : 'status-fail'}`}>
              {selectedTrajectory?.cumulativeMargin || '0'}
            </div>
            <div className="badge-row">
              {WITNESS_SHORT.map((w, i) => (
                <div key={i} className={`badge ${selectedTrajectory?.isSelectable ? 'is-pass' : 'is-fail'}`}>{w}</div>
              ))}
            </div>
          </div>

          <div className="metric-card" style={{ flex: 1 }}>
            <span className="section-label">Law Verification Witnesses</span>
            <div style={{ height: '230px', overflowY: 'auto', marginTop: '1rem' }}>
              {selectedTrajectory?.witnesses?.map((w, i) => (
                <div key={i} className="readout-item" style={{ borderBottom: '1px solid var(--border-subtle)', paddingBottom: '0.5rem', marginBottom: '0.5rem' }}>
                  <span className="monospace">STEP {i}</span>
                  <div className="badge-row">
                    <div className={`badge is-${w.c1.status}`}>C1</div>
                    <div className={`badge is-${w.c2.status}`}>C2</div>
                    <div className={`badge is-${w.c3.status}`}>C3</div>
                    <div className={`badge is-${w.c4.status}`}>C4</div>
                    <div className={`badge is-${w.c5.status}`}>C5</div>
                    <div className={`badge is-${w.c6.status}`}>C6</div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <button 
             className="monospace" 
             style={{ marginTop: 'auto', width: '100%', padding: '1rem', background: 'var(--neon-cyan)', color: 'black', border: 'none', fontWeight: 'bold' }}
             disabled={!selectedTrajectory?.isSelectable}
          >
             EXECUTE ADMISSIBLE PATH
          </button>
        </section>
      </main>

      <footer className="dashboard-footer">
        <div className="monospace">VALIDATOR: HARDENED_RUST_CORE::{data?.validatorVersion}</div>
        <div className="monospace">SESSION_ID: {data?.verification?.requestId}</div>
      </footer>
    </div>
  );
};


export default App;
