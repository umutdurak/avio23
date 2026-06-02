use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_l::{IomToCpiomL, CpiomLToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Extension/Retraction Partition");
    extension_retraction_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod extension_retraction_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_lg", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToLgPort;

    #[sampling_out(name = "ext_to_iom", msg_size = "1KB")]
    struct ExtToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_lg_port().unwrap();
        ctx.create_ext_to_iom_port().unwrap();
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
        info!("Started Extension/Retraction App");
        let mut gear_state = false;

        loop {
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
                    ctx.ext_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
