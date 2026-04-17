# Coh Integrity Dashboard

React/Vite dashboard for inspecting Coh receipt chains, slab summaries, and live verifier responses.

## What It Does

The dashboard has two operating modes:

1. **Fixture mode**: loads bundled JSONL/JSON demo artifacts from `public/demo/` and derives chain-break and slab-summary status in the browser.
2. **Live mode**: optionally posts the loaded chain to the Rust sidecar and displays the returned unified verification response.

This makes the dashboard useful both as a demo surface and as an operator-facing inspection console for local verification flows.

## Features

- **Scenario replay** for valid and invalid receipt chains.
- **Timeline inspection** for step-level continuity, accounting, and digest metadata.
- **Slab reconciliation** against the loaded chain summary.
- **Live sidecar fallback** using the Coh sidecar API when enabled.
- **Frontend test coverage** through Vitest and Testing Library.

## Prerequisites

- Node.js 20+
- npm
- Optional: a running Coh sidecar for live verification mode

## Development Setup

From `coh-dashboard/`:

```bash
npm install
npm run dev
```

## Available Scripts

| Script | Purpose |
|---|---|
| `npm run dev` | Start the Vite development server |
| `npm run build` | Produce a production build |
| `npm run preview` | Preview the production build locally |
| `npm run lint` | Run ESLint |
| `npm run test` | Start Vitest in watch mode |
| `npm run test:run` | Run the test suite once |
| `npm run test:ui` | Open the Vitest UI |

## Live Sidecar Integration

The dashboard defaults to the base URL defined in `src/data/cohData.js`:

```text
http://127.0.0.1:3030
```

That value can be overridden with the Vite environment variable `VITE_COH_SIDECAR_URL`.

To run the sidecar from the repository root:

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-sidecar --release
```

When live verification is enabled in the UI:

- the dashboard loads fixture receipts locally,
- posts them to `POST /v1/verify-chain`,
- and falls back to fixture-derived status if the sidecar is unavailable.

## Demo Scenarios

The scenario catalog currently includes:

| Scenario key | Meaning |
|---|---|
| `valid` | Happy-path valid chain and matching slab summary |
| `invalid_state_link` | Broken state-hash linkage |
| `reject_chain_digest` | Chain-digest mismatch |
| `reject_state_link_adv` | Adversarial state discontinuity |
| `reject_policy_violation` | Accounting-law violation |
| `reject_schema` | Schema or version rejection |
| `reject_numeric_parse` | Malformed numeric input |
| `reject_overflow` | Safe-math overflow rejection |
| `invalid_slab_summary` | Slab summary does not reconcile with the chain |

Fixtures are stored under `public/demo/`, and the scenario definitions live in `src/data/cohData.js`.

## Test and Build

```bash
npm run test:run
npm run build
```

## Relationship to the Rust Workspace

- `coh-dashboard/` is the visualization and operator-inspection layer.
- `coh-node/crates/coh-sidecar/` is the live HTTP verification service.
- `coh-node/vectors/` and `coh-dashboard/public/demo/` provide the bundled demo data used for replay and verification examples.

For the Rust verifier itself, see [`../coh-node/README.md`](../coh-node/README.md).
