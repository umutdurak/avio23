use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, warn, LevelFilter};
use cpm_e::{IomToCpiomE, CpiomEToIom};
use std::net::UdpSocket;

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting CPM-E Network I/O Gateway");
    cpm_e_gateway::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpm_e_gateway {
    use super::*;
    use a653rs_postcard::prelude::*;
    use a653rs::bindings::Validity;

    #[sampling_out(name = "iom_to_elec", msg_size = "1KB")]
    struct IomToElecPort;

    #[sampling_in(name = "gen_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct GenToIomPort;

    #[sampling_in(name = "shed_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct ShedToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_elec_port().unwrap();
        ctx.create_gen_to_iom_port().unwrap();
        ctx.create_shed_to_iom_port().unwrap();
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
        info!("Started CPM-E Gateway Loop");

        let socket: UdpSocket = a653rs_linux::partition::ApexLinuxPartition::get_udp_socket("0.0.0.0:49004")
            .unwrap()
            .expect("Could not get UDP socket from hypervisor config");
        socket.set_nonblocking(true).unwrap();

        let mut buf = [0u8; 1024];

        loop {
            // 1. Receive data from Sim Gateway and write to local port
            match socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if let Ok(data) = postcard::from_bytes::<IomToCpiomE>(&buf[..size]) {
                        ctx.iom_to_elec_port.as_ref().unwrap().send_type(data).unwrap();
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => warn!("UDP receive error: {:?}", e),
            }

            // 2. Read local sampling ports and send combined data back to Sim Gateway
            let mut combined = CpiomEToIom::default();
            let mut has_data = false;

            if let Ok((valid, data)) = ctx.gen_to_iom_port.as_ref().unwrap().recv_type::<CpiomEToIom>() {
                if valid == Validity::Valid {
                    combined.gen1_online = data.gen1_online;
                    combined.gen2_online = data.gen2_online;
                    combined.bus_tie_closed = data.bus_tie_closed;
                    combined.battery_charging = data.battery_charging;
                    has_data = true;
                }
            }
            if let Ok((valid, data)) = ctx.shed_to_iom_port.as_ref().unwrap().recv_type::<CpiomEToIom>() {
                if valid == Validity::Valid {
                    combined.load_shed_active = data.load_shed_active;
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
