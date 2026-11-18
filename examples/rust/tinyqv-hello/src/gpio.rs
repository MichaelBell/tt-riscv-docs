//! GPIO block at 0x8000040 plus the output function select and debug select registers.

const OUT: usize = 0x800_0040;
const IN: usize = 0x800_0044;
const FUNC_SEL: usize = 0x800_0060; // + 4*n, n = 0..7
const DEBUG_SEL: usize = 0x800_000C; // bits 6-7: out6/7 normal (1) or debug (0)
const REG_DEBUG: usize = 0x800_0030;

/// Function select values for out0..out7.
pub mod func {
    pub const DISABLED: u32 = 0;
    pub const GPIO: u32 = 1;
    pub const UART: u32 = 2;
    // 3..=15: user peripheral n; 16..=31: simple peripheral n-16; 32..=39: peripheral 16..23
}

/// Disable register/debug output on out2-5 and out6-7 (like the SDK's enable_all_outputs()).
pub fn enable_all_outputs() {
    crate::wr(REG_DEBUG, 0);
    crate::wr(DEBUG_SEL, 0xff);
}
pub fn set_func(pin: usize, f: u32) {
    crate::wr(FUNC_SEL + 4 * pin, f);
}
pub fn func(pin: usize) -> u32 {
    crate::rd(FUNC_SEL + 4 * pin)
}
pub fn set_outputs(v: u8) {
    crate::wr(OUT, v as u32);
}
pub fn outputs() -> u8 {
    crate::rd(OUT) as u8
}
pub fn set_high(pin: usize) {
    set_outputs(outputs() | (1 << pin));
}
pub fn set_low(pin: usize) {
    set_outputs(outputs() & !(1 << pin));
}
pub fn toggle(pin: usize) {
    set_outputs(outputs() ^ (1 << pin));
}
pub fn inputs() -> u8 {
    crate::rd(IN) as u8
}
