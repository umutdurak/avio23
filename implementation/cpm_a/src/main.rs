use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

// --- Data Structures ---

/// Sensor data received from the IOM (cabin temp, bleed air, pack request)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct IomToCpiomA {
    cabin_temp_c: f32,
    target_temp_c: f32,
    bleed_pressure_psi: f32,
    pack_flow_requested: bool,
}

/// Commands sent back to the IOM / other CPIOMs
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct CpiomAToIom {
    pack_valve_open: bool,
    bleed_valve_pct: f32,
    cabin_temp_error_c: f32,
    ecs_fault: bool,
}

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Air Conditioning / ECS Domain (CPIOM-A)");
    cpiom_a_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpiom_a_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    // --- ARINC 653 Ports ---
    #[sampling_in(name = "iom_to_ecs", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToEcsPort;

    #[sampling_out(name = "ecs_to_iom", msg_size = "1KB")]
    struct EcsToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_ecs_port().unwrap();
        ctx.create_ecs_to_iom_port().unwrap();
        ctx.create_bleed_air_app().unwrap().start().unwrap();
        ctx.create_temperature_reg_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    /// Bleed Air Control: reads engine bleed pressure and regulates the
    /// pack valve to supply conditioned air to the cabin.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn bleed_air_app(ctx: periodic::Context) {
        info!("Started Bleed Air App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_ecs_port.as_ref().unwrap().recv_type::<IomToCpiomA>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let pack_open = data.pack_flow_requested && data.bleed_pressure_psi > 10.0;
                    let bleed_pct = if pack_open {
                        // Proportional control: more bleed when cabin is too warm
                        let error = data.cabin_temp_c - data.target_temp_c;
                        (50.0 + error * 5.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };

                    let cmd = CpiomAToIom {
                        pack_valve_open: pack_open,
                        bleed_valve_pct: bleed_pct,
                        cabin_temp_error_c: data.cabin_temp_c - data.target_temp_c,
                        ecs_fault: data.bleed_pressure_psi < 5.0,
                    };

                    ctx.ecs_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }

    /// Temperature Regulation: computes pack output to maintain cabin target.
    /// In a full implementation, this would command the mixing valve.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 5,
        deadline = "Soft"
    )]
    fn temperature_reg_app(ctx: periodic::Context) {
        info!("Started Temperature Regulation App");
        loop {
            ctx.periodic_wait().unwrap();
        }
    }
}
