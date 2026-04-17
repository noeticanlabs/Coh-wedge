import Coh.Core.Hash
import Coh.Core.Law
import Coh.Core.Verify
import Coh.Core.ReceiptChain
import Coh.Core.Trace

/-!
Public semantic/trace theorem surface.

Stable named results intended for referee-facing use:
- `verify_accept_iff_lawful`
- `accepted_step_implies_chain_digest_correct`
- `accepted_step_implies_state_hash_link`
- `accepted_trace_closure`

Helper lemmas in imported modules remain available, but the names above are the
intended public sheet for the core/trace layer.
-/
