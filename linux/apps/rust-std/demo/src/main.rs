use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, process, thread};

fn section(name: &str, r: Result<String, String>) {
    match r {
        Ok(s) => println!("[ok]  {name}: {s}"),
        Err(e) => println!("[err] {name}: {e}"),
    }
}

fn collections() -> Result<String, String> {
    let mut v: Vec<u32> = (1..=20).rev().collect();
    v.sort();
    let s: String = v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
    let mut h = HashMap::new();
    for w in "the quick brown fox jumps over the lazy dog the end".split(' ') {
        *h.entry(w).or_insert(0u32) += 1;
    }
    let b: BTreeMap<_, _> = h.iter().collect();
    Ok(format!("sum={} first={} the={} keys={} pi~{:.4}", v.iter().sum::<u32>(), &s[..8], h["the"], b.len(), std::f64::consts::PI))
}

fn time() -> Result<String, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    thread::sleep(Duration::from_millis(250));
    let slept = t0.elapsed();
    let t1 = Instant::now();
    let mut n = 0u32;
    for i in 2..5000u32 {
        if (2..i).take_while(|d| d * d <= i).all(|d| i % d != 0) {
            n += 1;
        }
    }
    Ok(format!("unix={}s sleep(250ms)={:?} primes<5000={} in {:?}", now.as_secs(), slept, n, t1.elapsed()))
}

fn files() -> Result<String, String> {
    let e = |e: std::io::Error| e.to_string();
    fs::create_dir_all("/tmp/rust_std").map_err(e)?;
    let p = "/tmp/rust_std/hello.txt";
    fs::write(p, "line 1\n").map_err(e)?;
    let mut f = fs::OpenOptions::new().append(true).open(p).map_err(e)?;
    writeln!(f, "line 2").map_err(e)?;
    drop(f);
    let txt = fs::read_to_string(p).map_err(e)?;
    let md = fs::metadata(p).map_err(e)?;
    let mut names: Vec<String> = fs::read_dir("/root").map_err(e)?.filter_map(|d| d.ok()).map(|d| d.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    fs::rename(p, "/tmp/rust_std/renamed.txt").map_err(e)?;
    fs::remove_file("/tmp/rust_std/renamed.txt").map_err(e)?;
    fs::remove_dir("/tmp/rust_std").map_err(e)?;
    let issue = fs::read_to_string("/etc/issue").unwrap_or_default();
    Ok(format!("lines={} len={} /root={:?} issue={:?}", txt.lines().count(), md.len(), names, issue.trim()))
}

fn threads() -> Result<String, String> {
    let counter = Arc::new(Mutex::new(0u32));
    let (tx, rx) = mpsc::channel();
    let mut hs = Vec::new();
    for id in 0..3u32 {
        let c = Arc::clone(&counter);
        let tx = tx.clone();
        let h = thread::Builder::new()
            .name(format!("w{id}"))
            .stack_size(32 * 1024)
            .spawn(move || {
                let mut local = 0u32;
                for i in 0..1000 {
                    local += i;
                    *c.lock().unwrap() += 1;
                }
                tx.send((id, local)).unwrap();
                id * 10
            })
            .map_err(|e| e.to_string())?;
        hs.push(h);
    }
    drop(tx);
    let rets: Vec<u32> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    let mut got: Vec<(u32, u32)> = rx.iter().collect();
    got.sort();
    Ok(format!("joined={:?} counter={} msgs={:?}", rets, *counter.lock().unwrap(), got))
}

fn command() -> Result<String, String> {
    let out = process::Command::new("/bin/echo").arg("hi from child").output().map_err(|e| e.to_string())?;
    Ok(format!("status={} stdout={:?}", out.status, String::from_utf8_lossy(&out.stdout).trim()))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("rust_std_demo pid={} args={:?} PATH={:?}", process::id(), args, env::var("PATH").ok());
    let all = args.len() < 2;
    let want = |s: &str| all || args.iter().any(|a| a == s);
    if want("coll") { section("collections", collections()); }
    if want("time") { section("time", time()); }
    if want("fs") { section("fs", files()); }
    if want("thread") { section("thread", threads()); }
    if want("cmd") { section("command", command()); }
    println!("rust_std_demo done");
}
