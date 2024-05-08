# Overview

This tutorial introduces the concept of eBPF [Maps](https://docs.cilium.io/en/stable/bpf/architecture/#maps). eBPF Maps can be used to share data between the user-space and the kernel space.

Although they are called as Maps, they are essentially different data structures that may be suitable for different use-cases. The link above provides the details of these data structures.

The main learning objective of this tutorial is to understand how to work with the eBPF Maps using the kernel space and user space APIs provided by [aya-rs](https://aya-rs.dev/aya/). The tutorial would also explore the concept of atomic operations and how they are to be used (while updating the maps) and provide a motivation for using `PerCpuyArray` and use these APIs.

## Problem Statement

Initially, we will be using [Arrays](https://docs.aya-rs.dev/aya_ebpf/maps/array/struct.array) a kind of eBPF maps, to store data specific to store statistics specific to a particular packet received on an interface. For a packet received on an interface, these are the following actions -
1. `XDP_ABORTED` - Signaling error in packet processing
2. `XDP_DROP` - Drop the packet
3. `XDP_PASS` - Pass the packet to the next stage of packet processing
4. `XDP_TX` - Transmit the Packet
5. `XDP_REDIRECT` - Redirect the Packet to another interface

In this tutorial, we are maintaining a simple statistics about the total packets and the total bytes that are handled by individual action in the `XDP` maps of type `Array`. As seen in the previous tutorials, whenever an XDP program is attached to a specific interface, that program is invoked for every received packet on the interface and a specific action will be taken for each packet. Currently, we are just recording the packets received and take the `XDP_PASS` action. For each received packet, the `Array`(`Map`) created is updated.

The corresponding user space program will read this `Map` and log the received packet count on the console.

## Exercises

In addition to the existing programs, additional exercises are provided to work with -

1. Per CPU Arrays
2. Using the data from the `Context` to count the number of bytes.

Thus this exercise should serve as a good starting point for real `XDP` program.

# APIs

[aya-rs](https://aya-rs.dev/aya/) provides APIs for working with the Maps data structures in both the kernel space and the user space.

## Kernel Space API

The kernel space APIs are provided by [aya-ebpf](https://docs.aya-rs.dev/aya_ebpf/) crate. We'll look at a simple API for working with an Array.

An `Array` is a generic structure that can be created using one of the two APIs `with_max_entries` or `pinned`. For Recording the actual statistics we will be using a structure called `StatsRecord`. The structure contains entries for packet count and byte counts (this will be exercise). The eBPF Array then will be instantiated as follows. Since this is a Global array, we will be instantiating a 'static' array.


```rust
#[map]
static STATS_ARRAY: Array<StatsRecord> = Array::<StatsRecord>::with_max_entries(XDP_ACTION_MAX, 0);
```

The 'attribute' `map` above specifies the section in the generated eBPF binary. This stores the data in the `maps` section of the binary. This can be verified as follows

```shell

$ llvm-objdump --section-headers --section=maps target/bpfel-unknown-none/debug/basic-03

target/bpfel-unknown-none/debug/basic-03:	file format elf64-bpf

Sections:
Idx Name          Size     VMA              Type
  5 maps          00000054 0000000000000000 DATA
```

For updating the `STATS_ARRAY` above we will be making use of the `get_ptr_mut` API of the `Array`. Note: This API returns an Optional 'raw pointer', and we will be de-referencing this pointer to update the statistics. This action should be done in an `unsafe` block.


### A Note about atomic operations

A received packet can be processed on any of the CPUs and since this `Array` is shared across all the CPUs, we need to make sure that whenever the `Array` is updated, it should be done using 'atomic' instructions. In the kernel currently the `AtomicU*.fetch_add` instructions don't work, instead one should use the `intrinsics` versions as described [here](https://rust.docs.kernel.org/core/intrinsics/index.html).


## User space API

[Aya user space](https://docs.aya-rs.dev/aya/) provides User space APIs for working with the kernel space maps. This can be used as follows -

```rust
stats_array = Array::try_from(bpf.map("STATS_ARRAY").unwrap()).unwrap();
```
And then `get` API on the returned Array can be used to read the value specific to particular key (the XDP action is the key in this case).


## Common Code

The `StatsRecord` structure is used by both the kernel space API and the user space API. This structure is provided by a `common` package used by both kernel space and user space API. One more additional detail is - the trait called [`Pod`](https://docs.aya-rs.dev/aya/trait.pod) is required to be derived for our structure [`StatsRecord`] for reading the data from the kernel space.
