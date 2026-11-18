# Experimental: full Rust `std` on the KianV Linux

**Status: works on ttsky25a silicon, but needs a pinned nightly, local patches and a C shim.** Prefer
[`no_std`](../rust-kianv-linux.md) unless you need `std`.

The [KianV Linux image](../kianv-linux.md) includes `/root/rust_std_hello` and `/root/rust_std_demo` (source: [`linux/apps/rust-std`](../../linux/apps/rust-std)):

```
/ # /root/rust_std_demo
[ok]  collections: sum=210 first=1,2,3,4, the=3 keys=9 pi~3.1416
[ok]  time: unix=149s sleep(250ms)=294.797052ms primes<5000=669 in 1.357686423s
[ok]  fs: lines=2 len=14 /root=["hello_7seg",...,"rust_std_hello"] issue="Welcome to KianV ASIC Buildroot"
[ok]  thread: joined=[0, 10, 20] counter=3000 msgs=[(0, 499500), (1, 499500), (2, 499500)]
[err] command: Function not implemented (os error 38)
```

| Works                                                                   | Does not work                                                |
| ----------------------------------------------------------------------- | ------------------------------------------------------------ |
| `println!`, collections, `env`                                          | `std::process::Command`: there is no `fork()` without an MMU |
| `Instant`, `SystemTime` (time since boot: no RTC), `thread::sleep`      |                                                              |
| `fs` (create, write, read, `read_dir`, rename, remove)                  |                                                              |
| `thread` spawn/join, `Arc<Mutex>`, `mpsc` (max 7 threads, small stacks) |                                                              |

## How

All of it is in [`linux/apps/rust-std`](../../linux/apps/rust-std):

- **Custom target** `riscv32ima-unknown-linux-uclibc.json`: env `uclibc`, `+m,+a,-c,-f,-d`, ABI `ilp32`,
  no thread-local storage, static PIE, `panic=abort`.
- **Build:** `cargo +nightly-2026-11-17 build -Zbuild-std=std,panic_abort -Zjson-target-spec`.
- **Patches:**
  - `libc` 0.2.191: a new `uclibc/riscv32` module.
  - `std`: the time32 futex, and no guard-page stack-overflow handler.
- **`kianv_pthread.c`:** the image's uClibc has no pthreads. This shim adds threads via `clone`, keys,
  a lock around `malloc`, and a `fork()` that returns ENOSYS.
- **`kianv-flat-cc`:** a linker wrapper that produces FLAT binaries. `flthdr -s` raises the stack.
- **Size:** binaries are 280-420 KB. Compute is slow: primes below 5000 take about 1.4 s.

Upstreaming this would need a `libc` PR, small `std` changes and a tier-3 Rust target.
