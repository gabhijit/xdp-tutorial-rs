use std::time::Duration;

use anyhow::Context;

use aya::maps::{MapData, PerCpuArray};
use aya::programs::{Xdp, XdpFlags};
use aya::Ebpf;
use aya_log::EbpfLogger;

use clap::Parser;
use log::{info, warn};
use tokio::{signal, time};

use {{ to_snake_case tutorial_name }}_common::StatsRecord;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long)]
    program: String,

    #[clap(short, long)]
    file: String,

    #[clap(short, long, default_value = "lo")]
    iface: String,
}

async fn print_stats_for_action(stats_array: &PerCpuArray<&MapData, StatsRecord>, action: u32) {
    let values = stats_array.get(&action, 0).unwrap();
    for (i, value) in values.iter().enumerate() {
        info!(
            "CPU: {}, Action: {}, Packet Count: {}",
            i, action, value.pkt_count
        );
    }
}

// This is a Userspace program that is responsible for 'installing' the XDP eBPF binary in the
// kernel and attaching this binary to a particular network interface.

// The Packet Counter `PerCpuArray` that we have created in the Kernel space, will be accessed by this
// userspace program and we will periodically dump the statistics. This tutorial

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    stats().await
}

async fn stats() -> anyhow::Result<()> {
    let opts = Opt::parse();
    env_logger::init();

    let bpf_bin = format!("target/bpfel-unknown-none/debug/{}", opts.file);
    let bpf_bin = std::fs::read(&bpf_bin)?;
    let mut bpf = Ebpf::load(&bpf_bin)?;
    let xdp_program = bpf.program_mut(&opts.program);

    if let Some(xdp_program) = xdp_program {
        let xdp: &mut Xdp = xdp_program.try_into()?;
        xdp.load()?;
        let _linkid = xdp
        .attach(&opts.iface, XdpFlags::default())
        .context("Failed to attach the program to the interface using the `XdpFlags::default()`, try using `XdpFlags::SKB_MODE`")?;

        if let Err(e) = EbpfLogger::init(&mut bpf) {
            // This can happen if you remove all log statements from your eBPF program.
            warn!("failed to initialize eBPF logger: {}", e);
        }

        info!(
            "XDP Program '{}' attached to '{}'! Now waiting for Ctrl-C",
            &opts.program, &opts.iface
        );

        let mut stats_poller_interval = time::interval(Duration::from_secs(2));

        loop {
            tokio::select! {
                _ = stats_poller_interval.tick() => {
                    info!("tick!");
                    let stats_array = PerCpuArray::try_from(bpf.map("PINNED_PERCPU_ARRAY").unwrap()).unwrap();
                    // TODO: Right now just using XDP_PASS, make it a proper action
                    print_stats_for_action(&stats_array, 2).await;
                }
                _ = signal::ctrl_c() => {
                    info!("Exiting...");
                    break;
                }
            }
        }

        Ok(())
    } else {
        let mut progs = vec![];
        for (name, _program_type) in bpf.programs() {
            progs.push(name);
        }

        Err(anyhow::Error::msg(format!(
            "Unable to find the program '{}' in the loaded file '{}'. Available programs are: {}",
            opts.program,
            opts.file,
            progs.join(", "),
        )))
    }
}
