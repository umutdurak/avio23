# Avio23 -- Integrated Modular Avionics Teaching Platform

Avio23 is an educational IMA (Integrated Modular Avionics) system that demonstrates how multiple avionics domains are hosted on ARINC 653 computing modules and interconnected over a simulated AFDX network. It is the companion project to [OSAVI](https://github.com/umutdurak/osavi), which teaches how the ARINC 653 kernel itself is built.

## Architecture (Option C: Multi-Partition CPM with Network I/O Gateways)

```
                    +--------------------+
                    |  sim_gateway (IOM) |
                    |     172.20.0.2     |
                    +----------+---------+
                               |
             +-----------------+-----------------+
             |                 |                 |
      +------+------+   +------+------+   +------+------+
      |  CPM-L      |   |  CPM-F      |   |  CPM-A      |
      |  172.20.0.3 |   |  172.20.0.4 |   |  172.20.0.5 |
      +------+------+   +------+------+   +------+------+
             |
      +------+------+
      |  CPM-E      |
      |  172.20.0.6 |
      +-------------+
```

Under Option C, the system is deployed as **14 partitions across 5 physical nodes (Docker containers)**. Each container runs a local instance of the `a653rs-linux` hypervisor to schedule local partitions and wire them together via memory-mapped sampling ports. Inter-node communication is bridged using UDP sockets over a virtual AFDX network:

| Node | Domain | DAL | Partitions (Local Hypervisor) | Description |
|------|--------|-----|----------------------------|-------------|
| **sim_gateway** | IOM (sensor bridge) | C | `sim_gateway` | Reads external simulation UDP packets, maps telemetry signals to CPM ports |
| **cpm_l_node** | Landing Gear | B | `cpm_l_gateway`, `extension_retraction`, `braking`, `steering` | Processes gear lever state, wheel speed/pedal braking, and nose-wheel steering |
| **cpm_f_node** | Fuel | C | `cpm_f_gateway`, `fuel_quantity`, `transfer_pump` | Computes fuel level totalizer & manages tank balancing |
| **cpm_a_node** | ECS / Air Cond | D | `cpm_a_gateway`, `bleed_air`, `temperature_reg` | Regulates bleed air supply and cabin temperatures |
| **cpm_e_node** | Electrical / Energy | B | `cpm_e_gateway`, `generator_control`, `load_shedding` | Monitors generators & manages battery status / load shedding |

All inter-partition communication within the same CPM uses ARINC 653 **sampling ports** with `a653rs-postcard` serialization. The Network I/O Gateways (`cpm_*_gateway`) on each CPM handle UDP networking with `sim_gateway` and distribute the data to/from the local application partitions.

---

## Project Structure

```
Avio23/
  architecture/               SysML v2 architecture document
  configs/                    Per-CPM hypervisor YAML configs
    gateway.yaml              IOM configuration & scheduled ports
    cpm_l.yaml                Landing Gear configuration & schedule
    cpm_f.yaml                Fuel configuration & schedule
    cpm_a.yaml                ECS configuration & schedule
    cpm_e.yaml                Electrical configuration & schedule
  implementation/             Rust workspace
    sim_gateway/              IOM gateway crate (generates flight telemetry data)
    cpm_l/                    Landing Gear CPM library and binaries
      src/lib.rs              Data structure definitions
      src/bin/                Partition targets (cpm_l_gateway, braking, etc.)
    cpm_f/                    Fuel CPM library and binaries
    cpm_a/                    ECS CPM library and binaries
    cpm_e/                    Electrical CPM library and binaries
  platform/
    a653rs-linux/             ARINC 653 type-2 hypervisor for Linux (DLR)
  Dockerfile                  Multi-stage static musl container build
  docker-compose.yml          Orchestrates virtual AFDX network & static IPAM
```

---

## Prerequisites

- **Docker** and **Docker Compose**
- **Rust** 1.80+ (if working on local crate development)

Because `a653rs-linux` hypervisor depends on Linux-specific cgroups, PID/mount namespaces, and procfs, the workspace must be run inside a Linux virtual environment. Using the provided Docker container orchestration is the recommended layout for both macOS and Windows.

---

## Building and Running

To clean, build, and deploy the entire multi-node architecture:

```bash
# Stop any running containers
docker compose down

# Compile partitions and build the base image
docker compose build builder

# Launch all nodes in the background
docker compose up -d
```

### Inspecting Telemetry
To monitor the flight telemetry and verify communication flow, view the logs of the simulation gateway:

```bash
docker compose logs -f sim_gateway
```

This output displays the flight variables (airspeed ramp, gear switch) along with status reports returned from the CPM-L, CPM-F, CPM-A, and CPM-E partitions.

---

## Schedule

The local partition execution is scheduled around a periodic **40 ms major frame**:

```
|  0ms            10ms       20ms       30ms       40ms  |
|  cpm_x_gateway  |  app_1   |  app_2   |  idle    |
```

*   **cpm_x_gateway** (3 ms): Synchronizes incoming UDP data with local sampling ports.
*   **app_1 / app_2** (2 ms each): Process avionics application algorithms.
*   **idle** (Remaining ms): Margin for growth and worst-case execution time variations.

---

## Relationship to OSAVI

| Concern | OSAVI | Avio23 (Option C) |
|---------|-------|-------------------|
| **What it teaches** | How to build the ARINC 653 kernel | How to design and deploy IMA applications |
| **Platform** | Bare-metal Pi Zero 2 W | Linux containers (Docker) |
| **APEX** | ARINC 653 Part 1 Services in Rust | a653rs binding library |
| **Communication** | Shared-memory ports (single board) | Virtual AFDX network bridged via UDP over Ethernet |
| **Standards** | DO-178C, DO-297 (core software) | DO-297 (platform integration), ARP4754A |

---

## References

- ARINC 653 Parts 1--4: Application/Executive interface
- DO-297: Integrated Modular Avionics Development
- ARP4754A: Development of Civil Aircraft and Systems
- ARINC 664: Aircraft Data Network (AFDX)
- [a653rs](https://github.com/DLR-FT/a653rs): Rust ARINC 653 library (DLR)
- [a653rs-linux](https://github.com/DLR-FT/a653rs-linux): Linux ARINC 653 hypervisor (DLR)

## License

MIT
