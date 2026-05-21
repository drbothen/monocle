// semgrep-fixture: monocle-no-unbounded-channel
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 1 (single pattern, no pattern-either).

fn fixture_unbounded_channel() {
    // Single pattern match
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel::<u8>();
}
