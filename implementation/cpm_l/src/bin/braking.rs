use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_l::{IomToCpiomL, CpiomLToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Braking Partition");
    braking_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod braking_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_lg", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToLgPort;

    #[sampling_out(name = "brake_to_iom", msg_size = "1KB")]
    struct BrakeToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_lg_port().unwrap();
        ctx.create_brake_to_iom_port().unwrap();
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
        base_priority = 5,
        deadline = "Soft"
    )]
    fn run_app(ctx: run_app::Context) {
        info!("Started Braking App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_lg_port.as_ref().unwrap().recv_type::<IomToCpiomL>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let left_press = data.left_brake_pedal * 3000.0;
                    let right_press = data.right_brake_pedal * 3000.0;

                    let cmd = CpiomLToIom {
                        left_brake_pressure: left_press,
                        right_brake_pressure: right_press,
                        ..Default::default()
                    };
                    ctx.brake_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
