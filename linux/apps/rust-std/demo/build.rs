// The pthread layer and the linker wrapper are linked in by kianv-flat-cc,
// outside cargo's view: relink when they change.
fn main() {
    println!("cargo:rerun-if-changed=../kianv_pthread.c");
    println!("cargo:rerun-if-changed=../kianv-flat-cc");
}
