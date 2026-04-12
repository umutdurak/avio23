use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

// --- Data Structures ---

/// Sensor data received from the IOM (generator voltages, bus load, battery)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct IomToCpiomE {
    gen1_voltage: f32,
    gen2_voltage: f32,
    bus_load_amps: f32,
    battery_soc_pct: f32,
}

/// Commands sent back to the IOM / other CPIOMs
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct CpiomEToIom {
    gen1_online: bool,
    gen2_online: bool,
    bus_tie_closed: bool,
    load_shed_active: bool,
    battery_charging: bool,
}

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Electrical / Energy Domain (CPIOM-E)");
    cpiom_e_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpiom_e_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    // --- ARINC 653 Ports ---
    #[sampling_in(name = "iom_to_elec", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToElecPort;

    #[sampling_out(name = "elec_to_iom", msg_size = "1KB")]
    struct ElecToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_elec_port().unwrap();
        ctx.create_elec_to_iom_port().unwrap();
        ctx.create_generator_control_app().unwrap().start().unwrap();
        ctx.create_load_shedding_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    /// Generator Control: monitors engine-driven generators and manages the
    /// bus tie contactor. A generator is considered online if its voltage
    /// is within the nominal band (108-122 V AC).
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn generator_control_app(ctx: periodic::Context) {
        info!("Started Generator Control App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_elec_port.as_ref().unwrap().recv_type::<IomToCpiomE>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let gen1_ok = (108.0..=122.0).contains(&data.gen1_voltage);
                    let gen2_ok = (108.0..=122.0).contains(&data.gen2_voltage);

                    // Bus tie closes when both generators are healthy
                    let bus_tie = gen1_ok && gen2_ok;

                    // Load shedding when total demand exceeds single-generator capacity
                    let shed = !gen1_ok || !gen2_ok;

                    let cmd = CpiomEToIom {
                        gen1_online: gen1_ok,
                        gen2_online: gen2_ok,
                        bus_tie_closed: bus_tie,
                        load_shed_active: shed,
                        battery_charging: data.battery_soc_pct < 90.0 && (gen1_ok || gen2_ok),
                    };

                    ctx.elec_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }

    /// Load Shedding: monitors bus load and opens bus tie breakers if a
    /// generator fails, isolating the faulty side.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 5,
        deadline = "Soft"
    )]
    fn load_shedding_app(ctx: periodic::Context) {
        info!("Started Load Shedding App");
        loop {
            ctx.periodic_wait().unwrap();
        }
    }
}
