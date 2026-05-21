# Cross-Crate Constructor Audit Table — Vendored Copy

**Vendored from:** `.factory/specs/architecture/SS-engine-module.md` §Cross-Crate Constructor Audit Table

**SYNC POLICY:** When the audit table section in `SS-engine-module.md` changes (rows added,
removed, or modified), this file MUST be updated in the **same PR**. The table is the spec
source of truth; this file is a CI-accessible copy that avoids the `.factory/` worktree
not being materialized by `actions/checkout` on develop/PR refs.

**Drift detection:** CI job `audit-table-drift` validates this file against the canonical
block in `SS-engine-module.md` (on the `factory-artifacts` branch) on every PR. Manual
sync is NO LONGER the last line of defence — the CI job fails the build if the delimited
section drifts. To regenerate: run `scripts/extract_audit_table.py --source
.factory/specs/architecture/SS-engine-module.md --output /tmp/audit-table-new.md`, then
replace the delimited block in this file (from `<!-- BEGIN ... -->` through `<!-- END ... -->`)
and commit in the same PR as the SS-engine-module.md change.

**Delimiter note:** `check_audit_table.py` scans for the exact HTML comment delimiters
below. Do not alter them.

<!-- BEGIN: Cross-Crate Constructor Audit Table -->
| Struct | Defining crate | Source spec | Construction path | Constructor present? | Notes |
|--------|---------------|-------------|-------------------|---------------------|-------|
| `EngineMetadata` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (metadata()); test fixtures | Yes (`new(display_name, icon, config_paths, hook_schema_version)`, v1.1.7) | All 4 fields required |
| `ProcessSnapshot` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (detect/enrich path); `monocle-runtime/tests/engine_module.rs` | Yes (two: `new(pid, exe_path, cmdline, start_time_secs)` + `with_full_context(...)`, v1.1.7) | Two-tier: detect-only vs enrich |
| `EnrichedSession` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (enrich()); `monocle-runtime/tests/` | Yes (`new(session_id, harness_type, transcript_path, config_path, status, last_event_micros: Option<i64>)`, v1.1.8) | `last_event_micros: Option<i64>` — None on initial enrich |
| `HookResponse` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (on_hook()); `monocle-runtime/tests/` | Yes (`new(decision)` + `.with_diagnostic()` + `.with_redirect()`, v1.1.8) | Builder pattern for optional fields |
| `SpawnArgs` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (tests compile as separate `[[test]]` binaries) | Yes (`new(project_root)` + `.with_worktree()` + `.with_env_override()`, v1.1.8) | Builder for optional fields |
| `SessionHandle` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (separate `[[test]]` binaries) | Yes (`new(pid, session_id, hook_base_url)`, v1.1.8) | All 3 fields required |
| `EngineVersion` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (separate `[[test]]` binaries) | Yes (`new(version, binary_path)`, v1.1.8) | All 2 fields required |
| `HookEventRecord` | `monocle-runtime` | SS-daemon-lifecycle.md | struct-literal (cross-crate): `monocle-runtime/tests/jsonl_ring.rs` (separate `[[test]]` binary) | Yes (`new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input)`, v1.0.5); `RING_FORMAT_VERSION: u32 = 1` const | `format_version` always `RING_FORMAT_VERSION`; Phase 2 field evolution requires `#[non_exhaustive]` to avoid SemVer-major break |
| `SessionStartEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only: axum handlers call `serde_json::from_slice::<HookEvent>(&body)`; serde's `Deserialize` impl constructs internally within `monocle-core` — E0639 does not apply | No constructor required | Forward-compat: `#[non_exhaustive]` allows Phase 2+ field additions without breaking `Deserialize` impls in downstream crates. Enforce: if `Deserialize` is ever removed, re-audit. |
| `UserPromptSubmitEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `PreToolUseEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `NotificationEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `StopEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `FactoryDetection` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): `VsddFactoryAdapter::detect()` constructs via struct literal WITHIN `monocle-core::factory` — E0639 does not apply. Phase 3 WASM adapters implementing `FactoryAdapter::detect()` will construct cross-crate. | No constructor yet — add before Phase 3 when first cross-crate construction site materializes | Production-grade note: `#[non_exhaustive]` on `FactoryDetection` allows Phase 3+ field additions (e.g., adapter priority, schema version) without breaking existing `detect()` callers. |
| `FactoryState` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): `VsddFactoryAdapter::read_state()` constructs via struct literal WITHIN `monocle-core::factory` — E0639 does not apply. Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | `blocking_issues` and `convergence` are Phase 1 stubs (empty Vec / None) constructed inline. Phase 2 adds body parsing in `monocle-workflow` — that is a cross-crate construction site requiring a constructor. |
| `BlockingIssue` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): not constructed in Phase 1 (blocking_issues Vec is always empty). Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | Phase 2 table parser populates `Vec<BlockingIssue>` — that is the first cross-crate construction site. |
| `ConvergenceMetrics` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): not constructed in Phase 1 (convergence is always None). Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | Phase 2 §Session Resume Checkpoint parser populates `Option<ConvergenceMetrics>` — that is the first cross-crate construction site. |
<!-- END: Cross-Crate Constructor Audit Table -->
