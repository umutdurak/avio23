use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IomToCpiomF {
    pub left_tank_kg: f32,
    pub right_tank_kg: f32,
    pub center_tank_kg: f32,
    pub fuel_temp_c: f32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CpiomFToIom {
    pub total_fuel_kg: f32,
    pub fuel_imbalance_kg: f32,
    pub transfer_pump_active: bool,
    pub low_fuel_warning: bool,
}
