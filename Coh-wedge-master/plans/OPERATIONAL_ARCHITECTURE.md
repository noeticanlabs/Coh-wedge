# Coh Operational Architecture

> Complete deployment architecture, operational procedures, and failure mode documentation for the Coh verification system

## 1. Executive Summary

The Coh verification system is designed for operational simplicity while maintaining high security guarantees. The system consists of a deterministic verification kernel with no external dependencies, making it suitable for deployment in diverse environments ranging from local CLI usage to embedded library integration.

Key operational characteristics:

- **Stateless verification**: The verifier maintains no internal state; all state is external
- **Deterministic decisions**: Same receipt always produces same decision
- **No dependencies**: Pure computation; no external services required
- **Fail-safe defaults**: Invalid inputs are rejected

This document covers deployment modes, component architecture, configuration, failure modes, monitoring, and operational procedures.

## 2. Deployment Modes

### 2.1 Standalone CLI

The CLI is appropriate for local verification, CI/CD pipelines, and scripts.

#### 2.1.1 Installation

**Binary distribution** (recommended):

```bash
# Download the latest release
curl -L https://github.com/coh-wedge/releases/download/v1.0.0/coh-validator-x86_64-pc-windows-msvc.zip -o coh-validator.zip

# Extract
unzip coh-validator.zip

# Verify
./coh-validator.exe --version
```

**Build from source**:

```bash
# Clone repository
git clone https://github.com/coh-wedge/coh-node.git
cd coh-node

# Build
cargo build --release

# Verify
./target/release/coh-validator --version
```

#### 2.1.2 Usage

**Single receipt verification**:

```bash
# Verify a micro receipt
coh-validator verify-micro examples/micro_valid.json
# Output: ACCEPT

# Verify an invalid receipt
coh-validator verify-micro examples/micro_invalid_digest.json
# Output: REJECT
# Exit code: 1
```

**Chain verification**:

```bash
# Verify a multi-step chain
coh-validator verify-chain examples/chain_valid.jsonl
# Output: ACCEPT
```

**Slab verification**:

```bash
# Verify an aggregated receipt
coh-validator verify-slab examples/slab_valid.json
# Output: ACCEPT
```

**Build a slab**:

```bash
# Create a macro receipt from a chain
coh-validator build-slab examples/chain_valid.jsonl --out examples/demo_slab.json
# Output: ACCEPT (if chain is valid)
```

#### 2.1.3 Exit Codes

| Exit Code | Meaning | Description |
|----------|--------|-------------|
| 0 | ACCEPT | Verification passed |
| 1 | REJECT | Verification failed |
| 2 | MALFORMED | Input parse error |
| 3 | ERROR | Internal error |
| 4 | SOURCE | Build-slab had invalid source |

### 2.2 Sidecar Service (HTTP)

The sidecar provides an HTTP API for integration with agent frameworks, web services, and microservices.

#### 2.2.1 Installation

**Binary distribution**:

```bash
# Download the latest release
curl -L https://github.com/coh-wedge/releases/download/v1.0.0/coh-sidecar-x86_64-pc-windows-msvc.zip -o coh-sidecar.zip

# Extract
unzip coh-sidecar.zip

# Run
./coh-sidecar.exe
```

**Docker**:

```bash
# Build image
docker build -t coh-sidecar:latest -f Dockerfile.sidecar .

# Run container
docker run -p 8080:8080 coh-sidecar:latest
```

**Docker Compose**:

```yaml
version: '3.8'
services:
  coh-sidecar:
    image: coh-sidecar:latest
    ports:
      - "8080:8080"
    volumes:
      - ./policies:/app/policies
    environment:
      - LOG_LEVEL=info
```

#### 2.2.2 API Endpoints

**POST /verifyMicro**

Verify a single receipt:

```bash
curl -X POST http://localhost:8080/verifyMicro \
  -H "Content-Type: application/json" \
  -d @receipt.json
```

