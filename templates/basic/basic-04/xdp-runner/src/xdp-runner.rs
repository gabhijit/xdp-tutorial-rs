use std::time::Duration;

use anyhow::Context;

use aya::maps::{Map, MapData, PerCpuArray};
use aya::programs::{links::FdLink, Xdp, XdpFlags};
use aya::EbpfLoader;

use clap::{Parser, ValueEnum};
use tokio::{signal, time};

use {{to_snake_case tutorial_name}}_common::StatsRecord;

#[derive(Debug, Parser)]
enum CliCommand {
    /// Pin: Pins a given map
    Pin(PinOptions),

    /// Stats: Loads a given Map and
    Stats(StatsOptions),

    /// Unpin: Unpins, Removes a pinned map
    Unpin(UnpinOptions),

    /// List: Lists all Pinned Maps
    List(ListOptions),
}

// Handling of the 'pin' command.
#[derive(Debug, Parser)]
struct PinOptions {
    /// XDP Action for which the program is attached.
    #[clap(short, long)]
    action: XdpAction,

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
        XdpAction::Pass => ("pass", 2u32, "{{to_snake_case tutorial_name}}_action_pass"),
        XdpAction::Drop => ("drop", 1u32, "{{to_snake_case tutorial_name}}_action_drop"),
    }
}

