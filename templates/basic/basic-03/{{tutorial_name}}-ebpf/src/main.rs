#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::intrinsics::atomic_xadd_acquire;

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::Array,
    programs::XdpContext,
};
use aya_log_ebpf::{info, error};

use {{ to_snake_case tutorial_name }}_common::StatsRecord;


const XDP_ACTION_MAX: u32 = xdp_action::XDP_REDIRECT + 1;

#[map]
static STATS_ARRAY: Array<StatsRecord> = Array::<StatsRecord>::with_max_entries(XDP_ACTION_MAX, 0);

#[xdp]
pub fn {{to_snake_case tutorial_name}}_packet_stats(ctx: XdpContext) -> u32 {
    match try_{{to_snake_case tutorial_name}}_packet_stats(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_{{to_snake_case tutorial_name}}_packet_stats(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "Received a packet.");
    let record = STATS_ARRAY.get_ptr_mut(xdp_action::XDP_PASS);
    if let Some(record) = record {
        let _ = unsafe {
            atomic_xadd_acquire(&mut (*record).pkt_count, 1);
        };
        Ok(xdp_action::XDP_PASS)
    } else {
        error!(&ctx, "Entry for the action not found in the map!");
        Err(xdp_action::XDP_ABORTED)
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
