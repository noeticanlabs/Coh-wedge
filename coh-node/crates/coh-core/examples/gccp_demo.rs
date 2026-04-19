//! GCCP Demo - Demonstrates the Governed Compute Control Plane
use coh_core::gccp::state::{PowerState, QueueState, ThermalState};
fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║    GCCP Demo - Compute Control    ║");
    println!("╚══════════════════════════════╝\n");
    // Create GCCP states with u128 mill-units
    let thermal = ThermalState::new(45_000, 50_000, 2_000); // 45°C, 50°C, 2°C/s rise
    let power = PowerState::new(150_000, 400_000, 50_000); // 150W, 400W cap, 50W margin
    let queue = QueueState::new(25, 50, 80); // 25 deep, 50 age, 80% mix

    println!("Thermal (T):");
    println!(
        "  t_die:  {}mC ({}°C)",
        thermal.t_die,
        thermal.t_die as f64 / 1000.0
    );
    println!(
        "  t_hot:  {}mC ({}°C)",
        thermal.t_hot,
        thermal.t_hot as f64 / 1000.0
    );
    println!("  t_rise: {}mC/s", thermal.t_rise);
    println!();
    println!("Power (P):");
    println!(
        "  p_now:   {}mW ({}W)",
        power.p_now,
        power.p_now as f64 / 1000.0
    );
    println!(
        "  p_cap:  {}mW ({}W)",
        power.p_cap,
        power.p_cap as f64 / 1000.0
    );
    println!("  p_margin: {}mW", power.p_margin);
    println!();
    println!("Queue (Q):");
    println!("  q_depth: {}", queue.q_depth);
    println!("  q_age:  {}", queue.q_age);
    println!("  q_mix:  {}%", queue.q_mix);
    println!();
    println!("GCCP ready for compute governance!");
}
