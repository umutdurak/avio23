# Avio23 Student Manual

A short reference for the **Aeronautical Informatics** course assignment. Read this once before you start; come back to it when you get stuck.

---

## 1. What is Avio23?

Avio23 is a teaching platform that lets you write avionics applications the way real aircraft do — as **partitions** running on a **time-sliced, statically scheduled hypervisor**, communicating over **sampling ports**, and exchanging data between computers over a **deterministic network**.

It is a software-only stand-in for an **Integrated Modular Avionics (IMA)** system. Real IMA systems power the cockpit of every modern commercial aircraft (A350, A380, B787, Embraer E2, Bombardier Global, etc.). Avio23 lets you touch the same ideas — minus the certification cost.

Two pieces of vocabulary you will use today:

| Term | What it means in Avio23 |
|------|------------------------|
| **Partition** | An isolated process with its own memory and its own scheduled time slot. Think "container, but with a deadline." |
| **Major frame** | The repeating period (40 ms in Avio23) in which every partition gets exactly one turn to run. |
| **Time window** | One partition's slot within the major frame: `(offset, duration)`. |
| **Sampling port** | One-way mailbox between partitions. The writer overwrites the latest value; the reader sees whatever was last written. |
| **CPM** | Core Processing Module — one physical computer hosting several partitions. Avio23 has five: a sensor gateway and four domain CPMs (Landing Gear, Fuel, ECS, Electrical). |

The IMA standard behind all this is **ARINC 653**.

---

## 2. The Avio23 architecture (at a glance)

```
              +--------------------+
              |  sim_gateway (IOM) |   <- generates simulated sensor data
              +----------+---------+
                         |
       +--------+--------+--------+--------+
       |        |        |        |        |
     CPM-L   CPM-F     CPM-A    CPM-E
   (Gear)   (Fuel)    (ECS)    (Electrical)
```

Each box is a Docker container running a local hypervisor (`a653rs-linux`). The boxes talk over a virtual AFDX network — UDP packets on a dedicated Docker bridge.

**Your assignment lives entirely inside CPM-F.** You won't touch the other CPMs, the network, or the gateway. You will:

1. **Configure** a new partition (`fuel_controller`) inside CPM-F's hypervisor schedule.
2. **Implement** the decision logic for that partition.

You will do both things in a **self-contained Rust crate** (`assignment/starter/`) that runs on your laptop without Docker. The tutor will demo your passing solution on the full Avio23 stack at the end of the session.

---

## 3. Zoom in: CPM-F

The Fuel CPM normally hosts three partitions:

| Partition | What it does | Slot |
|-----------|-------------|------|
| `cpm_f_gateway` | Talks to the AFDX network, fans data in/out | 0 ms → 3 ms |
| `fuel_quantity` | Estimates total/per-tank fuel mass | 4 ms → 6 ms |
| `transfer_pump` | Drives the wing-to-wing transfer pumps | 8 ms → 10 ms |

For this assignment a **fourth partition** is added: **`fuel_controller`**. It receives the tank levels + fuel flow, and decides:

- which tank the **pump valve** draws from (Left or Right),
- which tank the **fill valve** routes refuel into (Left or Right),
- whether to set the **refueling-active** indicator.

Adding this partition to the schedule is **Part 1** of the assignment. Implementing its decision logic is **Part 2**.

---

## 4. Your working environment

Everything you need is in `assignment/starter/`:

```
starter/
├── Cargo.toml
├── cpm_f.yaml          <- config you edit in Part 1
├── src/
│   ├── lib.rs          <- given: sensor/actuator types
│   ├── controller.rs   <- given: controller skeleton + TODO function
│   └── bin/
│       └── test_bench.rs  <- runs your controller against a scenario
└── tests/
    └── grader.rs       <- the same tests the tutor will grade you with
```

You will edit **two files**:

- `cpm_f.yaml` — fill in the `fuel_controller` partition window + ports.
- `src/controller.rs` — implement `select_source_tank()` (and optional bonus functions).

Everything else is provided. You can read the rest to understand context, but you don't need to modify it.

---

## 5. How to build and run

You need Rust 1.80+ (`rustup install stable`). No Docker, no Linux VM.

```bash
cd assignment/starter

# Run the local test bench (replays a 600 s flight scenario through your controller)
cargo run --bin test_bench

# Validate your cpm_f.yaml
cargo run --bin test_bench -- --validate-config

# Run the full grader (config validation + scenario replay + scoring)
cargo test --release -- --nocapture
```

The test bench prints a live trace of fuel levels, valve positions, and any requirement violations:

```
 t=  0.04s  L=250.0  R=250.0  ΔL-R=  0.0  src=Left  valve_age=0.0  flow=0.10 lps
 t=  0.08s  L=250.0  R=249.9  ΔL-R=  0.1  src=Left  valve_age=0.04 flow=0.10 lps
 ...
[VIOLATION] t=18.36s — |L-R| = 10.4 L exceeds 10.0 L limit
```

It also writes `out/trace.csv` so you can plot the run (any spreadsheet or `gnuplot`).

---

## 6. How the grader scores you

The grader runs the same scenarios on every submission and produces a score out of 100.

| Stage | Points | Pass criterion |
|-------|--------|----------------|
| Config validation | 30 | `cpm_f.yaml` parses, major frame budget ≤ 40 ms, every port has a writer and a reader, partition periods consistent with windows |
| Scenario 1: Symmetric burn | 25 | `|left − right|` stays ≤ 10 L throughout, valve toggles ≥ 0.8 s apart |
| Scenario 2: Asymmetric start | 25 | Same constraints, starting from 200 L / 280 L |
| Scenario 3: Sensor noise | 10 | Same constraints with ±0.5 L noise on level sensors |
| Bonus: hysteresis | +5 | Valve toggles fewer than 60 times in the 600 s run |
| Bonus: refuel logic | +5 | `select_fill_tank` keeps balance while filling |

Pass mark is **60**. Anything above is honors-track.

---

## 7. Common pitfalls

- **Forgetting the cooldown.** Toggling on every tick where `|L−R| > 10` will violate the 0.8 s valve constraint. You need to read `seconds_since_switch` and refuse to switch if it's too small.
- **Chattering at the boundary.** Switching exactly at `|L−R| = 10.0` then back at `|L−R| = 10.0` again ping-pongs the valve. Add a small dead-band (the hysteresis bonus).
- **Wrong slot in the YAML.** If you put `fuel_controller` at `offset: 8ms, duration: 2ms` you overlap `transfer_pump`. The config validator catches this.
- **Period mismatch.** Your partition's `period` must equal `major_frame` (40 ms). The validator catches this too.
- **Forgetting to register the ports.** The controller can't read sensors if you didn't add the sampling-port wiring in `cpm_f.yaml`.

---

## 8. Submitting

At the end of the session, leave the following two files in `assignment/starter/`:

- `cpm_f.yaml`
- `src/controller.rs`

The tutor runs `cargo test --release` against your folder. That run produces your grade.

Good luck. The fuel pump is yours.
