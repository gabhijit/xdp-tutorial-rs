// For running the eBPF program
// Taken from `aya-template` and modified for our needs.
//
use std::process::Command;

use anyhow::Context as _;
use clap::Parser;

use crate::build_ebpf::{build_ebpf, Options as BuildOptions};
use crate::common::Architecture;

#[derive(Debug, Parser)]
pub struct Options {
    /// Set the name of the Userspace binary to run
    #[clap(name = "tutorial-name")]
    tutorial_name: String,

    /// Set the name of the program to run.
    #[clap(short, long)]
    program: String,

    /// Set the endianness of the BPF target
    #[clap(default_value = "bpfel-unknown-none", long)]
    pub bpf_target: Architecture,

    /// Build and run the release target
    #[clap(long)]
    pub release: bool,

    /// The command used to wrap your application
    #[clap(short, long, default_value = "sudo -E")]
    pub runner: String,

    /// Interface to attach the program to
    #[clap(short, name = "iface", default_value = "lo")]
    pub iface: String,

    /// Arguments to pass to your application
    #[clap(name = "args", last = true)]
    pub run_args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct RunOptions {
    /// Name of the tutorial to call the runner
    #[clap(name = "tutorial-name")]
    tutorial_name: String,

    /// Set the endianness of the BPF target
    #[clap(default_value = "bpfel-unknown-none", long)]
    pub target: Architecture,

    /// Build the release target
    #[clap(long)]
    pub release: bool,

    /// Log Verbosity Level (default: info, '-v': debug, '-vv...': trace)
    #[clap(short, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Arguments to be passed to the runner
    #[clap(name = "run-args", last = true)]
    run_args: Vec<String>,
}

/// Build the project
fn build(opts: &RunOptions) -> Result<(), anyhow::Error> {
    let mut args = vec!["build"];
    if opts.release {
        args.push("--release")
    }
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to build userspace");
    assert!(status.success());
    Ok(())
}

/// Build and run the project
pub fn run(opts: RunOptions) -> Result<(), anyhow::Error> {
    // build our ebpf program followed by our application
    build_ebpf(BuildOptions {
        name: opts.tutorial_name.clone(),
        target: opts.target,
        release: opts.release,
    })
    .context("Error while building eBPF program")?;

    // Build our 'xdp-loader' application
    build(&opts).context("Error while building userspace application")?;

    // profile we are building (release or debug)
    let profile = if opts.release { "release" } else { "debug" };

    // Obtain Path to the binary we will be running
    let bin_path = format!("target/{profile}/{0}-runner", &opts.tutorial_name);

    // arguments to pass to the application
    let mut run_args = opts.run_args.clone();

    // configure args
    let mut args: Vec<_> = vec!["sudo".to_string(), "-E".to_string()];
    args.push(bin_path);
    args.append(&mut run_args);

    let loglevel = match opts.verbosity {
        0 => "info",
        1 => "debug",
        2.. => "trace",
    };

    eprintln!("args: {}, loglevel: {}", args.join(" "), loglevel);
    // run the command
    let status = Command::new(args.first().expect("No first argument"))
        .env("RUST_LOG", loglevel)
        .args(args.iter().skip(1))
        .status()
        .expect("failed to run the command");

    if !status.success() {
        anyhow::bail!("Failed to run `{}`", args.join(" "));
    }
    Ok(())
}