Response:

```json
{
  "decision": "ACCEPT",
  "code": null,
  "message": "Verification passed",
  "step_index": null,
  "object_id": "agent.workflow.demo"
}
```

**POST /verifyChain**

Verify a multi-step chain:

```bash
curl -X POST http://localhost:8080/verifyChain \
  -H "Content-Type: application/json" \
  -d @chain.jsonl
```

**POST /verifySlab**

Verify a macro receipt:

```bash
curl -X POST http://localhost:8080/verifySlab \
  -H "Content-Type: application/json" \
  -d @slab.json
```

**POST /buildSlab**

Create a macro receipt:

```bash
curl -X POST http://localhost:8080/buildSlab \
  -H "Content-Type: application/json" \
  -d @chain.jsonl
```

**GET /health**

Health check:

```bash
curl http://localhost:8080/health
```

Response:

```json
{
  "status": "ok",
  "version": "1.0.0"
}
```

#### 2.2.3 Error Responses

Validation failures return structured errors:

```json
{
  "decision": "REJECT",
  "code": "RejectPolicyViolation",
  "message": "Macro inequality violated: v_post_last + total_spend (150) exceeds v_pre_first + total_defect (42)",
  "step_index": 2,
  "object_id": "agent.workflow.demo"
}
```

Malformed requests:

```json
{
  "error": {
    "code": "E005",
    "message": "Invalid JSON in request body"
  }
}
```

### 2.3 Embedded Library

For deep integration, use the Rust library directly or via Python bindings.

#### 2.3.1 Rust Integration

**Add dependency**:

```toml
[dependencies]
coh-core = { version = "1.0.0", path = "../coh-node/crates/coh-core" }
```

**Use in code**:

```rust
use coh_core::{verify_micro, Decision, Policy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = serde_json::from_str::<MicroReceipt>(receipt_json)?;
    let policy = Policy::default();
    
    let result = verify_micro(&receipt, &policy, &Config::default());
    
    match result.decision {
        Decision::Accept => println!("ACCEPT"),
        Decision::Reject(code) => {
            println!("REJECT: {:?}", code);
            if let Some(msg) = result.message {
                println!("{}", msg);
            }
        }
    }
    
    Ok(())
}
```

#### 2.3.2 Python Integration

**Install**:

```bash
pip install coh-python
```

**Use in code**:

```python
from coh_python import verify, Decision, Policy

# Load receipt
with open('receipt.json') as f:
    receipt = f.read()

# Verify
result = verify(receipt, Policy.default())

# Check result
if result.decision == Decision.ACCEPT:
    print("ACCEPT")
else:
    print(f"REJECT: {result.code}")
    if result.message:
        print(result.message)
```

#### 2.3.3 Performance Characteristics

| Mode | Latency | Throughput | Use Case |
|------|---------|------------|----------|
| Embedded Rust | ~31μs | 32K/sec | Highest performance |
| CLI | ~31μs | 32K/sec | Local scripts |
| Sidecar | +1-5ms | Limited by network | Agent integration |

## 3. Component Architecture

### 3.1 Core Components

| Component | Responsibility | Language | Dependencies |
|-----------|---------------|----------|--------------|
| `coh-core` | Verification kernel | Rust | None (pure) |
| `coh-cli` | CLI entry point | Rust | `coh-core` |
| `coh-sidecar` | HTTP API | Rust | `coh-core`, `axum` |
| `coh-python` | Language bindings | Python/Rust | `coh-core` |
| `ape` | Proposal generation | Rust | `coh-core` |

### 3.2 Data Flow

