use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_a::{IomToCpiomA, CpiomAToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Bleed Air Partition");
    bleed_air_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod bleed_air_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_ecs", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToEcsPort;

    #[sampling_out(name = "bleed_to_iom", msg_size = "1KB")]
    struct BleedToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_ecs_port().unwrap();
        ctx.create_bleed_to_iom_port().unwrap();
        ctx.create_run_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn run_app(ctx: run_app::Context) {
        info!("Started Bleed Air App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_ecs_port.as_ref().unwrap().recv_type::<IomToCpiomA>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let pack_open = data.pack_flow_requested && data.bleed_pressure_psi > 10.0;
                    let bleed_pct = if pack_open {
                        let error = data.cabin_temp_c - data.target_temp_c;
                        (50.0 + error * 5.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };

                    let cmd = CpiomAToIom {
                        pack_valve_open: pack_open,
                        bleed_valve_pct: bleed_pct,
                        ecs_fault: data.bleed_pressure_psi < 5.0,
                        ..Default::default()
                    };
                    ctx.bleed_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
