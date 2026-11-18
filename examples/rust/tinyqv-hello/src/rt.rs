//! Startup code. TinyQV has fixed vectors: reset at 0x0, trap (exceptions) at 0x4,
//! interrupts at 0x8. Interrupts are kept disabled by this runtime.

core::arch::global_asm!(
    r#"
    .section .boot, "ax"
    .globl _start
    .option push
    .option norvc
_start:
    j _tqv_reset            # 0x0 reset
    j _tqv_trap_entry       # 0x4 exception
    .option pop
    csrw mie, zero          # 0x8 interrupt: none expected, mask everything and return
    mret

_tqv_reset:
    .option push
    .option norelax
    li gp, 0x1000400        # hardwired on TinyQV, set anyway for other cores/sims
    .option pop
    li tp, 0x8000000        # peripheral base, also hardwired
    la sp, __stack_top
    csrw mie, zero
    csrci mstatus, 8

    # zero .bss
    la a0, __sbss
    la a1, __ebss
1:  bgeu a0, a1, 2f
    sw zero, 0(a0)
    addi a0, a0, 4
    j 1b
2:
    # copy .data from flash to PSRAM
    la a0, __sdata
    la a1, __edata
    la a2, __sidata
3:  bgeu a0, a1, 4f
    lw a3, 0(a2)
    sw a3, 0(a0)
    addi a0, a0, 4
    addi a2, a2, 4
    j 3b
4:
    call main
5:  j 5b

_tqv_trap_entry:
    la sp, __stack_top
    csrr a0, mcause
    csrr a1, mepc
    call _tqv_trap
6:  j 6b
"#
);

#[no_mangle]
extern "C" fn _tqv_trap(mcause: u32, mepc: u32) -> ! {
    let mut u = crate::uart::Uart;
    u.write_str_raw("\r\n*** TRAP mcause=0x");
    u.write_hex(mcause);
    u.write_str_raw(" mepc=0x");
    u.write_hex(mepc);
    u.write_str_raw("\r\n");
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;
    let _ = write!(crate::uart::Uart, "\r\n*** PANIC: {}\r\n", info);
    loop {}
}