```
+=====================================================================+
|                         DATA FLOW                                    |
|                                                                      |
|   +-------------+                                                  |
|   |   INPUT     |                                                  |
|   | (Receipt,  |                                                  |
|   |  Chain,     |                                                  |
|   |  Slab)      |                                                  |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +----------+                                  |
|   |   PARSE    | -> |MALFORMED| -> Error output                  |
|   +-------------+    +----------+                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +------------------+                          |
|   |   SCHEMA   | -> |   RejectSchema  | -> Error output          |
|   |  VALIDATE  |    +------------------+                          |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +------------------------+                   |
|   |   CANON    | -> | RejectCanonProfile   | -> Error output        |
|   |  VALIDATE |    +------------------------+                   |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +---------------------+                      |
|   |   CHAIN   | -> | RejectChainDigest   | -> Error output        |
|   | CONTINUITY|    +---------------------+                      |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +------------------------+                   |
|   |   STATE   | -> | RejectStateHashLink | -> Error output        |
|   |    HASH   |    +------------------------+                   |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +------------------------+                   |
|   |ACCOUNTING | -> | RejectPolicyViolation| -> Error output        |
|   |    LAW   |    +------------------------+                   |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+    +------------------------+                   |
|   |  POLICY   | -> | RejectPolicyViolation| -> Error output        |
|   |   CHECK  |    +------------------------+                   |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+                                                  |
|   |   ACCEPT  |                                                  |
|   +-------------+                                                  |
|         |                                                          |
|         v                                                          |
|   +-------------+                                                  |
|   | EXECUTION |                                                  |
|   |   LAYER   |                                                  |
|   +-------------+                                                  |
|                                                                      |
+=====================================================================+
```

### 3.3 Build Slab Flow

```
[Chain of receipts] -> [Aggregate summaries] -> [Build Merkle tree] -> [Produce slab]
                              |
                              v
                    +-------------------+
                    | Validate chain   | -> REJECT on invalid
                    +-------------------+
```

## 4. Configuration

### 4.1 Policy Configuration

Policies are defined in JSON:

```json
{
  "policy_id": "default",
  "version": "1.0.0",
  "rules": {
    "max_spend_per_action": 1000,
    "max_chain_length": 100,
    "max_slab_size": 1000,
    "authority_renewal": "periodic"
  },
  "valid_action_types": [
    "spend",
    "delegate",
    "stake"
  ]
}
```

**Loading policies**:

```bash
# CLI
coh-validator verify-micro --policy policy.json receipt.json

# Sidecar
# Place in policies/ directory
# Loaded automatically on startup
```

### 4.2 Canon Profile Configuration

Canon profiles define canonicalization behavior:

```json
{
  "profile_id": "coh.default",
  "version": "1.0.0",
  "domain_tag": "COH_V1",
  "hash_algorithm": "sha256",
  "canonicalization": "jcs"
}
```

**Default profiles**:

| Profile ID | Domain Tag | Use Case |
|-----------|-----------|----------|
| `coh.default` | `COH_V1` | Standard verification |
| `coh.strict` | `COH_STRICT` | High-security environments |

### 4.3 Configuration Precedence

Configuration values are resolved in this order:

1. **CLI flags** (highest precedence)
2. **Environment variables**
3. **Config file**
4. **Default values** (lowest precedence)

```bash
# Environment variable example
export COH_POLICY_PATH=/etc/coh/policy.json
export COH_LOG_LEVEL=debug
```

### 4.4 Configuration Files

**Directory structure** (recommended):

```
/etc/coh/
├── config.json
├── policies/
│   ├── default.json
│   ├── strict.json
│   └── custom.json
├── canon/
│   ├── default.json
│   └── strict.json
└── logs/
```

**Default paths**:

- CLI: `./config.json`, `./policies/`, `./canon/`
- Sidecar: `/etc/coh/config.json`, `/etc/coh/policies/`, `/etc/coh/canon/`

## 5. Operational Failure Modes

### 5.1 Input Failures

| Failure | Exit Code | Example | Resolution |
|---------|-----------|---------|-------------|
| File not found | 2 | `receipt.json: No such file` | Check file path |
| Invalid JSON | 2 | `Parse error at line 3` | Validate JSON syntax |
| Schema mismatch | 1 (REJECT) | Wrong schema_id | Use correct schema |

