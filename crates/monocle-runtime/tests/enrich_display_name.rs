//! Production enrich path display_name test (BC-2.06.006 PC-3, S-028 ADV Pass-3).
//!
//! Verifies that the production `ClaudeCodeModule::enrich()` path populates
//! `EnrichedSession::display_name` with a non-empty human-readable engine name
//! (e.g., `"Claude Code"` from `EngineMetadata::display_name`).
//!
//! # Red Gate rationale
//!
//! `ClaudeCodeModule::enrich()` currently calls `EnrichedSession::new()`, which sets
//! `display_name = ""` (empty). `EngineMetadata::display_name = "Claude Code"` is
//! computed by `ClaudeCodeModule::metadata()` but is NOT propagated into the
//! `EnrichedSession` returned by `enrich()`.
//!
//! Additionally, `SessionRegistry::snapshot_enriched_sessions()` also calls
//! `EnrichedSession::new()` directly (not `new_with_display_name`), so the daemon's
//! session list sent to the TUI via IPC also lacks `display_name`.
//!
//! BC-2.06.006 PC-3 requires that the sessions filter matches against
//! `session.display_name` — the daemon-populated IPC wire copy. When `display_name`
//! is empty, typing "Cla" (or "Claude") will NOT match any session by engine name
//! unless the TUI-side hardcoded map happens to cover the harness type, which is
//! incorrect architecture (hardcoded map is a fallback for legacy sessions, not the
//! production path).
//!
//! # BC coverage
//!
//! - BC-2.06.006 PC-3: sessions filter matches session.display_name.
//! - S-028 ADV Pass-3 finding MAJOR-2: production enrich uses EnrichedSession::new()
//!   which sets display_name="" instead of populating from EngineMetadata::display_name.
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic assertion amplification in test code.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_core::engine::{EngineModule, ProcessSnapshot};
use monocle_runtime::engine::ClaudeCodeModule;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construct a minimal `ProcessSnapshot` for a `claude` process.
fn claude_proc_snapshot() -> ProcessSnapshot {
    ProcessSnapshot::new(
        42000,
        Some(PathBuf::from("/usr/local/bin/claude")),
        vec!["claude".to_string()],
        0,
    )
}

/// Construct a `ClaudeCodeModule` instance for testing.
fn module() -> ClaudeCodeModule {
    ClaudeCodeModule::new("http://127.0.0.1:7891".to_string())
}

// ---------------------------------------------------------------------------
// BC-2.06.006 PC-3 — production enrich() output has non-empty display_name
//
// RED: behavioral failure — ClaudeCodeModule::enrich() calls EnrichedSession::new()
//      which sets display_name = "". The assertion `!display_name.is_empty()` fails
//      because the production code never populates display_name from
//      EngineMetadata::display_name ("Claude Code").
//
//      FIX: ClaudeCodeModule::enrich() must call either:
//        (a) self.metadata().map(|m| m.display_name) to retrieve the display name and
//            pass it to EnrichedSession::new_with_display_name(..., display_name.to_string()), OR
//        (b) EnrichedSession::new_with_display_name(..., "Claude Code".to_string()) directly.
//      Option (a) is preferred — it keeps display_name in sync with metadata() without
//      hardcoding it in a second location.
// ---------------------------------------------------------------------------

/// `ClaudeCodeModule::enrich()` must populate `EnrichedSession::display_name` with the
/// engine's human-readable name from `EngineMetadata::display_name` (e.g., `"Claude Code"`).
///
/// BC-2.06.006 PC-3: the TUI sessions filter matches `session.display_name` (the daemon
/// IPC wire copy) — NOT a hardcoded TUI-side lookup map.
///
/// RED: `enrich()` returns `EnrichedSession::new(...)` which sets `display_name = ""`.
/// The assertion `!session.display_name.is_empty()` fails.
///
/// FIX: `enrich()` must pass `display_name` via `EnrichedSession::new_with_display_name`.
#[tokio::test]
async fn test_BC_2_06_006_PC3_enrich_populates_display_name() {
    let m = module();
    let proc = claude_proc_snapshot();

    // Act: call the production enrich() path.
    // On machines where HOME is unset or HOME is not available, enrich() returns
    // Err(HomeUnresolvable) — skip the test in that environment.
    let result = m.enrich(&proc).await;
    let session = match result {
        Ok(s) => s,
        Err(e) => {
            // HomeUnresolvable is a legitimate environment failure (CI without HOME).
            // Skip the test rather than fail it — the display_name assertion is
            // unreachable in this case.
            eprintln!(
                "test_BC_2_06_006_PC3_enrich_populates_display_name: skipping — \
                 enrich() returned Err({e:?}) (HomeUnresolvable). \
                 Set HOME to run this test."
            );
            return;
        }
    };

    // Assert: display_name must NOT be empty.
    // Production path: enrich() must call new_with_display_name(..., "Claude Code".to_string())
    // or equivalent so that the IPC wire copy carries the engine's human-readable name.
    //
    // RED: current production code calls EnrichedSession::new(...) → display_name = "".
    // The assertion fails because "".is_empty() is true.
    assert!(
        !session.display_name.is_empty(),
        "BC-2.06.006 PC-3 defect (ADV Pass-3 MAJOR-2): \
         ClaudeCodeModule::enrich() must populate EnrichedSession::display_name \
         with the engine display name from EngineMetadata::display_name (e.g., \"Claude Code\"). \
         Current production code calls EnrichedSession::new() which defaults display_name to \"\". \
         FIX: change enrich() to call EnrichedSession::new_with_display_name(..., \
         self.metadata()?.display_name.to_string()) or equivalent."
    );

    // Assert: display_name must equal the value from ClaudeCodeModule::metadata().
    // This ensures the enrich() path is derived from metadata() — not a separate
    // hardcoded string that could drift out of sync.
    // metadata().display_name is &'static str; session.display_name is String.
    let expected_display_name: &str = m
        .metadata()
        .expect("metadata() must succeed when HOME is available")
        .display_name;
    assert_eq!(
        session.display_name.as_str(),
        expected_display_name,
        "BC-2.06.006 PC-3: EnrichedSession::display_name must equal \
         EngineMetadata::display_name (\"{expected_display_name}\")"
    );
}

