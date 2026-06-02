use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_f::{IomToCpiomF, CpiomFToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Fuel Quantity Partition");
    fuel_quantity_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod fuel_quantity_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_fuel", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToFuelPort;

    #[sampling_out(name = "fuel_qty_to_iom", msg_size = "1KB")]
    struct FuelQtyToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_fuel_qty_to_iom_port().unwrap();
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
        info!("Started Fuel Quantity App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_fuel_port.as_ref().unwrap().recv_type::<IomToCpiomF>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let total = data.left_tank_kg + data.right_tank_kg + data.center_tank_kg;
                    let imbalance = (data.left_tank_kg - data.right_tank_kg).abs();

                    let cmd = CpiomFToIom {
                        total_fuel_kg: total,
                        fuel_imbalance_kg: imbalance,
                        low_fuel_warning: total < 2000.0,
                        ..Default::default()
                    };
                    ctx.fuel_qty_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
