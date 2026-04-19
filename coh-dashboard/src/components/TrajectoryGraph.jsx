import React, { useMemo } from 'react';

const WITNESS_LABELS = ['C1', 'C2', 'C3', 'C4', 'C5', 'C6'];
const CONSTRAINT_LABELS = {
  C1: 'Schema',
  C2: 'Sigs',
  C3: 'Profile',
  C4: 'State',
  C5: 'Digest',
  C6: 'Policy'
};

const TrajectoryGraph = ({ candidates, selectedId, onSelect }) => {
  // Dense line probe layout - like an ECG/signal trace
  const width = 800;
  const height = 280;
  const padding = 40;

  const maxDepth = Math.max(...candidates.map(c => c.receipts?.length || 0), 1);
  const stepX = (width - padding * 2) / Math.max(maxDepth, 1);

  // Dense probe rendering
  const trajectories = useMemo(() => {
    return candidates.map(tau => {
      const isSelected = tau.id === selectedId;

      // Generate dense line points - continuous signal trace
      const points = [];
      const numPoints = tau.receipts?.length || 1;

      for (let i = 0; i < numPoints; i++) {
        const x = padding + i * stepX;
        // Generate y based on trajectory evaluation
        // Safety Bottleneck is the primary positioning driver
        const baseY = height * 0.7; // Lower baseline for safety-critical UI
        const safetyFactor = (tau.evaluation?.safetyBottleneck || 0);
        const alignmentFactor = (tau.evaluation?.alignment || 0);
        
        // y decreases as safety and alignment increase (higher on graph)
        const y = baseY - (safetyFactor * 80) - (alignmentFactor * 40);

        points.push({ x, y });
      }

      // Add intermediate points for smooth line (deterministic jitter using trajectory ID)
      const densePoints = [];
      const jitterSeed = tau.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0);
      for (let i = 0; i < points.length - 1; i++) {
        densePoints.push(points[i]);
        // Interpolate midpoint with deterministic jitter based on position
        const midX = (points[i].x + points[i + 1].x) / 2;
        const deterministicJitter = ((jitterSeed + i * 17) % 20) - 10;
        const midY = (points[i].y + points[i + 1].y) / 2 + deterministicJitter * 0.4;
        densePoints.push({ x: midX, y: midY });
      }
      if (points.length > 0) densePoints.push(points[points.length - 1]);

      return {
        ...tau,
        points: densePoints,
        originalPoints: points,
        isSelected
      };
    });
  }, [candidates, selectedId, stepX]);

  return (
    <div className="trajectory-viewport" style={{ background: 'var(--bg-base)', borderRadius: 'var(--radius-sm)' }}>
      <svg width="100%" height="100%" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="xMidYMid meet">
        <defs>
          <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="var(--brand-primary)" stopOpacity="0.3" />
            <stop offset="50%" stopColor="var(--brand-primary)" stopOpacity="0.8" />
            <stop offset="100%" stopColor="var(--brand-primary)" stopOpacity="0.3" />
          </linearGradient>
          <filter id="probe-glow">
            <feGaussianBlur stdDeviation="2" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Grid lines for depth markers */}
        {Array.from({ length: maxDepth + 1 }).map((_, i) => (
          <line
            key={`grid-${i}`}
            x1={padding + i * stepX}
            y1={height * 0.1}
            x2={padding + i * stepX}
            y2={height * 0.9}
            stroke="var(--border-muted)"
            strokeWidth="0.5"
            strokeDasharray="2,4"
            opacity="0.3"
          />
        ))}

        {/* Time axis label */}
        <text x={width / 2} y={height - 8} textAnchor="middle" fontSize="9" fill="var(--text-muted)">
          Execution Timeline (depth →
        </text>

        {trajectories.map(tau => {
          const { points, originalPoints, isSelectable, isSelected, firstFailureIndex, score } = tau;

          // Determine truncation point
          const truncateAt = !isSelectable && firstFailureIndex != null
            ? Math.min(firstFailureIndex + 1, points.length)
            : points.length;

          const visiblePoints = points.slice(0, truncateAt);
          const hasFailure = !isSelectable && firstFailureIndex != null;

          // Line color based on validity
          const lineColor = isSelected
            ? 'var(--brand-primary)'
            : hasFailure
              ? 'var(--brand-blocked)'
              : 'var(--text-muted)';

          return (
            <g key={tau.id} onClick={() => onSelect(tau.id)} style={{ cursor: 'pointer' }}>
              {/* Dense line probe - continuous signal */}
              <polyline
                points={visiblePoints.map(p => `${p.x},${p.y}`).join(' ')}
                fill="none"
                stroke={lineColor}
                strokeWidth={isSelected ? 2.5 : 1}
                strokeLinecap="round"
                strokeLinejoin="round"
                opacity={isSelected ? 1 : 0.4}
                filter={isSelected ? 'url(#probe-glow)' : 'none'}
                style={{ transition: 'all 0.3s ease' }}
              />

              {/* Failure indicator - spike down */}
              {hasFailure && firstFailureIndex < originalPoints.length && (
                <g transform={`translate(${originalPoints[firstFailureIndex].x}, ${originalPoints[firstFailureIndex].y})`}>
                  <line x1="0" y1="0" x2="0" y2={height * 0.3} stroke="var(--brand-blocked)" strokeWidth="2" strokeDasharray="4,2" />
                  <circle r="4" fill="var(--brand-blocked)" opacity="0.8" />
                </g>
              )}

              {/* Selected trajectory - show constraint badges at each probe point */}
              {isSelected && originalPoints.map((p, i) => {
                const nodeWitnesses = tau.witnesses?.[i] || {};
                const failedConstraint = WITNESS_LABELS.find(c => nodeWitnesses[c]?.status === 'fail');

                return (
                  <g key={`probe-${i}`} transform={`translate(${p.x}, ${p.y})`}>
                    {/* Probe point marker */}
                    <circle
                      r={3}
                      fill={failedConstraint ? 'var(--brand-blocked)' : 'var(--brand-primary)'}
                      stroke="var(--bg-base)"
                      strokeWidth="1"
                    />

                    {/* Constraint status bar below probe */}
                    <g transform="translate(0, 12)">
                      {WITNESS_LABELS.map((c, ci) => {
                        const status = nodeWitnesses[c]?.status || 'unknown';
                        const isFail = status === 'fail';
                        const isPass = status === 'pass';
                        return (
                          <rect
                            key={c}
                            x={(ci - 2.5) * 8}
                            y="0"
                            width="6"
                            height="3"
                            rx="1"
                            fill={isFail ? 'var(--brand-blocked)' : isPass ? 'var(--brand-primary)' : 'var(--text-muted)'}
                            opacity={isFail ? 1 : isPass ? 0.7 : 0.3}
                          />
                        );
                      })}
                    </g>
                  </g>
                );
              })}

              {/* Score label for selected */}
              {isSelected && tau.evaluation && (
                <g transform={`translate(${padding - 10}, ${originalPoints[0]?.y || height / 2})`}>
                  <text
                    textAnchor="end"
                    fontSize="8"
                    fill="var(--brand-primary)"
                    fontWeight="bold"
                  >
                    S:{tau.evaluation.safetyBottleneck.toFixed(2)}
                  </text>
                  <text
                    y="10"
                    textAnchor="end"
                    fontSize="8"
                    fill="var(--text-secondary)"
                  >
                    A:{tau.evaluation.alignment.toFixed(2)}
                  </text>
                </g>
              )}
            </g>
          );
        })}

        {/* Legend overlay */}
        <g transform={`translate(${width - 120}, 20)`}>
          <rect x="0" y="0" width="110" height="50" rx="4" fill="var(--bg-glass)" stroke="var(--border-muted)" />
          <text x="8" y="14" fontSize="7" fontWeight="bold" fill="var(--text-secondary)">LINE PROBE</text>
          <line x1="8" y1="24" x2="30" y2="24" stroke="var(--brand-primary)" strokeWidth="2" />
          <text x="34" y="27" fontSize="7" fill="var(--text-muted)">Selected</text>
          <line x1="8" y1="36" x2="30" y2="36" stroke="var(--brand-blocked)" strokeWidth="1.5" strokeDasharray="3,2" />
          <text x="34" y="39" fontSize="7" fill="var(--text-muted)">Blocked</text>
        </g>
      </svg>
    </div>
  );
};

export default TrajectoryGraph;