//! TinyQV custom instruction mul16: rd = rs1 * rs2[15:0] (R-type, opcode OP, funct7 = 2).
//! RV32EC has no M extension; compiler-builtins provides a shift-and-add __mulsi3.

#[inline(always)]
pub fn mul16(a: u32, b: u32) -> u32 {
    let r: u32;
    unsafe {
        core::arch::asm!(".insn r 0x33, 0, 2, {0}, {1}, {2}", out(reg) r, in(reg) a, in(reg) b,
                         options(pure, nomem, nostack))
    };
    r
}

/// 32x32 -> 32 multiply with two mul16 instructions (same as the SDK's __mulsi3).
#[inline(always)]
pub fn mul32(a: u32, b: u32) -> u32 {
    mul16(a, b).wrapping_add(mul16(a, b >> 16) << 16)
}
