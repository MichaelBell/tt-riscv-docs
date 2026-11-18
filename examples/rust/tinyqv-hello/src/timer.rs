//! Time: MTIME at 0xFFFFF00 (1 MHz at 64 MHz clock) and the cycle/time/instret CSRs.

const MTIME: usize = 0xFFF_FF00;

pub fn mtime() -> u32 {
    crate::rd(MTIME)
}
pub fn cycle() -> u32 {
    let r: u32;
    unsafe { core::arch::asm!("rdcycle {0}", out(reg) r) };
    r
}
pub fn time() -> u32 {
    let r: u32;
    unsafe { core::arch::asm!("rdtime {0}", out(reg) r) };
    r
}
pub fn instret() -> u32 {
    let r: u32;
    unsafe { core::arch::asm!("rdinstret {0}", out(reg) r) };
    r
}
pub fn delay_us(us: u32) {
    let end = time().wrapping_add(us);
    while (end.wrapping_sub(time()) as i32) > 0 {}
}
