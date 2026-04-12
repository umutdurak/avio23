use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

// --- Simulated IOM Data ---
// The gateway simulates the role of an Input/Output Module (IOM) that bridges
// the external world (flight simulator, sensor buses) to the ARINC 653 network.
// In a real aircraft, the IOM would receive ARINC 429, discrete I/O, and analog
// signals. Here we generate synthetic flight data for all four CPIOM domains.

/// Data sent from the IOM to CPIOM-L (Landing Gear domain)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IomToCpiomL {
    pub airspeed_kt: f32,
    pub lg_lever_down: bool,
    pub left_brake_pedal: f32,
    pub right_brake_pedal: f32,
    pub rudder_pedal: f32,
}

/// Data sent from the IOM to CPIOM-F (Fuel domain)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IomToCpiomF {
    pub left_tank_kg: f32,
    pub right_tank_kg: f32,
    pub center_tank_kg: f32,
    pub fuel_temp_c: f32,
}

/// Data sent from the IOM to CPIOM-A (Air Conditioning / ECS domain)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IomToCpiomA {
    pub cabin_temp_c: f32,
    pub target_temp_c: f32,
    pub bleed_pressure_psi: f32,
    pub pack_flow_requested: bool,
}

/// Data sent from the IOM to CPIOM-E (Electrical / Energy domain)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IomToCpiomE {
    pub gen1_voltage: f32,
    pub gen2_voltage: f32,
    pub bus_load_amps: f32,
    pub battery_soc_pct: f32,
}

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Simulation Gateway (IOM)");
    sim_gateway_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod sim_gateway_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    // --- Outbound ports: IOM -> CPIOMs ---
    #[sampling_out(name = "iom_to_lg", msg_size = "1KB")]
    struct IomToLgPort;

    #[sampling_out(name = "iom_to_fuel", msg_size = "1KB")]
    struct IomToFuelPort;

    #[sampling_out(name = "iom_to_ecs", msg_size = "1KB")]
    struct IomToEcsPort;

    #[sampling_out(name = "iom_to_elec", msg_size = "1KB")]
    struct IomToElecPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_lg_port().unwrap();
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_iom_to_ecs_port().unwrap();
        ctx.create_iom_to_elec_port().unwrap();
        ctx.create_sensor_broadcast_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    /// Periodically broadcasts synthetic sensor data to all four CPIOM domains.
    /// Simulates a flight scenario: takeoff, cruise, approach, landing.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn sensor_broadcast_app(ctx: periodic::Context) {
        info!("Started Sensor Broadcast App");

        let mut cycle: u32 = 0;

        loop {
            // Simulate a simple flight profile based on cycle count
            let phase_progress = (cycle % 1000) as f32 / 1000.0;
            let airspeed = 80.0 + phase_progress * 200.0; // 80-280 kt ramp
            let gear_down = airspeed < 160.0;

            // Landing Gear data
            let lg_data = IomToCpiomL {
                airspeed_kt: airspeed,
                lg_lever_down: gear_down,
                left_brake_pedal: if gear_down { 0.3 } else { 0.0 },
                right_brake_pedal: if gear_down { 0.3 } else { 0.0 },
                rudder_pedal: 0.0,
            };
            ctx.iom_to_lg_port.as_ref().unwrap().send_type(lg_data).unwrap();

            // Fuel data (slowly decreasing tanks)
            let fuel_remaining = 12000.0 - (cycle as f32 * 0.1);
            let fuel_data = IomToCpiomF {
                left_tank_kg: fuel_remaining * 0.4,
                right_tank_kg: fuel_remaining * 0.4,
                center_tank_kg: fuel_remaining * 0.2,
                fuel_temp_c: -15.0 + phase_progress * 5.0,
            };
            ctx.iom_to_fuel_port.as_ref().unwrap().send_type(fuel_data).unwrap();

            // ECS data
            let ecs_data = IomToCpiomA {
                cabin_temp_c: 22.0 + phase_progress * 2.0,
                target_temp_c: 22.0,
                bleed_pressure_psi: 35.0,
                pack_flow_requested: true,
            };
            ctx.iom_to_ecs_port.as_ref().unwrap().send_type(ecs_data).unwrap();

            // Electrical data
            let elec_data = IomToCpiomE {
                gen1_voltage: 115.0,
                gen2_voltage: 115.0,
                bus_load_amps: 120.0 + (cycle as f32 * 0.01).sin() * 20.0,
                battery_soc_pct: 95.0 - phase_progress * 5.0,
            };
            ctx.iom_to_elec_port.as_ref().unwrap().send_type(elec_data).unwrap();

            cycle = cycle.wrapping_add(1);
            ctx.periodic_wait().unwrap();
        }
    }
}
