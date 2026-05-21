// semgrep-fixture: monocle-no-naked-fs-write
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 2 (one per pattern-either arm).

fn fixture_arm_1_std_fs_write() {
    // Arm 1: std::fs::write
    let _ = std::fs::write("/tmp/x", b"data").unwrap();
}

async fn fixture_arm_2_tokio_fs_write() {
    // Arm 2: tokio::fs::write
    let _ = tokio::fs::write("/tmp/x", b"data").await.unwrap();
}
