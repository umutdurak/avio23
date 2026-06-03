//! Scenario replay engine.
//!
//! Drives the student's `Controller` through a deterministic flight scenario,
//! tracks requirement violations, and (optionally) writes a CSV trace.

use std::fs;
use std::io::Write;

use crate::{
    Controller, FuelSensors, Tank, IMBALANCE_LIMIT_L, MAJOR_FRAME_S, VALVE_COOLDOWN_S,
};

#[derive(Clone)]
pub struct Scenario {
    pub name: &'static str,
    pub left_start_l: f32,
    pub right_start_l: f32,
    pub flow_lps: f32,
    pub duration_s: f32,
    pub noise_l: f32,
}

impl Scenario {
    pub fn symmetric_burn() -> Self {
        Self {
            name: "symmetric_burn",
            left_start_l: 250.0,
            right_start_l: 250.0,
            flow_lps: 0.10,
            duration_s: 600.0,
            noise_l: 0.0,
        }
    }
    pub fn asymmetric_start() -> Self {
        Self {
            name: "asymmetric_start",
            left_start_l: 200.0,
            right_start_l: 280.0,
            flow_lps: 0.12,
            duration_s: 600.0,
            noise_l: 0.0,
        }
    }
    pub fn noisy_sensors() -> Self {
        Self {
            name: "noisy_sensors",
            left_start_l: 240.0,
            right_start_l: 260.0,
            flow_lps: 0.10,
            duration_s: 600.0,
            noise_l: 0.5,
        }
    }
}

pub struct ScenarioResult {
    pub name: &'static str,
    pub violations: Vec<String>,
    pub switch_count: usize,
    pub final_left: f32,
    pub final_right: f32,
}

impl ScenarioResult {
    pub fn passed(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn run(scenario: &Scenario, verbose: bool, write_csv: bool) -> ScenarioResult {
    let mut controller = Controller::new();
    let mut left = scenario.left_start_l;
    let mut right = scenario.right_start_l;
    let mut t = 0.0f32;

    let mut violations: Vec<String> = Vec::new();
    let mut last_source = controller.current_source();
    let mut last_switch_time = f32::NEG_INFINITY;
    let mut switch_count: usize = 0;

    let mut rng: u32 = 0xC0FFEE;
    let mut noise = || -> f32 {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let r = (rng >> 8) as f32 / ((1u32 << 24) as f32) - 0.5;
        r * 2.0 * scenario.noise_l
    };

    let mut csv = String::new();
    if write_csv {
        csv.push_str("t,left,right,delta,source,since_switch\n");
    }

    let mut next_log_t = 0.0f32;

    while t < scenario.duration_s && (left > 0.0 || right > 0.0) {
        let sensors = FuelSensors {
            left_liters: left + noise(),
            right_liters: right + noise(),
            fuel_flow_lps: scenario.flow_lps,
        };
        let (act, _ind) = controller.step(sensors, t);

        if act.pump_source != last_source {
            switch_count += 1;
            let interval = if last_switch_time.is_finite() {
                t - last_switch_time
            } else {
                f32::INFINITY
            };
            if interval < VALVE_COOLDOWN_S - 1e-4 {
                violations.push(format!(
                    "[VIOLATION] t={:.2}s — valve switched after only {:.3}s (< {:.1}s cooldown)",
                    t, interval, VALVE_COOLDOWN_S
                ));
            }
            last_switch_time = t;
            last_source = act.pump_source;
        }

        let consumed = scenario.flow_lps * MAJOR_FRAME_S;
        match act.pump_source {
            Tank::Left => left = (left - consumed).max(0.0),
            Tank::Right => right = (right - consumed).max(0.0),
        }

        if left > 0.0 && right > 0.0 && (left - right).abs() > IMBALANCE_LIMIT_L {
            violations.push(format!(
                "[VIOLATION] t={:.2}s — |L-R| = {:.2}L exceeds {:.1}L",
                t,
                (left - right).abs(),
                IMBALANCE_LIMIT_L
            ));
        }

        if write_csv {
            let since = if last_switch_time.is_finite() {
                t - last_switch_time
            } else {
                -1.0
            };
            csv.push_str(&format!(
                "{:.3},{:.3},{:.3},{:.3},{:?},{:.3}\n",
                t,
                left,
                right,
                left - right,
                act.pump_source,
                since
            ));
        }

        if verbose && t >= next_log_t {
            println!(
                " t={:6.2}s  L={:6.1}  R={:6.1}  ΔL-R={:6.2}  src={:?}",
                t,
                left,
                right,
                left - right,
                act.pump_source
            );
            next_log_t += 5.0;
        }

        t += MAJOR_FRAME_S;
    }

    if write_csv {
        let _ = fs::create_dir_all("out");
        if let Ok(mut f) = fs::File::create("out/trace.csv") {
            let _ = f.write_all(csv.as_bytes());
        }
    }

    ScenarioResult {
        name: scenario.name,
        violations,
        switch_count,
        final_left: left,
        final_right: right,
    }
}

pub fn print_summary(r: &ScenarioResult) {
    println!();
    if r.passed() {
        println!("[OK] scenario '{}' — no violations", r.name);
    } else {
        println!(
            "[FAIL] scenario '{}' — {} violations:",
            r.name,
            r.violations.len()
        );
        for v in r.violations.iter().take(10) {
            println!("  {}", v);
        }
        if r.violations.len() > 10 {
            println!("  ... and {} more", r.violations.len() - 10);
        }
    }
    println!(
        "[INFO] final state: L={:.1}L  R={:.1}L  valve switches={}",
        r.final_left, r.final_right, r.switch_count
    );
}
