use crate::types::{MicroReceipt, SlabReceipt, ChallengeOpening};
use crate::reject::RejectCode;
use crate::errors::CanonError;
use crate::merkle::{merkle_path, verify_merkle_path, micro_receipt_leaf};

pub fn open_challenge(
    _slab: &SlabReceipt,
    receipts: &[MicroReceipt],
    index: usize,
) -> Result<ChallengeOpening, CanonError> {
    let receipt = receipts[index].clone();
    let path = merkle_path(receipts, index)?;
    Ok(ChallengeOpening {
        index,
        receipt,
        merkle_path: path,
    })
}

pub fn verify_challenge_opening(
    slab: &SlabReceipt,
    opening: &ChallengeOpening,
) -> Result<(), RejectCode> {
    let leaf = micro_receipt_leaf(&opening.receipt).map_err(|_| RejectCode::RejectNumericParse)?;
    if !verify_merkle_path(leaf, opening.index, &opening.merkle_path, slab.merkle_root) {
        return Err(RejectCode::RejectMerkleRoot);
    }
    Ok(())
}
