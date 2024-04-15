// The following `no_std` is required for compiling for the eBPF target. That also means, care
// should be taken that the code here needs to use `core::*` definitions and not `std::*`
// definitions.
#![no_std]

/// Structure that maintains the Packet Statistics.
///
/// This structure will be shared by the Userspace and eBPF code.
pub struct StatsRecord {
    /// Number of Packets for a given `xdp_action`.
    pub pkt_count: u32,
    // TODO: Add bytes_count
}
