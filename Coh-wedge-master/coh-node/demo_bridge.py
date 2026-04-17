import json
from coh_sdk import AgentSession

print("--- Testing Coh SDK Pre-Seed Demo ---")

try:
    # 1. Initialize the cryptographic session 
    # (abstracts away chain hashes, schema_id, versioning, etc)
    session = AgentSession(
        object_id="agent.workflow.demo",
        canon_profile_hash="4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
        policy_hash="0"*64,
        initial_v=100
    )

    print("[1] Session initialized. Starting Agent turn 1.")
    # 2. Turn 1
    with session.step(spend=12, defect=0) as step:
        # Simulate LLM thinking...
        step.set_state_hash("1"*64)
    print("    Turn 1 accepted & verified deterministically.")
        
    print("[2] Starting Agent turn 2.")
    # 3. Turn 2
    with session.step(spend=6, defect=0) as step:
        # Simulate LLM thinking...
        step.set_state_hash("2"*64)
    print("    Turn 2 accepted & verified deterministically.")

    # 4. Verify Chain (macro check)
    print("[3] Finalizing Chain and verifying full contiguous run...")
    chain_res = session.verify_chain()
    print(f"    Decision: {chain_res['decision']}, Steps: {chain_res['steps_verified']}")

    # 5. Build Slab
    print("[4] Compressing state into Macro-Slab...")
    slab = session.build_slab()
    print(f"    Slab Built. Merkle Root: {slab.get('merkle_root')}")

    # 6. Verify Slab
    print("[5] Independent validator verifying Slab Envelope...")
    vs_res = session.verify_slab()
    print(f"    Decision: {vs_res['decision']}, Range: {vs_res['range_start']}-{vs_res['range_end']}")

    print("\n--- Pre-Seed Demo Complete ---")

except Exception as e:
    print(f"Error during execution: {e}")
    import traceback
    traceback.print_exc()
