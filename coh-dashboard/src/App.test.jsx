import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import App from './App';

// Mock components to simplify
vi.mock('./components/AdapterItem', () => ({
  default: ({ name, statusLabel }) => (
    <div data-testid="adapter-item">
      <span>{name}</span>
      <span>{statusLabel}</span>
    </div>
  ),
}));

// Mock cohData
vi.mock('./data/cohData', async () => {
  const actual = await vi.importActual('./data/cohData');
  return {
    ...actual,
    loadDashboardData: vi.fn(),
  };
});

import { loadDashboardData } from './data/cohData';

const mockDashboardData = {
  scenario: { label: 'Test Scenario', description: 'Test Description' },
  chainSteps: [
    {
      id: 'step-0',
      stepIndex: 0,
      hash: 'abc1234567890',
      status: 'TRUSTED',
      metrics: { 
        vPre: '100', vPost: '90', spend: '10', defect: '0', 
        isAdmissible: true, leftSide: '100', rightSide: '100',
        violationDelta: 0
      },
      continuity: { stateLabel: 'Continuous', digestLabel: 'Verified' },
      objectId: 'test-obj',
    }
  ],
  candidates: [
      {
          id: 'traj-1',
          steps: [],
          evaluation: {
              safetyBottleneck: 0.95,
              alignment: 0.8,
              normalizedCost: 0.1
          },
          isSelectable: true
      }
  ],
  isTrusted: true,
  verification: { status: 'ACCEPT', cohVersion: '0.1.0', source: 'sidecar', requestId: 'req-123' },
  slab: { objectId: 'slab-123', microCount: 1, merkleRoot: 'merkle-123' },
  slabCheck: { isValid: true, message: 'Slab verified' },
};

describe('App Behavioral Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders correctly and shows scenario info', async () => {
    loadDashboardData.mockResolvedValueOnce(mockDashboardData);

    render(<App />);

    // Wait for the new hardened loading state to finish
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // New Branding Assertions
    expect(screen.getByText(/Deterministic Execution Verification/i)).toBeInTheDocument();
    
    // Check for grounded metrics in Evidence Panel (rendered via evaluation mock)
    expect(screen.getByText(/0.95/)).toBeInTheDocument(); // safetyBottleneck
  });

  it('handles scenario selection change via CI markers', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    const select = screen.getByLabelText(/Scenario/i); // Matches the hidden label
    fireEvent.change(select, { target: { value: 'reject_policy_violation' } });

    expect(loadDashboardData).toHaveBeenCalledWith(expect.objectContaining({
      scenarioKey: 'reject_policy_violation'
    }));
  });
});
