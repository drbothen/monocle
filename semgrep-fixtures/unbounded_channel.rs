// semgrep-fixture: monocle-no-unbounded-channel
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 1 (single pattern, no pattern-either).

fn fixture_unbounded_channel() {
    // Single pattern match — deliberately NOT using turbofish form.
    // Semgrep 1.x does not match `tokio::sync::mpsc::unbounded_channel(...)` against
    // the turbofish form `unbounded_channel::<T>()` (type arguments are not `(...)` args).
    // The production anti-pattern rule fires on ANY call to unbounded_channel regardless
    // of turbofish usage; this fixture uses the basic call form that semgrep can match.
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}
