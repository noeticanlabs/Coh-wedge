import Coh.Core
import Coh.Contract
import Coh.Crypto
import Coh.Oplax
import Coh.Kernel
import Coh.Slack
import Coh.Trace
import Coh.Spectral
import Coh.Selection

/-!
Top-level Coh library surface.

Stable public theorem sheet:
- `verify_accept_iff_lawful`
- `strict_morphism_preserves_lawful`
- `oplax_compose_slack_add`
- `strict_embeds_as_zero_slack`
- `zero_slack_oplax_is_strict`
- `accepted_step_implies_chain_digest_correct`
- `accepted_step_implies_state_hash_link`
- `accepted_trace_closure`
- `rv_contract_correctness`

Auxiliary lemmas remain imported for development, but the names above are the
intended public theorem surface for the hardened three-pack kernel.
-/