// ---------------------------------------------------------------------------
// BC-2.06.006 PC-3 — snapshot_enriched_sessions output has non-empty display_name
//
// RED: behavioral failure — SessionRegistry::snapshot_enriched_sessions() calls
//      EnrichedSession::new() which sets display_name = "".
//      The assertion `!display_name.is_empty()` fails.
//
//      FIX: snapshot_enriched_sessions() must populate display_name.
//      The production path should call new_with_display_name(..., "Claude Code".to_string())
//      or resolve the display name from a registered ClaudeCodeModule::metadata() call.
// ---------------------------------------------------------------------------

/// `SessionRegistry::snapshot_enriched_sessions()` must populate
/// `EnrichedSession::display_name` on each session in the snapshot.
///
/// BC-2.06.006 PC-3: the IPC wire snapshot sent to the TUI must carry display_name
/// so that the TUI sessions filter can match by engine name without a hardcoded map.
///
/// RED: `snapshot_enriched_sessions()` calls `EnrichedSession::new(...)` which sets
/// `display_name = ""`. The assertion `!session.display_name.is_empty()` fails.
///
/// FIX: `snapshot_enriched_sessions()` must call `new_with_display_name` with the
/// harness type's display name (e.g., `"Claude Code"` for `harness_type = "claude-code"`).
#[test]
fn test_BC_2_06_006_PC3_snapshot_enriched_sessions_populates_display_name() {
    use monocle_runtime::hooks::SessionRegistry;

    // Arrange: registry with a single active session.
    // Use get_or_create — the production path that creates entries on first hook arrival.
    let registry = SessionRegistry::new();
    let _ = registry.get_or_create("sess-snap-001");

    // Act: snapshot the registry via the production path.
    let sessions = registry.snapshot_enriched_sessions();
    assert_eq!(
        sessions.len(),
        1,
        "precondition: registry must contain 1 session after record_session_start"
    );

    let session = &sessions[0];

    // Assert: display_name must NOT be empty.
    // RED: current code calls EnrichedSession::new(...) → display_name = "".
    // The assertion fails because "".is_empty() is true.
    assert!(
        !session.display_name.is_empty(),
        "BC-2.06.006 PC-3 defect (ADV Pass-3 MAJOR-2): \
         SessionRegistry::snapshot_enriched_sessions() must populate \
         EnrichedSession::display_name with the harness engine display name \
         (e.g., \"Claude Code\" for harness_type=\"claude-code\"). \
         Current code calls EnrichedSession::new() which defaults display_name to \"\". \
         FIX: call EnrichedSession::new_with_display_name(..., \"Claude Code\".to_string()) \
         or resolve via a harness-type registry mapping."
    );

    // Assert: harness_type is "claude-code" (the only registered harness in Phase 1).
    // This confirms we are testing the right session entry.
    assert_eq!(
        session.harness_type, "claude-code",
        "precondition: snapshot must carry harness_type=\"claude-code\" (Phase 1 only harness)"
    );

    // The expected display name for "claude-code" is "Claude Code" — same as
    // EngineMetadata::display_name returned by ClaudeCodeModule::metadata().
    // We do not hardcode the string here to avoid coupling the test to a specific
    // display name that might change — instead we assert it is non-empty and let
    // the implementer choose the canonical value.
    //
    // A second assertion: display_name must contain the substring "Claude" (case-sensitive)
    // to confirm the correct engine name is populated, not a generic placeholder.
    assert!(
        session.display_name.contains("Claude"),
        "BC-2.06.006 PC-3: display_name must contain \"Claude\" for harness_type=\"claude-code\". \
         Got: {:?}", session.display_name
    );
}
