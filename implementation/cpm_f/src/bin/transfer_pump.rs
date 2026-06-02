use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_f::{IomToCpiomF, CpiomFToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Transfer Pump Partition");
    transfer_pump_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod transfer_pump_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_fuel", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToFuelPort;

    #[sampling_out(name = "pump_to_iom", msg_size = "1KB")]
    struct PumpToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_pump_to_iom_port().unwrap();
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
        info!("Started Transfer Pump App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_fuel_port.as_ref().unwrap().recv_type::<IomToCpiomF>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let imbalance = (data.left_tank_kg - data.right_tank_kg).abs();
                    let active = imbalance > 200.0;

                    let cmd = CpiomFToIom {
                        transfer_pump_active: active,
                        ..Default::default()
                    };
                    ctx.pump_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
