# OPERATIONAL_ARCHITECTURE.md - Content Outline

## 1. Executive Summary

One-paragraph overview of deployment options, emphasizing the standalone nature of the Coh verification kernel and its operational simplicity.

## 2. Deployment Modes

### 2.1 Standalone CLI

Direct binary execution for local verification:

```
coh-validator verify-micro <receipt.json>
coh-validator verify-chain <chain.jsonl>
coh-validator verify-slab <slab.json>
```

**Characteristics:**
- No external dependencies
- Binary distribution
- Local state only
- Exit codes map to ACCEPT/REJECT

### 2.2 Sidecar Service (HTTP)

HTTP API for integration with agent frameworks:

```
POST /verifyMicro
POST /verifyChain  
POST /verifySlab
POST /buildSlab
```

**Characteristics:**
- JSON request/response
- Optional request signing
- Configurable policy cache
- Integrates with AI agents

### 2.3 Embedded Library

Direct Rust/Python API for deep integration:

```rust
let result = coh_core::verify_micro(receipt, policy);
```

```python
from coh_python import verify
result = verify(receipt, policy)
```

**Characteristics:**
- Zero-copy verification
- Custom state management
- High-performance workloads
- Language bindings

## 3. Component Architecture

### 3.1 Core Components

| Component | Responsibility | Dependencies |
|-----------|---------------|--------------|
| `coh-cli` | CLI entry point | None |
| `coh-core` | Verification engine | None (pure) |
| `coh-sidecar` | HTTP server | `coh-core` |
| `coh-python` | Python bindings | `coh-core` |
| `ape` | Proposal generation | `coh-core` |

### 3.2 Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Data Flow                                │
│                                                              │
│  [Input: Receipt/Chain/Slab]                                │
│            ↓                                                │
│  [Schema Validation] ──REJECT──→ [Error Output]             │
│            ↓                                                │
│  [Canon Validation] ──REJECT──→ [Error Output]             │
│            ↓                                                │
│  [Chain Continuity] ──REJECT──→ [Error Output]             │
│            ↓                                                │
│  [Accounting Law] ──REJECT──→ [Error Output]              │
│            ↓                                                │
│  [Policy Check] ───REJECT───→ [Error Output]               │
│            ↓                                                │
│  [ACCEPT] ──────────────→ [Execution Layer]               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 4. Configuration

### 4.1 Policy Configuration

JSON-based policy definitions:

```json
{
  "policy_id": "default",
  "max_spend_per_action": 1000,
  "max_chain_length": 100,
  "authority_renewal": "periodic"
}
```

### 4.2 Canon Profile Configuration

Determines canonical hashing behavior:

```json
{
  "profile_id": "coh.default",
  "domain_tag": "COH_V1",
  "hash_algo": "sha256"
}
```

## 5. Operational Failure Modes

### 5.1 Input Failures

| Failure | Cause | Resolution |
|---------|-------|------------|
| Malformed JSON | Input parse error | Validate input format |
| Schema mismatch | Wrong schema_id | Use correct schema |
| Version too old | Deprecated version | Upgrade to current |

### 5.2 Verification Failures (REJECT)

| Failure | Cause | Resolution |
|---------|-------|------------|
| RejectSchema | Invalid schema | Fix schema_id |
| RejectChainDigest | Broken chain | Fix chain_digest links |
| RejectStateHashLink | State discontinuity | Fix state_hash_prev |
| RejectPolicyViolation | Overspending | Reduce spend amount |
| RejectOverflow | Numeric overflow | Use smaller values |

### 5.3 System Failures (ERROR)

| Failure | Cause | Resolution |
|---------|-------|-------------|
| File not found | Missing input file | Check file path |
| Write error | Disk full / permissions | Fix filesystem |
| Internal error | Bug in verifier | Report issue |

## 6. Monitoring & Observability

### 6.1 Metrics

| Metric | Type | Purpose |
|--------|------|---------|
| Verification latency | Histogram | Performance |
| Acceptance rate | Gauge | Success ratio |
| Reject rate by code | Counter | Failure analysis |
| Chain length | Gauge | Workload size |

### 6.2 Logging

- Structured JSON logging
- REJECT includes code, message, step_index, object_id
- Full trace available for debugging

### 6.3 Health Checks

- `/health` endpoint for sidecar
- Returns OK if verifier functional

## 7. Performance Characteristics

### 7.1 Benchmarks

| Operation | Latency | Throughput |
|-----------|---------|-------------|
| Micro verify | ~31μs | 32K/sec |
| Chain verify (5-step) | ~1.7ms | - |
| Slab verify | p99: 280μs | - |

### 7.2 Scaling Considerations

- **Vertical**: Single instance handles high throughput
- **Horizontal**: Stateless verification; can scale horizontally
- **State**: State stored externally; verifier is stateless

### 7.3 Resource Requirements

- **Memory**: ~10MB baseline
- **CPU**: Minimal (pure computation)
- **Disk**: Only for logs/state

## 8. Deployment Patterns

### 8.1 Local Development

```bash
# Quick start
demo.bat

# Individual verification
coh-node\target\debug\coh-validator.exe verify-micro receipt.json
```

### 8.2 CI/CD Integration

```yaml
# Example CI step
- name: Verify receipts
  run: coh-validator verify-chain chain.jsonl
```

### 8.3 Agent Integration

```python
# AI agent calls sidecar
import requests

result = requests.post(
    "http://coh-sidecar:8080/verifyMicro",
    json={"receipt": receipt_data}
).json()
```

### 8.4 Production Container

```dockerfile
FROM rust:1.70 AS builder
COPY . /app
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/coh-validator /usr/local/bin/
ENTRYPOINT ["coh-validator"]
```

## 9. Operational Checklist

### 9.1 Pre-Deployment

- [ ] Select deployment mode (CLI/Sidecar/Library)
- [ ] Configure policy and canon profile
- [ ] Set up logging infrastructure
- [ ] Validate test receipts pass

### 9.2 Post-Deployment

- [ ] Verify health endpoint responds
- [ ] Monitor acceptance rate
- [ ] Set up alerting for ERROR conditions
- [ ] Document incident procedures

### 9.3 Ongoing

- [ ] Review reject rate by code
- [ ] Update policies as needed
- [ ] Monitor disk/logs
- [ ] Backup state store

## 10. Disaster Recovery

### 10.1 State Recovery

- Chain digest provides integrity
- Restore from known-good checkpoint
- Re-verify all pending receipts

### 10.2 Verifier Recovery

- Stateless; simply restart
- Re-fetch policies
- Resume verification

---

## Appendix: Related Documents

- [SYSTEM_ARCHITECTURE.md](./coh-node/SYSTEM_ARCHITECTURE.md) - System flow
- [ERROR_REJECT_CONTRACT.md](./ERROR_REJECT_CONTRACT.md) - Reject code taxonomy
- [COMPREHENSIVE_IMPROVEMENT_PLAN.md](./COMPREHENSIVE_IMPROVEMENT_PLAN.md) - Roadmap reference
- [SIDECAR_API.md](./SIDECAR_API.md) - HTTP API spec