use anyhow::Context;

use aya::programs::{Xdp, XdpFlags};
use aya::Ebpf;
use aya_log::EbpfLogger;

use clap::Parser;
use log::{info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long)]
    program: String,

    #[clap(short, long)]
    file: String,

    #[clap(short, long, default_value = "lo")]
    iface: String,
}

// This is a Userspace program that is responsible for 'installing' the XDP eBPF binary in the
// kernel and attaching this binary to a particular network interface.
//
// This program will be run when you are running `cargo xtask run`. The actual packet processing
// logic is implemented in the `<tutorial_name>-ebpf` package, whose generated output is passed
// as an argument to this program as `--file`. In a given file there may be more than one 'prgrams'
// in the `xdp` section, which program is to be attached is specified by the `--program` argument.
// Optionally, we can also give the interface to which the program is to be attached by specifying
// the `--iface` flag (default being `lo`).
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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
            "XDP Program '{}'attached to '{}'! Now waiting for Ctrl-C",
            &opts.program, &opts.iface
        );
        signal::ctrl_c().await?;
        info!("Exiting...");

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
