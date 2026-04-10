# Merkle Challenge Flow

The slab receipt includes a `merkle_root` computed from the `chain_digest_next` values of all micro-receipts in the chain. This enables compact audit proofs: any individual receipt can be verified against the slab without replaying the entire chain.

---

## Merkle Tree Construction

Leaves are the `chain_digest_next` hex values of each micro-receipt in chain order.

The tree is built bottom-up using the domain-tagged inner hash:

```
inner = SHA256("COH_V1_MERKLE" || "|" || left_bytes || "|" || right_bytes)
```

If a level has an odd number of nodes, the last node is paired with itself (self-duplication).

An empty leaf set returns the zero hash (`0x00...00`).

---

## Domain Separation

Two distinct domain tags prevent cross-context collisions:

| Tag | Used In |
|---|---|
| `COH_V1_CHAIN` | Chain digest computation (per micro-receipt) |
| `COH_V1_MERKLE` | Merkle inner node computation (slab) |

A hash produced in a chain context cannot be mistaken for a Merkle node, even if the input bytes are identical.

---

## Full Slab Verification (verify-slab-with-leaves)

`verify_slab_with_leaves` performs the complete audit:

1. Run `verify_slab` (schema, range, macro-inequality)
2. Extract `chain_digest_next` from each provided micro-receipt
3. Compute the Merkle root from those leaves
4. Compare computed root against `slab.merkle_root`

**Failure code**: `RejectSlabMerkle`

This gives an auditor a single, compact proof that a specific set of micro-receipts produced a given slab — without trusting the slab's summary fields alone.

---

## Example: 4-Leaf Tree

```
Leaves: [D0, D1, D2, D3]

Level 1:  H(D0, D1)    H(D2, D3)
Level 2:  H(H(D0,D1), H(D2,D3))  ← merkle_root
```

Example: 3-Leaf Tree (odd, last leaf self-duplicated):

```
Leaves: [D0, D1, D2]

Level 1:  H(D0, D1)    H(D2, D2)
Level 2:  H(H(D0,D1), H(D2,D2))  ← merkle_root
```
