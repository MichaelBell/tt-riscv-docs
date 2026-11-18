#![no_std]
#![no_main]

use core::hint::black_box;
use tinyqv::{gpio, mul, print, println, timer, uart::Uart};

const SIEVE_MAX: usize = 10_000;
static mut SIEVE: [u8; SIEVE_MAX] = [0; SIEVE_MAX]; // .bss in PSRAM
static mut COUNTER: u32 = 42; // .data, copied from flash at boot

fn sieve(n: usize) -> (u32, u32) {
    let s = unsafe { &mut *core::ptr::addr_of_mut!(SIEVE) };
    let n = n.min(SIEVE_MAX);
    s[..n].fill(1);
    let (mut count, mut last) = (0, 0);
    for i in 2..n {
        if s[i] != 0 {
            count += 1;
            last = i as u32;
            let mut j = i + i;
            while j < n {
                s[j] = 0;
                j += i;
            }
        }
    }
    (count, last)
}

fn bench_mul() {
    let t0 = timer::cycle();
    let mut acc = 0u32;
    for i in 0..1000u32 {
        acc = acc.wrapping_add(black_box(i).wrapping_mul(black_box(i)));
    }
    let t1 = timer::cycle();
    let mut acc2 = 0u32;
    for i in 0..1000u32 {
        acc2 = acc2.wrapping_add(mul::mul32(black_box(i), black_box(i)));
    }
    let t2 = timer::cycle();
    println!("sum i*i, i<1000: soft __mulsi3 {} ({} cycles), mul16 {} ({} cycles)",
             acc, t1.wrapping_sub(t0), acc2, t2.wrapping_sub(t1));
}

fn show_pins() {
    print!("func_sel out0..7 =");
    for p in 0..8 {
        print!(" {}", gpio::func(p));
    }
    println!(", OUT=0x{:02x}, IN=0x{:02x}", gpio::outputs(), gpio::inputs());
}

fn read_line(u: &mut Uart, buf: &mut [u8]) -> usize {
    let mut n = 0;
    loop {
        let b = u.read_byte();
        match b {
            b'\r' | b'\n' => {
                u.write_str_raw("\n");
                return n;
            }
            8 | 127 if n > 0 => {
                n -= 1;
                u.write_str_raw("\x08 \x08");
            }
            32..=126 if n < buf.len() => {
                buf[n] = b;
                n += 1;
                u.write_byte(b);
            }
            _ => {}
        }
    }
}

fn arg(it: &mut core::str::SplitWhitespace) -> Option<u32> {
    let s = it.next()?;
    if let Some(h) = s.strip_prefix("0x") {
        u32::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut u = Uart;
    println!("\n\nHello from Rust on TinyQV! (instance '{}', uart div {})",
             tinyqv::instance_id() as u8 as char, u.divider());
    let c = unsafe { core::ptr::addr_of_mut!(COUNTER).read_volatile() };
    println!(".data check: COUNTER={} (expect 42)", c);

    // time / cycle counters
    let (c0, m0) = (timer::cycle(), timer::mtime());
    timer::delay_us(10_000);
    let (c1, m1) = (timer::cycle(), timer::mtime());
    println!("delay_us(10000): {} cycles, mtime +{} us -> {} MHz",
             c1.wrapping_sub(c0), m1.wrapping_sub(m0), c1.wrapping_sub(c0) / m1.wrapping_sub(m0).max(1));

    // computation
    let t0 = timer::mtime();
    let (count, last) = sieve(SIEVE_MAX);
    let dt = timer::mtime().wrapping_sub(t0);
    println!("sieve: {} primes below {}, largest {}, {} us", count, SIEVE_MAX, last, dt);
    bench_mul();
    print!("no-fmt u32: ");
    u.write_u32(4_294_967_295);
    print!(" hex: 0x");
    u.write_hex(0xC0FFEE);
    println!();

    // GPIO: out0 stays UART TX; drive out1..out4 from the GPIO peripheral
    show_pins();
    gpio::enable_all_outputs();
    for p in 1..=4 {
        gpio::set_func(p, gpio::func::GPIO);
    }
    let mut pattern = 0u8;
    for step in 0..8u8 {
        pattern = (pattern & 1) | ((step & 0xf) << 1); // never touches bit0
        gpio::set_outputs(pattern);
        timer::delay_us(50_000);
        print!("{:02x} ", gpio::outputs());
    }
    println!();
    gpio::toggle(4);
    println!("toggle(4) -> OUT=0x{:02x}", gpio::outputs());
    show_pins();

    println!("Commands: help, in, out <v>, primes <n>, mul <a> <b>, time, bench");
    let mut buf = [0u8; 64];
    loop {
        print!("tqv> ");
        let n = read_line(&mut u, &mut buf);
        let line = core::str::from_utf8(&buf[..n]).unwrap_or("");
        let mut it = line.split_whitespace();
        match it.next() {
            None => {}
            Some("help") => println!("in | out <v> | primes <n<=10000> | mul <a> <b> | time | bench"),
            Some("in") => println!("IN=0x{:02x}", gpio::inputs()),
            Some("out") => match arg(&mut it) {
                Some(v) => {
                    gpio::set_outputs(v as u8);
                    println!("OUT=0x{:02x}", gpio::outputs());
                }
                None => println!("usage: out <v>"),
            },
            Some("primes") => {
                let n = arg(&mut it).unwrap_or(100) as usize;
                let t0 = timer::mtime();
                let (c, l) = sieve(n);
                println!("{} primes below {}, largest {} ({} us)", c, n.min(SIEVE_MAX), l,
                         timer::mtime().wrapping_sub(t0));
            }
            Some("mul") => match (arg(&mut it), arg(&mut it)) {
                (Some(a), Some(b)) => println!("{} * {} = {} (mul16: {})", a, b,
                                               a.wrapping_mul(b), mul::mul32(a, b)),
                _ => println!("usage: mul <a> <b>"),
            },
            Some("time") => println!("mtime={} us cycle={} instret={}", timer::mtime(),
                                     timer::cycle(), timer::instret()),
            Some("bench") => bench_mul(),
            Some(cmd) => println!("unknown command '{}'", cmd),
        }
    }
}
