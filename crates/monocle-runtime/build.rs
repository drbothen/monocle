//! Build script for monocle-runtime.
//!
//! Creates a symlink `<OUT_DIR>/../../deps/monocle-session-host` pointing to the
//! built `monocle-session-host` binary so that the Ruling-A integration test can
//! find it via `std::env::current_exe().parent().join("monocle-session-host")`.
//!
//! The test binary runs from `target/<profile>/deps/`, and the `monocle-session-host`
//! binary is produced at `target/<profile>/monocle-session-host`. This build script
//! bridges the two locations by creating a symlink in the `deps/` directory.

use std::path::Path;

fn main() {
    // OUT_DIR is set by cargo to something like:
    //   target/debug/build/monocle-runtime-<hash>/out
    // Walk up to find the profile directory (target/debug or target/release).
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cargo:warning=monocle-runtime build.rs: OUT_DIR not set: {e}");
            return;
        }
    };
    let out_path = Path::new(&out_dir);

    // OUT_DIR layout: <target>/<profile>/build/<crate>-<hash>/out
    // parent() x4 gives <target>/<profile>/
    let profile_dir = out_path
        .parent() // out → <hash>
        .and_then(|p| p.parent()) // <hash> → <crate>-<hash>
        .and_then(|p| p.parent()) // <crate>-<hash> → build
        .and_then(|p| p.parent()); // build → <profile>

    let profile_dir = match profile_dir {
        Some(d) => d,
        None => {
            eprintln!("cargo:warning=monocle-runtime build.rs: could not resolve profile dir from OUT_DIR={out_dir}");
            return;
        }
    };

    let binary = profile_dir.join("monocle-session-host");
    let symlink_target = profile_dir.join("deps").join("monocle-session-host");

    // Only create symlink if the target binary exists and the symlink doesn't.
    if binary.exists() && !symlink_target.exists() {
        if let Err(e) = std::os::unix::fs::symlink(&binary, &symlink_target) {
            eprintln!(
                "cargo:warning=monocle-runtime build.rs: could not create symlink {:?} → {:?}: {e}",
                symlink_target, binary
            );
        }
    } else if binary.exists() && symlink_target.is_symlink() {
        // Symlink already exists — check if it points to the right place.
        // If it does, nothing to do. If not, recreate it.
        match std::fs::read_link(&symlink_target) {
            Ok(target) if target == binary => {} // already correct
            Ok(_) | Err(_) => {
                let _ = std::fs::remove_file(&symlink_target);
                let _ = std::os::unix::fs::symlink(&binary, &symlink_target);
            }
        }
    }

    // Rerun if the binary changes.
    println!("cargo:rerun-if-changed=../monocle-session-host/src/main.rs");
    println!("cargo:rerun-if-changed=../monocle-session-host/Cargo.toml");
}
