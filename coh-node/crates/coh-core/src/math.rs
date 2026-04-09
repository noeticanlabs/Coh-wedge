use crate::reject::RejectCode;

pub type MathResult<T> = Result<T, RejectCode>;

pub trait CheckedMath: Sized {
    fn safe_add(self, other: Self) -> MathResult<Self>;
    fn safe_sub(self, other: Self) -> MathResult<Self>;
}

impl CheckedMath for u128 {
    fn safe_add(self, other: Self) -> MathResult<Self> {
        self.checked_add(other).ok_or(RejectCode::RejectOverflow)
    }

    fn safe_sub(self, other: Self) -> MathResult<Self> {
        self.checked_sub(other).ok_or(RejectCode::RejectOverflow)
    }
}