fn pin_program_and_maps(opts: PinOptions) -> Result<(), anyhow::Error> {
    let profile = if opts.release { "release" } else { "debug" };
    log::trace!("Using {profile} profile.");

    let bpf_file = format!("target/bpfel-unknown-none/{}/{}", profile, opts.file);

    // We create `map_pin_path` base directory to load the "pinned" maps. The "pinned" maps will be
    // loaded to "/map/pin/path/<MAP_NAME>" file.
    let map_pin_path = format!("/sys/fs/bpf/{}/{}/maps", opts.iface, opts.file);
    log::trace!("'map_pin_path set to {}", map_pin_path);

    // The dirs are rquired to be present for the maps to be loaded.
    std::fs::create_dir_all(&map_pin_path)?;

    // Instead of Using `Ebpf::load` API, we use the `EbpfLoader` API to customize the loading of
    // maps. This allows loading of the maps of 'unsupported' types to be loaded. But they won't be
    // accessible from the userspace (This avoids failure on loading unsupported maps).
    log::info!("Loading eBPF file: {bpf_file}");
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

        log::trace!("Loading XDP Program '{program_name}' in the Kernel");
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
        fd_link.pin(&program_pin_path)?;

        log::info!(
            "XDP Program: '{}' attached to interface: '{}' and pinned at path: '{}'",
            program_name,
            &opts.iface,
            program_pin_path,
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

// Handling of the 'stats' command.
#[derive(Debug, Parser)]
struct StatsOptions {
    /// Action for which the program is attached.
    #[clap(short, long)]
    action: XdpAction,

    /// Interface name to which the program is attached.
    #[clap(short, long, default_value = "lo")]
    iface: String,

    /// Name of the 'tutorial' to search Pinned Maps in `/sys/fs/bpf`
    #[clap(short, long, default_value = "{{tutorial_name}}")]
    name: String,
}

async fn stats(opts: StatsOptions) -> anyhow::Result<()> {
    let mut stats_poller_interval = time::interval(Duration::from_secs(2));

    let map_pin_path = format!(
        "/sys/fs/bpf/{}/{}/maps/PINNED_PERCPU_ARRAY",
        opts.iface, opts.name
    );

    if !std::path::Path::new(&map_pin_path).exists() {
        return Err(anyhow::Error::msg(
                "Map PINNED_PERCPU_ARRAY is not pinned. Please run 'pin --action <action>' to pin the map."
                ));
    }


    let map_data = MapData::from_pin(map_pin_path).unwrap();
    let map = Map::PerCpuArray(map_data);
    let stats_array = map.try_into().unwrap();

    let (action_name, action_val, _) = action_info_from_opts(&opts.action);

    loop {
        tokio::select! {
            _ = stats_poller_interval.tick() => {
                log::info!("tick!");
                // TODO: Right now just using XDP_PASS, make it a proper action
                print_stats_for_action(&stats_array, action_val, action_name).await;
            }
            _ = signal::ctrl_c() => {
                log::info!("Exiting...");
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
        log::info!(
            "CPU: {}, Action: {}, Packet Count: {}",
            i,
            action_name,
            value.pkt_count
        );
        total_packets += value.pkt_count;
    }
    log::info!("Action: {}, Total  Packets: {}", action_name, total_packets);
}

// Handling of the 'unpin' command.
#[derive(Debug, Parser)]
struct UnpinOptions {
    /// Name for searching the maps and programs in the 'pin' Path
    #[clap(short, long, default_value = "{{tutorial_name}}")]
    name: String,

    /// Interface to which the programs and maps are 'pinned'.
    #[clap(short, long, default_value = "lo")]
    iface: String,

    /// Unpin only maps
    #[clap(long)]
    maps_only: bool,

    /// Unpin only programs
    #[clap(long)]
    programs_only: bool,
}

fn unpin_program_and_maps(opts: UnpinOptions) -> Result<(), anyhow::Error> {
    let pin_base_path = format!("/sys/fs/bpf/{}/{}", opts.iface, opts.name);

    if !opts.programs_only || opts.maps_only {
        // Remove the pinned maps if any
        let map_pin_path = format!("{}/maps", pin_base_path);
        std::fs::remove_dir_all(map_pin_path)?;
        log::info!("Removed ALL pinned maps for '{}'!", opts.name);
    }

    if !opts.maps_only || opts.programs_only {
        // Remove the pinned programs if any
        let program_pin_path = format!("{}/programs", pin_base_path);
        std::fs::remove_dir_all(program_pin_path)?;
        log::info!("Removed ALL pinned programs for '{}'!", opts.name);
    }

    if std::fs::read_dir(&pin_base_path)?.next().is_none() {
        std::fs::remove_dir_all(&pin_base_path)?;
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct ListOptions {
    /// List only maps
    #[clap(long)]
    maps_only: bool,

    /// List only programs
    #[clap(long)]
    programs_only: bool,

    /// Use the 'release' mode of the binary
    #[clap(long)]
    release: bool,

    /// File to 'load' to list programs and maps
    #[clap(long, default_value = "{{tutorial_name}}")]
    file: String,

    /// Interface for which Maps and programs are loaded
    #[clap(short, long, default_value = "lo")]
    iface: String,
}

fn list_programs_and_maps(opts: ListOptions) -> Result<(), anyhow::Error> {
    let pin_base_path = format!("/sys/fs/bpf/{}/{}", opts.iface, opts.file);

    if !std::path::Path::new(&pin_base_path).exists() {
        println!("No Maps or Programs currently loaded.");
        return Ok(());
    }

    if !opts.maps_only || opts.programs_only {
        println!("programs: ");
        let programs_path = format!("{}/programs", pin_base_path);
        if !std::path::Path::new(&programs_path).exists() {
            println!("\tNo Programs currently pinned.");
        } else {
            for entry in std::fs::read_dir(programs_path)? {
                println!(
                    "\t{}",
                    entry?
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow::Error::msg("Cannot convert filename to String"))?
                );
            }
        }
    }

    if !opts.programs_only || opts.maps_only {
        println!("maps: ");
        let maps_path = format!("{}/maps", pin_base_path);
        if !std::path::Path::new(&maps_path).exists() {
            println!("\tNo Maps currently pinned.");
        } else {
            for entry in std::fs::read_dir(maps_path)? {
                println!(
                    "\t{}",
                    entry?
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow::Error::msg("Cannot convert filename to String"))?
                );
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let cli = CliCommand::parse();

    match cli {
        CliCommand::Pin(opts) => pin_program_and_maps(opts),
        CliCommand::Stats(opts) => stats(opts).await,
        CliCommand::Unpin(opts) => unpin_program_and_maps(opts),
        CliCommand::List(opts) => list_programs_and_maps(opts),
    }
}
