# ADR: Contract Layer Freeze (Option B) and Context Threading Roadmap

Status: Accepted

Decision

- Freeze the contract boundary on ByteArray digests (Digest := ByteArray) with a canon-bound profile.
- Introduce a minimal CanonProfile with fields: domainTag, jcsEncode, hashBytes.
- Freeze the canonical chain update rule chainUpdate(cp, prev, bytes) = cp.hashBytes (cp.domainTag ++ prev ++ bytes).
- Provide structural verifiers that deterministically return Decision RejectCode for micro/slab receipts.
- Keep the categorical core stable; add a context-threaded category alongside it and provide an adapter.

Rationale

- Canon-first contract: canonical bytes, domain-separated hashing, and explicit digests are load-bearing and must be frozen at the boundary.
- Avoid building additional layering on placeholder hash carriers that would have to be replaced.
- Maintain category/core stability during the boundary upgrade by using adapters instead of invasive rewrites.

Scope and Interfaces (frozen now)

- Boundary substrate
  - Digest := ByteArray
  - CanonProfile: { profileId : String, domainTag : ByteArray, jcsEncode : String → ByteArray, hashBytes : ByteArray → Digest }
  - chainUpdate : CanonProfile → Digest → ByteArray → Digest
  - BoundaryVerifier: RV : Digest → Receipt → Digest → Digest → Decision Code

- Structural verifiers
  - Micro (legacy receipts, core carriers): verifyMicroStruct bc encode prevState nextState prevChain r → Decision RejectCode
  - Micro V2 (ByteArray digests): verifyMicroStructV2 bc prevState nextState prevChain rV2 → Decision RejectCode
  - Slab V2 (ByteArray digests): verifySlabStructV2 cfg cp first last prev rV2 → Decision RejectCode

- Reject codes (baseline, already frozen): schema, canon profile, chain digest, state-hash link, numeric parse, interval invalid, overflow, policy violation, slab merkle, slab summary.

Context Threading (frozen substrate, additive next step)

- New contextful category GovCatCtx with StepCtx carrying at least prevChainDigest bytes.
- Adapter from the current context-free GovObj into GovObjCtx by ignoring context.
- Oplax composition with Δ-additive law preserved in contextful form.

Migration Plan

1) Contract boundary freeze [DONE]
   - Files: Coh/Contract/Boundary.lean, Coh/Contract/MicroV2.lean, Coh/Contract/SlabV2.lean
   - Canonical encoder for Micro receipts: encodeMicroJCS

2) Context-threaded category [DONE - initial]
   - File: Coh/Category/GovCatCtx.lean (StepCtx, GovObjCtx, Hom, OplaxHom, Δ-additive comp, fromGovObj adapter)

3) Bridge generalization to context [NEXT]
   - Add MicroBridgeHypCtx and govObjCtx constructor wiring BoundaryVerifier through StepCtx.
   - Keep MicroBridgeHyp.govObj compatibility; mark deprecation path in module headers.

4) Tightness preorder in GovCatCtx [NEXT]
   - Provide oplax tightness order f ≤ g :↔ f.Δ ≤ g.Δ mirroring GovCat; add Preorder instance.

5) Canonical test vectors [NEXT]
   - Extend Coh/Contract/TestVectors.lean to V2 types (MicroReceiptV2, SlabReceiptV2) and provide accept/reject examples.

6) Gradual legacy deprecation [LATER]
   - Migrate ContractConfig to include expectedPolicyHash explicitly; update proofs and wrappers; deprecate adapters from Core hash carriers where feasible.

Determinism and Canon Locks (Rigor-First)

- Freeze serialization: jcsEncode must be deterministic and stable; no floats or locale dependence.
- Freeze numeric domain: use Nat/u128 semantics at the boundary with explicit overflow checks.
- Freeze hash rules: chainUpdate must keep the domain tag constant and byte concatenation ordering stable.
- Freeze error surface: RejectCode family is closed under current baseline; changes require a version bump.
- No wall clock, RNG, or external hidden state in verifiers.

Failure Modes and Mitigations

- ByteArray semantics mismatch: ensure jcsEncode aligns with the existing JCS model; add golden vectors.
- Domain tag drift: pin domainTag per CanonProfile and audit via golden test.
- Adapter ambiguity: document that fromGovObj ignores StepCtx; prefer GovObjCtx-native objects for new modules.

Change Impact

- Category/core proofs remain green via adapters; boundary-only upgrades avoid churn.
- New contextful category enables future work on chain-aware morphisms without breaking current objects.

Versioning

- Introduce a module-level minor bump for Coh.Contract.* and Coh.Category.GovCatCtx.*; list affected test vector modules.

