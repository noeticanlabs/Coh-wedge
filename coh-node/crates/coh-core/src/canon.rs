use crate::errors::CanonError;
use crate::types::QFixed;
use serde::Serialize;

pub const SCALE: i128 = 1_000_000;

pub fn parse_qfixed(s: &str) -> Result<QFixed, CanonError> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() > 2 {
        return Err(CanonError::InvalidDecimal);
    }
    
    let integer_part = parts[0].parse::<i128>().map_err(|_| CanonError::InvalidDecimal)?;
    let mut fractional_part = 0i128;
    
    if parts.len() == 2 {
        let f_str = parts[1];
        if f_str.len() > 6 {
            return Err(CanonError::InvalidDecimal);
        }
        fractional_part = f_str.parse::<i128>().map_err(|_| CanonError::InvalidDecimal)?;
        // Scale fractional part (e.g., ".1" -> 100_000)
        for _ in 0..(6 - f_str.len()) {
            fractional_part *= 10;
        }
    }
    
    let total = integer_part * SCALE + (if integer_part < 0 { -fractional_part } else { fractional_part });
    Ok(QFixed(total))
}

pub fn qfixed_to_string(x: QFixed) -> String {
    let sign = if x.0 < 0 { "-" } else { "" };
    let abs_val = x.0.abs();
    let integer = abs_val / SCALE;
    let fraction = abs_val % SCALE;
    format!("{}{}.{:06}", sign, integer, fraction)
}

pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonError> {
    // For the demo, we use serde_json with a stable sort or similar.
    // In a production node, we would use a library that guarantees canonical JSON (RFC 8785).
    // Here we use sorted serialization if available, or just standard for the demo.
    serde_json::to_vec(value).map_err(|_| CanonError::InvalidCanonicalJson)
}
