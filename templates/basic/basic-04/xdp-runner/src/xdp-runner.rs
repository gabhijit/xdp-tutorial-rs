use std::time::Duration;

use anyhow::Context;

use aya::maps::{MapData, PerCpuArray};
use aya::programs::{Xdp, XdpFlags};
use aya::EbpfLoader;
use aya_log::EbpfLogger;

use clap::{Parser, ValueEnum};
use log::{info, warn};
use tokio::{signal, time};

use {{to_snake_case tutorial_name}}_common::StatsRecord;

#[derive(Debug, Parser)]
enum CliCommand {
    /// Pin: Pins a given map
    Pin,

    /// Load: Loads a given Map and
    Load(LoadCommand),

    /// Unpin: Unpins, Removes a pinned map
    Unpin,

    /// List: Lists all Pinned Maps
    List,
}

#[derive(Debug, Parser)]
struct LoadCommand {
    #[clap(short, long)]
    action: Action,

    #[clap(short, long)]
    file: String,

    #[clap(short, long, default_value = "lo")]
    iface: String,

    #[clap(long)]
    release: bool,
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum Action {
    #[default]
    /// Select the XDP_PASS action
    Pass,

    /// Select the XDP_DROP action
    Drop,
}

async fn print_stats_for_action(
    stats_array: &PerCpuArray<&MapData, StatsRecord>,
    action: u32,
    action_name: &str,
) {
    let values = stats_array.get(&action, 0).unwrap();
    for (i, value) in values.iter().enumerate() {
        info!(
            "CPU: {}, Action: {}, Packet Count: {}",
            i, action_name, value.pkt_count
        );
    }
}

// This is a Userspace program that is responsible for 'installing' the XDP eBPF binary in the
// kernel and attaching this binary to a particular network interface.

// The Packet Counter `PerCpuArray` that we have created in the Kernel space, will be accessed by this
// userspace program and we will periodically dump the statistics. This tutorial

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = CliCommand::parse();

    match cli {
        CliCommand::Load(opts) => stats(opts).await,
        _ => Err(anyhow::Error::msg("Unhandled command")),
    }
}

async fn stats(opts: LoadCommand) -> anyhow::Result<()> {
    env_logger::init();

    let profile = if opts.release { "release" } else { "debug" };

    let bpf_file = format!("target/bpfel-unknown-none/{}/{}", profile, opts.file);

    // We create `map_pin_path` base directory to load the "pinned" maps. The "pinned" maps will be
    // loaded to "/map/pin/path/<MAP_NAME>" file.
    let map_pin_path = format!("/sys/fs/bpf/{}/{}/", opts.iface, opts.file);

    // The dirs are rquired to be present for the maps to be loaded.
    std::fs::create_dir_all(&map_pin_path)?;

    // Instead of Using `Ebpf::load` API, we use the `EbpfLoader` API to customize the loading of
    // maps. This allows loading of the maps of 'unsupported' types to be loaded. But they won't be
    // accessible from the userspace (This avoids failure on loading unsupported maps).
    let mut bpf = EbpfLoader::new()
        .allow_unsupported_maps()
        .map_pin_path(map_pin_path)
        .load_file(bpf_file)?;

    let (action_name, action_val, program) = match opts.action {
        Action::Pass => ("pass", 2u32, "{{ to_snake_case tutorial_name }}_action_pass"),
        Action::Drop => ("drop", 1u32, "{{ to_snake_case tutorial_name }}_action_drop"),
    };

    let xdp_program = bpf.program_mut(program);

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
            program, &opts.iface
        );

        let mut stats_poller_interval = time::interval(Duration::from_secs(2));

        let map = bpf.map("PINNED_PERCPU_ARRAY").unwrap();
        let stats_array = PerCpuArray::try_from(map).unwrap();

        loop {
            tokio::select! {
                _ = stats_poller_interval.tick() => {
                    info!("tick!");
                    // TODO: Right now just using XDP_PASS, make it a proper action
                    print_stats_for_action(&stats_array, action_val, action_name).await;
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
            program,
            opts.file,
            progs.join(", "),
        )))
    }
}
