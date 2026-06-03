# Assignment: Fuel Management System on Avio23

**Course:** Aeronautical Informatics (TU Clausthal / DLR)
**Platform:** Avio23 — Integrated Modular Avionics teaching system
**Duration:** ~90 minutes hands-on within a 2-hour session
**Submission:** end of session, in-class grading

---

## Learning objectives

By the end of this assignment you will have:

1. **Configured** an ARINC 653 partition into a running hypervisor schedule — picked a time window, declared sampling ports, and reasoned about a major-frame budget.
2. **Implemented** a small piece of safety-related avionics logic — a fuel-balancing controller — that satisfies real time and resource constraints.
3. **Seen** your work execute on a real (simulated) IMA platform: five Docker-hosted CPMs, four domains, AFDX bridging.

---

## The system

You are designing the controller for a small two-tank fuel system. The aircraft has:

- **two wing tanks** (Left, Right) of equal capacity (default 300 L each),
- **one fuel pump** feeding the engine, sourced through a **pump valve** that connects to exactly one tank at a time,
- **one filler neck** feeding the tanks, routed through a **fill valve** that connects to exactly one tank at a time,
- **fuel level sensors** in each tank and a **fuel flow sensor** at the engine feed line.

The valve mechanism has a physical constraint: switching the pump valve **temporarily stalls fuel flow** while it disengages from one tank and engages the other. To prevent engine starvation and excessive wear:

> The pump valve must not be operated more than once every **0.8 seconds**.

For aircraft stability:

> The fuel level difference between the two tanks must not exceed **10 liters** at any time during normal operation.

---

## I/O contract

These are defined for you in `assignment/starter/src/lib.rs`. You do not modify them.

### Sensors (read-only inputs)

| Name | Type | Unit | Description |
|------|------|------|-------------|
| `left_liters` | `f32` | L | fuel in left tank |
| `right_liters` | `f32` | L | fuel in right tank |
| `fuel_flow_lps` | `f32` | L/s | flow rate at the engine feed |

### Actuators (your outputs)

| Name | Type | Description |
|------|------|-------------|
| `pump_source` | `enum Tank { Left, Right }` | which tank the pump draws from |
| `fill_target` | `enum Tank { Left, Right }` | which tank refueling routes into |
| `refueling_active` | `bool` | true while refueling is in progress |

### Indicators (computed for you)

| Name | Type | Unit | Description |
|------|------|------|-------------|
| `total_fuel_liters` | `f32` | L | sum of both tanks |
| `time_until_empty_s` | `f32` | s | `total_fuel_liters / fuel_flow_lps` |

The indicators are already implemented. Focus on the actuators.

### System parameters

| Name | Type | Unit | Default |
|------|------|------|---------|
| `left_capacity_liters` | `f32` | L | 300.0 |
| `right_capacity_liters` | `f32` | L | 300.0 |

---

## Part 1 — Configuration (15 min)

**File:** `assignment/starter/cpm_f.yaml`

The CPM-F hypervisor already runs three partitions: `cpm_f_gateway`, `fuel_quantity`, `transfer_pump`. You will add a fourth: **`fuel_controller`**.

### Step 1.1 — Add the partition entry

Find the `partitions:` block. There is a stub at the bottom:

```yaml
  - id: 3
    name: fuel_controller
    duration: TODO    # how long does your partition need to run each frame?
    offset:   TODO    # when in the frame does it start?
    period:   TODO    # how often does it run?
    image: fuel_controller
```

Fill in the three TODOs. Constraints:

