use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonError {
    #[error("invalid decimal string")]
    InvalidDecimal,
    #[error("overflow")]
    Overflow,
    #[error("invalid hex")]
    InvalidHex,
    #[error("invalid canonical json")]
    InvalidCanonicalJson,
}
