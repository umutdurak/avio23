//! Local test bench for the fuel controller.
//!
//! Usage:
//!   cargo run --bin test_bench                       # default scenario
//!   cargo run --bin test_bench -- --validate-config  # Part 1 validator
//!   cargo run --bin test_bench -- --all              # run every scenario
//!   cargo run --bin test_bench -- --scenario noisy   # specific scenario

use std::env;

use fuel_controller_starter::config;
use fuel_controller_starter::scenario::{self, Scenario};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--validate-config") {
        let report = config::validate("cpm_f.yaml");
        report.print();
        std::process::exit(if report.ok { 0 } else { 1 });
    }

    let scenarios: Vec<Scenario> = if args.iter().any(|a| a == "--all") {
        vec![
            Scenario::symmetric_burn(),
            Scenario::asymmetric_start(),
            Scenario::noisy_sensors(),
        ]
    } else if let Some(idx) = args.iter().position(|a| a == "--scenario") {
        let pick = args.get(idx + 1).map(String::as_str).unwrap_or("symmetric");
        match pick {
            "symmetric" => vec![Scenario::symmetric_burn()],
            "asymmetric" => vec![Scenario::asymmetric_start()],
            "noisy" => vec![Scenario::noisy_sensors()],
            other => {
                eprintln!("unknown scenario '{}'", other);
                std::process::exit(2);
            }
        }
    } else {
        vec![Scenario::symmetric_burn()]
    };

    let mut all_passed = true;
    for s in &scenarios {
        let result = scenario::run(s, /*verbose=*/ true, /*write_csv=*/ true);
        scenario::print_summary(&result);
        if !result.passed() {
            all_passed = false;
        }
    }

    if scenarios.len() == 1 {
        println!("\n[INFO] CSV trace written to out/trace.csv");
    }
    std::process::exit(if all_passed { 0 } else { 1 });
}
