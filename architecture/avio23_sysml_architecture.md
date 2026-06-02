# Avio23 IMA Architecture (SysML v2)

This document defines the architecture of the Avio23 Integrated Modular Avionics (IMA) platform using SysML v2 textual notation. It describes the domain decomposition, application allocation, inter-partition communication channels, and design assurance levels under the **Option C (Multi-Partition CPM with Network I/O Gateways)** implementation.

## Design Philosophy

Avio23 models a simplified but representative IMA system for a commuter category aeroplane (under CS-23 / Part 23 certification standards). The goal is to teach:

1. **Domain decomposition** -- how aircraft functions are grouped into domains.
2. **CPM allocation** -- which domains run on which computing modules (Core Processing Modules).
3. **Inter-partition communication (Option C)** -- how local application partitions communicate with their local CPM Network I/O Gateway via ARINC 653 sampling ports, and how the gateways bridge these signals to external UDP sockets over the containerized network (virtual AFDX).
4. **DAL assignment** -- how failure conditions drive assurance levels.
5. **Scheduling** -- how the ARINC 653 major frame is partitioned among local partitions on each CPM.

## Domain Decomposition

| CPM | Domain | DAL | Applications & Gateways | Rationale |
|-----|--------|-----|-------------------------|-----------|
| **CPM-L** | Landing Gear | B | `cpm_l_gateway`, `extension_retraction`, `braking`, `steering` | Loss of gear function is hazardous |
| **CPM-F** | Fuel | C | `cpm_f_gateway`, `fuel_quantity`, `transfer_pump` | Incorrect fuel indication is a major failure |
| **CPM-A** | Air Conditioning / ECS | D | `cpm_a_gateway`, `bleed_air`, `temperature_reg` | Loss of cabin comfort is minor |
| **CPM-E** | Electrical / Energy | B | `cpm_e_gateway`, `generator_control`, `load_shedding` | Loss of electrical power is hazardous |
| **IOM** | Simulation Gateway | C | `sim_gateway` | I/O gateway bridges external flight data |

**Segregation rules applied:**
- DAL B functions (Landing Gear, Electrical) are on separate CPMs to prevent common-mode failure.
- DAL D (ECS) is isolated from DAL B to avoid re-certification coupling.
- The IOM gateway is a separate node: a gateway fault shall not affect domain CPMs.

## Schedule & Major Frame

Each CPM runs its own local hypervisor scheduling partitions periodically over a **40 ms major frame**:

```
|  0ms            10ms       20ms       30ms       40ms  |
|  cpm_x_gateway  |  app_1   |  app_2   |  idle    |
```

For example, on **CPM-L**:
- `cpm_l_gateway` (0 - 3 ms): Receives UDP telemetry packets and writes to local sampling ports.
- `extension_retraction` (4 - 6 ms): Processes gear lever command & airspeed state.
- `braking` (8 - 10 ms): Computes anti-skid brake commands.
- `steering` (12 - 14 ms): Calculates nose wheel steering angles.
- **Idle** (14 - 40 ms): Margin for growth and worst-case execution time variations.

## Network & Channel Matrix (Option C)

All application partitions communicate using pure **ARINC 653 sampling ports** (last-write-wins). The local Network I/O gateway on each CPM translates these sampling ports to/from **UDP sockets** routed over the containerized bridge network.

### IP and UDP Socket Mapping

| Node | Container Service | Static IP | UDP Socket | Local Gateway Partition |
|------|-------------------|-----------|------------|-------------------------|
| **IOM** | `sim_gateway` | `172.20.0.2` | `0.0.0.0:49000` | N/A (direct UDP access) |
| **CPM-L** | `cpm_l_node` | `172.20.0.3` | `0.0.0.0:49001` | `cpm_l_gateway` |
| **CPM-F** | `cpm_f_node` | `172.20.0.4` | `0.0.0.0:49002` | `cpm_f_gateway` |
| **CPM-A** | `cpm_a_node` | `172.20.0.5` | `0.0.0.0:49003` | `cpm_a_gateway` |
| **CPM-E** | `cpm_e_node` | `172.20.0.6` | `0.0.0.0:49004` | `cpm_e_gateway` |

### Internal Sampling Port Connections

On each CPM, the local gateway acts as the data hub, connecting to application partitions:

| CPM | Source Port | Destination Port | Data Structure | Description |
|-----|-------------|------------------|----------------|-------------|
| **CPM-L** | `cpm_l_gateway.iom_to_lg` | `extension_retraction.iom_to_lg`<br>`braking.iom_to_lg`<br>`steering.iom_to_lg` | `IomToCpiomL` | Airspeed, gear lever, pedals |
| | `extension_retraction.ext_to_iom` | `cpm_l_gateway.ext_to_iom` | `CpiomLToIom` | Gear extension state |
| | `braking.brake_to_iom` | `cpm_l_gateway.brake_to_iom` | `CpiomLToIom` | Left/right brake pressures |
| | `steering.steer_to_iom` | `cpm_l_gateway.steer_to_iom` | `CpiomLToIom` | Nose wheel steering angle |
| **CPM-F** | `cpm_f_gateway.iom_to_fuel` | `fuel_quantity.iom_to_fuel`<br>`transfer_pump.iom_to_fuel` | `IomToCpiomF` | Fuel tank levels & temperature |
| | `fuel_quantity.fuel_qty_to_iom` | `cpm_f_gateway.fuel_qty_to_iom` | `CpiomFToIom` | Total fuel weight & alerts |
| | `transfer_pump.pump_to_iom` | `cpm_f_gateway.pump_to_iom` | `CpiomFToIom` | Fuel transfer pump active status |
| **CPM-A** | `cpm_a_gateway.iom_to_ecs` | `bleed_air.iom_to_ecs`<br>`temperature_reg.iom_to_ecs` | `IomToCpiomA` | Cabin/target temps, bleed pressure |
| | `bleed_air.bleed_to_iom` | `cpm_a_gateway.bleed_to_iom` | `CpiomAToIom` | Pack valve and bleed valve levels |
| | `temperature_reg.temp_to_iom` | `cpm_a_gateway.temp_to_iom` | `CpiomAToIom` | Cabin temperature warning & error |
| **CPM-E** | `cpm_e_gateway.iom_to_elec` | `generator_control.iom_to_elec`<br>`load_shedding.iom_to_elec` | `IomToCpiomE` | Generator voltage & bus loads |
| | `generator_control.gen_to_iom` | `cpm_e_gateway.gen_to_iom` | `CpiomEToIom` | Generator state, battery charging |
| | `load_shedding.shed_to_iom` | `cpm_e_gateway.shed_to_iom` | `CpiomEToIom` | Load shedding active warning |

## SysML v2 Model