- `period` must equal the major frame (`40ms`).
- `duration` must be ≥ 1 ms and ≤ 5 ms. (1 ms is plenty; 5 ms is the most you'll be granted.)
- `offset + duration` must not overlap with any existing window. Existing windows: 0–3 ms (`cpm_f_gateway`), 4–6 ms (`fuel_quantity`), 8–10 ms (`transfer_pump`).
- `offset + duration ≤ 40ms`.

### Step 1.2 — Wire the sampling ports

The `channel:` block at the bottom of the file lists every sampling-port connection. The gateway already publishes sensor data on `iom_to_fuel` to `fuel_quantity` and `transfer_pump`. You must:

(a) **Add `fuel_controller` as a destination** of the existing `iom_to_fuel` channel (so your partition can read sensors).

(b) **Create a new channel** that carries your actuator commands from `fuel_controller` back to `cpm_f_gateway` on port `controller_to_iom`.

Look at how the other channels are declared — your block follows the same shape.

### Step 1.3 — Validate

```bash
cd assignment/starter
cargo run --bin test_bench -- --validate-config
```

Validator must report **OK** before you proceed to Part 2. It will tell you exactly which constraint failed.

---

## Part 2 — Controller logic (40 min)

**File:** `assignment/starter/src/controller.rs`

The controller skeleton looks like this:

```rust
pub fn select_source_tank(
    left_liters: f32,
    right_liters: f32,
    current_source: Tank,
    seconds_since_switch: f32,
) -> Tank {
    // TODO(student): pick which tank the pump should draw from this tick.
    //
    // Requirements:
    //   - |left_liters - right_liters| must stay ≤ 10.0 L
    //   - You must not switch tanks unless seconds_since_switch >= 0.8
    //   - When fuel is exhausted in one tank, draw from the other regardless
    //
    // This function is invoked every 40 ms (one major frame).
    current_source  // naive placeholder: never switch
}
```

The function is called every frame by the surrounding controller code. The return value is sent verbatim to the pump valve. State you need to keep between calls (like the last switch time) is **maintained for you** in the controller struct — `seconds_since_switch` is passed in, ready to use.

### Verifying your work

```bash
cd assignment/starter
cargo run --bin test_bench
```

The test bench replays a 600-second flight scenario and prints violations as they occur. A passing implementation prints `[OK] no violations in 600 s` at the end.

You can also run the full grader (same code the tutor uses):

```bash
cargo test --release -- --nocapture
```

---

## Bonus tasks (optional)

If you finish early, attempt any of these. Each is graded independently.

### Bonus 1 — Hysteresis (+5 pts)

Add a dead-band so the valve doesn't chatter near the 10 L threshold. Aim for fewer than 60 valve switches over the 600 s default scenario.

### Bonus 2 — Refueling logic (+5 pts)

Implement `select_fill_tank(...)` in `controller.rs`. The same balance constraint applies during refueling: don't overshoot 10 L of imbalance.

### Bonus 3 — Sensor robustness (+5 pts)

Modify your controller to handle a sensor that returns `NaN` (simulated sensor failure). It must keep the remaining good tank from running dry and must not panic.

---

## Grading

| Component | Points |
|-----------|--------|
| Configuration validates | 30 |
| Scenario 1 (symmetric burn) | 25 |
| Scenario 2 (asymmetric start) | 25 |
| Scenario 3 (sensor noise) | 10 |
| Reflection question (below) | 10 |
| Bonus 1 (hysteresis) | +5 |
| Bonus 2 (refueling) | +5 |
| Bonus 3 (sensor robustness) | +5 |

**Pass mark: 60.** Maximum without bonuses: 100. Bonuses are added on top.

### Reflection question (answer in `REFLECTION.md` in the starter folder, ~3 sentences)

> If the major frame were changed from 40 ms to 100 ms (i.e. your controller is invoked 2.5× less often), would your implementation still satisfy the 10 L balance requirement? Why or why not?

---

## Submission

Leave these files in `assignment/starter/`:

1. `cpm_f.yaml`
2. `src/controller.rs`
3. `REFLECTION.md` (1 paragraph)

The tutor will run `cargo test --release` against your folder at the end of the session. Your grade is the test output.

---

## What happens after grading

For every passing solution, the tutor will copy your `controller.rs` into the live Avio23 stack, restart the CPM-F container, and you will watch your code drive the fuel valve across the simulated AFDX network in real time. That is the difference between writing a function and writing avionics.
