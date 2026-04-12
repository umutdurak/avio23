use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};

// --- Data Structures ---

/// Sensor data received from the IOM (via sim_gateway)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct IomToCpiomF {
    left_tank_kg: f32,
    right_tank_kg: f32,
    center_tank_kg: f32,
    fuel_temp_c: f32,
}

/// Commands sent back to the IOM / other CPIOMs
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
struct CpiomFToIom {
    total_fuel_kg: f32,
    fuel_imbalance_kg: f32,
    transfer_pump_active: bool,
    low_fuel_warning: bool,
}

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Fuel Domain (CPIOM-F)");
    cpiom_f_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpiom_f_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    // --- ARINC 653 Ports ---
    #[sampling_in(name = "iom_to_fuel", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToFuelPort;

    #[sampling_out(name = "fuel_to_iom", msg_size = "1KB")]
    struct FuelToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_fuel_to_iom_port().unwrap();
        ctx.create_fuel_quantity_app().unwrap().start().unwrap();
        ctx.create_transfer_pump_app().unwrap().start().unwrap();
    }

    #[start(warm)]
    fn warm_start(ctx: start::Context) {
        cold_start(ctx)
    }

    /// Fuel Quantity Computation: reads tank sensors, computes total and imbalance,
    /// publishes aggregated fuel state to the IOM sampling port.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 10,
        deadline = "Soft"
    )]
    fn fuel_quantity_app(ctx: periodic::Context) {
        info!("Started Fuel Quantity App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_fuel_port.as_ref().unwrap().recv_type::<IomToCpiomF>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let total = data.left_tank_kg + data.right_tank_kg + data.center_tank_kg;
                    let imbalance = (data.left_tank_kg - data.right_tank_kg).abs();

                    let cmd = CpiomFToIom {
                        total_fuel_kg: total,
                        fuel_imbalance_kg: imbalance,
                        transfer_pump_active: imbalance > 200.0,
                        low_fuel_warning: total < 2000.0,
                    };

                    ctx.fuel_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }

    /// Transfer Pump Control: monitors fuel imbalance and activates cross-feed
    /// when the left/right differential exceeds the threshold.
    #[periodic(
        period = "40ms",
        time_capacity = "Infinite",
        stack_size = "100KB",
        base_priority = 5,
        deadline = "Soft"
    )]
    fn transfer_pump_app(ctx: periodic::Context) {
        info!("Started Transfer Pump App");
        loop {
            // In a full implementation this would read the fuel_quantity output
            // and command the cross-feed valve. For now it yields each cycle.
            ctx.periodic_wait().unwrap();
        }
    }
}
