import React from 'react';
import { motion } from 'framer-motion';
import { Bot } from 'lucide-react';

export default function AdapterItem({
  name,
  subtitle,
  statusLabel,
  isActive,
  primaryLabel = 'Items',
  primaryValue = '—',
  secondaryLabel = 'Digest',
  secondaryValue = '—',
  note,
}) {
  return (
    <motion.div
      layout
      className={`adapter-card ${isActive ? 'is-active' : 'is-inactive'}`}
    >
      <div className="adapter-topline">
        <div className="adapter-identity">
          <div className={`adapter-icon ${isActive ? 'is-active' : ''}`}>
            <Bot size={20} />
          </div>
          <div>
            <h3>{name}</h3>
            <div className="adapter-status-row">
              <span className={`status-dot ${isActive ? 'is-active' : ''}`} />
              <span>{statusLabel}</span>
            </div>
            {subtitle ? <div className="adapter-subtitle">{subtitle}</div> : null}
          </div>
        </div>
      </div>

      <div className="adapter-metrics">
        <div>
          <span>{primaryLabel}</span>
          <strong>{primaryValue}</strong>
        </div>
        <div>
          <span>{secondaryLabel}</span>
          <strong>{secondaryValue}</strong>
        </div>
      </div>

      <div className="adapter-footer-note">{note}</div>
    </motion.div>
  );
}
