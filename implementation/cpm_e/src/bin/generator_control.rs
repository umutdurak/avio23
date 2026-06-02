use a653rs::partition;
use a653rs::prelude::PartitionExt;
use a653rs_linux::partition::ApexLogger;
use log::{info, LevelFilter};
use cpm_e::{IomToCpiomE, CpiomEToIom};

fn main() {
    ApexLogger::install_panic_hook();
    ApexLogger::install_logger(LevelFilter::Trace).unwrap();
    info!("Starting Generator Control Partition");
    generator_control_partition::Partition.run()
}

#[partition(a653rs_linux::partition::ApexLinuxPartition)]
mod generator_control_partition {
    use super::*;
    use a653rs_postcard::prelude::*;

    #[sampling_in(name = "iom_to_elec", msg_size = "1KB", refresh_period = "40ms")]
    struct IomToElecPort;

    #[sampling_out(name = "gen_to_iom", msg_size = "1KB")]
    struct GenToIomPort;

    #[start(cold)]
    fn cold_start(mut ctx: start::Context) {
        ctx.create_iom_to_elec_port().unwrap();
        ctx.create_gen_to_iom_port().unwrap();
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
        info!("Started Generator Control App");
        loop {
            if let Ok((valid, data)) = ctx.iom_to_elec_port.as_ref().unwrap().recv_type::<IomToCpiomE>() {
                if valid == a653rs::bindings::Validity::Valid {
                    let gen1_ok = (108.0..=122.0).contains(&data.gen1_voltage);
                    let gen2_ok = (108.0..=122.0).contains(&data.gen2_voltage);
                    let bus_tie = gen1_ok && gen2_ok;

                    let cmd = CpiomEToIom {
                        gen1_online: gen1_ok,
                        gen2_online: gen2_ok,
                        bus_tie_closed: bus_tie,
                        battery_charging: data.battery_soc_pct < 90.0 && (gen1_ok || gen2_ok),
                        ..Default::default()
                    };
                    ctx.gen_to_iom_port.as_ref().unwrap().send_type(cmd).unwrap();
                }
            }
            ctx.periodic_wait().unwrap();
        }
    }
}
