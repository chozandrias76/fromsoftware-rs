//! Refuses to build once upstream's NON-AI relicensing reaches the working tree.
//!
//! Upstream (vswarte/fromsoftware-rs) relicensed to "MIT/Apache-2.0 NON-AI" in
//! 603814c (2026-10-12) by renaming `LICENSE-MIT` and `LICENSE-ASL2` to
//! `*-NON-AI`. This fork stays on MIT OR Apache-2.0 and takes upstream only up
//! to 59fbd3b. Git hooks can stop a branch from moving onto that history, but a
//! blocked merge or checkout still leaves upstream's files in the working tree,
//! and path dependencies build whatever is in the working tree.

use std::{env, path::PathBuf, process};

const NON_AI_LICENSES: [&str; 2] = ["LICENSE-MIT-NON-AI", "LICENSE-ASL2-NON-AI"];

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");

    // Upstream's rename deletes these, which is what makes Cargo rerun this
    // script when that change lands.
    println!("cargo::rerun-if-changed=../../LICENSE-MIT");
    println!("cargo::rerun-if-changed=../../LICENSE-ASL2");

    let found: Vec<_> = NON_AI_LICENSES
        .into_iter()
        .filter(|name| root.join(name).exists())
        .collect();
    if !found.is_empty() {
        eprintln!(
            "{} found in {}: upstream's NON-AI relicensing (603814c) is in the working tree. \
             Merge upstream no further than 59fbd3b, and discard the upstream changes \
             shown by `git status`.",
            found.join(", "),
            root.display(),
        );
        process::exit(1);
    }
}
