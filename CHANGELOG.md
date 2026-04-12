# Changelog

All notable changes to the Coh Safety Wedge will be documented in this file.

## [1.0.0] - 2026-04-12
### Added
- **Lean T-Stack Ledger**: Initialized the modular formal ledger with the verified T1 theorem (Strict Coh ? Category).
- **Dashboard Behavioral Testing**: Implemented complete Vitest suite for UI state transitions and error rendering.
- **RejectIntervalInvalid**: New rejection code for detecting temporal gaps in receipt chains.
- **Schema Hardening**: Added `step_type` and `signatures` to `MicroReceiptWire` for V1 compliance.

### Changed
- **Modular Lean Architecture**: Transitioned from holistic `coh-lean` to the federated `coh-t-stack`.
- **JCS Canonicalization**: Now using RFC 8785 (JSON Canonicalization Scheme) for deterministic receipt digests.
- **Documentation Overhaul**: Synchronized all plans and specifications with the finalized V1 wedge implementation.

### Fixed
- **CI Stabilization**: Resolved environment issues with Dashboard UI testing and Lean build paths.
- **Checked Arithmetic**: Enforced `checked_add/sub/mul` across all verification kernels to prevent overflow attacks.
