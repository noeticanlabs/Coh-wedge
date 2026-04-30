use num_rational::Rational64 as Rational;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalClass {
    Timelike,
    Null,
    Spacelike,
}

#[derive(Debug, Clone)]
pub struct GmiConeConfig {
    pub c_g: Rational,
    pub dt_g: Rational,
}

#[derive(Debug, Clone)]
pub struct GmiConeCheck {
    pub distance: Rational,
    pub max_distance: Rational,
    pub interval_sq: Rational,
    pub class: CausalClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConeError {
    NegativeDistance,
    NonPositiveSpeedLimit,
    NonPositiveDeltaTime,
}

/// Classify the GMI interval using the causal cone approximation
pub fn classify_gmi_interval(distance: Rational, c_g: Rational, dt_g: Rational) -> Result<GmiConeCheck, ConeError> {
    if distance < Rational::from_integer(0) {
        return Err(ConeError::NegativeDistance);
    }
    if c_g <= Rational::from_integer(0) {
        return Err(ConeError::NonPositiveSpeedLimit);
    }
    if dt_g <= Rational::from_integer(0) {
        return Err(ConeError::NonPositiveDeltaTime);
    }

    let max_distance = c_g * dt_g;
    let interval_sq = max_distance * max_distance - distance * distance;

    let class = if distance < max_distance {
        CausalClass::Timelike
    } else if distance == max_distance {
        CausalClass::Null
    } else {
        CausalClass::Spacelike
    };

    Ok(GmiConeCheck {
        distance,
        max_distance,
        interval_sq,
        class,
    })
}
