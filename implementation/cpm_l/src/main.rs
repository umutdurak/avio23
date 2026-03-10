use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

// --- DATA STRUCTURES ---
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct IomToCpiomL {
    airspeed_kt: f32,
    lg_lever_down: bool,
    left_brake_pedal: f32,
    right_brake_pedal: f32,
    rudder_pedal: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct CpiomLToIom {
    gear_deployed: bool,
    left_brake_pressure: f32,
    right_brake_pressure: f32,
    nose_wheel_angle: f32,
}

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Landing Gear Domain (CPIOM-L)");
    cpiom_l_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpiom_l_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    // --- ARINC 653 PORTS ---
    #[sampling_in(name = "iom_to_lg", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToLgPort;

    #[sampling_out(name = "lg_to_iom", msg_size = "1KB")]
    struct LgToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_lg_port().unwrap();
        ctx.create_lg_to_iom_port().unwrap();
        ctx.create_extension_retraction_app().unwrap().start().unwrap();
        ctx.create_braking_app().unwrap().start().unwrap();
        ctx.create_steering_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    // --- APPLICATION 1: Extension/Retraction ---
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn extension_retraction_app(ctx: periodic::Context) {
        info!("Started Extension/Retraction App");
        let mut gear_state = false; // false = up, true = down

        loop {
            // Use postcard instead of raw bincode to read typed messages
            if let Ok((valid, data)) = ctx.iom_to_lg_port.as_ref().unwrap().recv_type::<IomToCpiomL>() {
                if valid == a653rs::bindings::Validity::Valid {
                    if data.lg_lever_down && data.airspeed_kt < 160.0 {
                        gear_state = true;
                    } else if !data.lg_lever_down {
                        gear_state = false;
                    }

                    let cmd = CpiomLToIom {
                        gear_deployed: gear_state,
                        ..Default::default()
                    };
                    
                    ctx.lg_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }

    // --- APPLICATION 2: Braking ---
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 5,
        deadline = "Soft"
    )]
    fn braking_app(ctx: periodic::Context) {
        info!("Started Braking App");
        loop {
            ctx.periodic_wait().unwrap();
        }
    }

    // --- APPLICATION 3: Steering ---
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 5,
        deadline = "Soft"
    )]
    fn steering_app(ctx: periodic::Context) {
        info!("Started Steering App");
        loop {
            ctx.periodic_wait().unwrap();
        }
    }
}
