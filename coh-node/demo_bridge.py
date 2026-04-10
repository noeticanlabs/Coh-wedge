import coh
import json

# Sample receipt
receipt_json = """
{
  "schema_id": "coh.receipt.micro.v1",
  "version": "1.0.0",
  "object_id": "agent.workflow.demo",
  "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
  "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "step_index": 0,
  "state_hash_prev": "1111111111111111111111111111111111111111111111111111111111111111",
  "state_hash_next": "2222222222222222222222222222222222222222222222222222222222222222",
  "chain_digest_prev": "0000000000000000000000000000000000000000000000000000000000000000",
  "chain_digest_next": "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c",
  "metrics": {
    "v_pre": "100",
    "v_post": "88",
    "spend": "12",
    "defect": "0"
  }
}
"""

print("--- Testing Coh PyO3 Bridge ---")
try:
    # 1. Normalize
    normalized = coh.normalize(receipt_json)
    print(f"[1] Normalized Output: {normalized[:60]}...")

    # 2. Compare (Semantic Equality)
    receipt_messy = receipt_json.replace(" ", "").replace("\n", "") 
    is_equal = coh.compare(receipt_json, receipt_messy)
    print(f"[2] Semantic Compare (Whitespace diff): {is_equal}")

    # 3. Hash calculation
    digest = coh.calculate_hash(receipt_json)
    print(f"[3] Calculated Hash: {digest}")

    # 4. Verify
    verify_res_json = coh.verify(receipt_json)
    verify_res = json.loads(verify_res_json)
    print(f"[4] Verify Decision: {verify_res['decision']}")
    
    print("\n--- Demo Complete ---")
except Exception as e:
    print(f"Error: {e}")
    print("\nNOTE: Ensure you have built and installed coh-python via 'maturin develop' first.")
