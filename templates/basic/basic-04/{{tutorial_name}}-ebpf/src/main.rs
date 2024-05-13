#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::PerCpuArray,
    programs::XdpContext,
};
use aya_log_ebpf::{debug, error};

use {{ to_snake_case tutorial_name }}_common::StatsRecord;

const XDP_ACTION_MAX: u32 = xdp_action::XDP_REDIRECT + 1;

#[map]
static PINNED_PERCPU_ARRAY: PerCpuArray<StatsRecord> =
    PerCpuArray::<StatsRecord>::pinned(XDP_ACTION_MAX, 0);

#[xdp]
pub fn {{ to_snake_case tutorial_name }}_action_pass(ctx: XdpContext) -> u32 {
    match try_{{ to_snake_case tutorial_name }}_packet_stats(ctx, xdp_action::XDP_PASS) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[xdp]
pub fn {{ to_snake_case tutorial_name }}_action_drop(ctx: XdpContext) -> u32 {
    match try_{{ to_snake_case tutorial_name }}_packet_stats(ctx, xdp_action::XDP_DROP) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)]
fn try_{{ to_snake_case tutorial_name }}_packet_stats(ctx: XdpContext, action: u32) -> Result<u32, u32> {
    debug!(&ctx, "Received a packet.");
    let record = PINNED_PERCPU_ARRAY.get_ptr_mut(action);
    if let Some(record) = record {
        let _ = unsafe { (*record).pkt_count += 1 };
        Ok(action)
    } else {
        error!(&ctx, "Entry for the action not found in the map!");
        Err(xdp_action::XDP_ABORTED)
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
