// semgrep-fixture: monocle-no-unbounded-channel
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 2
//   arm 1: basic call form     tokio::sync::mpsc::unbounded_channel()
//   arm 2: turbofish call form tokio::sync::mpsc::unbounded_channel::<T>()

fn fixture_unbounded_channel_basic() {
    // Basic call form — matches pattern arm 1.
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}

fn fixture_unbounded_channel_turbofish() {
    // Turbofish call form — matches pattern arm 2 (<...> ellipsis).
    // Previously NOT matched by the single `pattern: ...(...)` arm because
    // type arguments are syntactically distinct from call arguments in semgrep's
    // Rust parser. Now covered by the explicit turbofish arm.
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel::<String>();
}
