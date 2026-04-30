use coh_core::cohbit::CohBit;
use coh_core::atom::CohAtom;
use coh_physics::gauge::CohGaugeField;
use std::mem::size_of;

#[test]
fn test_memory_footprint() {
    println!("--- Resource Demand: Memory Footprint ---");
    println!("Size of CohBit: {} bytes", size_of::<CohBit>());
    println!("Size of CohAtom: {} bytes", size_of::<CohAtom>());
    println!("Size of CohGaugeField: {} bytes", size_of::<CohGaugeField>());
    
    // In a verifier node with 1M bits, how much RAM?
    let ram_1m_bits = (size_of::<CohBit>() * 1_000_000) as f64 / 1024.0 / 1024.0;
    println!("RAM for 1M CohBits: {:.2} MB", ram_1m_bits);
    
    assert!(size_of::<CohBit>() < 512); // Keep bits lean
}

#[test]
fn test_path_integral_scaling_demands() {
    use coh_core::trajectory::path_integral::CohHistory;
    use coh_core::types::{Hash32, Decision};
    use std::time::Instant;

    let steps_count = 10_000;
    let bit = CohBit {
        from_state: Hash32([0; 32]),
        to_state: Hash32([1; 32]),
        rv_status: Decision::Accept,
        ..Default::default()
    };
    
    let history = CohHistory {
        steps: vec![bit; steps_count],
    };
    
    let atom = CohAtom::default();
    
    let start = Instant::now();
    let prob = history.path_probability(&atom, 1.0, 0.0, 1.0, 1.0);
    let duration = start.elapsed();
    
    println!("--- Resource Demand: Path Integral Scaling ---");
    println!("Steps: {}", steps_count);
    println!("Duration: {:?}", duration);
    println!("Probability: {}", prob);
    
    assert!(duration.as_millis() < 500); // Should be sub-second for 10k steps
}
