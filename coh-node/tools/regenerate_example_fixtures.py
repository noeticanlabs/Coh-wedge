from __future__ import annotations

import hashlib
import json
from pathlib import Path


DIGEST_DOMAIN_TAG = b"COH_V1_CHAIN"
MERKLE_DOMAIN_TAG = b"COH_V1_MERKLE"
ZERO64 = "0" * 64
PROFILE_HASH = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
POLICY_HASH = ZERO64
OBJECT_ID = "obj_123"


def hx(value: int) -> str:
    return f"{value:064x}"


def canonical_prehash(receipt: dict) -> bytes:
    # Deep copy to avoid mutating the original
    prehash = json.loads(json.dumps(receipt))
    # chain_digest_next is the result of the hash, not part of the input
    if "chain_digest_next" in prehash:
        del prehash["chain_digest_next"]
    
    # Ensure metrics has authority for JCS
    if "metrics" in prehash and "authority" not in prehash["metrics"]:
        prehash["metrics"]["authority"] = "0"
    
    # Ensure SlabSummary has total_authority
    if "summary" in prehash and "total_authority" not in prehash["summary"]:
        prehash["summary"]["total_authority"] = "0"

    return json.dumps(prehash, sort_keys=True, separators=(",", ":")).encode("utf-8")


def compute_chain_digest(prev_digest_hex: str, receipt: dict) -> str:
    hasher = hashlib.sha256()
    hasher.update(DIGEST_DOMAIN_TAG)
    hasher.update(b"|")
    hasher.update(bytes.fromhex(prev_digest_hex))
    hasher.update(b"|")
    hasher.update(canonical_prehash(receipt))
    return hasher.hexdigest()


def compute_merkle_root(leaf_hexes: list[str]) -> str:
    if not leaf_hexes:
        return ZERO64

    layer = [bytes.fromhex(leaf) for leaf in leaf_hexes]
    while len(layer) > 1:
        next_layer: list[bytes] = []
        for i in range(0, len(layer), 2):
            left = layer[i]
            right = layer[i + 1] if i + 1 < len(layer) else left
            hasher = hashlib.sha256()
            hasher.update(MERKLE_DOMAIN_TAG)
            hasher.update(b"|")
            hasher.update(left)
            hasher.update(b"|")
            hasher.update(right)
            next_layer.append(hasher.digest())
        layer = next_layer
    return layer[0].hex()


def base_receipt(step_index: int, state_prev: str, state_next: str, v_pre: str, v_post: str, spend: str, defect: str) -> dict:
    return {
        "schema_id": "coh.receipt.micro.v1",
        "version": "1.0.0",
        "object_id": OBJECT_ID,
        "canon_profile_hash": PROFILE_HASH,
        "policy_hash": POLICY_HASH,
        "step_index": step_index,
        "state_hash_prev": state_prev,
        "state_hash_next": state_next,
        "chain_digest_prev": ZERO64,
        "chain_digest_next": ZERO64,
        "metrics": {
            "v_pre": v_pre,
            "v_post": v_post,
            "spend": spend,
            "defect": defect,
            "authority": "0",
        },
    }


def finalize_receipt(receipt: dict) -> dict:
    result = json.loads(json.dumps(receipt))
    result["chain_digest_next"] = compute_chain_digest(result["chain_digest_prev"], result)
    return result


def make_valid_chain() -> list[dict]:
    step0 = finalize_receipt(base_receipt(0, hx(0), hx(1), "100", "80", "15", "0"))
    step1 = base_receipt(1, hx(1), hx(2), "80", "60", "20", "0")
    step1["chain_digest_prev"] = step0["chain_digest_next"]
    step1 = finalize_receipt(step1)
    return [step0, step1]


