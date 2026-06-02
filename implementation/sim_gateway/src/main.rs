use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, warn, LevelFilter};
use std::net::UdpSocket;

// Import the structures from CPM libraries
use cpm_l::IomToCpiomL;
use cpm_f::IomToCpiomF;
use cpm_a::IomToCpiomA;
use cpm_e::IomToCpiomE;

use cpm_l::CpiomLToIom;
use cpm_f::CpiomFToIom;
use cpm_a::CpiomAToIom;
use cpm_e::CpiomEToIom;

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Simulation Gateway (IOM)");
    sim_gateway_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod sim_gateway_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_sensor_broadcast_app().unwrap().start().unwrap();
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
    fn sensor_broadcast_app(ctx: sensor_broadcast_app::Context) {
        info!("Started Simulation Gateway Loop");

        // Obtain the socket bound by the hypervisor (exposed to this partition)
        let socket: UdpSocket = a653rs_linux::partition::ApexLinuxPartition::get_udp_socket("0.0.0.0:49000")
            .unwrap()
            .expect("Could not get UDP socket from hypervisor config");
        socket.set_nonblocking(true).unwrap();

        let mut buf = [0u8; 1024];
        let mut cycle: u32 = 0;

        // Maintain the latest telemetry results received from CPMs
        let mut cpm_l_state = CpiomLToIom::default();
        let mut cpm_f_state = CpiomFToIom::default();
        let mut cpm_a_state = CpiomAToIom::default();
        let mut cpm_e_state = CpiomEToIom::default();

        loop {
            // 1. Send synthetic telemetry data to each CPM gateway over the network
            let phase_progress = (cycle % 1000) as f32 / 1000.0;
            let airspeed = 80.0 + phase_progress * 200.0; // 80-280 kt ramp
            let gear_down = airspeed < 160.0;

            // Landing Gear data
            let lg_data = IomToCpiomL {
                airspeed_kt: airspeed,
                lg_lever_down: gear_down,
                left_brake_pedal: if gear_down { 0.3 } else { 0.0 },
                right_brake_pedal: if gear_down { 0.3 } else { 0.0 },
                rudder_pedal: (cycle as f32 * 0.05).sin(),
            };
            if let Ok(bytes) = postcard::to_allocvec(&lg_data) {
                let _ = socket.send_to(&bytes, "172.20.0.3:49001");
            }

            // Fuel data
            let fuel_remaining = 12000.0 - (cycle as f32 * 0.1);
            let fuel_data = IomToCpiomF {
                left_tank_kg: fuel_remaining * 0.4,
                right_tank_kg: fuel_remaining * 0.4,
                center_tank_kg: fuel_remaining * 0.2,
                fuel_temp_c: -15.0 + phase_progress * 5.0,
            };
            if let Ok(bytes) = postcard::to_allocvec(&fuel_data) {
                let _ = socket.send_to(&bytes, "172.20.0.4:49002");
            }

            // ECS data
            let ecs_data = IomToCpiomA {
                cabin_temp_c: 22.0 + phase_progress * 2.0,
                target_temp_c: 22.0,
                bleed_pressure_psi: 35.0,
                pack_flow_requested: true,
            };
            if let Ok(bytes) = postcard::to_allocvec(&ecs_data) {
                let _ = socket.send_to(&bytes, "172.20.0.5:49003");
            }

            // Electrical data
            let elec_data = IomToCpiomE {
                gen1_voltage: 115.0,
                gen2_voltage: 115.0,
                bus_load_amps: 120.0 + (cycle as f32 * 0.01).sin() * 20.0,
                battery_soc_pct: 95.0 - phase_progress * 5.0,
            };
            if let Ok(bytes) = postcard::to_allocvec(&elec_data) {
                let _ = socket.send_to(&bytes, "172.20.0.6:49004");
            }

            // 2. Read incoming UDP packets from CPM gateways
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, src)) => {
                        let ip = src.ip();
                        if ip == std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 3)) {
                            // CPM-L
                            if let Ok(data) = postcard::from_bytes::<CpiomLToIom>(&buf[..size]) {
                                // Merge updates (only replace non-default fields since it's a combined state)
                                if data.gear_deployed {
                                    cpm_l_state.gear_deployed = true;
                                } else if data.left_brake_pressure == 0.0 && data.right_brake_pressure == 0.0 && data.nose_wheel_angle == 0.0 {
                                    cpm_l_state.gear_deployed = false;
                                }
                                if data.left_brake_pressure > 0.0 || data.right_brake_pressure > 0.0 {
                                    cpm_l_state.left_brake_pressure = data.left_brake_pressure;
                                    cpm_l_state.right_brake_pressure = data.right_brake_pressure;
                                }
                                if data.nose_wheel_angle != 0.0 {
                                    cpm_l_state.nose_wheel_angle = data.nose_wheel_angle;
                                }
                            }
                        } else if ip == std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 4)) {
                            // CPM-F
                            if let Ok(data) = postcard::from_bytes::<CpiomFToIom>(&buf[..size]) {
                                if data.total_fuel_kg > 0.0 {
                                    cpm_f_state.total_fuel_kg = data.total_fuel_kg;
                                    cpm_f_state.fuel_imbalance_kg = data.fuel_imbalance_kg;
                                    cpm_f_state.low_fuel_warning = data.low_fuel_warning;
                                }
                                if data.transfer_pump_active {
                                    cpm_f_state.transfer_pump_active = true;
                                } else if data.total_fuel_kg == 0.0 {
                                    cpm_f_state.transfer_pump_active = false;
                                }
                            }
                        } else if ip == std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 5)) {
                            // CPM-A
                            if let Ok(data) = postcard::from_bytes::<CpiomAToIom>(&buf[..size]) {
                                if data.pack_valve_open || data.bleed_valve_pct > 0.0 || data.ecs_fault {
                                    cpm_a_state.pack_valve_open = data.pack_valve_open;
                                    cpm_a_state.bleed_valve_pct = data.bleed_valve_pct;
                                    cpm_a_state.ecs_fault = data.ecs_fault;
                                }
                                if data.cabin_temp_error_c != 0.0 {
                                    cpm_a_state.cabin_temp_error_c = data.cabin_temp_error_c;
                                }
                            }
                        } else if ip == std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 6)) {
                            // CPM-E
                            if let Ok(data) = postcard::from_bytes::<CpiomEToIom>(&buf[..size]) {
                                if data.gen1_online || data.gen2_online || data.bus_tie_closed || data.battery_charging {
                                    cpm_e_state.gen1_online = data.gen1_online;
                                    cpm_e_state.gen2_online = data.gen2_online;
                                    cpm_e_state.bus_tie_closed = data.bus_tie_closed;
                                    cpm_e_state.battery_charging = data.battery_charging;
                                }
                                if data.load_shed_active {
                                    cpm_e_state.load_shed_active = true;
                                } else if !data.gen1_online && !data.gen2_online {
                                    cpm_e_state.load_shed_active = false;
                                }
                            }
                        } else {
                            warn!("Received UDP packet from unknown source: {:?}", src);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break; // No more packets available
                    }
                    Err(e) => {
                        warn!("UDP receive error in Sim Gateway: {:?}", e);
                        break;
                    }
                }
            }

            // 3. LogTelemetry once per second (every 25 cycles)
            if cycle % 25 == 0 {
                info!("=== Avio23 Flight Telemetry ===");
                info!("Airspeed: {:.1} kt, Gear Down Switch: {}", airspeed, gear_down);
                info!("  CPM-L: Gear Deployed = {}, L/R Brake = {:.1}/{:.1} psi, Steer Angle = {:.1} deg", 
                    cpm_l_state.gear_deployed, cpm_l_state.left_brake_pressure, cpm_l_state.right_brake_pressure, cpm_l_state.nose_wheel_angle);
                info!("  CPM-F: Total Fuel = {:.1} kg, Warning = {}, Transfer Pump = {}", 
                    cpm_f_state.total_fuel_kg, cpm_f_state.low_fuel_warning, cpm_f_state.transfer_pump_active);
                info!("  CPM-A: Bleed Valve = {:.1}%, Pack Valve Open = {}, ECS Fault = {}", 
                    cpm_a_state.bleed_valve_pct, cpm_a_state.pack_valve_open, cpm_a_state.ecs_fault);
                info!("  CPM-E: Gen1/Gen2 Online = {}/{}, Battery Charging = {}, Load Shed = {}", 
                    cpm_e_state.gen1_online, cpm_e_state.gen2_online, cpm_e_state.battery_charging, cpm_e_state.load_shed_active);
            }

            cycle = cycle.wrapping_add(1);
            ctx.periodic_wait().unwrap();
        }
    }
}
