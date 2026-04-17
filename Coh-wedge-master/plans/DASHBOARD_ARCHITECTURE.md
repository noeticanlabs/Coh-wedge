# Coh Dashboard Architecture

> Documentation for the React-based verification dashboard

## Overview

The Dashboard is a React application that provides a visual interface for:
- Loading and viewing demo verification scenarios
- Visualizing receipt chains and slab summaries
- Displaying chain break detection
- Switching between fixture mode and live sidecar verification

---

## Component Structure

```
App.jsx (main)
├── Header component
├── Scenario selector (dropdown)
├── Dashboard display
│   ├── Overview cards (4)
│   ├── Chain viewer (scrollable)
│   ├── Slab viewer
│   ├── Break details
│   └── Verification status
└── Footer
```

---

## Data Flow

### 1. Scenario Selection

User selects a scenario from the dropdown → triggers `loadDashboardData()`

### 2. Fixture Loading

```
Scenario SCENARIO → chainPath + slabPath 
                        ↓
              load from /demo/*.jsonl|*.json
                        ↓
              parseJsonLines() / JSON.parse()
```

### 3. Data Processing

```
Receipts → normalizeStep() → chainSteps[]
                                     ↓
                    deriveChainBreak() → breakInfo
                                     ↓
                    deriveSlabCheck() → slabCheck

Slab → normalizeSlab() → normalizedSlab
```

### 4. Verification

- **Fixture mode**: Derives verification result from data (no live call)
- **Live mode**: Calls sidecar API (`/v1/verify-chain`)

---

## Key Functions (cohData.js)

### parseJsonLines(text)

Parses JSONL (JSON Lines) into array of objects.

```javascript
function parseJsonLines(text) {
    return text.split(/\r?\n/)
        .filter(line => line.trim())
        .map(line => JSON.parse(line));
}
```

### normalizeStep(receipt, index, receipts, breakInfo)

Normalizes a single receipt to display format with:
- Continuity analysis (state_label, digest_label)
- Metrics calculation (isAdmissible)

### deriveChainBreak(normalizedReceipts)

Detects deterministic chain breaks:
- Policy violation
- State link failure
- Chain digest mismatch

### deriveSlabCheck(receipts, slab)

Validates slab summary against actual chain totals.

---

## Scenarios

| Key | Label | Description |
|-----|-------|-------------|
| valid | Valid Chain | Happy-path with consistent summary |
| invalid_state_link | Broken State Link | State continuity failure |
| reject_chain_digest | Chain Digest Mismatch | Cryptographic linkage failure |
| reject_state_link_adv | State Discontinuity | Adversarial state transition |
| reject_policy_violation | Policy Violation | v_post + spend > v_pre + defect |
| reject_schema | Schema Reject | Unsupported schema/version |
| reject_numeric_parse | Malformed Numeric | Invalid hex/numeric format |
| reject_overflow | Arithmetic Overflow | u128 overflow |
| invalid_slab_summary | Invalid Slab Summary | Chain/slab mismatch |

---

## Scenario Configuration (SCENARIOS)

```javascript
const SCENARIOS = {
  valid: {
    key: 'valid',
    label: 'Valid Chain',
    description: 'Happy-path receipt chain...',
    chainPath: '/demo/ai_workflow_chain_valid.jsonl',
    slabPath: '/demo/ai_workflow_slab_valid.json',
  },
  // ... more scenarios
};
```

---

## App State

```javascript
const [scenarioKey, setScenarioKey] = useState('valid');
const [preferLiveVerification, setPreferLiveVerification] = useState(false);
const [reloadTick, setReloadTick] = useState(0);
const [selectedIdx, setSelectedIdx] = useState(0);
const [dashboardData, setDashboardData] = useState(null);
const [loading, setLoading] = useState(true);
const [loadError, setLoadError] = useState(null);
```

---

## Dashboard Data Structure

Loaded via `loadDashboardData()` returns:

```typescript
interface DashboardData {
  scenario: Scenario;
  chainSteps: NormalizedStep[];
  verification: VerificationResult;
  slab: SlabReceipt;
  slabCheck: SlabCheckResult;
  breakInfo: ChainBreak | null;
  isTrusted: boolean;
  liveError: string | null;
  sidecarBaseUrl: string;
}
```

---

## Demo Files (public/demo/)

| File | Purpose |
|------|---------|
| `ai_workflow_chain_valid.jsonl` | Valid demo chain |
| `ai_workflow_slab_valid.json` | Valid demo slab |
| `reject_policy_violation.jsonl` | Policy violation |
| `reject_state_link.jsonl` | State link failure |
| `reject_chain_digest.jsonl` | Digest mismatch |
| `reject_schema.jsonl` | Schema rejection |
| `reject_numeric_parse.jsonl` | Parse error |
| `reject_overflow.jsonl` | Arithmetic error |

---

## Live vs Fixture Mode

### Fixture Mode (default)

- Loads data from `/demo/*.json` files
- Derives verification result algorithmically
- No network calls

### Live Mode

- Calls sidecar API (`POST /v1/verify-chain`)
- Requires sidecar running at `VITE_COH_SIDECAR_URL` or `http://127.0.0.1:3030`
- Real-time verification

---

## Visual Components

### Overview Cards (4)

1. **Chain Steps** - Number of receipts loaded
2. **Break Index** - Where break detected (or "—")
3. **Slab Check** - PASS/FAIL
4. **Validator** - Coh version + source

### Chain Viewer

Displays each receipt with:
- Step number
- Status (✓ valid, ✗ rejected)
- Metrics summary

### Break Details

Shows:
- Break type (T1-T6)
- Type label
- Message

---

## UI/UX Libraries

- **React 18** - UI framework
- **Framer Motion** - Animations
- **Lucide React** - Icons
- **Tailwind CSS** - Styling

---

## Integration with Core

The dashboard uses the same `cohData.js` functions that mirror the Rust verification:

| Dashboard Function | Rust Equivalent |
|--------------------|-------------------|
| parseJsonLines | MicroReceiptWire parsing |
| normalizeStep | verify_micro |
| deriveChainBreak | verify_chain |
| deriveSlabCheck | verify_slab |
| normalizeSlab | SlabReceipt conversion |

This ensures visual verification matches CLI/sidecar behavior.

Reference: `plans/RECEIPT_SCHEMA_SPEC.md`, `plans/ERROR_REJECT_CONTRACT.md`