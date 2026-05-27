---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T00:00:00Z
phase: phase-1-expansion
inputs:
  - {path: .factory/specs/architecture/SS-config.md, version: "1.0.0"}
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-07
capability: CAP-007
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.07.006: CCR Detection via `ccr_path` Config Field

## Description

Claude Code Router (`ccr`) is an integrate-external dependency: monocle detects it
at runtime and surfaces the result in the TUI status bar. The detection algorithm
has two steps: first check the explicit `ccr_path` field in `config.json`; if unset
or the path does not exist on disk, fall back to a PATH search via the `which` crate.
The detection is not cached — it is re-evaluated on each call to `detect_ccr()`,
so that a user who installs `ccr` while monocle is running sees the updated status
within the next status bar refresh cycle. No CCR API changes are required or expected;
monocle only needs to locate the binary.

## Preconditions

1. `detect_ccr(config: &MonocleConfig)` is called with the current in-memory config.
2. The `which` crate is available as a dependency of `monocle-config` (see
   SS-config.md §Dependency Graph: `which` for PATH search for CCR binary detection).
3. The function is called from a synchronous context (monocle-config is sync-only;
   no async runtime is required for `detect_ccr`).

## Postconditions

**Step 1 — Explicit config path check:**
1. If `config.ccr_path` is `Some(path_str)`:
   a. Construct `PathBuf::from(path_str)`.
   b. Call `path.is_file()` to verify the path exists and is a regular file.
   c. If `path.is_file()` is `true`: `detect_ccr` returns `Some(path)`. PATH search
      is NOT performed.
   d. If `path.is_file()` is `false` (path does not exist, is a directory, or is
      inaccessible): a `tracing::warn!("ccr_path configured at {:?} but not found; falling through to PATH search", path)` is emitted. Execution falls through to Step 2.
   e. The configured path not existing is a non-fatal warning — it does NOT cause an
      error return or block monocle startup.

**Step 2 — PATH search (fallback):**
2. If `config.ccr_path` is `None` OR the configured path did not resolve (Step 1d):
   `which::which("ccr")` is called.
3. If `which::which("ccr")` returns `Ok(path)`: `detect_ccr` returns `Some(path)`.
4. If `which::which("ccr")` returns `Err(_)` (not found on PATH): `detect_ccr`
   returns `None`.

**TUI status bar rendering:**
5. The result of `detect_ccr()` is surfaced in the TUI status bar:
   - `Some(path)`: the status bar renders "CCR: <path>" (or "CCR: detected" depending
     on space constraints).
   - `None`: the status bar renders "CCR: not found". This does NOT block other TUI
     functionality; it is an informational notice.
