**(Note: Not Ready Yet! Do not use!)**

# Overview

Code for [xdp-project's xdp-tutorial](https://github.com/xdp-project/xdp-tutorial/) in Rust.

This project implements all the examples and assignments of the XDP Tutorial in Rust. The intended audience for this tutorial is - developers looking to get started with Rust and XDP. This tutorial also uses parts from the [aya-rs book](https://aya-rs.dev/book/), but for the sake of completion, will describe those parts here fully.


# Setting Up

Before starting the tutorial, one needs to install the development tools.

## New to Rust?

If you are new to rust, it's better to get started with installing Rust following the instructions from [the official website](https://www.rust-lang.org/learn/get-started). This should install `rustup` a tool for managing the Rust toolchains.


## New to `eBPF` and `XDP`?

If you are totally new to `eBPF` you might want to go quickly through [getting started guide](https://ebpf.io/get-started/) on the eBPF website.

Quickly going through the [XDP academic paper](https://github.com/xdp-project/xdp-paper/blob/master/xdp-the-express-data-path.pdf) might also be a good idea for most of the users, unless you are an XDP developer already.

You might want to quickly go through the [introduction](https://github.com/xdp-project/xdp-tutorial/#introduction) from the original tutorial's website.


## Common Instructions

First let's get started by installing the required dependencies.

1. Install the [BPF Linker](https://github.com/aya-rs/bpf-linker). This is required for generating the binaries for the eBPF programs. This will work with LLVM provided by `rustc`.

```bash
cargo install ebpf-linker
```

2. Optional but recommended to install `llvm-objdump`. This tool is required for inspecting the generated object files for the BPF programs.

```bash
cargo install llvm
```

# Starting the tutorial

Once the required dependencies are setup properly, we can start running the tutorial. This is done in two parts. First part would install `cargo-xdp-tutorial` command from the repository of the root and then in the second part, we will use the command `cargo-xdp-tutorial` to actually get started with the tutorial. This is described below.

## Installing `cargo-xdp-tutorial`

```bash
# This will install the latest `cargo-xdp-tutorial` binary from the current directory (crate).
$ cargo install --path . --force

# Verify that `xdp-tutorial` sub command is installed.
$ cargo xdp-tutorial help
```

## Running your first tutorial.

We create a very simple tutorial that prints a simple log statement when a packet is received on an interface.
```bash

# Following command will copy 'templates' from the templates/initial directory and creating a basic
# scaffolding to which individual tutorials can be added subsequently.
$ cargo xdp-tutorial init templates/initial tutorial

# Add the 'basic-01' to the tutorial created above.
# Follow the instructions when prompted.
$ cargo xdp-tutorial add templates/basic/basic-01 tutorial

```