def make_slab(valid_chain: list[dict]) -> dict:
    total_spend = sum(int(step["metrics"]["spend"]) for step in valid_chain)
    total_defect = sum(int(step["metrics"]["defect"]) for step in valid_chain)
    leaf_hexes = [step["chain_digest_next"] for step in valid_chain]
    return {
        "schema_id": "coh.receipt.slab.v1",
        "version": "1.0.0",
        "object_id": valid_chain[0]["object_id"],
        "canon_profile_hash": valid_chain[0]["canon_profile_hash"],
        "policy_hash": valid_chain[0]["policy_hash"],
        "range_start": valid_chain[0]["step_index"],
        "range_end": valid_chain[-1]["step_index"],
        "micro_count": len(valid_chain),
        "chain_digest_prev": valid_chain[0]["chain_digest_prev"],
        "chain_digest_next": valid_chain[-1]["chain_digest_next"],
        "state_hash_first": valid_chain[0]["state_hash_prev"],
        "state_hash_last": valid_chain[-1]["state_hash_next"],
        "merkle_root": compute_merkle_root(leaf_hexes),
        "summary": {
            "total_spend": str(total_spend),
            "total_defect": str(total_defect),
            "total_authority": "0",
            "v_pre_first": valid_chain[0]["metrics"]["v_pre"],
            "v_post_last": valid_chain[-1]["metrics"]["v_post"],
        },
    }


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def write_jsonl(path: Path, records: list[dict]) -> None:
    path.write_text("\n".join(json.dumps(record, sort_keys=True, separators=(",", ":")) for record in records) + "\n", encoding="utf-8")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    examples = repo_root / "examples"

    valid_chain = make_valid_chain()
    valid_slab = make_slab(valid_chain)

    micro_valid = valid_chain[0]

    micro_invalid_policy = json.loads(json.dumps(micro_valid))
    micro_invalid_policy["metrics"]["v_post"] = "110"
    micro_invalid_policy = finalize_receipt(micro_invalid_policy)

    micro_invalid_digest = json.loads(json.dumps(micro_valid))
    micro_invalid_digest["chain_digest_next"] = ZERO64

    chain_valid = valid_chain

    chain_invalid_digest = json.loads(json.dumps(valid_chain))
    chain_invalid_digest[1]["chain_digest_prev"] = ZERO64
    chain_invalid_digest[1] = finalize_receipt(chain_invalid_digest[1])

    chain_invalid_state = json.loads(json.dumps(valid_chain))
    chain_invalid_state[1]["state_hash_prev"] = "F" * 64
    chain_invalid_state[1] = finalize_receipt(chain_invalid_state[1])

    chain_invalid_index = json.loads(json.dumps(valid_chain))
    chain_invalid_index[1]["step_index"] = 2
    chain_invalid_index[1] = finalize_receipt(chain_invalid_index[1])

    chain_invalid_step_index = json.loads(json.dumps(chain_invalid_index))
    chain_invalid_state_link = json.loads(json.dumps(chain_invalid_state))

    slab_invalid_summary = json.loads(json.dumps(valid_slab))
    slab_invalid_summary["summary"]["v_post_last"] = "120"

    slab_invalid_policy = json.loads(json.dumps(valid_slab))
    slab_invalid_policy["summary"]["total_spend"] = "500"
    slab_invalid_policy["summary"]["v_post_last"] = "80"

    slab_invalid_merkle = json.loads(json.dumps(valid_slab))
    slab_invalid_merkle["merkle_root"] = "DEADC0DE" * 8

    write_json(examples / "micro_valid.json", micro_valid)
    write_json(examples / "micro_invalid_policy.json", micro_invalid_policy)
    write_json(examples / "micro_invalid_digest.json", micro_invalid_digest)

    write_jsonl(examples / "chain_valid.jsonl", chain_valid)
    write_jsonl(examples / "chain_invalid_digest.jsonl", chain_invalid_digest)
    write_jsonl(examples / "chain_invalid_state.jsonl", chain_invalid_state)
    write_jsonl(examples / "chain_invalid_state_link.jsonl", chain_invalid_state_link)
    write_jsonl(examples / "chain_invalid_index.jsonl", chain_invalid_index)
    write_jsonl(examples / "chain_invalid_step_index.jsonl", chain_invalid_step_index)

    documented_lines = [
        "# Phase 1: Initial reduction",
        json.dumps(chain_valid[0], separators=(",", ":")),
        "",
        "# Step transition",
        "# Note: The chain link must precisely match the previous digest result.",
        json.dumps(chain_valid[1], separators=(",", ":")),
        "",
    ]
    (examples / "chain_documented.jsonl").write_text("\n".join(documented_lines), encoding="utf-8")

    write_json(examples / "slab_valid.json", valid_slab)
    write_json(examples / "slab_new.json", valid_slab)
    write_json(examples / "slab_final.json", valid_slab)
    write_json(examples / "slab_invalid_summary.json", slab_invalid_summary)
    write_json(examples / "slab_invalid_policy.json", slab_invalid_policy)
    write_json(examples / "slab_invalid_merkle.json", slab_invalid_merkle)

    print("Regenerated example fixtures.")
    print(f"step0={chain_valid[0]['chain_digest_next']}")
    print(f"step1={chain_valid[1]['chain_digest_next']}")
    print(f"merkle={valid_slab['merkle_root']}")


if __name__ == "__main__":
    main()
