#![no_std]
#![no_main]

use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::{warn, info};

#[xdp]
pub fn {{to_snake_case tutorial_name}}_pass(ctx: XdpContext) -> u32 {
    match try_{{to_snake_case tutorial_name}}_pass(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_{{to_snake_case tutorial_name}}_pass(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "Received a packet.");
    Ok(xdp_action::XDP_PASS)
}

#[xdp]
pub fn {{to_snake_case tutorial_name}}_drop(ctx: XdpContext) -> u32 {
    match try_{{to_snake_case tutorial_name}}_drop(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_{{to_snake_case tutorial_name}}_drop(ctx: XdpContext) -> Result<u32, u32> {
    warn!(&ctx, "Received a packet. Dropping!");
    Ok(xdp_action::XDP_DROP)
}

// Write a program that returns `XDP_ABORTED` Error.
//
// There are following XDP Actions available
//
// ```
// xdp_action::XDP_ABORTED, xdp_action::XDP_DROP, xdp_action::XDP_PASS,
// xdp_action::XDP_REDIRECT, xdp_action::XDP_TX
// ```
// You will need to write functions similar to the above. Note: the return type should be the `Err`
// variant of the Result.
//
// ```ignore
//
// pub fn {{to_snake_case tutorial_name}}_abort(ctx: XdpContext) -> u32 {
// // Add code here
// }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

