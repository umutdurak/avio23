# Avio23 -- Integrated Modular Avionics Teaching Platform

Avio23 is an educational IMA (Integrated Modular Avionics) system that demonstrates how multiple avionics domains are hosted on ARINC 653 computing modules and interconnected over a simulated AFDX network. It is the companion project to [OSAVI](https://github.com/<your-org>/osavi), which teaches how the ARINC 653 kernel itself is built.

## Architecture

```
                    +-----------+
                    |  IOM      |
                    | (gateway) |
                    +-----+-----+
                          |
            +-----+-----+-----+-----+
            |     |           |     |
       +----+  +--+--+  +----+  +--+--+
       | L  |  |  F  |  |  A |  |  E  |
       +----+  +-----+  +----+  +-----+
      Landing   Fuel      ECS   Electrical
       Gear
```

**5 partitions across 5 nodes**, each running an `a653rs-linux` hypervisor instance:

| Node | Domain | DAL | Applications |
|------|--------|-----|-------------|
| sim_gateway | IOM (sensor bridge) | C | Sensor Broadcast |
| CPIOM-L | Landing Gear | B | Extension/Retraction, Braking, Steering |
| CPIOM-F | Fuel | C | Fuel Quantity, Transfer Pump |
| CPIOM-A | Air Conditioning / ECS | D | Bleed Air, Temperature Regulation |
| CPIOM-E | Electrical / Energy | B | Generator Control, Load Shedding |

All inter-partition communication uses ARINC 653 **sampling ports** with `a653rs-postcard` serialization.

## Project Structure

```
Avio23/
  architecture/           SysML v2 architecture model
  configs/                Per-CPIOM hypervisor YAML configs
    gateway.yaml          IOM schedule + outbound channels
    cpm_l.yaml            Landing Gear schedule + channels
    cpm_f.yaml            Fuel schedule + channels
    cpm_a.yaml            ECS schedule + channels
    cpm_e.yaml            Electrical schedule + channels
  implementation/         Rust workspace
    ima_config.yaml       Unified 5-partition config (single-host demo)
    sim_gateway/          IOM partition (generates synthetic flight data)
    cpm_l/                Landing Gear partition
    cpm_f/                Fuel partition
    cpm_a/                ECS partition
    cpm_e/                Electrical partition
  platform/
    a653rs-linux/         ARINC 653 type-2 hypervisor for Linux (DLR)
  Dockerfile              Container build
  docker-compose.yml      Multi-node deployment
```

## Prerequisites

- **Rust** 1.80+ with `x86_64-unknown-linux-musl` target
- **Docker** and **Docker Compose** (for multi-node deployment)
- Linux host (the `a653rs-linux` hypervisor uses cgroups and namespaces)

## Building

### Single-host (all partitions in one hypervisor)

```bash
cd implementation
cargo build --release --target x86_64-unknown-linux-musl

# Add partition binaries to PATH
export PATH="$(pwd)/target/x86_64-unknown-linux-musl/release:$PATH"

# Run with the unified config
cd ../platform/a653rs-linux
RUST_LOG=info cargo run --package a653rs-linux-hypervisor --release -- \
    ../../implementation/ima_config.yaml
```

### Multi-node (Docker Compose)

```bash
docker compose build
docker compose up
```

Each service runs its own hypervisor with a per-CPIOM config from `configs/`.

## Schedule

40 ms major frame:

```
|  0ms        10ms  15ms  20ms  25ms  30ms       40ms  |
|  sim_gateway | L  |  F  |  A  |  E  |   idle   |
```

## Adding a New Domain

1. Create a new crate under `implementation/` (copy `cpm_f/` as a template).
2. Define your data structures and ARINC 653 ports.
3. Add the partition to `implementation/Cargo.toml` workspace members.
4. Create a config in `configs/` with the schedule slot and channels.
5. Add the partition to `ima_config.yaml` and `docker-compose.yml`.

## Relationship to OSAVI

| | OSAVI | Avio23 |
|---|---|---|
| **What it teaches** | How to build the ARINC 653 kernel | How to design and deploy IMA applications |
| **Platform** | Bare-metal Pi Zero 2 W | Linux containers (Docker) |
| **APEX** | 14 Part 4 services in Rust | a653rs library |
| **Communication** | Shared-memory ports (single board) | Memory-mapped ports (multi-node) |
| **Standards** | DO-178C, DO-297 (core software) | DO-297 (platform integration), ARP4754A |

## References

- ARINC 653 Parts 1--4: Application/Executive interface
- DO-297: Integrated Modular Avionics Development
- ARP4754A: Development of Civil Aircraft and Systems
- ARINC 664: Aircraft Data Network (AFDX)
- [a653rs](https://github.com/DLR-FT/a653rs): Rust ARINC 653 library (DLR)
- [a653rs-linux](https://github.com/DLR-FT/a653rs-linux): Linux ARINC 653 hypervisor (DLR)

## License

MIT
