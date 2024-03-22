// For building the eBPF (XDP) program.
// Taken from `aya-template` and modified for our needs.
//
use std::{path::PathBuf, process::Command};

use clap::Parser;

use crate::common::Architecture;

#[derive(Debug, Parser)]
pub struct Options {
    /// Name of the eBPF target to build
    #[clap(short, long)]
    pub name: String,

    /// Set the endianness of the BPF target
    #[clap(default_value = "bpfel-unknown-none", long)]
    pub target: Architecture,

    /// Build the release target
    #[clap(long)]
    pub release: bool,
}

pub fn build_ebpf(opts: Options) -> Result<(), anyhow::Error> {
    let binary = format!("{}-ebpf", opts.name);
    let dir = PathBuf::from(&binary);
    let target = format!("--target={}", opts.target);
    let mut args = vec!["build", target.as_str(), "-Z", "build-std=core"];
    if opts.release {
        args.push("--release")
    }

    // Command::new creates a child process which inherits all env variables. This means env
    // vars set by the cargo xtask command are also inherited. RUSTUP_TOOLCHAIN is removed
    // so the rust-toolchain.toml file in the -ebpf folder is honored.

    let status = Command::new("cargo")
        .current_dir(dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(&args)
        .status()
        .expect("failed to build bpf program");
    assert!(status.success());
    Ok(())
}
