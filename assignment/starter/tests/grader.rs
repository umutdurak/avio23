//! Integration grader.
//!
//! Run with `cargo test --release -- --nocapture`. The tutor uses this exact
//! file to grade your submission, so what passes here is what scores points.

use fuel_controller_starter::config;
use fuel_controller_starter::scenario::{self, Scenario};

const CONFIG_POINTS: u32 = 30;
const SYM_POINTS: u32 = 25;
const ASYM_POINTS: u32 = 25;
const NOISY_POINTS: u32 = 10;
const HYSTERESIS_BONUS: u32 = 5;

#[test]
fn grade() {
    let mut score: u32 = 0;
    let mut bonus: u32 = 0;
    let mut log: Vec<String> = Vec::new();

    let cfg = config::validate("cpm_f.yaml");
    if cfg.ok {
        score += CONFIG_POINTS;
        log.push(format!("[+{}] config validates", CONFIG_POINTS));
    } else {
        log.push("[+0]  config has errors:".into());
        for m in &cfg.messages {
            log.push(format!("       - {}", m));
        }
    }

    let mut total_switches = 0;
    for (sc, pts) in [
        (Scenario::symmetric_burn(), SYM_POINTS),
        (Scenario::asymmetric_start(), ASYM_POINTS),
        (Scenario::noisy_sensors(), NOISY_POINTS),
    ] {
        let r = scenario::run(&sc, false, false);
        total_switches += r.switch_count;
        if r.passed() {
            score += pts;
            log.push(format!(
                "[+{}] scenario '{}' passed (switches={})",
                pts, sc.name, r.switch_count
            ));
        } else {
            log.push(format!(
                "[+0]  scenario '{}' — {} violations (first: {})",
                sc.name,
                r.violations.len(),
                r.violations.first().map(String::as_str).unwrap_or("?")
            ));
        }
    }

    if total_switches < 60 * 3 {
        bonus += HYSTERESIS_BONUS;
        log.push(format!(
            "[+{}] hysteresis bonus (total switches across scenarios: {})",
            HYSTERESIS_BONUS, total_switches
        ));
    }

    println!();
    println!("============================================================");
    println!("  Avio23 Fuel Management Assignment — Grader");
    println!("============================================================");
    for l in &log {
        println!("{}", l);
    }
    println!("------------------------------------------------------------");
    println!(" Score:  {} / 100   (bonus +{})", score, bonus);
    println!(" Total:  {} / 100", score + bonus);
    println!("============================================================");

    assert!(
        score >= 60,
        "Score {} below pass threshold of 60",
        score
    );
}
