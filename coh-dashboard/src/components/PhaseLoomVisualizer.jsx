import React, { useEffect, useRef, useState } from 'react';
import { motion } from 'framer-motion';

const PhaseLoomVisualizer = () => {
  const canvasRef = useRef(null);
  const [simulationData, setSimulationData] = useState(null);
  const [hoveredStrand, setHoveredStrand] = useState(null);

  // Colors matching the cinematic theme
  const THEME = {
    BG: '#05080d',
    GOLD: '#d9a233',
    CYAN: '#8ed8ff',
    RED: '#ff4e3f',
    WHITE: '#eaf2ff',
    MUTED: '#6b7280'
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch('/data/simulation_state.json');
        const data = await response.json();
        setSimulationData(data);
      } catch (err) {
        console.error('Failed to load simulation state:', err);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000); // Poll every 5s
    return () => clearInterval(interval);
  }, []);

  const THEME = {
    BG: '#05080d',
    GOLD: '#d9a233',   // Coherence Margin
    CYAN: '#8ed8ff',   // Chaos Margin
    MAGENTA: '#ff00ff', // Semantic Envelope
    RED: '#ff4e3f',    // Violation
    WHITE: '#eaf2ff',
    MUTED: '#6b7280'
  };

  useEffect(() => {
    if (!simulationData || !canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    const width = canvas.width;
    const height = canvas.height;

    const render = () => {
      ctx.clearRect(0, 0, width, height);
      
      // Draw Grid
      ctx.strokeStyle = THEME.CYAN;
      ctx.globalAlpha = 0.05;
      ctx.lineWidth = 0.5;
      for (let i = 0; i < 10; i++) {
        const x = (width / 10) * i;
        ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, height); ctx.stroke();
        const y = (height / 10) * i;
        ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke();
      }

      // Draw Trajectories
      simulationData.trajectories.forEach((traj, i) => {
        const isAccept = traj.outcome === 'ACCEPT';
        const isChaosExhausted = Math.abs(traj.chaos_margin || 100) < 1;
        const isCohExhausted = Math.abs(traj.coherence_margin || 100) < 1;
        const isEnvelopeExhausted = Math.abs(traj.envelope_margin || 100) < 1;

        // Choose color based on active boundary
        let color = THEME.MUTED;
        if (isAccept) {
          if (isChaosExhausted) color = THEME.CYAN;
          else if (isCohExhausted) color = THEME.GOLD;
          else if (isEnvelopeExhausted) color = THEME.MAGENTA;
          else color = THEME.WHITE;
        } else {
          color = THEME.RED;
        }

        ctx.strokeStyle = color;
        ctx.globalAlpha = isAccept ? (isChaosExhausted || isCohExhausted ? 0.9 : 0.6) : 0.15;
        ctx.lineWidth = isAccept ? (isChaosExhausted || isCohExhausted ? 2.5 : 1.5) : 0.8;

        ctx.beginPath();
        traj.curve.forEach((yVal, xIdx) => {
          const x = (xIdx / (traj.curve.length - 1)) * width;
          const y = height / 2 + (yVal * height / 2);
          if (xIdx === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        });
        ctx.stroke();

        // Glow for active boundary
        if (isAccept && (isChaosExhausted || isCohExhausted)) {
          ctx.shadowBlur = 10;
          ctx.shadowColor = color;
          ctx.stroke();
          ctx.shadowBlur = 0;
        }
      });

      // Draw Formation Gates
      ctx.globalAlpha = 0.8;
      ctx.lineWidth = 2;
      
      // RV Gate
      ctx.strokeStyle = THEME.GOLD;
      ctx.beginPath();
      ctx.ellipse(width * 0.55, height / 2, 5, height * 0.35, 0, 0, Math.PI * 2);
      ctx.stroke();

      // Chaos Boundary
      ctx.strokeStyle = THEME.CYAN;
      ctx.setLineDash([5, 5]);
      ctx.beginPath();
      ctx.moveTo(width * 0.2, 0); ctx.lineTo(width * 0.2, height);
      ctx.stroke();
      ctx.setLineDash([]);
    };

    render();
  }, [simulationData]);

  return (
    <div className="card" style={{ height: '500px', marginBottom: '1.5rem', overflow: 'hidden', position: 'relative' }}>
      <div className="card-header">
        <div className="card-title">FORMATION_BOUNDARY_INSPECTOR</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <div style={{ width: 8, height: 8, background: THEME.CYAN, borderRadius: '50%' }} />
            <span className="monospace" style={{ fontSize: '0.6rem' }}>ΔCh</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <div style={{ width: 8, height: 8, background: THEME.GOLD, borderRadius: '50%' }} />
            <span className="monospace" style={{ fontSize: '0.6rem' }}>ΔCoh</span>
          </div>
          <div className="live-indicator" />
          <span className="monospace" style={{ fontSize: '0.65rem', color: 'var(--text-muted)' }}>FORMATION_V2_FEED</span>
        </div>
      </div>
      
      <div className="card-body" style={{ padding: 0, position: 'relative', overflow: 'hidden' }}>
        <canvas 
          ref={canvasRef} 
          width={1200} 
          height={400} 
          style={{ width: '100%', height: '100%', objectFit: 'cover', opacity: 0.9 }}
        />
        
        {/* HUD: Diagnostics Card */}
        {simulationData && (
          <motion.div 
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            style={{ 
              position: 'absolute', top: '1.25rem', left: '1.25rem', 
              padding: '1rem', background: 'var(--bg-glass)', 
              border: '1px solid var(--border-bright)', borderRadius: 'var(--radius-md)',
              backdropFilter: 'blur(10px)', zIndex: 10
            }}
          >
            <div className="metric-label" style={{ color: 'var(--brand-primary)' }}>FORMATION_DIAGNOSTICS</div>
            <div className="metric-group" style={{ flexDirection: 'column', gap: '8px', marginTop: '8px' }}>
              <div className="metric-item">
                <span className="metric-label">Acceptance (Π)</span>
                <span className="metric-value" style={{ color: 'var(--brand-caution)' }}>
                  {((simulationData.metadata.outcomes.ACCEPT / simulationData.metadata.n_proposals) * 100).toFixed(1)}%
                </span>
              </div>
              <div className="metric-item">
                <span className="metric-label">Chaos Pressure</span>
                <span className="metric-value">
                  {simulationData.diagnostics.mean_margin.toFixed(3)}
                </span>
              </div>
              <div className="metric-item" style={{ marginTop: '4px', borderTop: '1px solid var(--border-subtle)', paddingTop: '4px' }}>
                <span className="monospace" style={{ fontSize: '0.55rem', opacity: 0.6 }}>ACTIVE_BOUNDARY: </span>
                <span className="monospace" style={{ fontSize: '0.55rem', color: THEME.CYAN }}>{simulationData.diagnostics.active_gate || 'NOMINAL'}</span>
              </div>
            </div>
          </motion.div>
        )}

        {/* Legend / Status HUD */}
        <div style={{ position: 'absolute', bottom: '1.25rem', right: '1.25rem', display: 'flex', gap: '1rem' }}>
           <div className="monospace" style={{ fontSize: '0.6rem', color: THEME.MUTED }}>
             PROFILE: <span style={{ color: THEME.WHITE }}>FORMATION_V2</span>
           </div>
        </div>

        {/* Signature */}
        <div className="monospace" style={{ 
          position: 'absolute', bottom: '1.5rem', left: '50%', 
          transform: 'translateX(-50%)', fontSize: '0.6rem', 
          fontWeight: 'bold', letterSpacing: '0.3em', color: 'var(--brand-caution)',
          opacity: 0.5
        }}>
          FORMATION IS THE INTERSECTION.
        </div>
      </div>
    </div>
  );
};

export default PhaseLoomVisualizer;
