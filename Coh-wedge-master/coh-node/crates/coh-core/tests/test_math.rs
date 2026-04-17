use coh_core::math::*;
use coh_core::reject::RejectCode;

#[test]
fn test_safe_add_overflow() {
    let a = u128::MAX;
    let b = 1u128;
    let res = a.safe_add(b);
    assert_eq!(res, Err(RejectCode::RejectOverflow));
}

#[test]
fn test_safe_add_valid() {
    let a = 100u128;
    let b = 50u128;
    let res = a.safe_add(b);
    assert_eq!(res, Ok(150u128));
}
