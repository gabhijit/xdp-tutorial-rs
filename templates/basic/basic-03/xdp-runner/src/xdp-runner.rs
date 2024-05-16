use std::time::Duration;

use anyhow::Context;

use aya::maps::{Array, MapData};
use aya::programs::{Xdp, XdpFlags};
use aya::Ebpf;
use aya_log::EbpfLogger;

use clap::{ValueEnum, Parser};
use tokio::{signal, time};

use {{ to_snake_case tutorial_name }}_common::StatsRecord;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(long)]
    action: XdpAction,

    #[clap(short, long, default_value = "{{tutorial_name}}")]
    file: String,

    #[clap(short, long, default_value = "lo")]
    iface: String,

    #[clap(long)]
    release: bool,
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum XdpAction {
    #[default]
    /// Select the XDP_PASS action
    Pass,

    /// Select the XDP_DROP action
    Drop,
}

fn action_info_from_opts(opts_action: &XdpAction) -> (&str, u32, &str) {
    match opts_action {
        XdpAction::Pass => ("pass", 2u32, "{{to_snake_case tutorial_name}}_pass_packet_stats"),
        XdpAction::Drop => ("drop", 1u32, "{{to_snake_case tutorial_name}}_drop_packet_stats"),
    }
}
async fn print_stats(stats_array: &Array<&MapData, StatsRecord>, action: u32, action_name: &str) {
    let stats = stats_array.get(&action, 0);
    log::info!("Packet Count for action '{}' : {}", action_name, stats.unwrap().pkt_count);
}

// This is a Userspace program that is responsible for 'installing' the XDP eBPF binary in the
// kernel and attaching this binary to a particular network interface.

// The Packet Counter `Array` that we have created in the Kernel space, will be accessed by this
// userspace program and we will periodically dump the statistics. This tutorial

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let opts = Opt::parse();

    let profile = if opts.release { "release" } else { "debug" };

    let bpf_bin = format!("target/bpfel-unknown-none/{}/{}", profile, opts.file);
    log::info!("Loading eBPF file: '{}'", bpf_bin);

    let bpf_bin = std::fs::read(&bpf_bin)?;
    let mut bpf = Ebpf::load(&bpf_bin)?;

    let (action_name, action, program_name) = action_info_from_opts(&opts.action);
    let xdp_program = bpf.program_mut(program_name);
    if let Some(xdp_program) = xdp_program {
        log::trace!("Attaching program '{}' to interface '{}'", program_name, opts.iface);
        let xdp: &mut Xdp = xdp_program.try_into()?;
        xdp.load()?;
        let _linkid = xdp
        .attach(&opts.iface, XdpFlags::default())
        .context("Failed to attach the program to the interface using the `XdpFlags::default()`, try using `XdpFlags::SKB_MODE`")?;

        if let Err(e) = EbpfLogger::init(&mut bpf) {
            // This can happen if you remove all log statements from your eBPF program.
            log::warn!("failed to initialize eBPF logger: {}", e);
        }


        log::info!(
            "XDP Program '{}' attached to '{}'! Now waiting for Ctrl-C",
            program_name, &opts.iface
        );

        let mut stats_poller_interval = time::interval(Duration::from_secs(2));


        loop {
            tokio::select! {
                _ = stats_poller_interval.tick() => {
                    log::info!("tick!");
                    let stats_array = Array::try_from(bpf.map("STATS_ARRAY").unwrap()).unwrap();
                    print_stats(&stats_array, action, action_name).await;
                }
                _ = signal::ctrl_c() => {
                    log::info!("Exiting...");
                    break;
                }
            }
        }

        Ok(())
    } else {
        let mut progs = vec![];
        for (name, _) in bpf.programs() {
            progs.push(name);
        }

        Err(anyhow::Error::msg(format!(
            "Unable to find the program '{}' in the loaded file '{}'. Available programs are: {}",
            program_name,
            opts.file,
            progs.join(", "),
        )))
    }
}
