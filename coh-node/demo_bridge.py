import coh
import json

# Sample receipt as a native Python DICTIONARY (Polymorphic support)
receipt = {
    "schema_id": "coh.receipt.micro.v1",
    "version": "1.0.0",
    "object_id": "agent.workflow.demo",
    "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
    "policy_hash": "0" * 64,
    "step_index": 0,
    "state_hash_prev": "1" * 64,
    "state_hash_next": "2" * 64,
    "chain_digest_prev": "0" * 64,
    "chain_digest_next": "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c",
    "metrics": {
        "v_pre": "100",
        "v_post": "88",
        "spend": "12",
        "defect": "0"
    }
}

print("--- Testing Coh Refined Python API ---")

try:
    # 1. Normalize (Returns CohResult object)
    result = coh.normalize(receipt)
    print(f"[1] Normalized Hash: {result.hash}")
    print(f"[1] Normalized Dict snippet: {str(result.normalized)[:100]}...")

    # 2. Verify (Assertion Contract)
    print("[2] Verifying valid receipt (expecting None)...")
    coh.verify(receipt)
    print("    Success: verify() returned None")

    # 3. Compare (Canonical-aware)
    receipt_alternate = receipt.copy()
    # Change format but keep semantic identity
    is_same = coh.compare(receipt, receipt_alternate)
    print(f"[3] Semantic Compare (Self): {is_same}")
    
    # 4. Hash (Canonical-aware)
    h1 = coh.hash(receipt)
    print(f"[4] Canonical Hash: {h1}")

    # 5. Assert Equivalent
    print("[5] Running coh.assert_equivalent()...")
    coh.assert_equivalent(receipt, receipt_alternate)
    print("    Success: assertion passed")

    # 6. Test Exception Handling (Verification Failure)
    print("[6] Testing CohVerificationError...")
    tampered = receipt.copy()
    tampered["metrics"]["spend"] = "999" # Violate accounting law
    
    try:
        coh.verify(tampered)
    except coh.CohVerificationError as e:
        print(f"    Caught expected error: {e}")
        print(f"    Reason metadata: {getattr(e, 'reason', 'N/A')}")

    print("\n--- Refinement Demo Complete ---")

except Exception as e:
    print(f"Error during execution: {e}")
    import traceback
    traceback.print_exc()

