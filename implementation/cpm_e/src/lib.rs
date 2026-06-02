use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IomToCpiomE {
    pub gen1_voltage: f32,
    pub gen2_voltage: f32,
    pub bus_load_amps: f32,
    pub battery_soc_pct: f32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CpiomEToIom {
    pub gen1_online: bool,
    pub gen2_online: bool,
    pub bus_tie_closed: bool,
    pub load_shed_active: bool,
    pub battery_charging: bool,
}
