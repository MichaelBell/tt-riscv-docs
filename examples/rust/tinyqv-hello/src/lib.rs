//! Minimal runtime and peripheral access for TinyQV (ttsky25a, RV32EC + Zcb + Zicond).
//! Register map: ttsky25a-tinyQV/docs/info.md. Startup mirrors tinyQV-sdk start.s / runtime.c.
#![no_std]

pub mod rt;
pub mod uart;
pub mod gpio;
pub mod timer;
pub mod mul;

#[inline(always)]
pub(crate) fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
#[inline(always)]
pub(crate) fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) }
}

/// DEBUG block: returns the TinyQV instance ID (0x41 'A' on the "Asteroids" build).
pub fn instance_id() -> u32 {
    rd(0x800_0008)
}
