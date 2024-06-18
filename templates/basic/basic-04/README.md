# Overview

The Goal of this tutorial is to introduce the concept of [Pinned eBPF maps](https://ebpf-docs.dylanreimerink.nl/linux/concepts/pinning/). In the previous tutorial we introduced eBPF Maps. This tutorial is a continuation of the previous tutorial. With 'pinning', it is possible for one process (or program) to load an eBPF file and pin 'maps' and 'programs' from that eBPF file and another process to read data from these pinned maps. We will explore 'pinning' in this tutorial.

# Problem Statement

Our `xdp-runner` program is now modified to support following commands -

1. `pin` - Command used to `pin` maps (and programs) in the BPF file system.
2. `stats` - Command that uses the `pin`ned maps from the file systems and displays packet processing stats corresponding to a specific action.

Continuing from [`basic-03`](../basic-03/README.md) tutorial, we make use of Per CPU arrays for storing the statistics.

# APIs

[aya-rs](https://aya-rs.dev/aya/) provides APIs for working with the Maps data structures in both the kernel space and the user space.

## User Space API

Builder type API for loading eBPF files are provided by [`EbpfLoader`](https://docs.aya-rs.dev/aya/struct.ebpfloader). This allows customization of how the eBPF files are loaded. More concretely we will customize the path in the [BPF virtual file system]() where our maps and programs are pinned using this builder API.

The BPF programs loaded and attached to the interface remain loaded in the kernel as long as the process that loaded the programs is running. These programs can be 'pinned' by using the [`Xdp.pin`](https://docs.aya-rs.dev/aya/programs/xdp/struct.xdp#method.pin) API. However, the `Drop` implementation of the `Xdp`

## Kernel Space API

We will be using [`PerCpuArray`](https://docs.aya-rs.dev/aya_ebpf/maps/array/struct.percpuarray) type of map in the kernel space code. The map is initialized as follows -

```rust
#[map]
static PINNED_PERCPU_ARRAY: PerCpuArray<StatsRecord> =
    PerCpuArray::<StatsRecord>::pinned(XDP_ACTION_MAX, 0);
```
As in the previous tutorial, the attribute `#[map]` is used for assigning the section in the eBPF binary. Note above we are using `PerCpuArray::<_>::pinned` API. This API creates a `pinned` Map. The path to which the map gets pinned in the BPF file system is determined as discussed above in the [User Space Api](#user-space-api).

The map is updated using the `get_ptr_mut` (unsafe) API. Note however that unlike the `Array` which is shared by multiple CPUs, this particular API does not require use of 'atomic' instructions as this data is guaranteed to be accessed by one CPU at any give time (`PerCpu...` prefix).

# Exercises

Implement the following commands in the tutorial runner -

1. `list` - List the currently 'pinned' maps and programs (Hint: Only read the corresponding directories)
2. `unpin` - Unpin currently 'pinned' maps and programs. Optionally take `--maps-only` or `--programs-only` CLI switches.