### 5.2 Verification Failures (REJECT)

When verification fails, the decision is REJECT with a specific reject code:

| Reject Code | Exit Code | Cause | Resolution |
|------------|-----------|-------|-------------|
| `RejectSchema` | 1 | Invalid schema_id | Use valid schema_id |
| `RejectCanonProfile` | 1 | Canon profile mismatch | Use matching profile |
| `RejectChainDigest` | 1 | Broken chain link | Fix chain_digest_prev |
| `RejectStateHashLink` | 1 | State discontinuity | Fix state_hash_prev |
| `RejectNumericParse` | 1 | Invalid number format | Fix numeric encoding |
| `RejectOverflow` | 1 | Arithmetic overflow | Use smaller values |
| `RejectPolicyViolation` | 1 | Overspending | Reduce spend amount |
| `RejectSlabSummary` | 1 | Summary mismatch | Fix summary calculation |
| `RejectSlabMerkle` | 1 | Merkle root mismatch | Rebuild Merkle tree |
| `RejectIntervalInvalid` | 1 | Invalid interval | Fix interval values |

**Example REJECT output**:

```
REJECT
code: RejectPolicyViolation
message: Policy violation: spend (100) exceeds allowed (50)
step_index: 2
object_id: agent.workflow.demo
```

### 5.3 System Failures (ERROR)

| Failure | Exit Code | Cause | Resolution |
|---------|-----------|-------|-------------|
| Internal error | 3 | Bug in verifier | Report issue |
| File write error | 3 | Disk full | Fix filesystem |
| Invalid config | 3 | Config syntax | Fix config file |

### 5.4 Failure Recovery

#### 5.4.1 Recovery from REJECT

REJECT means the state was unchanged. To recover:

1. Analyze the reject code and message
2. Fix the invalid input
3. Re-submit the corrected receipt

**Never modify the state manually** — always use the verification flow.

#### 5.4.2 Recovery from ERROR

ERROR indicates a system failure. To recover:

1. Check logs for error details
2. Restart the verifier
3. Re-verify pending receipts

```bash
# Restart sidecar
docker restart coh-sidecar

# Or for CLI, simply re-run
coh-validator verify-chain pending_chain.jsonl
```

#### 5.4.3 Recovery from State Corruption

If the state store is corrupted:

1. Identify the last valid state via chain digest
2. Restore state from backup at that point
3. Re-verify all receipts after that point

## 6. Monitoring and Observability

### 6.1 Metrics

Metrics are exposed via the sidecar at `/metrics`:

| Metric | Type | Description |
|--------|------|-------------|
| `coh_verify_total` | Counter | Total verifications |
| `coh_verify_accept_total` | Counter | Total accepts |
| `coh_verify_reject_total` | Counter | Total rejects |
| `coh_verify_reject_code_total` | Counter | Rejects by code |
| `coh_verify_latency_seconds` | Histogram | Verification latency |
| `coh_chain_length` | Gauge | Current chain length |

**Prometheus scrape config**:

```yaml
scrape_configs:
  - job_name: 'coh-sidecar'
    static_configs:
      - targets: ['localhost:8080']
```

### 6.2 Logging

#### 6.2.1 Log Levels

| Level | Use Case |
|-------|----------|
| ERROR | Verification failures, system errors |
| WARN | Configuration warnings |
| INFO | Accept/reject decisions |
| DEBUG | Detailed verification trace |

#### 6.2.2 Log Format

Structured JSON logging:

```json
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "INFO",
  "message": "verification_complete",
  "decision": "REJECT",
  "code": "RejectPolicyViolation",
  "step_index": 2,
  "object_id": "agent.workflow.demo",
  "latency_ms": 0.031
}
```

#### 6.2.3 Log Output

**CLI**: Console output

```bash
coh-validator verify-micro receipt.json 2>&1 | jq '.'
```

