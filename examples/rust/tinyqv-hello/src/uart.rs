//! UART at 0x8000080: 115200 baud at 64 MHz by default (DIVIDER register), TX on out0, RX on in7.

const DATA: usize = 0x800_0080;
const STATUS: usize = 0x800_0084; // bit0 TX busy, bit1 RX byte available
const DIVIDER: usize = 0x800_0088;

pub struct Uart;

impl Uart {
    pub fn write_byte(&mut self, b: u8) {
        while crate::rd(STATUS) & 1 != 0 {}
        crate::wr(DATA, b as u32);
    }
    /// Writes bytes, translating "\n" into "\r\n".
    pub fn write_str_raw(&mut self, s: &str) {
        for &b in s.as_bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }
    pub fn write_hex(&mut self, v: u32) {
        for i in (0..8).rev() {
            let n = ((v >> (i * 4)) & 0xf) as u8;
            self.write_byte(if n < 10 { b'0' + n } else { b'a' + n - 10 });
        }
    }
    /// Decimal without core::fmt.
    pub fn write_u32(&mut self, mut v: u32) {
        let mut buf = [0u8; 10];
        let mut i = buf.len();
        loop {
            i -= 1;
            buf[i] = b'0' + (v % 10) as u8;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        for &b in &buf[i..] {
            self.write_byte(b);
        }
    }
    pub fn byte_available(&self) -> bool {
        crate::rd(STATUS) & 2 != 0
    }
    pub fn try_read_byte(&mut self) -> Option<u8> {
        if self.byte_available() {
            Some(crate::rd(DATA) as u8)
        } else {
            None
        }
    }
    pub fn read_byte(&mut self) -> u8 {
        loop {
            if let Some(b) = self.try_read_byte() {
                return b;
            }
        }
    }
    pub fn divider(&self) -> u32 {
        crate::rd(DIVIDER)
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str_raw(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{ use core::fmt::Write; let _ = write!($crate::uart::Uart, $($arg)*); }};
}
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{ $crate::print!($($arg)*); $crate::print!("\n"); }};
}
