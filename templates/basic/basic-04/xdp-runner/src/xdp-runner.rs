use std::time::Duration;

use anyhow::Context;

use aya::maps::{Map, MapData, PerCpuArray};
use aya::programs::{links::FdLink, Xdp, XdpFlags};
use aya::EbpfLoader;

use clap::{Parser, ValueEnum};
use log::info;
use tokio::{signal, time};

use {{to_snake_case tutorial_name}}_common::StatsRecord;

#[derive(Debug, Parser)]
enum CliCommand {
    /// Pin: Pins a given map
    Pin(PinCommand),

    /// Stats: Loads a given Map and
    Stats(StatsCommand),

    /// Unpin: Unpins, Removes a pinned map
    Unpin,

    /// List: Lists all Pinned Maps
    List,
}

#[derive(Debug, Parser)]
struct PinCommand {
    /// Action for which the program is attached.
    #[clap(short, long)]
    action: Action,

    /// Name of the 'eBPF' binary file.
    #[clap(short, long, default_value = "{{tutorial_name}}")]
    file: String,

    /// Interface name to which the program is attached.
    #[clap(short, long, default_value = "lo")]
    iface: String,

    /// Run the binary in 'release' mode
    #[clap(long)]
    release: bool,
}

#[derive(Debug, Parser)]
struct StatsCommand {
    /// Action for which the program is attached.
    #[clap(short, long)]
    action: Action,

    /// Interface name to which the program is attached.
    #[clap(short, long, default_value = "lo")]
    iface: String,

    /// Name of the 'eBPF' binary file.
    #[clap(short, long, default_value = "{{tutorial_name}}")]
    file: String,

    /// Run the binary in 'release' mode
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

fn pin_program_and_maps(opts: PinCommand) -> Result<(), anyhow::Error> {
    let profile = if opts.release { "release" } else { "debug" };

    let bpf_file = format!("target/bpfel-unknown-none/{}/{}", profile, opts.file);

    // We create `map_pin_path` base directory to load the "pinned" maps. The "pinned" maps will be
    // loaded to "/map/pin/path/<MAP_NAME>" file.
    let map_pin_path = format!("/sys/fs/bpf/{}/{}/maps", opts.iface, opts.file);

    // The dirs are rquired to be present for the maps to be loaded.
    std::fs::create_dir_all(&map_pin_path)?;

    // Instead of Using `Ebpf::load` API, we use the `EbpfLoader` API to customize the loading of
    // maps. This allows loading of the maps of 'unsupported' types to be loaded. But they won't be
    // accessible from the userspace (This avoids failure on loading unsupported maps).
    let mut bpf = EbpfLoader::new()
        .allow_unsupported_maps()
        .map_pin_path(map_pin_path)
        .load_file(bpf_file)?;

    let (_, _, program_name) = action_info_from_opts(&opts.action);

    // xdp_program returned is an `enum Program` which can be converted into an `Xdp` program
    let xdp_program = bpf.program_mut(program_name);
    if let Some(xdp_program) = xdp_program {
        // The `Xdp` program variant of the Enum
        let xdp: &mut Xdp = xdp_program.try_into()?;

        // Load the program in the kernel.
        xdp.load()?;

        let link_id = xdp
        .attach(&opts.iface, XdpFlags::default())
        .context("Failed to attach the program to the interface using the `XdpFlags::default()`, try using `XdpFlags::SKB_MODE`")?;

        let xdp_link = xdp.take_link(link_id)?;
        let fd_link: FdLink = xdp_link.try_into().unwrap();

        let program_pin_path = format!("/sys/fs/bpf/{}/{}/programs", opts.iface, opts.file);
        std::fs::create_dir_all(&program_pin_path)?;
        let program_pin_path = format!("{}/{}", program_pin_path, program_name);
        let x = fd_link.pin(program_pin_path);
        eprintln!("x: {:#?}", x);

        info!(
            "XDP Program '{}' attached to '{}'! Now waiting for Ctrl-C",
            program_name, &opts.iface
        );

        Ok(())
    } else {
        let mut progs = vec![];
        for (name, _program_type) in bpf.programs() {
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

fn action_info_from_opts(opts_action: &Action) -> (&str, u32, &str) {
    match opts_action {
        Action::Pass => ("pass", 2u32, "basic_04_action_pass"),
        Action::Drop => ("drop", 1u32, "basic_04_action_drop"),
    }
}

// This is a Userspace program that is responsible for 'installing' the XDP eBPF binary in the
// kernel and attaching this binary to a particular network interface.

// The Packet Counter `PerCpuArray` that we have created in the Kernel space, will be accessed by this
// userspace program and we will periodically dump the statistics. This tutorial

async fn stats(opts: StatsCommand) -> anyhow::Result<()> {
    env_logger::init();

    let mut stats_poller_interval = time::interval(Duration::from_secs(2));

    let map_pin_path = format!(
        "/sys/fs/bpf/{}/{}/maps/PINNED_PERCPU_ARRAY",
        opts.iface, opts.file
    );
    let map_data = MapData::from_pin(map_pin_path).unwrap();
    let map = Map::PerCpuArray(map_data);
    let stats_array = map.try_into().unwrap();

    let (action_name, action_val, _) = action_info_from_opts(&opts.action);

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
}

async fn print_stats_for_action(
    stats_array: &PerCpuArray<MapData, StatsRecord>,
    action: u32,
    action_name: &str,
) {
    let mut total_packets = 0;
    let values = stats_array.get(&action, 0).unwrap();
    for (i, value) in values.iter().enumerate() {
        info!(
            "CPU: {}, Action: {}, Packet Count: {}",
            i, action_name, value.pkt_count
        );
        total_packets += value.pkt_count;
    }
    info!("Action: {}, Total  Packets: {}", action_name, total_packets);
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = CliCommand::parse();

    match cli {
        CliCommand::Stats(opts) => stats(opts).await,
        CliCommand::Pin(opts) => pin_program_and_maps(opts),
        _ => Err(anyhow::Error::msg("Unhandled command")),
    }
}
