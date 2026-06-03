//! Fuel controller starter library.
//!
//! Students edit `controller.rs`. Everything else (sensor/actuator types,
//! scenario runner, config validator) is fixed.

pub mod config;
pub mod controller;
pub mod scenario;

pub use controller::{Controller, select_fill_tank, select_source_tank};

/// Which fuel tank a valve is connected to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tank {
    Left,
    Right,
}

/// Sensor readings made available to the controller every frame.
#[derive(Copy, Clone, Debug)]
pub struct FuelSensors {
    pub left_liters: f32,
    pub right_liters: f32,
    pub fuel_flow_lps: f32,
}

/// Actuator commands the controller produces every frame.
#[derive(Copy, Clone, Debug)]
pub struct FuelActuators {
    pub pump_source: Tank,
    pub fill_target: Tank,
    pub refueling_active: bool,
}

/// Cockpit indicators the controller drives every frame.
#[derive(Copy, Clone, Debug)]
pub struct FuelIndicators {
    pub total_fuel_liters: f32,
    pub time_until_empty_s: f32,
}

/// Static system parameters.
#[derive(Copy, Clone, Debug)]
pub struct FuelParameters {
    pub left_capacity_liters: f32,
    pub right_capacity_liters: f32,
}

impl Default for FuelParameters {
    fn default() -> Self {
        Self {
            left_capacity_liters: 300.0,
            right_capacity_liters: 300.0,
        }
    }
}

/// Real-time / safety limits the controller must respect.
pub const VALVE_COOLDOWN_S: f32 = 0.8;
pub const IMBALANCE_LIMIT_L: f32 = 10.0;
pub const MAJOR_FRAME_S: f32 = 0.04;
