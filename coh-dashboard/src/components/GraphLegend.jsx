import React from 'react';

export default function GraphLegend() {
    return (
        <div className="graph-legend" style={{
            display: 'flex',
            gap: 16,
            padding: '8px 12px',
            background: '#f8fafc',
            borderTop: '1px solid #e2e8f0',
            fontSize: 11,
            color: '#475569',
        }}>
            <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                <span style={{ fontWeight: 'bold', marginRight: 4 }}>Path states:</span>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 2 }}>
                    <span style={{ width: 10, height: 10, borderRadius: '50%', background: '#22c55e' }} />
                    Valid
                </span>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 2 }}>
                    <span style={{ width: 10, height: 10, borderRadius: '50%', background: '#fef3c7', border: '1px solid #f59e0b' }} />
                    Constrained
                </span>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 2 }}>
                    <span style={{ width: 10, height: 10, borderRadius: '50%', background: '#ef4444' }} />
                    Rejected
                </span>
            </div>

            <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                <span style={{ fontWeight: 'bold', marginRight: 4 }}>Constraints:</span>
                <span>C1 Schema</span>
                <span>C2 Sigs</span>
                <span>C3 Profile</span>
                <span>C4 State</span>
                <span>C5 Digest</span>
                <span>C6 Policy</span>
            </div>

            <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                <span style={{ fontWeight: 'bold', marginRight: 4 }}>Highlight:</span>
                <span>★ = Selected trajectory (max score)</span>
            </div>
        </div>
    );
}