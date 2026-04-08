use crate::types::Hash32;

pub const MICRO_SCHEMA_ID: &str = "coh.receipt.micro.v1";
pub const SLAB_SCHEMA_ID: &str = "coh.receipt.slab.v1";
pub const VERSION: u32 = 1;
pub const CHAIN_TAG: &[u8] = b"COH_V1";
pub const MERKLE_TAG: &[u8] = b"COH_MERKLE_V1";

pub fn expected_schema_micro() -> &'static str { MICRO_SCHEMA_ID }
pub fn expected_schema_slab() -> &'static str { SLAB_SCHEMA_ID }
