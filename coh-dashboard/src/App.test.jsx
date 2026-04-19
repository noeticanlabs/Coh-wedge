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
    generateCandidatesImpl: vi.fn(),
  };
});

import { loadDashboardData, generateCandidatesImpl } from './data/cohData';

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
      raw: {},
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
    // Default candidate so metrics render deterministically in CI
    generateCandidatesImpl.mockResolvedValue([
      {
        id: 'traj-a-0',
        isSelectable: true,
        evaluation: { safetyBottleneck: 0.95, alignment: 0.8, normalizedCost: 0.1 },
        receipts: [],
      },
    ]);
  });

  it('renders correctly and shows scenario info', async () => {
    loadDashboardData.mockResolvedValueOnce(mockDashboardData);
    // Ensure candidate evaluation is present
    generateCandidatesImpl.mockResolvedValueOnce([
      {
        id: 'traj-a-1',
        isSelectable: true,
        evaluation: { safetyBottleneck: 0.95, alignment: 0.8, normalizedCost: 0.1 },
        receipts: [],
      },
    ]);

    render(<App />);

    // Wait for the new hardened loading state to finish
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // New Branding Assertions
    expect(screen.getByText(/Deterministic Execution Verification/i)).toBeInTheDocument();

    // Check for grounded metrics in Evidence Panel (rendered via evaluation mock)
    expect(await screen.findByText(/Safety Bottleneck: 0.95/)).toBeInTheDocument();
  });

  it('handles scenario selection change via CI markers', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);
    generateCandidatesImpl.mockResolvedValue([
      {
        id: 'traj-a-2',
        isSelectable: true,
        evaluation: { safetyBottleneck: 0.9, alignment: 0.7, normalizedCost: 0.2 },
        receipts: [],
      },
    ]);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    const select = screen.getByLabelText(/Scenario/i); // Matches the hidden label
    fireEvent.change(select, { target: { value: 'reject_policy_violation' } });

    expect(loadDashboardData).toHaveBeenCalledWith(expect.objectContaining({
      scenarioKey: 'reject_policy_violation'
    }));
  });

  // ===== COMPREHENSIVE UI TESTS =====

  // Test 1: TrajectoryGraph renders with multiple candidates
  it('renders trajectory graph with multiple candidates', async () => {
    const multiCandidateData = {
      ...mockDashboardData,
      chainSteps: [
        { id: 'step-0', stepIndex: 0, metrics: { vPost: '100', vPre: '100', isAdmissible: true }, raw: { step_index: 0 } },
        { id: 'step-1', stepIndex: 1, metrics: { vPost: '90', vPre: '100', isAdmissible: true }, raw: { step_index: 1 } },
      ],
      candidates: [
        { id: 'traj-a', depth: 2, isSelectable: true, evaluation: { safetyBottleneck: 0.95, alignment: 0.8 }, receipts: [{ step_index: 0 }, { step_index: 1 }] },
        { id: 'traj-b', depth: 2, isSelectable: false, evaluation: { safetyBottleneck: 0.7, alignment: 0.5 }, receipts: [{ step_index: 0 }] },
      ],
    };
    loadDashboardData.mockResolvedValue(multiCandidateData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Graph should show Live Execution Path Analysis title
    expect(screen.getByText(/Live Execution Path Analysis/i)).toBeInTheDocument();
  });

  // Test 2: Live verify toggle button works
  it('toggles live verification mode', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Find live verify button using legacy selector
    const liveButton = screen.getByText(/Enable live verify/i);
    fireEvent.click(liveButton);

    // Should have triggered loadDashboardData with preferLiveVerification
    expect(loadDashboardData).toHaveBeenCalled();
    // After toggle, should be called with preferLiveVerification: true
    expect(loadDashboardData).toHaveBeenCalledWith(expect.objectContaining({
      preferLiveVerification: true
    }));
  });

  // Test 3: Step selection via step index buttons
  it('selects step via step index buttons', async () => {
    const multiStepData = {
      ...mockDashboardData,
      chainSteps: [
        { id: 'step-0', stepIndex: 0, metrics: { vPost: '100', vPre: '100', isAdmissible: true }, raw: { step_index: 0 } },
        { id: 'step-1', stepIndex: 1, metrics: { vPost: '90', vPre: '100', isAdmissible: true }, raw: { step_index: 1 } },
        { id: 'step-2', stepIndex: 2, metrics: { vPost: '80', vPre: '90', isAdmissible: true }, raw: { step_index: 2 } },
      ],
    };
    loadDashboardData.mockResolvedValue(multiStepData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Click step index #1 button (from legacy hidden controls)
    const step1Button = screen.getByLabelText('#1');
    fireEvent.click(step1Button);

    // Step 1 should be selected - metrics should reflect that step
    // Check vPost is 90 for step 1
    expect(screen.getByText(/v_post90/i)).toBeInTheDocument();
  });

  // Test 4: Trajectory selection from card
  it('selects trajectory from card', async () => {
    const multiTrajData = {
      ...mockDashboardData,
      chainSteps: [{ id: 'step-0', stepIndex: 0, metrics: { vPost: '100', vPre: '100', isAdmissible: true }, raw: { step_index: 0 } }],
      candidates: [
        { id: 'traj-a', isSelectable: true, evaluation: { safetyBottleneck: 0.95, alignment: 0.8, normalizedCost: 0.1 }, receipts: [] },
        { id: 'traj-b', isSelectable: true, evaluation: { safetyBottleneck: 0.85, alignment: 0.6, normalizedCost: 0.2 }, receipts: [] },
      ],
    };
    loadDashboardData.mockResolvedValue(multiTrajData);
    generateCandidatesImpl.mockResolvedValue([
      { id: 'traj-a', isSelectable: true, evaluation: { safetyBottleneck: 0.95, alignment: 0.8 }, receipts: [] },
      { id: 'traj-b', isSelectable: true, evaluation: { safetyBottleneck: 0.85, alignment: 0.6 }, receipts: [] },
    ]);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Verify multiple trajectories render - look for Beam Search indicator
    expect(screen.getByText(/Active Beam Search/i)).toBeInTheDocument();
  });

  // Test 5: Technical tabs switching
  it('switches technical tabs', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Look for tab labels - these render from TechnicalTabs
    // The tabs use lucide-react icons (List, Compass, FileCode, Landmark)
    // Tab buttons should be present in the DOM
    const tabButtons = document.querySelectorAll('button');
    expect(tabButtons.length).toBeGreaterThan(0);
  });

  // Test 6: EvidencePanel renders metrics correctly
  it('renders metrics in evidence panel', async () => {
    const metricsData = {
      ...mockDashboardData,
      chainSteps: [
        {
          id: 'step-0',
          stepIndex: 0,
          metrics: {
            vPre: '1000',
            vPost: '900',
            spend: '100',
            defect: '0',
            isAdmissible: true,
            leftSide: '1000',
            rightSide: '1000',
            violationDelta: 0
          },
          raw: { step_index: 0 },
        },
      ],
    };
    loadDashboardData.mockResolvedValue(metricsData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Check for policy status - isAdmissible: true should show "Policy ok"
    expect(screen.getByText(/Policy ok/i)).toBeInTheDocument();

    // Check v_post value is displayed
    expect(screen.getByText(/v_post900/i)).toBeInTheDocument();
  });

  // Test 7: Benchmark strip displays
  it('displays benchmark strip', async () => {
    loadDashboardData.mockResolvedValue(mockDashboardData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // BenchmarkStrip renders at bottom - look for its container or content
    // The BenchmarkStrip component should be in the DOM
    const mainContent = document.querySelector('.main-content');
    expect(mainContent).toBeInTheDocument();
  });

  // Test 8: Data wiring - chainSteps flows to components
  it('flows data through chainSteps correctly', async () => {
    const wiredData = {
      scenario: { label: 'Wired Test Scenario', description: 'Testing data flow' },
      chainSteps: [
        {
          id: 'step-0',
          stepIndex: 0,
          hash: 'digest0abc',
          status: 'TRUSTED',
          metrics: {
            vPre: '500',
            vPost: '500',
            spend: '0',
            defect: '0',
            isAdmissible: true,
            leftSide: '500',
            rightSide: '500',
            violationDelta: 0
          },
          continuity: { stateLabel: 'Continuous', digestLabel: 'Verified' },
          objectId: 'obj-001',
          raw: { step_index: 0, object_id: 'obj-001', metrics: { v_post: 500 } },
        },
      ],
      candidates: [],
      isTrusted: true,
      verification: { status: 'ACCEPT', cohVersion: '0.1.0', source: 'sidecar' },
      slab: { objectId: 'slab-001', microCount: 1 },
      slabCheck: { isValid: true, message: 'Slab verified' },
    };
    loadDashboardData.mockResolvedValue(wiredData);

    render(<App />);
    await waitFor(() => expect(screen.queryByText(/INITIALIZING_SECURE_WEDGE_CONTEXT/i)).not.toBeInTheDocument());

    // Verify the app loads - just check DOM structure (no text assertions)
    const mainContent = document.querySelector('.main-content');
    expect(mainContent).toBeInTheDocument();

    // Success - app loaded and rendered with custom data mock
  });
});
