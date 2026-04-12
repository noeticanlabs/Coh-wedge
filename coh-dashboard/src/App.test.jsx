import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import App from './App';

// Mock the components/AdapterItem to simplify
vi.mock('./components/AdapterItem', () => ({
  default: ({ name, statusLabel }) => (
    <div data-testid="adapter-item">
      <span>{name}</span>
      <span>{statusLabel}</span>
    </div>
  ),
}));

// Mock the cohData to avoid actual fetch calls
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
      metrics: { vPre: '100', vPost: '90', spend: '10', defect: '0', isAdmissible: true, leftSide: '100', rightSide: '100' },
      continuity: { stateLabel: 'Continuous', digestLabel: 'Verified' },
      objectId: 'test-obj',
      schemaId: 'coh.receipt.micro.v1',
      version: '1.0.0',
      canonProfileHash: 'hash-123',
      stateHashPrev: 'prev-123',
      stateHashNext: 'next-123',
      chainDigestPrev: 'dprev-123',
      chainDigestNext: 'dnext-123',
    },
    {
      id: 'step-1',
      stepIndex: 1,
      hash: 'def1234567890',
      status: 'TAMPERED',
      metrics: { vPre: '90', vPost: '100', spend: '10', defect: '0', isAdmissible: false, leftSide: '110', rightSide: '90' },
      continuity: { stateLabel: 'Continuous', digestLabel: 'Verified' },
      objectId: 'test-obj',
      schemaId: 'coh.receipt.micro.v1',
      version: '1.0.0',
      canonProfileHash: 'hash-123',
      stateHashPrev: 'next-123',
      stateHashNext: 'next-456',
      chainDigestPrev: 'dnext-123',
      chainDigestNext: 'dnext-456',
    }
  ],
  isTrusted: false,
  verification: { status: 'REJECT', cohVersion: '0.1.0', source: 'fixture', requestId: 'req-123' },
  slab: { objectId: 'slab-123', microCount: 2, merkleRoot: 'merkle-123' },
  slabCheck: { isValid: true, message: 'Slab verified' },
  breakInfo: { stepIndex: 1, typeLabel: 'Policy Violation', message: 'Math bad', expected: '90', actual: '110' },
};

describe('App Behavioral Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders correctly and shows scenario info', async () => {
    loadDashboardData.mockResolvedValueOnce(mockDashboardData);
    
    render(<App />);
    
    // Wait for loading to finish
    await waitFor(() => expect(screen.queryByText(/Loading AI demo data/i)).not.toBeInTheDocument());

    expect(screen.getByText(/Integrity Operations Dashboard/i)).toBeInTheDocument();
    expect(screen.getByText(/Attention Required/i)).toBeInTheDocument();
    expect(screen.getByText(/Policy Violation/i)).toBeInTheDocument();
  });

  it('handles scenario selection change', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);
    
    render(<App />);
    await waitFor(() => expect(screen.queryByText(/Loading AI demo data/i)).not.toBeInTheDocument());

    const select = screen.getByLabelText(/Scenario/i);
    fireEvent.change(select, { target: { value: 'reject_policy_violation' } });

    expect(loadDashboardData).toHaveBeenCalledWith(expect.objectContaining({
      scenarioKey: 'reject_policy_violation'
    }));
  });

  it('updates inspector when a timeline step is clicked', async () => {
    loadDashboardData.mockResolvedValueOnce(mockDashboardData);
    
    render(<App />);
    await waitFor(() => expect(screen.queryByText(/Loading AI demo data/i)).not.toBeInTheDocument());

    // Click step #1
    const step1Button = screen.getByRole('button', { name: /#1/i });
    fireEvent.click(step1Button);

    // Inspector should now show step #1 metrics
    const inspector = screen.getByText(/Audit inspector/i).closest('.panel');
    expect(inspector).toHaveTextContent(/v_post100/i); // simplified match for kv-list
    expect(inspector).toHaveTextContent(/Policy violated/i);
  });

  it('toggles live verification mode', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);
    
    render(<App />);
    await waitFor(() => expect(screen.queryByText(/Loading AI demo data/i)).not.toBeInTheDocument());

    const liveToggle = screen.getByText(/Enable live verify/i);
    fireEvent.click(liveToggle);

    expect(loadDashboardData).toHaveBeenCalledWith(expect.objectContaining({
      preferLiveVerification: true
    }));
    
    expect(screen.getByText(/Live verify enabled/i)).toBeInTheDocument();
  });
});
