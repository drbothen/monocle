---
document_type: behavioral-contract
level: L3
version: "1.1.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md, architecture/SS-session-manager.md]
input-hash: "4843ba9"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.006: ClaudeCodeModule.spawn_recipe() — CCR Base URL Injection

## Description

When a Claude Code Router (CCR) base URL is configured for the selected harness profile and
passed via `SpawnOptions.ccr_base_url`, `ClaudeCodeModule::spawn_recipe()` injects the URL
as the `ANTHROPIC_BASE_URL` environment variable in the returned `SpawnRecipe.env`. This
routes the spawned Claude Code session through CCR for API proxying. The injection is
additive — it supplements the `MONOCLE_SESSION_ID` injection defined in BC-2.03.005.
`SpawnRecipe.env` is an OVERLAY on the session-host process's inherited environment: the
session-host inherits its own process env (PATH, HOME, TERM, etc.) first, then overlays
the `SpawnRecipe.env` fields on top. The env map does NOT replace the full environment.

## Preconditions

1. All preconditions of BC-2.03.005 are satisfied (valid `claude` binary, valid UTF-8 path).
2. `opts.ccr_base_url` is `Some(url)` where `url` is a non-empty string.

## Postconditions

1. `recipe.env` contains the key `"ANTHROPIC_BASE_URL"` mapped to the value of
   `opts.ccr_base_url.unwrap()` verbatim. No URL normalization or validation is performed
   by `spawn_recipe()` — the value is passed through as-is.
2. `recipe.env` also contains `"MONOCLE_SESSION_ID"` per BC-2.03.005 PC-1.
3. When `opts.ccr_base_url` is `None`, `"ANTHROPIC_BASE_URL"` is NOT present in
   `recipe.env`. The absence of a CCR URL is a valid and expected operating mode.
4. All other fields of `SpawnRecipe` are populated per BC-2.03.005.

## Invariants

1. `ANTHROPIC_BASE_URL` injection is the sole mechanism by which CCR routing is enabled
   for monocle-spawned sessions. No other environment variable, CLI flag, or config file
   mutation is used for CCR routing at spawn time.
2. The CCR URL is sourced from the harness profile's `ccr_path` field (BC-2.07.006), read
   by the daemon and passed to `SessionManager`, which passes it as `opts.ccr_base_url`
   to `spawn_recipe()`. The `ClaudeCodeModule` does not read config directly.
3. `spawn_recipe()` performs no URL validation. If the CCR URL is malformed, the spawned
   Claude Code session will fail to reach the CCR endpoint at runtime; the spawn itself
   succeeds from monocle's perspective.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-105 | `opts.ccr_base_url` is `Some("")` (empty string) | `ANTHROPIC_BASE_URL` is set to `""` in `recipe.env` — no validation; spawn proceeds; CCR will likely fail at runtime but monocle's spawn is not responsible for CCR reachability |
| EC-106 | `opts.ccr_base_url` is `Some("http://localhost:8080")` and the CCR process is not running | `recipe.env` contains `ANTHROPIC_BASE_URL = "http://localhost:8080"`; spawn proceeds; Claude Code will fail to reach CCR at runtime; monocle surfaces the session-host's `StateChanged::Terminated` to the TUI |
| EC-107 | `opts.ccr_base_url` is `None` (CCR not configured) | `ANTHROPIC_BASE_URL` absent from `recipe.env`; `recipe.env` contains only `MONOCLE_SESSION_ID`; Claude Code uses its default API endpoint |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ccr_base_url: Some("http://127.0.0.1:8080")` with valid binary and hooks_settings_path | `recipe.env = {"MONOCLE_SESSION_ID": "<id>", "ANTHROPIC_BASE_URL": "http://127.0.0.1:8080"}` | happy-path |
| `ccr_base_url: None` | `recipe.env = {"MONOCLE_SESSION_ID": "<id>"}` — no ANTHROPIC_BASE_URL key | happy-path |
| `ccr_base_url: Some("http://[::1]:9090")` (IPv6) | `recipe.env = {"MONOCLE_SESSION_ID": "<id>", "ANTHROPIC_BASE_URL": "http://[::1]:9090"}` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | When `ccr_base_url = Some(url)`, `recipe.env["ANTHROPIC_BASE_URL"] == url` | unit |
| VP-TBD | When `ccr_base_url = None`, `recipe.env` does NOT contain `"ANTHROPIC_BASE_URL"` | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC governs the CCR routing injection that enables monocle to transparently route spawned Claude Code sessions through CCR, a key capability of the engine abstraction |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness — CCR injection is purely via environment variable; no config file is written to the harness's configuration directory) |
| Architecture Module | monocle-runtime (ClaudeCodeModule — `monocle-runtime/src/engine/claude_code.rs`) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.6.0 §ClaudeCodeModule::spawn_recipe() implementation spec (env injection block) |
| Cross-Ref | BC-2.07.006 (CCR Detection via `ccr_path` Config Field — source of the CCR URL that becomes opts.ccr_base_url) |
| Test Name | test_BC_2_03_006_spawn_recipe_ccr_base_url_injected |

## Related BCs

- [BC-2.03.005] — extends: CCR URL injection supplements the base spawn_recipe() behavior
- [BC-2.07.006] — depends on: CCR path config field is the source of ccr_base_url at spawn time

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#claudecodemodulespawn_recipe-implementation-spec` — env injection block (CCR base URL conditional)

## Story Anchor

S-045 — Same story as BC-2.03.005 (ClaudeCodeModule::spawn_recipe() implementation)

## VP Anchors

VP-TBD — CCR base URL injection presence/absence unit tests (filled after VP creation)

## §Trace v1.1.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-045** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition (clusters with BC-2.03.005). No behavioral content changed.

## §Trace v1.1.1

**Arch-source pin v1.4.0→v1.4.1 (architect C34-001 bump)** (2026-06-13 / D-276):
- Architecture Source updated: SS-engine-module-v2-delta.md v1.4.0 → v1.4.1.
- Reason: architect bumped SS-engine-module-v2-delta.md to v1.4.1 to correct the null-byte
  detection mechanism in spawn_recipe() (C34-001). No behavioral content in this BC changes.
- Patch bump only.

## §Trace v1.1.0

**Architect-delegated BC edit — SpawnRecipe.env is an OVERLAY (I2-006)** (2026-06-03):
- I2-006 finding: BC-2.03.006 description did not explicitly state that `SpawnRecipe.env`
  is an OVERLAY on the inherited process environment (not a full replacement). The architecture
  (SS-session-manager.md v1.3.0 §SpawnRecipe integration + startup step 4 env inheritance fix)
  mandates overlay semantics: session-host inherits its own process env (PATH, HOME, etc.)
  FIRST, then overlays the recipe.env fields. Without env inheritance, the harness child
  launches without PATH or HOME, breaking hooks and binary resolution.
- Description: added explicit "SpawnRecipe.env is an OVERLAY" statement.
- Invariant 3 was already correct ("MERGED with the child process's inherited environment") —
  no change needed there; description now aligned with it.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.006 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Covers: ANTHROPIC_BASE_URL injection when ccr_base_url is Some; absence when None.
- Companion to BC-2.03.005 (base spawn_recipe() happy path).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
