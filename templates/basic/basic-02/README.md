# Overview

This tutorial introduces various actions that are possible to be taken on a packet during packet processing using `XDP`. In this tutorial, we attach eBPF code that performs different actions on the received packets. We will be introducing two main actions 'PASS' and 'DROP'. We further discuss another action called 'ABORT' during packet processing, which signals error in the packet processing.

# Problem Statement

Following actions are possible on a received packet during `XDP` processing -

1. `XDP_ABORTED` - Signaling error in packet processing
2. `XDP_DROP` - Drop the packet
3. `XDP_PASS` - Pass the packet to the next stage of packet processing
4. `XDP_TX` - Transmit the Packet
5. `XDP_REDIRECT` - Redirect the Packet to another interface

There are two functions implemented one for action `XDP_PASS` and one for action `XDP_DROP`. When the `{{tutorial_name}}-ebpf` is compiled, both these functions are part of the `xdp` section of the eBPF binary.

The tutorial can be run as follows -

1. To run `XDP_PASS` action on packets -

```shell
$ cargo xtask run basic-02 -- --program basic_02_pass
```

2. To run `XDP_ABORT` action on packets -

```shell
$ cargo xtask run basic-02 -- --program basic_02_drop
```

# Exercises

Additional exercise in this tutorial include to implement the `XDP_ABORTED` action (log it as `error`) and return `Err` variant of the result.

Also, add a command line switch `--list-programs` (which is mutually exclusive with `--program` CLI switch) and lists all the programs available in the `xdp` section of the binary. (Hint: Make use of the [`programs`](https://docs.aya-rs.dev/aya/struct.ebpf#method.programs) method of the `Ebpf` structure from the `aya` crate.)


# Notes

No additional notes
