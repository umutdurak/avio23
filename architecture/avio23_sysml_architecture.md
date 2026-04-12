# Avio23 IMA Architecture (SysML v2)

This document defines the architecture of the Avio23 Integrated Modular Avionics (IMA) platform using SysML v2 textual notation. It describes the domain decomposition, application allocation, inter-partition communication channels, and design assurance levels.

## Design Philosophy

Avio23 models a simplified but representative IMA system inspired by modern wide-body aircraft (A350, 787). The goal is to teach:

1. **Domain decomposition** -- how aircraft functions are grouped into domains.
2. **CPIOM allocation** -- which domains run on which computing modules.
3. **Inter-partition communication** -- how ARINC 653 sampling ports carry data between CPIOMs over the AFDX network.
4. **DAL assignment** -- how failure conditions drive assurance levels.
5. **Scheduling** -- how the ARINC 653 major frame is partitioned among domains.

## Domain Decomposition

| CPIOM | Domain | DAL | Applications | Rationale |
|-------|--------|-----|-------------|-----------|
| CPIOM-L | Landing Gear | B | Extension/Retraction, Braking, Steering | Loss of gear function is hazardous |
| CPIOM-F | Fuel | C | Fuel Quantity, Transfer Pump | Incorrect fuel indication is a major failure |
| CPIOM-A | Air Conditioning / ECS | D | Bleed Air Control, Temperature Regulation | Loss of cabin comfort is minor |
| CPIOM-E | Electrical / Energy | B | Generator Control, Load Shedding | Loss of electrical power is hazardous |
| IOM | Simulation Gateway | C | Sensor Broadcast | I/O gateway bridges external data |

**Segregation rules applied:**
- DAL B functions (Landing Gear, Electrical) are on separate CPIOMs to prevent common-mode failure.
- DAL D (ECS) is isolated from DAL B to avoid re-certification coupling.
- The IOM gateway is a separate node: a gateway fault shall not affect domain CPIOMs.

## Schedule

The system operates on a 40 ms major frame:

```
|  0ms        10ms  15ms  20ms  25ms  30ms       40ms  |
|  sim_gateway | L  |  F  |  A  |  E  |   idle   |
```

- **sim_gateway** (10 ms): generates and broadcasts sensor data to all four domains.
- **CPIOM-L** (5 ms): processes gear, brake, and steering logic.
- **CPIOM-F** (5 ms): computes fuel quantity, controls transfer pumps.
- **CPIOM-A** (5 ms): regulates bleed air and cabin temperature.
- **CPIOM-E** (5 ms): monitors generators, manages load shedding.
- **Idle** (10 ms): margin for growth and worst-case jitter.

## Channel Matrix

All inter-partition communication uses ARINC 653 **sampling ports** (last-write-wins). Each channel carries a serialized struct via `a653rs-postcard`.

| Channel | Source | Destination | Data | Size |
|---------|--------|-------------|------|------|
| iom_to_lg | sim_gateway | cpiom_l | Airspeed, lever, pedals | 1 KB |
| lg_to_iom | cpiom_l | sim_gateway | Gear state, brake pressure, steering | 1 KB |
| iom_to_fuel | sim_gateway | cpiom_f | Tank levels, fuel temp | 1 KB |
| fuel_to_iom | cpiom_f | sim_gateway | Total fuel, imbalance, warnings | 1 KB |
| iom_to_ecs | sim_gateway | cpiom_a | Cabin temp, bleed pressure | 1 KB |
| ecs_to_iom | cpiom_a | sim_gateway | Pack valve, bleed valve, faults | 1 KB |
| iom_to_elec | sim_gateway | cpiom_e | Generator voltages, bus load | 1 KB |
| elec_to_iom | cpiom_e | sim_gateway | Generator status, load shedding | 1 KB |

## SysML v2 Model

