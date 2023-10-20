use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};

use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;

use clap::Parser;
use log::{debug, info, warn};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long)]
    program: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::parse();
    env_logger::init();

    let bpf_program = format!("target/bpfel-unknown-none/debug/{}", opts.program);
    let bpf_program = std::fs::read(&bpf_program)?;
    let mut bpf = Bpf::load(&bpf_program)?;

    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }

    Ok(())
}
