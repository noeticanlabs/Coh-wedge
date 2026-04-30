# Dashboard Integration

The Coh Safety Wedge includes a React/Vite dashboard (`coh-dashboard/`) for inspecting Coh receipt chains, slab summaries, and interacting with the live verifier via the Sidecar API.

## Operating Modes

The dashboard supports two primary modes of operation:

### 1. Fixture Mode (Offline)
In Fixture Mode, the dashboard loads bundled JSONL/JSON demo artifacts directly from its `public/demo/` directory. It parses these files and derives chain-break and slab-summary status entirely within the browser. This is useful for demonstrating the UI and for reviewing static, pre-calculated workflows without needing the Rust backend running.

### 2. Live Mode (Online)
In Live Mode, the dashboard posts the loaded chain or receipt to the active Rust sidecar (`coh-sidecar`) and displays the real-time, unified verification response from the actual Verifier Kernel.

## Live Sidecar Integration

The dashboard communicates with the `coh-sidecar` service. By default, it expects the sidecar to be available at:

```text
http://127.0.0.1:3030
```

This can be overridden by setting the `VITE_COH_SIDECAR_URL` environment variable during the dashboard's build or dev server startup.

### Starting the Sidecar

To enable Live Mode, start the sidecar from the repository root:

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-sidecar --release
```

### API Endpoints Used

The dashboard relies on the following endpoints exposed by `coh-sidecar`:

- `GET /health` : Used to detect if the sidecar is online and toggle Live Mode availability.
- `POST /v1/verify-micro` : Used to verify individual micro-receipts (V1, V2, or V3 schemas).
- `POST /v1/verify-chain` : Used to verify a contiguous chain of receipts in a single payload.

When Live Mode is active, the dashboard will attempt to use these endpoints. If the sidecar becomes unavailable, the dashboard will gracefully fall back to Fixture Mode (client-side derived status).

## Scenario Catalog

The dashboard comes with a built-in scenario catalog (defined in `src/data/cohData.js`) that maps to fixtures in `public/demo/`. These cover various acceptance and rejection cases:

*   **`valid`**: A happy-path valid chain and matching slab summary.
*   **`invalid_state_link`**: A chain where the state-hash linkage is broken.
*   **`reject_chain_digest`**: A chain where the cryptographic digest mismatch occurs.
*   **`reject_policy_violation`**: A chain that violates the Law of Coherence (spending exceeds allowed variance).
*   **`reject_schema`**: A receipt with an invalid schema or version.
*   **`reject_numeric_parse`**: A receipt containing malformed numeric inputs.
*   **`reject_overflow`**: A receipt that triggers a safe-math overflow.
*   **`invalid_slab_summary`**: A slab summary that does not correctly reconcile with its underlying chain.

## Data Flow

1. **Selection:** User selects a scenario from the UI.
2. **Load:** Dashboard fetches the corresponding `.jsonl` or `.json` file from `public/demo/`.
3. **Dispatch:**
    *   If **Live Mode** is on, the dashboard sends the payload to `http://127.0.0.1:3030/v1/verify-chain` (or `verify-micro`).
    *   If **Fixture Mode** is on (or sidecar is unreachable), the dashboard parses the JSON and runs lightweight, client-side validation logic to determine the UI state.
4. **Render:** The UI updates the timeline, highlights the accepted steps, and marks the exact step where a rejection occurred (if any), displaying the precise `RejectCode`.