**Sidecar**: File or syslog

```json
{
  "log_driver": "json-file",
  "options": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

### 6.3 Health Checks

#### 6.3.1 Health Endpoint

**GET /health**

Returns:

```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

#### 6.3.2 Readiness Endpoint

**GET /ready**

Returns:

```json
{
  "ready": true,
  "checks": {
    "config": "ok",
    "policies": "ok"
  }
}
```

### 6.4 Tracing

Distributed tracing via OpenTelemetry:

```bash
# Enable tracing
export COH_TRACE_ENABLED=true
export COH_trace_ENDPOINT=http://tempo:4317
```

Trace spans include:

- `verify_micro` - Full verification
- `parse` - Input parsing
- `schema_check` - Schema validation
- `canon_check` - Canonical validation
- `chain_check` - Chain continuity
- `state_check` - State hash validation
- `accounting_check` - Law validation
- `policy_check` - Policy validation

## 7. Performance

### 7.1 Benchmarks

Measured on Intel i7-11700K, 32GB RAM:

| Operation | P50 | P95 | P99 | Throughput |
|-----------|-----|-----|-----|------------|
| Micro verify | 31μs | 45μs | 280μs | 32K/sec |
| Chain verify (5-step) | 1.7ms | 2.5ms | 3.2ms | 590/sec |
| Slab verify (100-step) | 18ms | 25ms | 31ms | 55/sec |

### 7.2 Scaling Considerations

#### 7.2.1 Vertical Scaling

The verifier is CPU-bound. Single instances handle high throughput.

**Recommended instance types**:

| Throughput | vCPU | Memory |
|------------|------|--------|
| < 1K/sec | 2 | 4GB |
| 1K-10K/sec | 4 | 8GB |
| > 10K/sec | 8+ | 16GB+ |

#### 7.2.2 Horizontal Scaling

The verifier is stateless. Scale horizontally by running multiple instances.

**Load balancer configuration**:

```yaml
upstream coh_verifiers {
    server coh-1:8080;
    server coh-2:8080;
    server coh-3:8080;
}
```

#### 7.2.3 State Store Scaling

State is the bottleneck. Use:

- SSD-backed storage
- In-memory caching (Redis)
- Read replicas for queries

### 7.3 Resource Requirements

| Component | CPU | Memory | Disk |
|-----------|-----|--------|-----|
| CLI | Minimal | ~10MB | Minimal |
| Sidecar | 1-2 cores | ~100MB | Logs |
| Python binding | 1 core | ~50MB | Minimal |

## 8. Deployment Patterns

### 8.1 Local Development

**Quick start**:

```bash
# Clone and build
git clone https://github.com/coh-wedge/coh-node.git
cd coh-node
cargo build --release

# Run demo
cd examples
../target/release/coh-validator verify-micro micro_valid.json
# OUTPUT: ACCEPT
```

**Use demo scripts**:

```bash
# From repo root
demo.bat

# AI workflow demo
ai_demo.bat

# Full benchmark
bench.bat
```

### 8.2 CI/CD Integration

**GitHub Actions**:

```yaml
name: Verify receipts

on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build
        run: cargo build --release
      
      - name: Verify chain
        run: |
          ./target/release/coh-validator verify-chain \
            examples/chain_valid.jsonl
```

**GitLab CI**:

```yaml
verify:
  image: rust:latest
  script:
    - cargo build --release
    - ./target/release/coh-validator verify-chain examples/chain_valid.jsonl
```

### 8.3 AI Agent Integration

**Agent calls sidecar**:

```python
import requests

def verify_agent_action(action):
    response = requests.post(
        "http://coh-sidecar:8080/verifyMicro",
        json={
            "receipt": action.receipt,
            "policy": action.policy
        }
    )
    result = response.json()
    
    if result["decision"] == "ACCEPT":
        execute(action)
    else:
        log_rejection(result)
        return False
    
    return True
```

