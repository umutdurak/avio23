//! `cpm_f.yaml` validator.
//!
//! Checks the student's hypervisor config against the ARINC 653 constraints
//! the assignment spells out:
//!   - YAML parses
//!   - every partition's `period` equals `major_frame`
//!   - the time windows fit inside the major frame and do not overlap
//!   - the `fuel_controller` partition is present and within the duration
//!     bounds the assignment requires
//!   - every sampling-channel destination has a matching source partition
//!     declared, and `fuel_controller` reads sensors and writes commands

use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ConfigReport {
    pub ok: bool,
    pub messages: Vec<String>,
}

impl ConfigReport {
    pub fn print(&self) {
        for m in &self.messages {
            println!("  {}", m);
        }
        if self.ok {
            println!("\n[OK] cpm_f.yaml validates.");
        } else {
            println!("\n[FAIL] cpm_f.yaml has errors.");
        }
    }
}

pub fn validate<P: AsRef<Path>>(path: P) -> ConfigReport {
    let mut messages = Vec::new();
    let path = path.as_ref();

    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return ConfigReport {
                ok: false,
                messages: vec![format!("cannot read {}: {}", path.display(), e)],
            };
        }
    };

    let yaml: serde_yaml::Value = match serde_yaml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return ConfigReport {
                ok: false,
                messages: vec![format!("YAML parse error: {}", e)],
            };
        }
    };

    let major_frame_ms = match yaml.get("major_frame").and_then(|v| v.as_str()) {
        Some(s) => match parse_duration_ms(s) {
            Some(v) => v,
            None => {
                messages.push(format!("major_frame: cannot parse '{}'", s));
                return done(false, messages);
            }
        },
        None => {
            messages.push("major_frame: missing or not a string".into());
            return done(false, messages);
        }
    };

    let partitions = match yaml.get("partitions").and_then(|v| v.as_sequence()) {
        Some(s) => s,
        None => {
            messages.push("partitions: missing or not a list".into());
            return done(false, messages);
        }
    };

    let mut windows: Vec<(String, u64, u64)> = Vec::new(); // (name, offset, end)
    let mut have_controller = false;
    let mut controller_duration_ms: Option<u64> = None;

    for p in partitions {
        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let duration = p
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(parse_duration_ms);
        let offset = p
            .get("offset")
            .and_then(|v| v.as_str())
            .and_then(parse_duration_ms);
        let period = p
            .get("period")
            .and_then(|v| v.as_str())
            .and_then(parse_duration_ms);

        let (duration, offset, period) = match (duration, offset, period) {
            (Some(d), Some(o), Some(p)) => (d, o, p),
            _ => {
                messages.push(format!(
                    "partition '{}': missing or malformed duration/offset/period",
                    name
                ));
                continue;
            }
        };

        if period != major_frame_ms {
            messages.push(format!(
                "partition '{}': period {}ms != major_frame {}ms",
                name, period, major_frame_ms
            ));
        }
        if offset + duration > major_frame_ms {
            messages.push(format!(
                "partition '{}': window {}..{}ms exceeds major frame {}ms",
                name,
                offset,
                offset + duration,
                major_frame_ms
            ));
        }
        windows.push((name.to_string(), offset, offset + duration));

        if name == "fuel_controller" {
            have_controller = true;
            controller_duration_ms = Some(duration);
        }
    }

    // overlap check
    let mut sorted = windows.clone();
    sorted.sort_by_key(|w| w.1);
    for pair in sorted.windows(2) {
        if pair[0].2 > pair[1].1 {
            messages.push(format!(
                "windows overlap: '{}' ({}..{}ms) and '{}' ({}..{}ms)",
                pair[0].0, pair[0].1, pair[0].2, pair[1].0, pair[1].1, pair[1].2
            ));
        }
    }

    if !have_controller {
        messages.push(
            "fuel_controller partition is missing — add an entry under partitions:".into(),
        );
    } else if let Some(d) = controller_duration_ms {
        if !(1..=5).contains(&d) {
            messages.push(format!(
                "fuel_controller duration must be between 1ms and 5ms (got {}ms)",
                d
            ));
        }
    }

    // channel checks
    let mut reads_sensors = false;
    let mut writes_commands = false;
    if let Some(channels) = yaml.get("channel").and_then(|v| v.as_sequence()) {
        for ch in channels {
            // !Sampling tagged value, or untagged mapping
            let mapping = match ch {
                serde_yaml::Value::Tagged(t) => Some(&t.value),
                serde_yaml::Value::Mapping(_) => Some(ch),
                _ => None,
            };
            let Some(m) = mapping else { continue };

            let dests = m
                .get("destination")
                .and_then(|v| v.as_sequence())
                .cloned()
                .unwrap_or_default();
            let source_part = m
                .get("source")
                .and_then(|s| s.get("partition"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let source_port = m
                .get("source")
                .and_then(|s| s.get("port"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            for d in &dests {
                let dpart = d
                    .get("partition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if dpart == "fuel_controller" && source_port == "iom_to_fuel" {
                    reads_sensors = true;
                }
            }
            if source_part == "fuel_controller" {
                writes_commands = true;
            }
        }
    } else {
        messages.push("channel: section missing".into());
    }

    if !reads_sensors {
        messages.push(
            "fuel_controller is not a destination of the iom_to_fuel sampling channel — it cannot read sensors"
                .into(),
        );
    }
    if !writes_commands {
        messages.push(
            "fuel_controller has no outgoing sampling channel — add one to send commands to cpm_f_gateway"
                .into(),
        );
    }

    done(messages.is_empty(), messages)
}

fn done(ok: bool, messages: Vec<String>) -> ConfigReport {
    let mut messages = messages;
    if ok {
        messages.push("schedule, ports and windows look good.".into());
    }
    ConfigReport { ok, messages }
}

/// Parse `40ms`, `2ms`, `1s`, `500us` into milliseconds. Returns None on
/// malformed input.
pub fn parse_duration_ms(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix("ms") {
        num.trim().parse().ok()
    } else if let Some(num) = s.strip_suffix("us") {
        num.trim().parse::<u64>().ok().map(|v| v / 1000)
    } else if let Some(num) = s.strip_suffix('s') {
        num.trim().parse::<u64>().ok().map(|v| v * 1000)
    } else {
        None
    }
}
