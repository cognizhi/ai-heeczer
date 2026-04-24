//! Build script: generate C headers via cbindgen (if cbindgen is installed).
//!
//! cbindgen is an optional dev tool. If not installed, the build succeeds but
//! `heeczer_core_c.h` is NOT regenerated. CI should run `make cbindgen` to
//! update the header before a release.

fn main() {
    // Trigger rebuild only when the C ABI surface changes.
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