```sysml
package Avio23_IMA_Architecture {

    import ScalarValues::*;

    // ==========================================
    // Platform Defs & Network Interfaces
    // ==========================================

    part def Virtual_AFDX_Network {
        doc /* Container-bridged Ethernet simulating AFDX */
    }

    part def CPM {
        doc /* Core Processing Module running local a653rs hypervisor */
        port networkPort;
    }

    part def IOM {
        doc /* Input/Output Module (Simulation Gateway) */
        port networkPort;
    }

    // ==========================================
    // System Context
    // ==========================================

    part Avio23_System {

        // --- Hardware Node Allocation ---
        part gatewayNode : IOM;
        part landingGearNode : CPM;     // DAL B
        part fuelNode : CPM;            // DAL C
        part ecsNode : CPM;             // DAL D
        part electricalNode : CPM;      // DAL B
        part virtualNetwork : Virtual_AFDX_Network;

        // --- Network Topology (UDP Socket routing) ---
        connect gatewayNode.networkPort to virtualNetwork;
        connect landingGearNode.networkPort to virtualNetwork;
        connect fuelNode.networkPort to virtualNetwork;
        connect ecsNode.networkPort to virtualNetwork;
        connect electricalNode.networkPort to virtualNetwork;

        // ==========================================
        // Landing Gear CPM Node -- DAL B
        // ==========================================
        part landingGearNodeDetails {
            
            // Gateway Partition (Network Interface)
            part gateway {
                doc /* Bridges UDP socket 49001 to local sampling ports */
                port udpSocket;
                port iom_to_lg;
                port ext_to_iom;
                port brake_to_iom;
                port steer_to_iom;
            }

            // Application Partitions
            part extensionRetractionApp {
                doc /* Gear sequencing: evaluates gear lever and airspeed */
                port inData;
                port outCmd;
            }

            part brakingApp {
                doc /* Auto-braking and skid protection */
                port inData;
                port outCmd;
            }

            part steeringApp {
                doc /* Nose-wheel steering control */
                port inData;
                port outCmd;
            }

            // Local ARINC 653 Sampling Port Connections
            connect gateway.iom_to_lg to extensionRetractionApp.inData;
            connect gateway.iom_to_lg to brakingApp.inData;
            connect gateway.iom_to_lg to steeringApp.inData;
            connect extensionRetractionApp.outCmd to gateway.ext_to_iom;
            connect brakingApp.outCmd to gateway.brake_to_iom;
            connect steeringApp.outCmd to gateway.steer_to_iom;
        }

        // ==========================================
        // Fuel CPM Node -- DAL C
        // ==========================================
        part fuelNodeDetails {

            part gateway {
                doc /* Bridges UDP socket 49002 to local ports */
                port udpSocket;
                port iom_to_fuel;
                port fuel_qty_to_iom;
                port pump_to_iom;
            }

            part fuelQuantityApp {
                doc /* Totalizes fuel levels & alarms */
                port inData;
                port outCmd;
            }

            part transferPumpApp {
                doc /* Manages tank crossfeed/balancing */
                port inData;
                port outCmd;
            }

            connect gateway.iom_to_fuel to fuelQuantityApp.inData;
            connect gateway.iom_to_fuel to transferPumpApp.inData;
            connect fuelQuantityApp.outCmd to gateway.fuel_qty_to_iom;
            connect transferPumpApp.outCmd to gateway.pump_to_iom;
        }

        // ==========================================
        // Air Conditioning/ECS CPM Node -- DAL D
        // ==========================================
        part ecsNodeDetails {

            part gateway {
                doc /* Bridges UDP socket 49003 to local ports */
                port udpSocket;
                port iom_to_ecs;
                port bleed_to_iom;
                port temp_to_iom;
            }

            part bleedAirApp {
                doc /* Bleed air extraction control */
                port inData;
                port outCmd;
            }

            part temperatureRegApp {
                doc /* Cabin temp pack control */
                port inData;
                port outCmd;
            }

            connect gateway.iom_to_ecs to bleedAirApp.inData;
            connect gateway.iom_to_ecs to temperatureRegApp.inData;
            connect bleedAirApp.outCmd to gateway.bleed_to_iom;
            connect temperatureRegApp.outCmd to gateway.temp_to_iom;
        }

        // ==========================================
        // Electrical CPM Node -- DAL B
        // ==========================================
        part electricalNodeDetails {

            part gateway {
                doc /* Bridges UDP socket 49004 to local ports */
                port udpSocket;
                port iom_to_elec;
                port gen_to_iom;
                port shed_to_iom;
            }

            part generatorControlApp {
                doc /* Generator health monitor and regulator */
                port inData;
                port outCmd;
            }

            part loadSheddingApp {
                doc /* Electrical load shedding manager */
                port inData;
                port outCmd;
            }

            connect gateway.iom_to_elec to generatorControlApp.inData;
            connect gateway.iom_to_elec to loadSheddingApp.inData;
            connect generatorControlApp.outCmd to gateway.gen_to_iom;
            connect loadSheddingApp.outCmd to gateway.shed_to_iom;
        }
    }
}
```

## Relationship to OSAVI

OSAVI is the **core software** (the ARINC 653 kernel itself), running bare-metal on a single board (e.g. Raspberry Pi). Avio23 is the **IMA system** that uses such a kernel (hosted via the `a653rs-linux` Linux-hosted type-2 hypervisor) to run multiple avionics domains across isolated CPMs mapped to distinct Docker containers.

| Concern | OSAVI | Avio23 (Option C) |
|---------|-------|-------------------|
| **Scope** | Single-board ARINC 653 kernel | Multi-CPM IMA platform |
| **Partitioning** | MMU + HW timer (bare metal) | CGroups + Linux PID/mount namespaces |
| **APEX API** | ARINC 653 Part 1 Services | Rust `a653rs` binding library |
| **CPM Gateways** | N/A (single CPU focus) | Local Gateway partitions forwarding UDP to sampling ports |
| **Communication** | Core sampling/queuing | Virtual AFDX network routed via UDP over Ethernet |
| **Configuration** | system_config.xml | Individual CPM YAML hypervisor configs |
| **Target** | Pi Zero 2 W | Multi-container Docker on x86_64/AArch64 |

Together they cover the full DO-297 stack: OSAVI teaches how to **build** the platform; Avio23 teaches how to **design, distribute, and deploy** complex applications on it.
