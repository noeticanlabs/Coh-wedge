use coh_core::types::CertifiedMorphism;
use serde::{Deserialize, Serialize};

/// A Trace is a verified sequence of morphisms where the post-state
/// of each morphism matches the pre-state of the next.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trace {
    pub morphisms: Vec<CertifiedMorphism>,
}

#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("Trace is empty")]
    EmptyTrace,
    #[error("Trace composition mismatch at index {0}: v_post {1} != v_pre {2}")]
    CompositionMismatch(usize, u128, u128),
    #[error("Arithmetic overflow during trace collapse")]
    Overflow,
}

impl Trace {
    /// Create a new Trace from a sequence of morphisms, verifying composition.
    pub fn try_from_morphisms(morphisms: Vec<CertifiedMorphism>) -> Result<Self, TraceError> {
        if morphisms.is_empty() {
            return Err(TraceError::EmptyTrace);
        }

        for i in 0..morphisms.len() - 1 {
            let current = &morphisms[i];
            let next = &morphisms[i + 1];
            if current.v_post != next.v_pre {
                return Err(TraceError::CompositionMismatch(i, current.v_post, next.v_pre));
            }
        }

        Ok(Self { morphisms })
    }

    /// Collapse the entire trace into a single aggregate CertifiedMorphism.
    /// This is the "Trace Law" implementation.
    pub fn collapse(&self) -> Result<CertifiedMorphism, TraceError> {
        if self.morphisms.is_empty() {
            return Err(TraceError::EmptyTrace);
        }

        let first = &self.morphisms[0];
        let last = &self.morphisms[self.morphisms.len() - 1];

        let mut total_spend: u128 = 0;
        let mut total_defect: u128 = 0;
        let mut total_authority: u128 = 0;

        for m in &self.morphisms {
            total_spend = total_spend.checked_add(m.spend).ok_or(TraceError::Overflow)?;
            total_defect = total_defect.checked_add(m.defect).ok_or(TraceError::Overflow)?;
            total_authority = total_authority.checked_add(m.authority).ok_or(TraceError::Overflow)?;
        }

        Ok(CertifiedMorphism {
            v_pre: first.v_pre,
            v_post: last.v_post,
            spend: total_spend,
            defect: total_defect,
            authority: total_authority,
        })
    }

    /// Get a segment of the trace as a Slab.
    pub fn segment(&self, start: usize, end: usize) -> Result<Slab, TraceError> {
        if start >= self.morphisms.len() || end > self.morphisms.len() || start >= end {
            return Err(TraceError::EmptyTrace); // Or a specific RangeError
        }

        let sub_morphisms = self.morphisms[start..end].to_vec();
        let sub_trace = Trace::try_from_morphisms(sub_morphisms)?;
        
        Ok(Slab {
            trace: sub_trace,
            range: (start, end),
        })
    }
}

/// A Slab is a named or bounded segment of a Trace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slab {
    pub trace: Trace,
    pub range: (usize, usize),
}

impl Slab {
    /// Returns the aggregated morphism for this slab.
    pub fn aggregate(&self) -> Result<CertifiedMorphism, TraceError> {
        self.trace.collapse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_trace_composition() {
        let m1 = CertifiedMorphism::new(100, 80, 20, 0, 0);
        let m2 = CertifiedMorphism::new(80, 50, 30, 0, 0);
        let trace = Trace::try_from_morphisms(vec![m1, m2]).unwrap();
        
        let collapsed = trace.collapse().unwrap();
        assert_eq!(collapsed.v_pre, 100);
        assert_eq!(collapsed.v_post, 50);
        assert_eq!(collapsed.spend, 50);
        assert!(collapsed.is_certified());
    }

    #[test]
    fn test_invalid_trace_mismatch() {
        let m1 = CertifiedMorphism::new(100, 80, 20, 0, 0);
        let m2 = CertifiedMorphism::new(70, 50, 20, 0, 0); // 80 != 70
        let result = Trace::try_from_morphisms(vec![m1, m2]);
        assert!(matches!(result, Err(TraceError::CompositionMismatch(0, 80, 70))));
    }

    #[test]
    fn test_trace_with_authority() {
        let m1 = CertifiedMorphism::new(100, 110, 0, 0, 10); // Auth used to increase V
        let m2 = CertifiedMorphism::new(110, 50, 60, 0, 0);
        let trace = Trace::try_from_morphisms(vec![m1, m2]).unwrap();
        
        let collapsed = trace.collapse().unwrap();
        assert_eq!(collapsed.authority, 10);
        assert!(collapsed.is_certified());
    }

    #[test]
    fn test_slab_segmentation() {
        let m1 = CertifiedMorphism::new(100, 90, 10, 0, 0);
        let m2 = CertifiedMorphism::new(90, 80, 10, 0, 0);
        let m3 = CertifiedMorphism::new(80, 70, 10, 0, 0);
        let trace = Trace::try_from_morphisms(vec![m1, m2, m3]).unwrap();
        
        let slab = trace.segment(1, 3).unwrap(); // m2, m3
        let agg = slab.aggregate().unwrap();
        assert_eq!(agg.v_pre, 90);
        assert_eq!(agg.v_post, 70);
        assert_eq!(agg.spend, 20);
    }
}
