use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IomToCpiomA {
    pub cabin_temp_c: f32,
    pub target_temp_c: f32,
    pub bleed_pressure_psi: f32,
    pub pack_flow_requested: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CpiomAToIom {
    pub pack_valve_open: bool,
    pub bleed_valve_pct: f32,
    pub cabin_temp_error_c: f32,
    pub ecs_fault: bool,
}
