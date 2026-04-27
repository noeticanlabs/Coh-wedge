use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThermalState {
    pub die_temp: f64,
    pub hotspot_temp: f64,
    pub thermal_slope: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerState {
    pub draw_watts: f64,
    pub cap_watts: f64,
    pub margin_watts: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueueState {
    pub depth: u32,
    pub age_pressure: f64,
    pub class_mix: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UtilizationState {
    pub compute_percent: f64,
    pub memory_bw_percent: f64,
    pub interconnect_percent: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryState {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub fragmentation: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskState {
    pub retry_count: u32,
    pub timeout_ms: u32,
    pub is_throttled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetState {
    pub energy_joules: f64,
    pub latency_ms: f64,
    pub stability_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlContext {
    pub policy_hash: String,
    pub profile: String,
    pub mode: String,
    pub class: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GccpState {
    pub thermal: ThermalState,
    pub power: PowerState,
    pub queue: QueueState,
    pub utilization: UtilizationState,
    pub memory: MemoryState,
    pub risk: RiskState,
    pub budget: BudgetState,
    pub context: ControlContext,
}

impl Default for GccpState {
    fn default() -> Self {
        Self {
            thermal: ThermalState { die_temp: 45.0, hotspot_temp: 50.0, thermal_slope: 0.0 },
            power: PowerState { draw_watts: 100.0, cap_watts: 300.0, margin_watts: 200.0 },
            queue: QueueState { depth: 0, age_pressure: 0.0, class_mix: "default".to_string() },
            utilization: UtilizationState { compute_percent: 0.0, memory_bw_percent: 0.0, interconnect_percent: 0.0 },
            memory: MemoryState { used_bytes: 0, total_bytes: 32 * 1024 * 1024 * 1024, fragmentation: 0.0 },
            risk: RiskState { retry_count: 0, timeout_ms: 100, is_throttled: false },
            budget: BudgetState { energy_joules: 1000.0, latency_ms: 10.0, stability_score: 1.0 },
            context: ControlContext { 
                policy_hash: "0000".to_string(), 
                profile: "default".to_string(), 
                mode: "normal".to_string(), 
                class: "standard".to_string() 
            },
        }
    }
}
