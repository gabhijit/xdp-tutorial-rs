use aya::Ebpf;
use aya_log::EbpfLogger;

use clap::Parser;
use log::warn;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long)]
    program: String,

    #[clap(short, long)]
    file: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::parse();
    env_logger::init();

    let bpf_bin = format!("target/bpfel-unknown-none/debug/{}", opts.file);
    let bpf_bin = std::fs::read(&bpf_bin)?;
    let mut bpf = Ebpf::load(&bpf_bin)?;
    let mut _bpf_prorgam = bpf.program_mut(&opts.program);

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }

    Ok(())
}
