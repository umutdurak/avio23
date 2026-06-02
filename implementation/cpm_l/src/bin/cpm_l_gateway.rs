use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, warn, LevelFilter};
use cpm_l::{IomToCpiomL, CpiomLToIom};
use std::net::UdpSocket;

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting CPM-L Network I/O Gateway");
    cpm_l_gateway::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod cpm_l_gateway {
    use super::*;
    use a653rs_postcard::prelude::*;
    use a653rs::bindings::Validity;

    #[sampling_out(name = "iom_to_lg", msg_size = "1KB")]
    struct IomToLgPort;

    #[sampling_in(name = "ext_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct ExtToIomPort;

    #[sampling_in(name = "brake_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct BrakeToIomPort;

    #[sampling_in(name = "steer_to_iom", msg_size = "1KB", refresh_period = "40ms")]
    struct SteerToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_lg_port().unwrap();
        ctx.create_ext_to_iom_port().unwrap();
        ctx.create_brake_to_iom_port().unwrap();
        ctx.create_steer_to_iom_port().unwrap();
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
        info!("Started CPM-L Gateway Loop");

        // Obtain the socket bound by the hypervisor
        let socket: UdpSocket = a653rs_linux::partition::ApexLinuxPartition::get_udp_socket("0.0.0.0:49001")
            .unwrap()
            .expect("Could not get UDP socket from hypervisor config");
        socket.set_nonblocking(true).unwrap();

        let mut buf = [0u8; 1024];

        loop {
            // 1. Receive data from Sim Gateway (IOM) and write to local port
            match socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    if let Ok(data) = postcard::from_bytes::<IomToCpiomL>(&buf[..size]) {
                        ctx.iom_to_lg_port.as_ref().unwrap().send_type(data).unwrap();
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => warn!("UDP receive error: {:?}", e),
            }

            // 2. Read local sampling ports and send combined data back to Sim Gateway
            let mut combined = CpiomLToIom::default();
            let mut has_data = false;

            if let Ok((valid, data)) = ctx.ext_to_iom_port.as_ref().unwrap().recv_type::<CpiomLToIom>() {
                if valid == Validity::Valid {
                    combined.gear_deployed = data.gear_deployed;
                    has_data = true;
                }
            }
            if let Ok((valid, data)) = ctx.brake_to_iom_port.as_ref().unwrap().recv_type::<CpiomLToIom>() {
                if valid == Validity::Valid {
                    combined.left_brake_pressure = data.left_brake_pressure;
                    combined.right_brake_pressure = data.right_brake_pressure;
                    has_data = true;
                }
            }
            if let Ok((valid, data)) = ctx.steer_to_iom_port.as_ref().unwrap().recv_type::<CpiomLToIom>() {
                if valid == Validity::Valid {
                    combined.nose_wheel_angle = data.nose_wheel_angle;
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
