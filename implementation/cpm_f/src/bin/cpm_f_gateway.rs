use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, warn, LevelFilter};
use cpm_f::{IomToCpiomF, CpiomFToIom};
use std::net::UdpSocket;

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting CPM-F Network I/O Gateway");
    cpm_f_gateway::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpm_f_gateway {
    use super::*;
    use a653rs_postcard::prelude::*;
    use a653rs::bindings::Validity;

    #[sampling_out(name = "iom_to_fuel", msg_size = "1KB")]
    struct IomToFuelPort;

    #[sampling_in(name = "fuel_qty_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct FuelQtyToIomPort;

    #[sampling_in(name = "pump_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct PumpToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_fuel_port().unwrap();
        ctx.create_fuel_qty_to_iom_port().unwrap();
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
        base_priority = 10,
        deadline = "Soft"
    )]
    fn run_app(ctx: run_app::Context) {
        info!("Started CPM-F Gateway Loop");

        let socket: UdpSocket = a653rs_linux::partition::ApexLinuxPartition::get_udp_socket("0.0.0.0:49002")
            .unwrap()
            .expect("Could not get UDP socket from hypervisor config");
        socket.set_nonblocking(true).unwrap();

        let mut buf = [0u8; 1024];

        loop {
            // 1. Receive data from Sim Gateway and write to local port
            match socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if let Ok(data) = postcard::from_bytes::<IomToCpiomF>(&buf[..size]) {
                        ctx.iom_to_fuel_port.as_ref().unwrap().send_type(data).unwrap();
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => warn!("UDP receive error: {:?}", e),
            }

            // 2. Read local sampling ports and send combined data back to Sim Gateway
            let mut combined = CpiomFToIom::default();
            let mut has_data = false;

            if let Ok((valid, data)) = ctx.fuel_qty_to_iom_port.as_ref().unwrap().recv_type::<CpiomFToIom>() {
                if valid == Validity::Valid {
                    combined.total_fuel_kg = data.total_fuel_kg;
                    combined.fuel_imbalance_kg = data.fuel_imbalance_kg;
                    combined.low_fuel_warning = data.low_fuel_warning;
                    has_data = true;
                }
            }
            if let Ok((valid, data)) = ctx.pump_to_iom_port.as_ref().unwrap().recv_type::<CpiomFToIom>() {
                if valid == Validity::Valid {
                    combined.transfer_pump_active = data.transfer_pump_active;
                    has_data = true;
                }
            }

            if has_data {
                if let Ok(bytes) = postcard::to_allocvec(&combined) {
                    if let Err(e) = socket.send_to(&bytes, "172.20.0.2:49000") {
                        warn!("Failed to send UDP packet to sim_gateway: {:?}", e);
                    }
                }
            }

            ctx.periodic_wait().unwrap();
        }
    }
}