6. The status bar rendering is updated on each status bar refresh cycle (the refresh
   interval is governed by the TUI's render loop; typically ≤1 second). `detect_ccr`
   is called on each refresh, not cached from startup.

**No caching:**
7. `detect_ccr` does NOT cache its result. Each invocation performs a fresh filesystem
   check on the configured path (if set) and a fresh PATH search (if needed). This
   design ensures that installing `ccr` while monocle is running is reflected in the
   next status bar update without requiring a daemon or TUI restart.

## Invariants

1. `detect_ccr` never panics. All fallible operations (`path.is_file()`, `which::which`)
   return `bool` or `Result` respectively; neither causes a panic.
2. `detect_ccr` never returns an `Err`. The function signature is
   `pub fn detect_ccr(config: &MonocleConfig) -> Option<PathBuf>`. All failure modes
   (path not found, PATH search failure) result in `None`, not an error.
3. A configured `ccr_path` that does not exist on disk triggers a warning log and
   falls through to PATH search. It does NOT return an error and does NOT suppress
   the PATH fallback.
4. The detection result does NOT affect daemon startup or TUI startup. monocle starts
   and runs normally when CCR is not found; the only consequence is the "CCR: not found"
   status bar indicator.
5. `detect_ccr` is classified as an effectful-shell function (reads filesystem + PATH)
   per SS-config.md §Purity Boundary. It is not unit-testable without filesystem setup;
   integration tests use `tempfile::TempDir` and `PATH` manipulation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-114 | `ccr_path` is `None` and `ccr` is on PATH | `which::which("ccr")` returns `Ok(path)`; `detect_ccr` returns `Some(path)`; status bar: "CCR: detected" |
| EC-115 | `ccr_path` is `None` and `ccr` is NOT on PATH | `which::which("ccr")` returns `Err`; `detect_ccr` returns `None`; status bar: "CCR: not found" |
| EC-116 | `ccr_path` is `Some("/usr/local/bin/ccr")` and that path exists | `path.is_file()` true; `detect_ccr` returns `Some("/usr/local/bin/ccr")`; no PATH search performed |
| EC-117 | `ccr_path` is `Some("/usr/local/bin/ccr")` and that path does NOT exist | `tracing::warn!` emitted; falls through to PATH search; result depends on whether `ccr` is on PATH |
| EC-118 | `ccr_path` is `Some("/usr/local/bin/ccr")` and path exists but is a directory | `path.is_file()` false (directories are not files); `tracing::warn!` emitted; falls through to PATH search |
| EC-119 | `ccr_path` is `Some("")` (empty string) | `PathBuf::from("")` is an empty path; `path.is_file()` returns false; `tracing::warn!` emitted; falls through to PATH search |
| EC-120 | User installs `ccr` to PATH while monocle is running | Next status bar refresh cycle calls `detect_ccr` fresh; `which::which("ccr")` now returns `Ok`; status bar updates from "CCR: not found" to "CCR: detected" |
| EC-121 | `ccr_path` configured correctly; user uninstalls `ccr`; path no longer exists | Next refresh: `path.is_file()` false; `tracing::warn!`; PATH fallback also fails; `None` returned; status bar: "CCR: not found" |
| EC-122 | Multiple rapid status bar refreshes while filesystem is busy | `detect_ccr` is called multiple times; each is independent; no shared mutable state; no race condition |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `config.ccr_path = None`; `ccr` on PATH at `/usr/bin/ccr` | `Some(PathBuf::from("/usr/bin/ccr"))` | happy-path |
| `config.ccr_path = None`; `ccr` not on PATH | `None` | happy-path |
| `config.ccr_path = Some("/tmp/fake-ccr")`; file exists at that path | `Some(PathBuf::from("/tmp/fake-ccr"))` | happy-path |
| `config.ccr_path = Some("/tmp/fake-ccr")`; file does NOT exist; `ccr` on PATH | `tracing::warn!` emitted; `Some(<PATH-resolved path>)` | edge-case |
| `config.ccr_path = Some("/tmp/fake-ccr")`; file does NOT exist; `ccr` NOT on PATH | `tracing::warn!` emitted; `None` | edge-case |
| Status bar re-renders after `ccr` installed to PATH mid-session | Status bar transitions from "CCR: not found" to "CCR: detected" on next refresh | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `detect_ccr` returns `Some` when configured path exists | integration (TempDir with a fake `ccr` binary) |
| VP-TBD | `detect_ccr` returns `Some` via PATH when `ccr_path` is None and binary is on PATH | integration (modify PATH env var in test; place fake binary) |
| VP-TBD | `detect_ccr` returns `None` when both config path is absent and `ccr` not on PATH | integration |
| VP-TBD | Warning log emitted when configured path does not exist | integration (tracing-test subscriber; assert warn! event) |
| VP-TBD | No panic under any input combination | unit + integration (assert no unwrap) |
| VP-TBD | TUI status bar renders "CCR: not found" when `detect_ccr` returns `None` | integration (mock detect_ccr via trait or fixture; assert status bar widget) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability §SS-07 |
| Capability Anchor Justification | CAP-007 ("Configuration persistence; harness profile management; profile picker; CCR detection") per ARCH-INDEX §Capability Traceability — this BC specifies the CCR detection algorithm that is the named CCR detection component of CAP-007 |
| L2 Domain Invariants | No domain-spec/invariants.md exists for this project; authority is ARCH-INDEX §SS-07 and SS-config.md §CCR Detection |
| Architecture Module | monocle-config (detect_ccr function) per ARCH-INDEX Subsystem Registry SS-07; monocle-tui (status bar rendering of CCR detection result) |
| Architecture Source | SS-config.md v1.0.0 §CCR Detection (BC-2.07.006) |
| Cross-Ref | BC-2.07.002 (ccr_path field in config schema); BC-2.07.003 (config load path that provides config to detect_ccr); product-brief.md §D-010 (CCR as integrate-external dependency: detect on PATH, no CCR API changes) |
| Brief Features | F-55 (ccr_path field in config; detection surfaced in TUI status bar) |
| Test File | `monocle-config/tests/ccr_detection.rs` |
| Test Name | `test_BC_2_07_006_ccr_detection_config_path_then_path_fallback` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.07.002] — depends on: ccr_path schema field defined in BC-2.07.002 is consumed here
- [BC-2.07.003] — depends on: config (possibly default) provides ccr_path value to detect_ccr

## Architecture Anchors

- `architecture/SS-config.md#ccr-detection-bc-2-07-006` — detection algorithm, two-step fallback, no-cache rationale
- `architecture/SS-config.md#purity-boundary` — detect_ccr classified as effectful-shell

## Story Anchor

S-TBD — Implement monocle-config CCR detection and TUI status bar integration (filled by story-writer)

## VP Anchors

VP-TBD — CCR detection integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- Created as new artifact for SS-07 (Config subsystem) per `prd-expansion-scope.md` §3.4 BC-2.07.006
  and `SS-config.md` §CCR Detection.
- Two-step algorithm (config path → PATH fallback) fully specified with all warning/fallthrough
  semantics.
- No-cache property explicitly stated and justified (live install detection).
- Grounded in product-brief.md D-010 (integrate-external: detect on PATH, no API changes).
- Brief feature traced: F-55.
- SE-16d: 2026-05-26T00:00:00Z >= chain high-water (new artifact; no prior chain).
