use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IomToCpiomL {
    pub airspeed_kt: f32,
    pub lg_lever_down: bool,
    pub left_brake_pedal: f32,
    pub right_brake_pedal: f32,
    pub rudder_pedal: f32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CpiomLToIom {
    pub gear_deployed: bool,
    pub left_brake_pressure: f32,
    pub right_brake_pressure: f32,
    pub nose_wheel_angle: f32,
}
