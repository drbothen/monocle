//! Integration tests for BC-2.07.006: CCR Detection via `ccr_path` config field.
//!
//! All tests use filesystem fixtures via `tempfile::TempDir`. PATH manipulation
//! (for `which::which("ccr")` fallback) is done via `std::env::set_var` with
//! care to avoid test pollution.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause |
//! |---------------|----|-----------|
//! | test_BC_2_07_006_ccr_path_none_ccr_not_on_path_returns_none | AC-008 | BC-2.07.006 PC-4, EC-115 |
//! | test_BC_2_07_006_ccr_path_some_file_exists_returns_some | AC-007 | BC-2.07.006 PC-1a,b,c / EC-116 |
//! | test_BC_2_07_006_ccr_path_some_file_missing_falls_through | AC-007 | BC-2.07.006 PC-1d / EC-117 |
//! | test_BC_2_07_006_ccr_path_some_is_directory_falls_through | — | BC-2.07.006 EC-118 |
//! | test_BC_2_07_006_ccr_path_some_empty_string_falls_through | — | BC-2.07.006 EC-119 |
//! | test_BC_2_07_006_detect_ccr_never_returns_err | AC-008 | BC-2.07.006 INV-2 |
//! | test_BC_2_07_006_detect_ccr_no_caching | AC-009 | BC-2.07.006 INV-3, PC-7 |
//! | test_BC_2_07_006_invariant_no_panic_under_any_input | AC-008 | BC-2.07.006 INV-1 |
//! | test_BC_2_07_006_ccr_path_none_ccr_on_path_returns_some | AC-007 | BC-2.07.006 PC-2,3 / EC-114 |

// Tests use expect/unwrap as assertion amplification — not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// BC-ID-encoded test names.
#![allow(non_snake_case)]
// catch_unwind + todo!() panics in stubs.
#![allow(clippy::panic)]

use monocle_config::{detect_ccr, MonocleConfig};
use std::panic::AssertUnwindSafe;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Helper: make a dummy executable file in a TempDir
// ---------------------------------------------------------------------------

/// Create a fake executable file at `dir/name` with a shebang so the OS treats it
/// as executable. Returns the full path.
///
/// On Unix this sets mode 0o755 so `path.is_file()` and `which::which` can find it.
#[cfg(unix)]
fn make_fake_executable(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path).expect("create fake executable");
    file.write_all(b"#!/bin/sh\necho ccr\n")
        .expect("write fake executable");
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("set executable permissions");
    path
}

// ---------------------------------------------------------------------------
// BC-2.07.006 PC-4 / EC-115 — ccr_path None, ccr not on PATH → None
// ---------------------------------------------------------------------------