```sysml
package Avio23_IMA_Architecture {

    import ScalarValues::*;

    // ==========================================
    // Platform Definition
    // ==========================================

    part def IMA_Network {
        doc /* AFDX / ARINC 664 network backbone */
    }

    part def CPIOM {
        doc /* Core Processing Input/Output Module */
        port networkPort;
    }

    part def IOM {
        doc /* Input/Output Module (gateway to external systems) */
        port networkPort;
        port externalSimulatorPort;
    }

    // ==========================================
    // System Context
    // ==========================================

    part Avio23_System {

        // --- Hardware Nodes ---
        part gatewayNode : IOM;
        part landingGearNode : CPIOM;    // DAL B
        part fuelNode : CPIOM;           // DAL C
        part ecsNode : CPIOM;            // DAL D
        part electricalNode : CPIOM;     // DAL B
        part afdxBus : IMA_Network;

        // --- Network Topology ---
        connect gatewayNode.networkPort to afdxBus;
        connect landingGearNode.networkPort to afdxBus;
        connect fuelNode.networkPort to afdxBus;
        connect ecsNode.networkPort to afdxBus;
        connect electricalNode.networkPort to afdxBus;

        // ==========================================
        // Landing Gear Domain (CPIOM-L) -- DAL B
        // ==========================================

        part landingGearDomain {
            port inPort : IOM_to_LG_Data;
            port outPort : LG_to_IOM_Cmd;

            part extensionRetractionApp {
                doc /* Gear sequencing: evaluates lever + airspeed */
                attribute period = 40ms;
                attribute priority = 10;
                port inData;
                port outCmd;
            }

            part brakingApp {
                doc /* Anti-skid and auto-brake */
                attribute period = 40ms;
                attribute priority = 5;
                port inData;
                port outCmd;
            }

            part steeringApp {
                doc /* Nose-wheel steering from rudder pedal */
                attribute period = 40ms;
                attribute priority = 5;
                port inData;
                port outCmd;
            }

            connect inPort to extensionRetractionApp.inData;
            connect inPort to brakingApp.inData;
            connect inPort to steeringApp.inData;
            connect extensionRetractionApp.outCmd to outPort;
            connect brakingApp.outCmd to outPort;
            connect steeringApp.outCmd to outPort;
        }

        // ==========================================
        // Fuel Domain (CPIOM-F) -- DAL C
        // ==========================================

        part fuelDomain {
            port inPort : IOM_to_Fuel_Data;
            port outPort : Fuel_to_IOM_Cmd;

            part fuelQuantityApp {
                doc /* Computes total fuel from tank sensors */
                attribute period = 40ms;
                attribute priority = 10;
                port inData;
                port outCmd;
            }

            part transferPumpApp {
                doc /* Cross-feed valve control for tank balancing */
                attribute period = 40ms;
                attribute priority = 5;
                port inData;
                port outCmd;
            }

            connect inPort to fuelQuantityApp.inData;
            connect inPort to transferPumpApp.inData;
            connect fuelQuantityApp.outCmd to outPort;
            connect transferPumpApp.outCmd to outPort;
        }

        // ==========================================
        // Air Conditioning / ECS Domain (CPIOM-A) -- DAL D
        // ==========================================

        part airConditioningDomain {
            port inPort : IOM_to_ECS_Data;
            port outPort : ECS_to_IOM_Cmd;

            part bleedAirApp {
                doc /* Bleed valve extraction from engines */
                attribute period = 40ms;
                attribute priority = 10;
                port inData;
                port outCmd;
            }

            part temperatureRegApp {
                doc /* Pack mixing valve for cabin target temp */
                attribute period = 40ms;
                attribute priority = 5;
                port inData;
                port outCmd;
            }

            connect inPort to bleedAirApp.inData;
            connect inPort to temperatureRegApp.inData;
            connect bleedAirApp.outCmd to outPort;
            connect temperatureRegApp.outCmd to outPort;
        }

        // ==========================================
        // Electrical / Energy Domain (CPIOM-E) -- DAL B
        // ==========================================

        part energyDomain {
            port inPort : IOM_to_ELEC_Data;
            port outPort : ELEC_to_IOM_Cmd;

            part generatorControlApp {
                doc /* Monitors engine-driven generators */
                attribute period = 40ms;
                attribute priority = 10;
                port inData;
                port outCmd;
            }

            part loadSheddingApp {
                doc /* Bus tie breaker on generator failure */
                attribute period = 40ms;
                attribute priority = 5;
                port inData;
                port outCmd;
            }

            connect inPort to generatorControlApp.inData;
            connect inPort to loadSheddingApp.inData;
            connect generatorControlApp.outCmd to outPort;
            connect loadSheddingApp.outCmd to outPort;
        }

        // --- Allocation ---
        allocate gatewayNode to sim_gateway;
        allocate landingGearDomain to landingGearNode;
        allocate fuelDomain to fuelNode;
        allocate airConditioningDomain to ecsNode;
        allocate energyDomain to electricalNode;
    }
}
```

## Relationship to OSAVI

OSAVI is the **core software** (the ARINC 653 kernel itself), running bare-metal on a single Cortex-A53 board. Avio23 is the **IMA system** that uses such a kernel (here, `a653rs-linux` as a Linux-hosted type-2 hypervisor) to host multiple avionics domains across multiple computing modules.

| Concern | OSAVI | Avio23 |
|---------|-------|--------|
| Scope | Single-board ARINC 653 kernel | Multi-CPIOM IMA platform |
| Partitioning | MMU + timer (bare metal) | CGroups + namespaces (Linux) |
| APEX API | 14 Part 4 services in Rust | a653rs library (Rust) |
| Communication | Shared-memory sampling ports | Memory-mapped sampling ports over IPC |
| Configuration | system_config.xml | per-CPIOM YAML configs |
| Target | Pi Zero 2 W | Docker containers on x86_64 |

Together they cover the full DO-297 stack: OSAVI teaches how to **build** the platform; Avio23 teaches how to **design and deploy** applications on it.
