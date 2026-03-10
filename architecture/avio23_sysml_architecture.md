# Avio23 IMA Architecture (SysML v2)

This document formally defines the architecture of the Avio23 Integrated Modular Avionics (IMA) platform using the Systems Modeling Language version 2 (SysML v2) textual notation. 

We will iteratively refine this model, drilling down into each domain (Landing Gear, Fuel, Air Conditioning, Electrical) to define specific avionics applications and their interfaces over the ARINC 653 / AFDX network.

## Current SysML v2 Model

```sysml
package Avio23_IMA_Architecture {
    
    // Core Data Types for IMA Communication
    import ScalarValues::*;
    
    // ==========================================
    // Platform Definition (The IMA Network & Nodes)
    // ==========================================
    
    part def IMA_Network {
        // Represents the AFDX / ARINC 664 network backbone
    }
    
    part def CPIOM {
        // Generic Core Processing Input/Output Module (hosts partitions)
        port networkPort;
    }
    
    part def IOM {
        // Generic Input/Output Module (Gateway to external systems)
        port networkPort;
        port externalSimulatorPort;
    }
    
    // ==========================================
    // System Context
    // ==========================================
    
    part Avio23_System {
        
        // --- Hardware Node Allocations ---
        part gatewayNode : IOM;
        part landingGearNode : CPIOM;
        part fuelNode : CPIOM;
        part ecsNode : CPIOM;
        part electricalNode : CPIOM;
        part afdxBus : IMA_Network;
        
        // --- Network Topology ---
        connect gatewayNode.networkPort to afdxBus;
        connect landingGearNode.networkPort to afdxBus;
        connect fuelNode.networkPort to afdxBus;
        connect ecsNode.networkPort to afdxBus;
        connect electricalNode.networkPort to afdxBus;
        
        // ==========================================
        // Domain Details: Landing Gear (CPIOM-L)
        // ==========================================
        
        part landingGearDomain {
            // Logical Ports representing ARINC 653 Sampling/Queuing connections
            port inPort : IOM_to_LG_Data;
            port outPort : LG_to_IOM_Cmd;
            
            // Applications (ARINC 653 Processes)
            part extensionRetractionApp {
                // Evaluates gear lever and airspeed to actuate gear sequencing
                port inData;
                port outCmd;
            }
            
            part brakingApp {
                // Anti-skid and auto-brake logic
                port inData;
                port outCmd;
            }
            
            part steeringApp {
                // Nose-wheel steering based on rudder pedal input
                port inData;
                port outCmd;
            }
            
            // Domain Internal Routing (Ports to Apps)
            connect inPort to extensionRetractionApp.inData;
            connect inPort to brakingApp.inData;
            connect inPort to steeringApp.inData;
            
            connect extensionRetractionApp.outCmd to outPort;
            connect brakingApp.outCmd to outPort;
            connect steeringApp.outCmd to outPort;
        }
        
        // ==========================================
        // Domain Details: Fuel (CPIOM-F)
        // ==========================================
        
        part fuelDomain {
            port inPort : IOM_to_Fuel_Data;
            port outPort : Fuel_to_IOM_Cmd;
            
            part fuelQuantityApp {
                // Calculates total fuel based on tank sensor inputs
                port inData;
            }
            
            part transferPumpApp {
                // Logic to balance tanks or supply engines
                port inData;
                port outCmd;
            }
            
            connect inPort to fuelQuantityApp.inData;
            connect inPort to transferPumpApp.inData;
            connect transferPumpApp.outCmd to outPort;
        }
        
        // ==========================================
        // Domain Details: Air Conditioning / ECS (CPIOM-A)
        // ==========================================
        
        part airConditioningDomain {
            port inPort : IOM_to_ECS_Data;
            port outPort : ECS_to_IOM_Cmd;
            
            part bleedAirApp {
                // Controls bleed valve extraction from engines
                port inData;
                port outCmd;
            }
            
            part temperatureRegApp {
                // Computes pack output to maintain cabin target temp
                port inData;
                port outCmd;
            }
            
            connect inPort to bleedAirApp.inData;
            connect inPort to temperatureRegApp.inData;
            connect bleedAirApp.outCmd to outPort;
            connect temperatureRegApp.outCmd to outPort;
        }
        
        // ==========================================
        // Domain Details: Energy / Electrical (CPIOM-E)
        // ==========================================
        
        part energyDomain {
            port inPort : IOM_to_ELEC_Data;
            port outPort : ELEC_to_IOM_Cmd;
            
            part generatorControlApp {
                // Monitors voltage/frequency of engine generators
                port inData;
                port outCmd;
            }
            
            part loadSheddingApp {
                // Opens bus tie breakers if a generator fails
                port inData;
                port outCmd;
            }
            
            connect inPort to generatorControlApp.inData;
            connect inPort to loadSheddingApp.inData;
            connect generatorControlApp.outCmd to outPort;
            connect loadSheddingApp.outCmd to outPort;
        }
        
        // --- Software to Hardware Allocation ---
        // (Mapping our logical domains to physical/simulated CPIOM nodes)
        allocate externalGateway to gatewayNode;
        allocate landingGearDomain to landingGearNode;
        allocate fuelDomain to fuelNode;
        allocate airConditioningDomain to ecsNode;
        allocate energyDomain to electricalNode;
    }
}
```

## User Review Required

> [!IMPORTANT]
> The complete IMA platform architecture is defined above in SysML v2, detailing the Landing Gear, Fuel, Air Conditioning, and Energy partitions. 
> 
> Please review the applications running within these partitions. Once you are satisfied with this architectural blueprint, we can transition to writing the concrete `a653rs-linux` hypervisor configurations and the Rust code for these applications!
