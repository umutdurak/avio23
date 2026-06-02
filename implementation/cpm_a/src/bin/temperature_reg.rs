use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_a::{IomToCpiomA, CpiomAToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Temperature Regulation Partition");
    temperature_reg_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod temperature_reg_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_ecs", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToEcsPort;

    #[sampling_out(name = "temp_reg_to_iom", msg_size = "1KB")]
    struct TempRegToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_ecs_port().unwrap();
        ctx.create_temp_reg_to_iom_port().unwrap();
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
        info!("Started Temperature Reg App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_ecs_port.as_ref().unwrap().recv_type::<IomToCpiomA>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let temp_err = data.cabin_temp_c - data.target_temp_c;

                    let cmd = CpiomAToIom {
                        cabin_temp_error_c: temp_err,
                        ..Default::default()
                    };
                    ctx.temp_reg_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
