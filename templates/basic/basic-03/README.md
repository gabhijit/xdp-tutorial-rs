# Overview

This tutorial introduces the concept of eBPF [Maps](https://docs.cilium.io/en/stable/bpf/architecture/#maps). eBPF Maps can be used to share data between the user-space and the kernel space.

Although they are called as Maps, they are essentially different data structures that may be suitable for different use-cases. The link above provides the details of these data structures.

The main learning objective of this tutorial is to understand how to work with the eBPF Maps using the kernel space and user space APIs provided by [aya-rs](https://aya-rs.dev/aya/). This tutorial would also explore the concept of atomic operations and how they are to be used (while updating the maps) and provide a motivation for using `PerCpuyArray` and use these APIs.

## Problem Statement

In this tutorial, we will be using [Arrays]() a kind of eBPF Maps. For a packet received on an interface, these are the following actions -
1. `XDP_ABORTED` - Signaling error in packet processing
2. `XDP_DROP` - Drop the packet
1. `XDP_PASS` - Pass the packet to the next stage of packet processing
3. `XDP_TX` - Transmit the Packet
4. `XDP_REDIRECT` - Redirect the Packet to another interface

In this tutorial, we are maintaining a simple statistics about the packets and the total bytes that are handled by individual action in the `XDP` maps of type `Array`. As seen in the previous tutorials, whenever an XDP program is attached to a specific interface, that program is invoked for every received packet on the interface and a specific action will be taken for each packet. Currently, we are just recording the packets received and take the `XDP_PASS` action. For each received packet, the `Array`(`Map`) created is updated.

The corresponding Userspace program will read this `Map` and log the received packet count.

## Exercises

In addition to the existing programs, additional exercises are provided to work with -

1. Per CPU Arrays
2. Using the data from the `Context` to count the number of bytes.

Thus this exercise should serve as a good starting point for real `XDP` program.

# APIs

[aya-rs](https://aya-rs.dev/aya/) provides APIs for working with the Maps data structures in both the kernel space and the user space.

## Kernel Space API

The kernel space APIs are provided by [aya-ebpf]() crate. We'll look at a simple API for working with an Array.
