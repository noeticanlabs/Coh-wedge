import React, { useMemo } from 'react';

const WITNESS_LABELS = ['C1', 'C2', 'C3', 'C4', 'C5', 'C6'];

const TrajectoryGraph = ({ candidates, selectedId, onSelect }) => {
  // SVG Layout Constants
  const width = 800;
  const height = 500;
  const padding = 60;
  const nodeRadius = 6;
  
  const maxDepth = Math.max(...candidates.map(c => c.depth), 1);
  const stepX = (width - padding * 2) / maxDepth;

  const trajectories = useMemo(() => {
    return candidates.map(tau => {
      const isSelected = tau.id === selectedId;
      const points = tau.receipts.map((r, i) => {
        const x = padding + i * stepX;
        // Simple vertical distribution for demo branching
        const siblingIndex = candidates.filter(c => c.depth === tau.depth).indexOf(tau);
        const y = height / 2 + (siblingIndex - 1) * 60;
        return { x, y };
      });

      return {
        ...tau,
        points,
        isSelected
      };
    });
  }, [candidates, selectedId, stepX]);

  return (
    <div className="trajectory-container">
      <div className="hud-overlay" />
      <svg className="svg-viewport" viewBox={`0 0 ${width} ${height}`}>
        <defs>
          <filter id="glow">
            <feGaussianBlur stdDeviation="2.5" result="coloredBlur"/>
            <feMerge>
              <feMergeNode in="coloredBlur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>

        {trajectories.map(tau => {
          const { points, isSelectable, isSelected, firstFailureIndex } = tau;
          const truncateAt = !isSelectable && firstFailureIndex !== null ? firstFailureIndex + 1 : points.length;
          const visiblePoints = points.slice(0, truncateAt);

          return (
            <g key={tau.id} onClick={() => onSelect(tau.id)} style={{ cursor: 'pointer' }}>
              {/* Path Line */}
              <polyline
                points={visiblePoints.map(p => `${p.x},${p.y}`).join(' ')}
                fill="none"
                stroke={isSelected ? 'var(--neon-cyan)' : isSelectable ? 'var(--text-muted)' : 'var(--neon-crimson)'}
                strokeWidth={isSelected ? 4 : 2}
                strokeDasharray={tau.warnCount > 0 ? '5,5' : '0'}
                filter={isSelected ? 'url(#glow)' : 'none'}
                opacity={isSelected ? 1 : 0.4}
                style={{ transition: 'all 0.3s' }}
              />

              {/* Path Status Indicator (Truncation) */}
              {!isSelectable && firstFailureIndex !== null && (
                <g transform={`translate(${points[firstFailureIndex].x}, ${points[firstFailureIndex].y})`}>
                  <circle r="12" fill="none" stroke="var(--neon-crimson)" strokeWidth="1" strokeDasharray="2,2" />
                  <line x1="-6" y1="-6" x2="6" y2="6" stroke="var(--neon-crimson)" strokeWidth="2" />
                  <line x1="6" y1="-6" x2="-6" y2="6" stroke="var(--neon-crimson)" strokeWidth="2" />
                </g>
              )}

              {/* Nodes and Badges */}
              {visiblePoints.map((p, i) => (
                <g key={`n-${i}`} transform={`translate(${p.x}, ${p.y})`}>
                  <circle 
                    r={nodeRadius} 
                    fill={isSelected ? 'var(--neon-cyan)' : 'var(--bg-carbon)'} 
                    stroke={tau.witnesses?.[i]?.c1?.status === 'pass' ? 'var(--neon-cyan)' : 'var(--neon-crimson)'}
                    strokeWidth="1"
                  />
                  
                  {/* Witness Badges for the last visible node */}
                  {i === visiblePoints.length - 1 && isSelected && (
                    <g transform="translate(10, -25)">
                      <rect width="130" height="24" rx="2" fill="var(--bg-surface)" stroke="var(--neon-cyan)" strokeWidth="0.5" />
                      <text x="5" y="16" fontSize="8" fill="var(--neon-cyan)" className="monospace">LAW WITNESS:</text>
                      {WITNESS_LABELS.map((lbl, j) => (
                        <g key={lbl} transform={`translate(${75 + j * 9}, 10)`}>
                           <rect width="7" height="7" rx="1" fill={tau.witnesses?.[i]?.[`c${j+1}`]?.status === 'pass' ? 'var(--neon-cyan)' : 'var(--neon-crimson)'} opacity="0.8" />
                        </g>
                      ))}
                    </g>
                  )}
                </g>
              ))}
            </g>
          );
        })}
      </svg>

      {/* Selected Trajectory Floating Readout */}
      {selectedId && trajectories.find(t => t.id === selectedId) && (
        <div className="selected-readout panel">
          <div className="section-label">Trajectory Signature</div>
          {(() => {
            const tau = trajectories.find(t => t.id === selectedId);
            return (
              <>
                <div className="readout-item">
                  <span>Selection Score S(τ)</span>
                  <strong>{tau.score?.toFixed(2) || '0.00'}</strong>
                </div>
                <div className="readout-item">
                  <span>Cumulative Margin</span>
                  <strong className={tau.isSelectable ? 'status-pass' : 'status-fail'}>
                    {tau.cumulativeMargin}
                  </strong>
                </div>
                <div className="readout-item">
                  <span>Warn/Penalty Count</span>
                  <strong className={tau.warnCount > 0 ? 'status-warn' : 'status-pass'}>
                    {tau.warnCount}
                  </strong>
                </div>
                <div className="readout-item">
                  <span>Legality Status</span>
                  <strong className={tau.isSelectable ? 'status-pass' : 'status-fail'}>
                    {tau.pathStatus}
                  </strong>
                </div>
              </>
            );
          })()}
        </div>
      )}
    </div>
  );
};

export default TrajectoryGraph;