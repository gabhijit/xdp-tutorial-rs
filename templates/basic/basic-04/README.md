# Overview

The Goal of this tutorial is to introduce the concept of [Pinned eBPF maps](). In the previous tutorial we introduced eBPF Maps. This tutorial is a continuation of the previous tutorial.

## Problem Statement

In this tutorial, we will be using [Per CPU Arrays]() a kind of eBPF Maps. For a packet received on an interface, these are the following actions -
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
