# Overview

The main objective of this tutorial is to develop the basic understanding of the ecosystem for XDP development. This focuses on using the [`aya`](https://github.com/aya-rs/aya) crates for XDP development.

Typically any XDP development will consist of two parts -
1. The kernel space part - The actual eBPF XDP programs that are attached to network devices and will take different actions. The kernel space API is provided by [aya-ebpf](https://docs.aya-rs.dev/aya-ebpf/) crate.
2. The user space part - This is an application part that will deal with loading, pinning etc. of the eBPF programs in the kernel space. The user space API is provided by the [aya](https://docs.aya-rs.dev/aya/) crate.

## Code Organization

The code is organized into following directories -

- `{{tutorial_name}}-ebpf` directory contains the kernel space code. Think of this program as 'data-path' for packet processing.
- `common` directory contains the code used by both kernel space and the user space code. This typically contains the data structures shared by kernel space and user space code. When we learn more about 'eBPF maps' this will be more clear. In the current tutorial this directory is empty.
- `xdp-runner` - This directory contains the user space program that will be used as driver for installing the eBPF program and running the eBPF program in the kernel code. Think of this program as the 'control path' for packet processing.

## Top Level Runner for all Tutorials

In addition, there is a `xdp-tutorial-xtask` crate provides the required tooling for running the actual tutorials. This crate is essentially a binary that provides following options
- `build-ebpf` build the eBPF binary for individual tutorials
- `run` runs the user-space program (binary in the `xdp-runner` directory above). This command will re-build the `ebpf` kernel code if required.

Thus for running any tutorial you will typically run the following command -

```shell
$ cargo xtask run {{tutorial_name}} -- [options for the binary]
```

The command output will provide help about missing options if any etc. Note the options that are to be passed to the binary are to be separated by `--` (or are to be passed as 'last' options) when passed to `cargo xtask`.

# Problem Statement

A very simple XDP program, that just logs "Packet received!" is implemented in this tutorial. Once this tutorial is 'run', A log line like following is printed for each packet received. As the goal of this tutorial is to get started with the ecosystem, we are not performing any "packet processing" in this tutorial.

# Exercises

No additional exercises.

# Notes

## Kernel Logging with `aya-log-ebpf`

One might have noticed, how can we trivially use the same logging API as in user space in the kernel spaces? The way it is actually achieved by `aya-log-ebpf` crate is using what is called as `perf` events. When we will learn more about 'eBPF maps' we will revisit this and discuss more about how the kernel level logging happens when we run the user space program.