/// BC-2.07.006 PC-4 / EC-115: When `config.ccr_path` is `None` and `ccr` is NOT
/// on PATH, `detect_ccr()` must return `None`.
///
/// We isolate the PATH environment variable to a temp dir containing no `ccr` binary.
#[test]
fn test_BC_2_07_006_ccr_path_none_ccr_not_on_path_returns_none() {
    let tmp = tempdir().expect("create temp dir");

    // Use a PATH that contains only our empty temp dir — no ccr binary there.
    let old_path = std::env::var("PATH").unwrap_or_default();
    // SAFETY: test-only, single-threaded — but tests run in parallel with other
    // tests. We set PATH to an empty/nonexistent location so which::which("ccr") fails.
    // Restore after test.
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let config = MonocleConfig::default(); // ccr_path: None

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));

    // Restore PATH before any assertions (cleanup happens even if test panics).
    std::env::set_var("PATH", &old_path);

    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 PC-4 / EC-115)"
        )
    });

    assert_eq!(
        result, None,
        "detect_ccr() must return None when ccr_path is None and ccr is not on PATH \
        (BC-2.07.006 PC-4 / EC-115)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 PC-1a,b,c / EC-116 — ccr_path Some, file exists → Some(path)
// ---------------------------------------------------------------------------

/// BC-2.07.006 PC-1c / EC-116: When `config.ccr_path` is `Some(path)` and the
/// file at that path exists (is_file() == true), `detect_ccr()` returns `Some(path)`.
/// PATH search is NOT performed (Step 2 is short-circuited).
#[cfg(unix)]
#[test]
fn test_BC_2_07_006_ccr_path_some_file_exists_returns_some() {
    let tmp = tempdir().expect("create temp dir");
    let fake_ccr = make_fake_executable(tmp.path(), "ccr");

    let config = MonocleConfig {
        ccr_path: Some(fake_ccr.to_str().unwrap().to_string()),
        ..Default::default()
    };

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 PC-1c / EC-116)"
        )
    });

    assert_eq!(
        result,
        Some(fake_ccr.clone()),
        "detect_ccr() must return Some(configured_path) when ccr_path points to an \
        existing file (BC-2.07.006 PC-1c / EC-116); got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 PC-1d / EC-117 — ccr_path Some, file missing → fallthrough to PATH
// ---------------------------------------------------------------------------

/// BC-2.07.006 PC-1d / EC-117: When `config.ccr_path` is `Some(path)` but the file
/// does NOT exist, `detect_ccr()` must emit `tracing::warn!` and fall through to
/// Step 2 (PATH search). The configured-path-missing is non-fatal.
///
/// Since ccr is also not on PATH in this test, the final result is `None`.
#[test]
fn test_BC_2_07_006_ccr_path_some_file_missing_falls_through() {
    let tmp = tempdir().expect("create temp dir");

    // Point ccr_path to a file that does NOT exist.
    let nonexistent = tmp.path().join("does_not_exist_ccr");
    assert!(!nonexistent.exists(), "file must not exist for this test");

    let config = MonocleConfig {
        ccr_path: Some(nonexistent.to_str().unwrap().to_string()),
        ..Default::default()
    };

    // Set PATH to empty so ccr is not found via PATH either.
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));

    std::env::set_var("PATH", &old_path);

    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 PC-1d / EC-117)"
        )
    });

    // File not found → falls through to PATH search → PATH also has no ccr → None.
    assert_eq!(
        result, None,
        "detect_ccr() must return None when ccr_path points to a missing file \
        AND ccr is not on PATH (BC-2.07.006 PC-1d / EC-117); got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 EC-118 — ccr_path is a directory → falls through (is_file() == false)
// ---------------------------------------------------------------------------

/// BC-2.07.006 EC-118: When `config.ccr_path` points to a directory (not a file),
/// `path.is_file()` returns false → `tracing::warn!` → falls through to PATH search.
#[test]
fn test_BC_2_07_006_ccr_path_some_is_directory_falls_through() {
    let tmp = tempdir().expect("create temp dir");

    // The temp dir itself is a directory — not a file.
    let config = MonocleConfig {
        ccr_path: Some(tmp.path().to_str().unwrap().to_string()),
        ..Default::default()
    };

    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));

    std::env::set_var("PATH", &old_path);

    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 EC-118)"
        )
    });

    // A directory fails is_file() → falls through → PATH has no ccr → None.
    // (Result may be Some if ccr happens to be on PATH in the test environment,
    // but we've isolated PATH so it should be None.)
    // The critical invariant is no panic; None is the expected result with empty PATH.
    assert_eq!(
        result, None,
        "detect_ccr() must return None when ccr_path points to a directory \
        AND ccr is not on PATH (BC-2.07.006 EC-118); got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 EC-119 — ccr_path is empty string → is_file() false → fallthrough
// ---------------------------------------------------------------------------

/// BC-2.07.006 EC-119: When `config.ccr_path` is `Some("")` (empty string),
/// `PathBuf::from("")` is an empty path, `path.is_file()` returns false,
/// `tracing::warn!` is emitted, and execution falls through to PATH search.
#[test]
fn test_BC_2_07_006_ccr_path_some_empty_string_falls_through() {
    let tmp = tempdir().expect("create temp dir");

    let config = MonocleConfig {
        ccr_path: Some(String::new()),
        ..Default::default()
    };

    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));

    std::env::set_var("PATH", &old_path);

    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) on empty string ccr_path — \
            Red Gate: implementation not yet written (BC-2.07.006 EC-119)"
        )
    });

    assert_eq!(
        result, None,
        "detect_ccr() must return None when ccr_path is empty string \
        AND ccr is not on PATH (BC-2.07.006 EC-119); got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 INV-2 — detect_ccr never returns Err (return type is Option)
// ---------------------------------------------------------------------------

/// BC-2.07.006 INV-2: `detect_ccr()` always returns `Option<PathBuf>`, never
/// `Result`. This is a compile-time guarantee, but we verify at runtime that
/// every code path produces a non-panicking `Option` value (not just a type check).
///
/// This test calls detect_ccr with several configs that exercise different branches
/// and verifies that the result is always `Some` or `None` — never a panic.
#[test]
fn test_BC_2_07_006_detect_ccr_never_returns_err() {
    let tmp = tempdir().expect("create temp dir");

    let configs = vec![
        MonocleConfig::default(),
        MonocleConfig {
            ccr_path: Some("/absolutely/does/not/exist/ccr".to_string()),
            ..Default::default()
        },
        MonocleConfig {
            ccr_path: Some(String::new()),
            ..Default::default()
        },
        MonocleConfig {
            ccr_path: Some(tmp.path().to_str().unwrap().to_string()),
            ..Default::default()
        },
    ];

    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    for (i, config) in configs.iter().enumerate() {
        let config_clone = config.clone();
        let result = std::panic::catch_unwind(AssertUnwindSafe(move || detect_ccr(&config_clone)));
        assert!(
            result.is_ok(),
            "detect_ccr() must not panic for config case {i} \
            (BC-2.07.006 INV-1 / INV-2 / story AC-008)"
        );
        // The inner result is Option<PathBuf> — both Some and None are acceptable.
        // If the result is Some/None, the invariant is satisfied (no Err, no panic).
    }

    std::env::set_var("PATH", &old_path);
}

// ---------------------------------------------------------------------------
// BC-2.07.006 INV-3 / PC-7 — no caching (each call is a fresh probe)
// ---------------------------------------------------------------------------

/// BC-2.07.006 INV-3 / PC-7: `detect_ccr()` must NOT cache its result. Each call
/// performs a fresh filesystem probe. This test verifies that a file created AFTER
/// the first call to `detect_ccr()` is detected on the second call.
///
/// Two calls with the same config: first returns None (file absent), second returns
/// Some (file present). If caching were used, the second call would also return None.
#[cfg(unix)]
#[test]
fn test_BC_2_07_006_detect_ccr_no_caching() {
    let tmp = tempdir().expect("create temp dir");
    let ccr_path = tmp.path().join("ccr");

    let config = MonocleConfig {
        ccr_path: Some(ccr_path.to_str().unwrap().to_string()),
        ..Default::default()
    };

    // First call: file does not exist → falls through to PATH → PATH is empty → None.
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let first_result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));
    let first = first_result.unwrap_or_else(|_| {
        std::env::set_var("PATH", &old_path);
        panic!(
            "detect_ccr() panicked on first call (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 INV-3)"
        )
    });

    // The file is absent — first result should be None (since PATH also has no ccr).
    // Note: if test env already has 'ccr' on PATH this test would have a different first result;
    // but we've overridden PATH to only the temp dir, which has no ccr yet.
    assert_eq!(
        first, None,
        "First detect_ccr() call must return None when file is absent \
        (BC-2.07.006 INV-3 no-caching pre-condition)"
    );

    // Now create the file.
    make_fake_executable(tmp.path(), "ccr");
    assert!(ccr_path.exists(), "fake ccr must exist after creation");

    // Second call: file now exists → should return Some(ccr_path).
    let second_result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));
    std::env::set_var("PATH", &old_path);

    let second = second_result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked on second call (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 INV-3)"
        )
    });

    assert_eq!(
        second,
        Some(ccr_path.clone()),
        "Second detect_ccr() call must return Some after file is created \
        (BC-2.07.006 INV-3 / PC-7 / story AC-009 — no caching). \
        If this test fails, detect_ccr() is caching the first None result."
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006 INV-1 — no panic under any input combination
// ---------------------------------------------------------------------------

/// BC-2.07.006 INV-1: `detect_ccr()` must not panic under any input combination.
/// All fallible operations (`path.is_file()`, `which::which`) return bool or Result;
/// neither should cause a panic.
#[test]
fn test_BC_2_07_006_invariant_no_panic_under_any_input() {
    let tmp = tempdir().expect("create temp dir");
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let edge_cases = vec![
        MonocleConfig::default(),
        MonocleConfig {
            ccr_path: Some("/".to_string()),
            ..Default::default()
        },
        MonocleConfig {
            ccr_path: Some("relative/path/ccr".to_string()),
            ..Default::default()
        },
        MonocleConfig {
            ccr_path: Some("/dev/null".to_string()),
            ..Default::default()
        },
        MonocleConfig {
            ccr_path: Some("\x00invalid\x00".to_string()),
            ..Default::default()
        },
    ];

    for (i, config) in edge_cases.into_iter().enumerate() {
        let result = std::panic::catch_unwind(AssertUnwindSafe(move || detect_ccr(&config)));
        assert!(
            result.is_ok(),
            "detect_ccr() must not panic for edge case config {i} \
            (BC-2.07.006 INV-1 / story AC-008 — never panics)"
        );
    }

    std::env::set_var("PATH", &old_path);
}

// ---------------------------------------------------------------------------
// BC-2.07.006 PC-2,3 / EC-114 — ccr_path None, ccr on PATH → Some
// ---------------------------------------------------------------------------

/// BC-2.07.006 PC-2,3 / EC-114: When `config.ccr_path` is `None` and `ccr` IS
/// on PATH, `detect_ccr()` must return `Some(path)` from `which::which("ccr")`.
///
/// We create a fake `ccr` binary in a temp dir and set PATH to that dir.
#[cfg(unix)]
#[test]
fn test_BC_2_07_006_ccr_path_none_ccr_on_path_returns_some() {
    let tmp = tempdir().expect("create temp dir");
    let fake_ccr = make_fake_executable(tmp.path(), "ccr");

    // Set PATH to only the temp dir containing the fake ccr.
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", tmp.path().to_str().unwrap());

    let config = MonocleConfig::default(); // ccr_path: None

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| detect_ccr(&config)));

    std::env::set_var("PATH", &old_path);

    let result = result.unwrap_or_else(|_| {
        panic!(
            "detect_ccr() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.006 PC-2,3 / EC-114)"
        )
    });

    assert!(
        result.is_some(),
        "detect_ccr() must return Some when ccr_path is None but ccr IS on PATH \
        (BC-2.07.006 PC-2,3 / EC-114 / story AC-007); got None. \
        Ensure which::which(\"ccr\") is used for PATH fallback."
    );

    let found = result.expect("must be Some");
    // The returned path should resolve to the fake ccr binary.
    // which::which returns a canonicalized path, so compare file names.
    assert_eq!(
        found.file_name(),
        fake_ccr.file_name(),
        "detect_ccr() PATH result must point to 'ccr' binary \
        (BC-2.07.006 PC-3 / EC-114)"
    );
}