**Agent embedded**:

```python
from coh_python import verify, Decision

result = verify(receipt_json, policy_json)

if result.decision == Decision.ACCEPT:
    execute(action)
```

### 8.4 Production Container

**Dockerfile**:

```dockerfile
# Build stage
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates
COPY --from=builder /app/target/release/coh-validator /usr/local/bin/
ENTRYPOINT ["coh-validator"]
```

**Docker Compose (full stack)**:

```yaml
version: '3.8'

services:
  coh-validator:
    image: coh-validator:latest
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config
      - ./policies:/app/policies
      - ./logs:/app/logs
    environment:
      - LOG_LEVEL=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    
volumes:
  redis-data:
```

## 9. Operational Checklist

### 9.1 Pre-Deployment

- [ ] Verify build compiles without warnings
- [ ] Validate all policy configurations
- [ ] Validate all canon profiles
- [ ] Test with known-good receipts
- [ ] Configure logging destination
- [ ] Configure metrics export
- [ ] Set up health check endpoints
- [ ] Document policy update process

### 9.2 Post-Deployment

- [ ] Verify health endpoint responds
- [ ] Verify health check is green
- [ ] Send test receipts (valid and invalid)
- [ ] Verify acceptance rate metrics
- [ ] Verify reject rate metrics
- [ ] Verify logging output
- [ ] Set up alerting for ERROR conditions

### 9.3 Ongoing Operations

- [ ] Monitor acceptance rate (target: > 95%)
- [ ] Monitor reject rate by code
- [ ] Review reject distribution
- [ ] Update policies as needed
- [ ] Rotate canon profiles if needed
- [ ] Monitor disk/logs usage
- [ ] Back up configuration
- [ ] Review security model updates

### 9.4 Incident Response

| Severity | Example | Response |
|----------|---------|----------|
| SEV1 | All verifications failing | Check logs, restart service |
| SEV2 | High reject rate | Analyze reject codes |
| SEV3 | Performance degradation | Scale vertically/horizontally |
| INFO | Configuration change | Update and verify |

## 10. Disaster Recovery

### 10.1 State Recovery

The chain digest provides state integrity verification.

**Recovery steps**:

1. **Identify last valid state**: Find chain digest from known-good checkpoint
2. **Restore state**: Load state from backup
3. **Verify chain**: Re-verify all receipts after checkpoint
4. **Resume**: Continue operations

```bash
# Verify chain integrity
coh-validator verify-chain --checkpoint <last_digest> chain.jsonl
```

### 10.2 Configuration Recovery

Back up configurations:

```bash
# Backup
tar -czf config-backup-$(date +%Y%m%d).tar.gz /etc/coh/

# Restore
tar -xzf config-backup-20240115.tar.gz -C /
```

### 10.3 Verifier Recovery

The verifier is stateless; recovery is simple:

1. **Restart service**: `docker restart coh-sidecar`
2. **Reload configuration**: Configuration is reloaded on restart
3. **Resume verification**: Continue from last known state

### 10.4 Complete System Recovery

If all state is lost:

1. **Restore from backup**: Load last backup
2. **Verify chain**: Re-verify entire chain
3. **Resume operations**: Continue with last verified state

---

## Appendix: Related Documents

- [`SYSTEM_ARCHITECTURE.md`](coh-node/SYSTEM_ARCHITECTURE.md) - System flow and component mapping
- [`ERROR_REJECT_CONTRACT.md`](ERROR_REJECT_CONTRACT.md) - Complete reject code taxonomy
- [`SECURITY_MODEL.md`](SECURITY_MODEL.md) - Security model and threat model
- [`SIDECAR_API.md`](SIDECAR_API.md) - HTTP API specification
- [`COMPREHENSIVE_IMPROVEMENT_PLAN.md`](COMPREHENSIVE_IMPROVEMENT_PLAN.md) - Roadmap reference

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial document |