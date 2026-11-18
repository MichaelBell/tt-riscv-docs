//! no_std Rust userspace demo for the KianV RV32IMA uLinux SoC (Tiny Tapeout).
//!
//! Built as a staticlib and linked by the Buildroot uClibc gcc into a FLAT
//! (elf2flt) binary. Talks to uClibc through extern "C" and drives the
//! 7-segment display on uo_out[7:0] through the GPIO MMIO block (no MMU).
#![no_std]
#![feature(used_with_arg)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::{read_volatile, write_volatile};

extern "C" {
    fn write(fd: c_int, buf: *const c_void, n: usize) -> isize;
    fn usleep(us: c_uint) -> c_int;
    fn malloc(n: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn posix_memalign(out: *mut *mut c_void, align: usize, n: usize) -> c_int;
    fn abort() -> !;
}

// ---- FLAT loader workaround -------------------------------------------
// binfmt_flat stores the data-segment start in the 4 bytes just before
// .data. RISC-V uses no data-start gap, so that word is the last word of
// .text and gets overwritten at load time (here it was the tail jump of
// the weak memcpy). elf2flt.ld places .call_table_text last in .text, so
// this sacrificial word absorbs the write. used(linker) survives
// --gc-sections.
#[used(linker)]
#[link_section = ".call_table_text"]
static FLAT_DATA_PTR_SLOT: [u32; 1] = [0];

// ---- allocator over uClibc malloc/free --------------------------------
struct LibcAlloc;

unsafe impl GlobalAlloc for LibcAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if l.align() <= 8 {
            malloc(l.size()) as *mut u8
        } else {
            let mut p: *mut c_void = core::ptr::null_mut();
            if posix_memalign(&mut p, l.align(), l.size()) != 0 {
                return core::ptr::null_mut();
            }
            p as *mut u8
        }
    }
    unsafe fn dealloc(&self, p: *mut u8, _l: Layout) {
        free(p as *mut c_void)
    }
}

#[global_allocator]
static ALLOC: LibcAlloc = LibcAlloc;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    out("rust panic\n");
    unsafe { abort() }
}

fn out(s: &str) {
    let mut b = s.as_bytes();
    while !b.is_empty() {
        let n = unsafe { write(1, b.as_ptr() as *const c_void, b.len()) };
        if n <= 0 {
            return;
        }
        b = &b[n as usize..];
    }
}

// ---- 7-segment on uo_out via GPIO MMIO ------------------------------
const GPIO_UO_EN: *mut u32 = 0x1060_0000 as *mut u32;
const GPIO_UO_OUT: *mut u32 = 0x1060_0004 as *mut u32;
const GPIO_UI_IN: *const u32 = 0x1060_0008 as *const u32;

// bit0=A .. bit6=G, bit7=DP
const DIGITS: [u8; 10] = [0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F];
// "rust": r = e g, u = c d e, S = a c d f g, t = d e f g
const RUST: [u8; 4] = [0x50, 0x1C, 0x6D, 0x78];

fn show(seq: &[u8]) {
    for &g in seq {
        unsafe { write_volatile(GPIO_UO_OUT, g as u32) };
        unsafe { usleep(400_000) };
        unsafe { write_volatile(GPIO_UO_OUT, 0) };
        unsafe { usleep(100_000) };
    }
}

fn primes(n: usize) -> Vec<u32> {
    let mut v = Vec::new();
    let mut k = 2u32;
    while v.len() < n {
        if v.iter().take_while(|&&p| p * p <= k).all(|&p| k % p != 0) {
            v.push(k);
        }
        k += 1;
    }
    v
}

#[no_mangle]
pub extern "C" fn main(argc: c_int, _argv: *const *const c_char) -> c_int {
    out("Hello from Rust on Tiny Tapeout KianV\n");
    let p = primes(16);
    let list: Vec<String> = p.iter().map(|x| format!("{}", x)).collect();
    out(&format!("first {} primes: {}\n", p.len(), list.join(" ")));
    let sum: u32 = p.iter().sum();
    out(&format!("sum = {}, 2^31/7 = {}, argc = {}\n", sum, (1u32 << 31) / 7, argc));
    let ui = unsafe { read_volatile(GPIO_UI_IN) };
    out(&format!("ui_in = 0x{:02x}\n", ui & 0xff));
    out("7-seg: rust 0-9 (UART TX muted ~7 s)\n");
    unsafe { usleep(200_000) }; // let the UART drain before stealing uo_out[4]

    unsafe { write_volatile(GPIO_UO_EN, 0xFF) };
    show(&RUST);
    show(&DIGITS);
    unsafe { write_volatile(GPIO_UO_EN, 0) };

    out("Rust done, uo released\n");
    0
}
