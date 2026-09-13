//! Tells cargo when the embedded web UI has changed.
//!
//! `src/rest/ui.rs` compiles `apps/frontend/dist` into the binary with
//! `rust-embed`. The derive expands to one `include_bytes!` per file, so cargo
//! already rebuilds when the *content* of a file it embedded changes — but it
//! learns nothing about files being added or removed, which is exactly what a
//! Vite rebuild does (every chunk is content-hashed, so `index-A1B2.js` becomes
//! `index-C3D4.js`). Without the directives below, `pnpm build` followed by
//! `cargo build` would happily ship the previous bundle.
//!
//! This script must never fail: `apps/frontend/dist` is gitignored, so a fresh
//! checkout has no UI at all and `cargo build` still has to work.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // Keep in sync with the `#[folder]` attribute in src/rest/ui.rs.
    let dist = manifest.join("../../apps/frontend/dist");

    // A path that does not exist counts as "changed", which is what we want: the
    // script re-runs the moment `pnpm build` creates it.
    println!("cargo:rerun-if-changed={}", dist.display());
    watch_subdirs(&dist);

    if !dist.join("index.html").is_file() {
        println!(
            "cargo:warning=apps/frontend/dist/index.html not found — building a control plane \
             with no web UI. Run `just ui-build` and rebuild to embed it."
        );
    }
}

/// Cargo only watches the exact paths it is given, and a directory's mtime only
/// moves when its own entries change — not when a nested file is added. Walk the
/// tree so `dist/assets/` is watched too.
fn watch_subdirs(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            let path = entry.path();
            println!("cargo:rerun-if-changed={}", path.display());
            watch_subdirs(&path);
        }
    }
}
