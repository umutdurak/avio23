//! Fuel controller.
//!
//! You edit this file. The pieces you must implement are marked with
//! `TODO(student)`. Everything else is provided so you can focus on
//! the decision logic.

use crate::{FuelActuators, FuelIndicators, FuelSensors, Tank};

/// Holds the controller's state between frames.
///
/// Avio23 invokes `step` every 40 ms (one major frame). The state struct
/// remembers which tank the valve is currently connected to and when it
/// last switched, so your `select_source_tank` function gets the
/// `seconds_since_switch` value pre-computed.
pub struct Controller {
    current_source: Tank,
    current_fill: Tank,
    last_switch_time_s: f32,
    refueling: bool,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Self {
        Self {
            current_source: Tank::Left,
            current_fill: Tank::Right,
            last_switch_time_s: f32::NEG_INFINITY,
            refueling: false,
        }
    }

    /// One control tick. Reads sensors, decides actuator state,
    /// computes indicators. Called every 40 ms by the test bench
    /// and by the real `fuel_controller` partition.
    pub fn step(&mut self, sensors: FuelSensors, t_s: f32) -> (FuelActuators, FuelIndicators) {
        let seconds_since_switch = if self.last_switch_time_s.is_finite() {
            t_s - self.last_switch_time_s
        } else {
            f32::INFINITY
        };

        let new_source = select_source_tank(
            sensors.left_liters,
            sensors.right_liters,
            self.current_source,
            seconds_since_switch,
        );

        if new_source != self.current_source {
            self.current_source = new_source;
            self.last_switch_time_s = t_s;
        }

        let new_fill = select_fill_tank(
            sensors.left_liters,
            sensors.right_liters,
            self.current_fill,
            seconds_since_switch,
        );
        self.current_fill = new_fill;

        let actuators = FuelActuators {
            pump_source: self.current_source,
            fill_target: self.current_fill,
            refueling_active: self.refueling,
        };

        let total = sensors.left_liters.max(0.0) + sensors.right_liters.max(0.0);
        let time_until_empty_s = if sensors.fuel_flow_lps > 1e-3 {
            total / sensors.fuel_flow_lps
        } else {
            f32::INFINITY
        };
        let indicators = FuelIndicators {
            total_fuel_liters: total,
            time_until_empty_s,
        };

        (actuators, indicators)
    }

    /// Test-bench / refueling control hook. Not part of the student task.
    pub fn set_refueling(&mut self, on: bool) {
        self.refueling = on;
    }

    /// Read-only accessor for tests.
    pub fn current_source(&self) -> Tank {
        self.current_source
    }
}

// ============================================================
//                     STUDENT FUNCTIONS
// ============================================================

/// Decide which tank the fuel pump valve should draw from on this tick.
///
/// # Inputs
/// - `left_liters`           — current fuel quantity, left tank
/// - `right_liters`          — current fuel quantity, right tank
/// - `current_source`        — tank the valve is connected to right now
/// - `seconds_since_switch`  — elapsed time since the last valve change
///                              (`f32::INFINITY` on the very first call)
///
/// # Requirements (from the assignment spec)
/// 1. `|left_liters - right_liters|` must stay ≤ 10.0 L at all times.
/// 2. You **must not** switch tanks unless `seconds_since_switch >= 0.8`.
/// 3. When one tank is empty, draw from the other regardless of the
///    imbalance constraint (engine starvation > balance).
///
/// This function is invoked every 40 ms (one major frame).
pub fn select_source_tank(
    left_liters: f32,
    right_liters: f32,
    current_source: Tank,
    seconds_since_switch: f32,
) -> Tank {
    // TODO(student): implement.
    //
    // The naive body below never switches the valve and will fail the
    // 10 L balance constraint after a few seconds of burn from one tank.
    let _ = (left_liters, right_liters, seconds_since_switch);
    current_source
}

/// (Bonus) Decide which tank the fill valve should route refuel into.
///
/// Same signature shape as `select_source_tank`. Only graded if you tackle
/// Bonus 2.
pub fn select_fill_tank(
    left_liters: f32,
    right_liters: f32,
    current_target: Tank,
    seconds_since_switch: f32,
) -> Tank {
    // TODO(student, optional): implement for Bonus 2.
    let _ = (left_liters, right_liters, seconds_since_switch);
    current_target
}

// ============================================================
//                          TESTS
// ============================================================
// Small unit checks you can run with `cargo test`. The real grader
// lives in tests/grader.rs and runs full scenarios.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_tanks_do_not_force_a_switch() {
        // Equal tanks → no reason to switch. Whatever you return,
        // it should not force a switch in fewer than 0.8 s.
        let _ = select_source_tank(250.0, 250.0, Tank::Left, 0.1);
    }

    #[test]
    fn empty_left_must_use_right() {
        assert_eq!(
            select_source_tank(0.0, 200.0, Tank::Left, f32::INFINITY),
            Tank::Right,
            "controller must abandon an empty tank"
        );
    }
}
