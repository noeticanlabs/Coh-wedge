import coh
import json
from contextlib import contextmanager

class AgentSession:
    def __init__(self, object_id: str, canon_profile_hash: str, policy_hash: str, initial_v: int = 100):
        self.object_id = object_id
        self.canon_profile_hash = canon_profile_hash
        self.policy_hash = policy_hash
        
        self.step_index = 0
        self.chain_digest_prev = "0" * 64
        self.state_hash_prev = "0" * 64
        self.v_pre = str(initial_v)
        
        self.receipts = []

    @contextmanager
    def step(self, spend: int, defect: int = 0):
        class StepContext:
            def __init__(self):
                self.state_hash_next = "0" * 64
                self.v_post = None
            
            def set_state_hash(self, hash_str: str):
                self.state_hash_next = hash_str

        expected_v_post = int(self.v_pre) - spend + defect
        
        ctx = StepContext()
        ctx.v_post = str(expected_v_post)
        
        yield ctx
        
        receipt = {
            "schema_id": "coh.receipt.micro.v1",
            "version": "1.0.0",
            "object_id": self.object_id,
            "canon_profile_hash": self.canon_profile_hash,
            "policy_hash": self.policy_hash,
            "step_index": self.step_index,
            "state_hash_prev": self.state_hash_prev,
            "state_hash_next": ctx.state_hash_next,
            "chain_digest_prev": self.chain_digest_prev,
            "chain_digest_next": "0" * 64, 
            "metrics": {
                "v_pre": self.v_pre,
                "v_post": ctx.v_post,
                "spend": str(spend),
                "defect": str(defect)
            }
        }
        
        v_res = coh.normalize(receipt)
        receipt["chain_digest_next"] = v_res.hash
        
        coh.verify(receipt)
        
        self.receipts.append(receipt)
        self.step_index += 1
        self.state_hash_prev = ctx.state_hash_next
        self.chain_digest_prev = v_res.hash
        self.v_pre = ctx.v_post

    def export_chain(self) -> str:
        return "\n".join(json.dumps(r, separators=(',', ':')) for r in self.receipts)

    def verify_chain(self):
        return coh.verify_chain(self.export_chain())

    def build_slab(self):
        if not self.receipts:
            raise ValueError("No steps in session")
        return coh.build_slab(self.receipts)

    def verify_slab(self):
        slab = self.build_slab()
        return coh.verify_slab(slab)

