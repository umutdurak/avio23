# Avio23 Developer Guide

Everything you need to write applications for, and configure, the Avio23 IMA platform.

This guide assumes you're comfortable with Rust and `cargo`, and have used Docker before. It does *not* assume any prior ARINC 653 or DO-297 knowledge.

---

## Table of contents

1. [What Avio23 is](#1-what-avio23-is)
2. [Mental model — five concepts](#2-mental-model--five-concepts)
3. [Repository layout](#3-repository-layout)
4. [Toolchain & prerequisites](#4-toolchain--prerequisites)
5. [Anatomy of a partition](#5-anatomy-of-a-partition)
6. [Configuration reference (`*.yaml`)](#6-configuration-reference-yaml)
7. [Inter-partition messaging](#7-inter-partition-messaging)
8. [Cross-CPM messaging — the gateway pattern](#8-cross-cpm-messaging--the-gateway-pattern)
9. [Building and running](#9-building-and-running)
10. [Observability & debugging](#10-observability--debugging)
11. [Tutorial — add a new partition to an existing CPM](#11-tutorial--add-a-new-partition-to-an-existing-cpm)
12. [Tutorial — add a new CPM (domain)](#12-tutorial--add-a-new-cpm-domain)
13. [API cheatsheet](#13-api-cheatsheet)

---

## 1. What Avio23 is

Avio23 is a **software-only IMA (Integrated Modular Avionics) teaching platform**. It runs on Linux containers and reproduces the architectural pattern of a real airliner cockpit:

- Multiple **Core Processing Modules (CPMs)**, each a Docker container, host a small number of **partitions** (isolated applications) scheduled by a local hypervisor.
- A **virtual AFDX network** (UDP on a Docker bridge) connects the CPMs to one another and to an I/O Module (`sim_gateway`) that injects simulated sensor data.
- The hypervisor implements **ARINC 653 Part 1** services through the [`a653rs`](https://github.com/DLR-FT/a653rs) Rust library; the Linux-specific kernel layer is [`a653rs-linux`](https://github.com/DLR-FT/a653rs-linux) from DLR-FT.

The platform is built around real CS-23 / Part 23 commuter-category aircraft domains:

| CPM | Domain | Partitions |
|-----|--------|-----------|
| `sim_gateway` | I/O Module — synthetic sensor source | `sim_gateway` |
| `cpm_l_node` | Landing Gear | `cpm_l_gateway`, `extension_retraction`, `braking`, `steering` |
| `cpm_f_node` | Fuel | `cpm_f_gateway`, `fuel_quantity`, `transfer_pump` |
| `cpm_a_node` | ECS / Air Conditioning | `cpm_a_gateway`, `bleed_air`, `temperature_reg` |
| `cpm_e_node` | Electrical / Energy | `cpm_e_gateway`, `generator_control`, `load_shedding` |

You develop on Avio23 by writing **Rust partition binaries** and **YAML hypervisor configurations**. Both are first-class artifacts: a partition without a config slot doesn't run, and a config slot without a binary fails at startup.

---

## 2. Mental model — five concepts

If you internalize these five, every other Avio23 concept is a refinement.

### 2.1 Partition

An isolated application. Same idea as a Linux process, with two extra guarantees:

1. **Space isolation** — memory is segregated; one partition cannot read or corrupt another.
2. **Time isolation** — CPU time is allotted in fixed slots; one partition cannot steal another's slice.

In Avio23, a partition is a single Rust binary, scheduled by `a653rs-linux` and identified by an `id` (0..N) and a `name`.

### 2.2 Major frame

The hypervisor scheduler is **static and periodic**. Every CPM has a **major frame** (40 ms in Avio23) that contains exactly one **time window** per partition. The schedule repeats forever.

```
Major frame = 40 ms
| gateway 3ms | fuel_qty 2ms | transfer_pump 2ms | …idle… | gateway … (next frame)
```

A partition runs only inside its window. If it overruns, the hypervisor preempts it and the next partition starts on time. Timing is deterministic — you can prove the system meets its real-time constraints by inspection.

### 2.3 Time window

A `(offset, duration)` pair inside the major frame. Two windows cannot overlap. Their `offset + duration` must not exceed the major frame.

A partition's `period` always equals the major frame in Avio23 (one window per partition per frame).

### 2.4 Sampling port

A one-way, **latest-value** mailbox between partitions.

- The **writer** (`sampling_out`) overwrites the slot every time it sends.
- The **reader** (`sampling_in`) sees whatever was most recently written. No queue. No back-pressure.
- A `refresh_period` on the reader tells the hypervisor how long a value is considered "fresh"; older data is flagged `Invalid`.

Sampling ports are the right primitive for **periodic sensor data**, which is most of an aircraft's traffic. If you need ordered, never-dropped messages (commands, events) you'd use a queueing port instead — Avio23 currently only uses sampling.

### 2.5 Gateway

A special partition that bridges its CPM to the AFDX network. There are two patterns:

- **IOM gateway** (`sim_gateway`): generates synthetic sensor data and pushes it to peer CPMs over UDP.
- **CPM gateway** (`cpm_x_gateway` per CPM): receives UDP from peers, deserializes, and writes to local sampling ports; reads local ports and serializes back to UDP.

Gateways are how you talk **across** CPMs. Application partitions don't see UDP — they only see sampling ports.

---

## 3. Repository layout

```
Avio23/
├── README.md                       project overview
├── architecture/                   SysML v2 architecture document
├── configs/                        Per-CPM hypervisor YAML
│   ├── gateway.yaml                sim_gateway
│   ├── cpm_l.yaml                  Landing Gear CPM
│   ├── cpm_f.yaml                  Fuel CPM
│   ├── cpm_a.yaml                  ECS CPM
│   └── cpm_e.yaml                  Electrical CPM
├── implementation/                 Rust workspace
│   ├── sim_gateway/                IOM crate
│   ├── cpm_l/                      Landing Gear crate
│   │   ├── src/lib.rs              shared types (sensor structs)
│   │   └── src/bin/                one .rs per partition target
│   ├── cpm_f/                      Fuel crate
│   ├── cpm_a/                      ECS crate
│   └── cpm_e/                      Electrical crate
├── platform/
│   └── a653rs-linux/               vendored hypervisor (DLR-FT)
├── Dockerfile                      multi-stage musl static build
├── docker-compose.yml              5-container orchestration + AFDX bridge
└── assignment/                     teaching assignment kit (see assignment/README.md)
```

Each CPM is **one Rust crate** containing N partition binaries (under `src/bin/`) plus a shared `lib.rs` that defines the message types they exchange.

---

## 4. Toolchain & prerequisites

| Tool | Used for | Why |
|------|----------|-----|
| **Rust 1.80+** (stable) | Compiling partitions and the hypervisor | The `a653rs` macros require a recent stable. |
| **`musl` target** (`<arch>-unknown-linux-musl`) | Static-linked partition binaries | Hypervisor execs partitions as bare ELF; static linking avoids glibc surprises. The Dockerfile adds this target automatically. |
| **Docker + Docker Compose** | Multi-CPM orchestration | Each CPM is a privileged container running its own hypervisor. `a653rs-linux` needs cgroups + PID/mount namespaces, so a Linux VM is mandatory; on macOS/Windows that means Docker. |
| **`docker compose`** v2 | Service orchestration | The v1 `docker-compose` CLI also works but v2 is what `docker-compose.yml` is written against. |

For pure-Rust development you can iterate on a partition's logic on your laptop without Docker — but to actually exercise the hypervisor and the AFDX bridging, you must use the container stack.

---

## 5. Anatomy of a partition

Every partition is a single Rust binary with the same skeleton. Below is a fully annotated walk-through of [`implementation/cpm_f/src/bin/cpm_f_gateway.rs`](../implementation/cpm_f/src/bin/cpm_f_gateway.rs), which is the most feature-rich because it also handles UDP — application partitions are simpler.

### 5.1 The `main()` entry

```rust
use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting CPM-F Network I/O Gateway");
    cpm_f_gateway::Partition.run()
}
```

Three things happen here:

1. The **ApexLogger** is installed. This routes Rust `log` macros (`info!`, `warn!`, `error!`) through the ARINC 653 health-monitoring channel into the hypervisor's stdout — which is what you see in `docker compose logs`.
2. A panic hook is installed so panics surface as health-monitor events instead of silently killing the partition.
3. `Partition::run()` hands control to the hypervisor, which calls back into our `cold_start` / `warm_start` / `periodic` handlers at the right times.

### 5.2 The `#[partition]` module

```rust
#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpm_f_gateway {
    use super::*;
    use a653rs_postcard::prelude::*;
```

The `#[partition(...)]` macro turns the inner module into a partition definition for the `a653rs-linux` backend. The macro generates:

- A `Partition` struct with `.run()`.
- Context types (`start::Context`, `run_app::Context`) that expose the ports you declare below.
- Wiring between your sampling-port symbols and the hypervisor.

`a653rs_postcard` provides `send_type`/`recv_type` methods that serialize/deserialize port payloads with [`postcard`](https://github.com/jamesmunns/postcard) — the same library can run on bare-metal, on Linux, anywhere.

### 5.3 Declaring sampling ports

```rust
    #[sampling_out(name = "iom_to_fuel", msg_size = "1KB")]
    struct IomToFuelPort;

    #[sampling_in(name = "fuel_qty_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct FuelQtyToIomPort;

    #[sampling_in(name = "pump_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct PumpToIomPort;
```

Each `#[sampling_in]` / `#[sampling_out]` attribute declares one port. The `name` must match a port name in the YAML (otherwise the hypervisor refuses to start). `msg_size` is the maximum serialized payload, and `refresh_period` (on inputs) is how long a value stays "fresh".

The macro generates Zero-Sized-Type marker structs (`IomToFuelPort`, etc.) whose only purpose is to anchor the port in the partition context.

### 5.4 Cold and warm start

```rust
    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_fuel_qty_to_iom_port().unwrap();
        ctx.create_pump_to_iom_port().unwrap();
        ctx.create_run_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }
```

`cold_start` is called once on first boot. `warm_start` is called after a restart that preserved state. Both must:

1. **Create every port** declared above. The hypervisor refuses to operate a port you forgot to create — there is no implicit construction.
2. **Create and start every process** (here, only `run_app`).

For simple partitions, `warm_start` just delegates to `cold_start`. Real systems use the difference for state recovery; in Avio23 it does not currently matter.

### 5.5 The periodic process

```rust
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn run_app(ctx: run_app::Context) {
        info!("Started CPM-F Gateway Loop");
        loop {
            // ... read ports, do work, write ports ...
            ctx.periodic_wait().unwrap();
        }
    }
```

`period` must equal the major frame. `ctx.periodic_wait()` blocks until the partition's next window begins — this is what makes the loop **rate-limited** instead of busy-spinning. Skipping `periodic_wait` is the most common bug: your CPU will pin at 100% inside the partition's time slot and accomplish nothing.

`time_capacity` allows finer-than-window WCET control; Avio23 leaves it `Infinite`. `stack_size` is the partition's stack budget — bump it if you blow it. `deadline = Soft` means a missed deadline produces a health-monitor warning but is not fatal.

### 5.6 Sending and receiving typed data

Inside `run_app`:

```rust
ctx.iom_to_fuel_port.as_ref().unwrap()
   .send_type(my_struct).unwrap();           // serialize MyStruct → port

if let Ok((valid, data)) = ctx.fuel_qty_to_iom_port.as_ref().unwrap()
    .recv_type::<MyStruct>()                 // deserialize port → MyStruct
{
    if valid == Validity::Valid {
        // data is fresh; use it
    }
}
```

`send_type` is a one-liner — write whatever your type is, it gets postcard-encoded. `recv_type::<T>` returns a `(Validity, T)` tuple: `Validity::Valid` means the value was written more recently than the reader's `refresh_period`; `Invalid` means the slot is stale.

**Every port read should branch on `Validity`.** A partition that trusts stale data silently is a partition that will pump fuel out of an empty tank.

### 5.7 Application partition vs. gateway partition

Application partitions look exactly like the gateway, minus the UDP socket. See [`transfer_pump.rs`](../implementation/cpm_f/src/bin/transfer_pump.rs) for the simplest example:

```rust
#[periodic(period = "40ms", …)]
fn run_app(ctx: run_app::Context) {
    loop {
        if let Ok((valid, data)) = ctx.iom_to_fuel_port.as_ref().unwrap()
            .recv_type::<IomToCpiomF>()
        {
            if valid == Validity::Valid {
                let cmd = compute_command(&data);
                ctx.pump_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
            }
        }
        ctx.periodic_wait().unwrap();
    }
}
```

The shape is always: **read → compute → write → wait**.

---

## 6. Configuration reference (`*.yaml`)

Each CPM is described by one YAML file in `configs/`. The file is read by the `a653rs-linux` hypervisor at startup; it defines the schedule, the partitions, and the sampling channels.

### 6.1 Top-level shape

```yaml
major_frame: 40ms     # the repeating period

partitions:           # list of partitions on this CPM
  - id: 0
    name: ...
    ...

channel:              # list of sampling channels (inter-partition messaging)
  - !Sampling
    ...
```

### 6.2 Partition entry

```yaml
- id: 0                        # unique on this CPM, 0..N
  name: cpm_f_gateway          # must match the `#[partition]` module name
  duration: 3ms                # length of this partition's window
  offset:   0ms                # start within the major frame
  period:   40ms               # always equal to major_frame
  image: cpm_f_gateway         # binary name (under target/.../release/)
  sockets:                     # optional: only for gateway partitions
    - type: udp
      address: 0.0.0.0:49002
```

Constraints the hypervisor enforces at startup:

- `period` must equal `major_frame`.
- `offset + duration` must not exceed `major_frame`.
- No two windows on the same CPM may overlap.
- `name` must match the partition module name in the binary's `#[partition]` macro.
- `image` must be a binary findable on the hypervisor's `PATH`.

If `sockets:` is present, the hypervisor grants the partition access to that socket via `ApexLinuxPartition::get_udp_socket(addr)`. This is how gateway partitions bypass the normal "no syscalls" rule for AFDX bridging.

### 6.3 Sampling channel entry

```yaml
- !Sampling
  msg_size: 1KB            # max serialized payload
  source:                  # exactly one writer
    partition: cpm_f_gateway
    port: iom_to_fuel
  destination:             # one or more readers
    - partition: fuel_quantity
      port: iom_to_fuel
    - partition: transfer_pump
      port: iom_to_fuel
```

The `!Sampling` tag is YAML 1.2 syntax — it tells the hypervisor this is a sampling channel (as opposed to a queueing channel). One source, N destinations.

Port names in `source.port` and each `destination.port` must match the `name = "..."` argument of the corresponding `#[sampling_in]` / `#[sampling_out]` attribute in Rust.

### 6.4 Major frame budget

```
Σ (partition.duration)  ≤  major_frame
```

Avio23 keeps significant idle margin — the existing CPM-F frame uses 7 ms out of 40 ms (~83% idle). This is intentional: real avionics programs accumulate timing demands over years of integration and you want headroom.

---

## 7. Inter-partition messaging

Within a single CPM, partitions communicate **only** through sampling ports. There is no shared memory, no global state, no IPC. This is the same as ARINC 653 on real hardware — Avio23 does not loosen the model.

### 7.1 The contract

Two artifacts make a port work:

1. **Both partitions** declare the port with matching `name` and `msg_size`.
2. **The YAML** declares a channel that connects the writer's port to the reader's port (same names).

If any one of these three names disagrees, the hypervisor refuses to create the port and the partition's `cold_start` panics.

### 7.2 Choosing a payload type

Define your message in the CPM's `lib.rs` so both writer and reader can `use` it:

```rust
// implementation/cpm_f/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IomToCpiomF {
    pub left_tank_kg: f32,
    pub right_tank_kg: f32,
    pub center_tank_kg: f32,
    pub fuel_temp_c: f32,
}
```

Constraints:

- The type must implement `Serialize + Deserialize`.
- Its serialized size must fit in `msg_size`. `postcard` is compact, so 1 KB easily holds dozens of fields.
- Both sides must `use` the exact same definition. Schema drift between writer and reader is silent corruption — discipline matters.

### 7.3 One writer, many readers

A sampling channel may have many destinations. All of them see the same overwritten value. If you want **one-to-many fan-out** of sensor data, this is your tool.

If you want **many-to-one fan-in** (e.g. multiple monitor partitions reporting to a single supervisor), you need **multiple channels**, one per writer. The supervisor reads from each in turn.

---

## 8. Cross-CPM messaging — the gateway pattern

Sampling ports are local to one CPM. To reach a different CPM, you need a gateway pair.

### 8.1 The flow

```
[ app_part ] --port-->  [ local_gateway ] --UDP--> [ peer_gateway ] --port--> [ app_part ]
              (in CPM-X)                                                       (in CPM-Y)
```

Each direction is **one local sampling channel + one UDP datagram**. The gateway:

1. Reads the local port (`recv_type`).
2. Postcard-serializes the payload.
3. `socket.send_to(bytes, "172.20.0.X:49000")` to the peer.

The peer's gateway runs the same loop in reverse: `recv_from` → deserialize → `send_type` to its local port.

### 8.2 Why we don't expose UDP to application partitions

You could, in principle, give every partition its own socket. We don't, for three reasons:

1. **Discipline.** Application code is unchanged whether it runs on Avio23 or on real ARINC 664 hardware. Only the gateway differs.
2. **Schedule budget.** UDP work concentrates in one window (`cpm_f_gateway` at 0–3 ms), so the other partitions don't pay the latency cost.
3. **DAL.** Gateways are simpler than apps and can be certified separately.

### 8.3 The Docker AFDX bridge

`docker-compose.yml` defines an `afdx` bridge with subnet `172.20.0.0/16` and pins each CPM to a fixed IP (172.20.0.2 .. 172.20.0.6). This is the only "wiring" the platform layer requires — there is no AFDX simulator, just plain UDP on a bridge network.

UDP ports used today:

| Port | Direction | Used by |
|------|-----------|---------|
| 49000 | inbound to `sim_gateway` | CPMs send telemetry back |
| 49002 | inbound to `cpm_f_gateway` | `sim_gateway` sends fuel telemetry |
| 49003..49006 | inbound to other CPMs | one per domain |

If you add a new CPM, pick a new IP and a new port and update the YAML + Docker compose together.

---

## 9. Building and running

### 9.1 Quick path (Docker)

```bash
# Build the base image (compiles every partition + the hypervisor with musl)
docker compose build builder

# Boot all 5 CPMs
docker compose up -d

# Watch flight telemetry
docker compose logs -f sim_gateway

# Stop
docker compose down
```

The `builder` image is reused by every CPM service — it contains the partition binaries and the hypervisor. Each service runs `a653rs-linux-hypervisor /usr/src/avio23/configs/<that-cpm>.yaml`.

### 9.2 Iterative path (cargo on Linux)

If you're on Linux, you can run the hypervisor without Docker:

```bash
cd implementation
cargo build --release --target $(uname -m)-unknown-linux-musl

cd ../platform/a653rs-linux
cargo build --release --package a653rs-linux-hypervisor

# Run a single CPM directly
sudo ./target/release/a653rs-linux-hypervisor ../../configs/cpm_f.yaml
```

`sudo` is required because the hypervisor uses cgroups and namespaces. macOS/Windows users should use the Docker path instead.

### 9.3 Iterative path (laptop, no hypervisor)

For pure logic development, you can pull your partition's algorithm into a plain library and exercise it with a `cargo run` test harness. See [`assignment/starter/`](../assignment/starter/) for the canonical pattern — a tiny scenario runner that calls the same function shape your partition exposes, with no ARINC 653 in the loop. Fast iteration; you re-deploy into a partition only when the algorithm is ready.

---

## 10. Observability & debugging

### 10.1 Logs

`RUST_LOG=info` is set in `docker-compose.yml`. All `info!` / `warn!` / `error!` calls from a partition surface in:

```bash
docker compose logs -f <service>             # one CPM
docker compose logs -f --tail=200            # all CPMs
```

Bump to `RUST_LOG=trace` in the compose file for full firehose.

### 10.2 The hypervisor's own logs

The hypervisor prints schedule decisions and health events to the same stream. Look for lines starting with `[a653rs-linux]`. A partition that overruns its window will log a `WindowExpired` event here.

### 10.3 What to check when nothing works

| Symptom | Likely cause |
|---------|--------------|
| `Could not get UDP socket from hypervisor config` | Partition's binary expects a socket the YAML didn't grant — add a `sockets:` block to the partition. |
| `port creation failed` at cold start | Port name in Rust doesn't match the YAML, or `msg_size` mismatch. |
| Reader always sees `Validity::Invalid` | Refresh period too short, or writer never sends (check that the writer's loop reaches `send_type`, and that the channel destination matches). |
| Partition silently does nothing | Forgot `ctx.periodic_wait()` in the loop, or the binary panicked at startup — check `docker compose logs <service>` for a Rust panic. |
| Container exits immediately | YAML parse error or binary not found — run `docker compose run <service>` to see startup output. |

---

## 11. Tutorial — add a new partition to an existing CPM

Goal: add a partition `fuel_alarm` to CPM-F that watches for low fuel and raises a warning on the AFDX network.

### Step 1 — Define the message type

Edit `implementation/cpm_f/src/lib.rs`:

```rust
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FuelAlarm {
    pub low_fuel_warning: bool,
    pub remaining_total_kg: f32,
}
```

### Step 2 — Create the partition binary

Create `implementation/cpm_f/src/bin/fuel_alarm.rs`:

```rust
use a653rs::bindings::Validity;
use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use cpm_f::{FuelAlarm, IomToCpiomF};
use log::{info, LevelFilter};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Info).unwrap();
    info!("Starting Fuel Alarm partition");
    fuel_alarm::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod fuel_alarm {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_fuel", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToFuelPort;

    #[sampling_out(name = "alarm_to_iom", msg_size = "1KB")]
    struct AlarmToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_alarm_to_iom_port().unwrap();
        ctx.create_run_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) { cold_start(ctx) }

    #[periodic(period = "40ms", time_capacity = "Infinite",
               stack_size = "100KB", base_priority = 5, deadline = "Soft")]
    fn run_app(ctx: run_app::Context) {
        loop {
            if let Ok((Validity::Valid, data)) =
                ctx.iom_to_fuel_port.as_ref().unwrap().recv_type::<IomToCpiomF>()
            {
                let total = data.left_tank_kg + data.right_tank_kg + data.center_tank_kg;
                let alarm = FuelAlarm {
                    low_fuel_warning: total < 100.0,
                    remaining_total_kg: total,
                };
                ctx.alarm_to_iom_port.as_ref().unwrap().send_type(alarm).unwrap();
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
```

### Step 3 — Register the binary in `Cargo.toml`

`implementation/cpm_f/Cargo.toml` lists each `[[bin]]`:

```toml
[[bin]]
name = "fuel_alarm"
path = "src/bin/fuel_alarm.rs"
```

(Cargo auto-discovers `src/bin/*.rs`, so this is usually optional — but explicit is clearer.)

### Step 4 — Add the partition to `configs/cpm_f.yaml`

```yaml
partitions:
  # ... existing entries ...
  - id: 3
    name: fuel_alarm
    duration: 1ms
    offset: 12ms
    period: 40ms
    image: fuel_alarm
```

Pick an offset that doesn't overlap existing windows. The major-frame budget is 40 ms; current usage tops out at 10 ms, so anywhere from 10 ms onward is free.

### Step 5 — Add the channels

The new partition reads `iom_to_fuel` (already exists — just add a destination) and writes a new `alarm_to_iom` channel:

```yaml
channel:
  - !Sampling
    msg_size: 1KB
    source:
      partition: cpm_f_gateway
      port: iom_to_fuel
    destination:
      - partition: fuel_quantity
        port: iom_to_fuel
      - partition: transfer_pump
        port: iom_to_fuel
      - partition: fuel_alarm        # NEW
        port: iom_to_fuel

  # NEW channel
  - !Sampling
    msg_size: 1KB
    source:
      partition: fuel_alarm
      port: alarm_to_iom
    destination:
      - partition: cpm_f_gateway
        port: alarm_to_iom
```

### Step 6 — Wire the gateway to forward over UDP

Edit `cpm_f_gateway.rs` to add a `sampling_in` for `alarm_to_iom` and serialize+UDP-send it to `sim_gateway`. (Or, decide that `fuel_alarm` data stays local and only the gateway acts on it — your call.)

### Step 7 — Rebuild and run

```bash
docker compose build builder    # picks up the new binary
docker compose up -d
docker compose logs -f cpm_f_node | grep fuel_alarm
```

If the partition starts but you see no logs, you forgot `periodic_wait`. If the partition refuses to start, you have a port-name mismatch or a schedule overlap.

---

## 12. Tutorial — add a new CPM (domain)

Adding a brand-new domain is a superset of adding a partition.

### Step 1 — Create the crate

```bash
cd implementation
cargo new --lib cpm_h          # e.g. Hydraulic
```

Mirror the structure of `cpm_f`: `src/lib.rs` for shared types, `src/bin/cpm_h_gateway.rs`, `src/bin/<app1>.rs`, etc.

### Step 2 — Write the gateway

The gateway must:

- Read incoming UDP from `sim_gateway` (port 49007 if continuing the existing sequence).
- Write the deserialized sensor struct to a local `iom_to_hyd` sampling port.
- Read local app outputs.
- Serialize and UDP-send back to `sim_gateway` at `172.20.0.2:49000`.

Copy `cpm_f_gateway.rs` and adapt the port names, message types, and UDP address.

### Step 3 — Write the application partitions

Standard pattern: `#[sampling_in]` reads sensors, `#[sampling_out]` writes commands, periodic loop in between.

### Step 4 — Create `configs/cpm_h.yaml`

Copy `cpm_f.yaml` and update:
- Each partition's `name` and `image` to match your new binaries.
- The `sockets.address` of the gateway (e.g. `0.0.0.0:49007`).
- The channel `source.partition` / `destination.partition` names.

### Step 5 — Add the container to `docker-compose.yml`

```yaml
  cpm_h_node:
    image: avio23-builder:latest
    depends_on: [builder]
    command: a653rs-linux-hypervisor /usr/src/avio23/configs/cpm_h.yaml
    privileged: true
    environment: { RUST_LOG: info }
    networks:
      afdx:
        ipv4_address: 172.20.0.7
```

### Step 6 — Teach `sim_gateway` about it

Open `implementation/sim_gateway/` and add:
- A new payload type for the hydraulics domain.
- A new UDP send to `172.20.0.7:49007`.
- A new UDP receive from the CPM-H gateway's outbound stream.

The exact place to add this depends on the structure of the sim_gateway; the principle is symmetric to how it talks to CPM-F today.

### Step 7 — Rebuild and run

```bash
docker compose build builder
docker compose up -d
docker compose logs -f cpm_h_node
```

---

## 13. API cheatsheet

### Macros

| Macro | Purpose |
|-------|---------|
| `#[partition(<backend>)]` | Marks an inner module as the partition definition. Backend is `a653rs_linux::partition::ApexLinuxPartition`. |
| `#[sampling_in(name, msg_size, refresh_period)]` | Declares an inbound sampling port. |
| `#[sampling_out(name, msg_size)]` | Declares an outbound sampling port. |
| `#[start(cold)]` / `#[start(warm)]` | Boot handlers. Must create ports + processes. |
| `#[periodic(period, time_capacity, stack_size, base_priority, deadline)]` | Marks a periodic process function. |

### Methods on a context

| Call | Effect |
|------|--------|
| `ctx.create_<port>_port().unwrap()` | Allocate the port (in `cold_start`). |
| `ctx.create_<process>().unwrap().start().unwrap()` | Start a process. |
| `ctx.<port>.as_ref().unwrap().send_type(value)` | Postcard-serialize and write to a sampling-out port. |
| `ctx.<port>.as_ref().unwrap().recv_type::<T>()` | Read + deserialize a sampling-in port; returns `Result<(Validity, T), _>`. |
| `ctx.periodic_wait().unwrap()` | Block until the next time window starts. |

### Methods on `ApexLinuxPartition`

| Call | Effect |
|------|--------|
| `ApexLinuxPartition::get_udp_socket(addr)` | Get a UDP socket the hypervisor pre-allocated (matches `sockets.address` in the YAML). |
| `ApexLogger::install_logger(level)` | Route `log` macros into the health-monitor stream. |
| `ApexLogger::install_panic_hook()` | Convert panics into health-monitor events. |

### YAML schema (mini)

```yaml
major_frame: <duration>
partitions:
  - { id, name, duration, offset, period, image }
  - { ..., sockets: [{type, address}] }    # only gateway partitions
channel:
  - !Sampling
    msg_size: <size>
    source: { partition, port }
    destination: [{ partition, port }, ...]
```

---

## Further reading

- ARINC 653 Part 1: Application/Executive interface (the standard Avio23 follows).
- DO-297: Integrated Modular Avionics Development Guidance and Certification Considerations.
- ARP4754A: Development of Civil Aircraft and Systems.
- ARINC 664: Aircraft Data Network (AFDX — the physical thing Avio23 simulates with UDP).
- [DLR-FT/a653rs](https://github.com/DLR-FT/a653rs) — the Rust ARINC 653 binding.
- [DLR-FT/a653rs-linux](https://github.com/DLR-FT/a653rs-linux) — the Linux hypervisor used by Avio23.
- [`architecture/avio23_sysml_architecture.md`](../architecture/avio23_sysml_architecture.md) — SysML model of the platform.
- [`assignment/`](../assignment/) — a worked teaching assignment that exercises everything in this guide on a small scope.
