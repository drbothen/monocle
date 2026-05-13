---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
phase: pre-phase-1-final-gate-post-fix-burst
timestamp: 2026-05-13T10:00:00Z
traces_to: "round-13 fix burst commits 2cdd8d2 + 1178797 + 1108029; trajectory round-14"
project: monocle
---

# Consistency Audit — Round 14 (Post-Round-13 Fix Burst)

**Scope:** All 17 artifacts (14 specs + 3 planning files).
**Verdict: GAPS_FOUND**

---

## Executive Summary

All 13 round-13 defects (F-FC-C001..C003, F-FC-I001..I005, and related adversary findings) are RESOLVED in the artifacts. Three new defects are detected. None is blocking for pre-Phase-1 gate entry, but two must be remediated before Phase 1 PRD dispatch.

---

## Round-13 Fix Verification

### F-FC-C001 RESOLVED
**Brief removes Phase1Permission from #[non_exhaustive] list; ClaudeCodeTool added as exhaustive-exempt; ADR-0004 cited.**
- Brief §Scope line 158: lists `HookType, HookEvent, DenyReason, AllowPattern, DenyPattern` as carrying `#[non_exhaustive]`; explicitly exempts `Phase1Permission` and `ClaudeCodeTool` per ADR-0004. Correct.
- SS-core-types-and-abi.md §Enum Extensibility: §Exhaustive Enums — Forbidden List explicitly lists both `Phase1Permission` AND `ClaudeCodeTool` with ADR-0004 reference. Correct.
- ADR-0004: formalizes exemption for both enums. Correct.

### F-FC-C002 RESOLVED
**`unsafe impl private::Sealed` gone; replaced with `plugin-sdk-escape-hatch` feature flag pattern.**
- SS-core-types-and-abi.md §Sealed Pattern Relaxation: documents the original error and the corrected approach using `#[cfg(feature = "plugin-sdk-escape-hatch")] pub mod __plugin_sdk_only { pub use super::private::Sealed; }`. Correct.
- SS-engine-module.md §Sealed Pattern Relaxation: same feature-flag pattern applied to `EngineModule`. Correct.

### F-FC-C003 RESOLVED
**`read_state` parser uses `phase:`, `status:`, `awaiting:`, `current_cycle:` matching real STATE.md frontmatter.**
- SS-core-types-and-abi.md §VsddFactoryAdapter: `parse_frontmatter_field(&content, "phase")`, `parse_frontmatter_field(&content, "status")`, `parse_frontmatter_field(&content, "awaiting")`, `parse_frontmatter_field(&content, "current_cycle")`. Correct.

### F-FC-I001 RESOLVED
**FactoryState: 7 fields (phase, status, awaiting, blocking_issues, convergence, cycle, custom_fields). No `raw_content`.**
- SS-core-types-and-abi.md §FactoryAdapter Trait: struct verified to have exactly 7 fields. Correct.

### F-FC-I002 RESOLVED
**All 5 HookEvent inner-variant structs fully specified with no placeholders.**
- SS-core-types-and-abi.md §Non-Exhaustive Inner Structs: `SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`, `NotificationEvent`, `StopEvent` all fully defined with field lists and rationale. Correct.

### F-FC-I003/I004 (EngineModule / ADR-0004) RESOLVED
**SS-engine-module.md and ADR-0004 created in round-13 burst.**
- SS-engine-module.md: EngineModule trait signature, ClaudeCodeModule implementation, BC-ENGINE-001/002 pre-staged. Correct.
- ADR-0004: formalizes Phase1Permission and ClaudeCodeTool exhaustive exemptions. Correct.

### All Numerical Checks

| Check | Claimed | Actual | Result |
|-------|---------|--------|--------|
| Artifact count | 17 | 17 (14 specs + 3 planning) | PASS |
| EXACT-pinned crates | 9 | 9 (tokio, prost, wasmtime, russh, rmcp, reqwest, axum, serde_json, rand) | PASS |
| Named workspace pins in manifest | 28 | 28 | PASS |
| FactoryState fields | 7 | 7 | PASS |
| Hook event types | 5 | 5 (SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop) | PASS |
| Hook paths | 5 | 5 (all match across brief, SS-daemon, SS-engine-module) | PASS |
| /healthz, /status, /shutdown | present | present in SS-daemon-lifecycle §Health and Status Endpoints | PASS |
| Pre-staged BCs (trajectory claim: 13) | 13 | 13 (10 FC-burst + BC-DTU-001 + BC-ENGINE-001/002) | PASS |
| Brief supplements | claimed 10 | 10 (confirmed in frontmatter) | PASS |

### URL/Path Coherence

All 5 hook paths consistent across brief, SS-daemon-lifecycle, SS-engine-module `hook_paths()`:
- `/hooks/session-start`, `/hooks/prompt-submit`, `/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`

`/healthz` — unauthenticated, GET, 200/503. Present in SS-daemon-lifecycle §Health and Status Endpoints and brief.
`/status` — authenticated, GET, full daemon JSON. Present. `abi_version` field present.
`/shutdown` — authenticated, POST. Present in SS-daemon-lifecycle §Shutdown Signal Handling.

---

## New Defects Found

### G-R14-001 — IMPORTANT
**Brief supplements missing SS-engine-module.md and ADR-0004**

Brief v1.4.8 frontmatter `supplements:` lists 10 entries. Round-13 produced two new primary spec artifacts — `SS-engine-module.md` and `ADR-0004` — but neither is listed in `supplements:`. This means a Phase 1 PRD agent loading only the brief and its listed supplements would NOT load these artifacts, missing BC-ENGINE-001/002 and the exhaustive-enum exemption rationale.

**Remediation:** Add `SS-engine-module.md` and `ADR-0004` to brief `supplements:` frontmatter. Count rises to 12. Brief version bumps to v1.4.9.

**Affected artifact:** `.factory/specs/product-brief.md` §frontmatter `supplements:`

---

### G-R14-002 — IMPORTANT
**SS-permissions-phase1.md §Consequences overstates exhaustiveness scope**

SS-permissions-phase1.md §Consequences line 265 states: "No `#[non_exhaustive]` anywhere in this module." This is factually incorrect. ADR-0004 (which postdates SS-permissions-phase1.md) exempts only `Phase1Permission` and `ClaudeCodeTool` from the `#[non_exhaustive]` default. `DenyReason`, `AllowPattern`, and `DenyPattern` are NOT in ADR-0004's exemption list and therefore MUST carry `#[non_exhaustive]` per BC-TYPES-001.

The brief (line 158) is CORRECT: it lists `DenyReason`, `AllowPattern`, `DenyPattern` as carrying `#[non_exhaustive]`. SS-permissions-phase1.md is the artifact that needs correction.

**Remediation:** Update SS-permissions-phase1.md §Consequences to read: "`Phase1Permission` and `ClaudeCodeTool` are exhaustive by design (documented in ADR-0004); `DenyReason`, `AllowPattern`, and `DenyPattern` carry `#[non_exhaustive]` per BC-TYPES-001 (they are not in ADR-0004's exemption list)."

**Affected artifact:** `.factory/specs/architecture/SS-permissions-phase1.md` §Consequences, line 265.

---

### G-R14-003 — MINOR
**SS-deps-pin-manifest.md missing `async-trait` entry required by SS-engine-module.md**

SS-engine-module.md §Trace explicitly states: "SS-deps-pin-manifest.md — `async-trait` crate (add to Phase 1 pin table: `async-trait = "^0.1"`, caret pin; widely used, no untrusted-input path)." The `async-trait` crate is not in the manifest's Phase 1 pin table. The manifest currently lists 28 entries; adding `async-trait` would make 29.

**Remediation:** Add `async-trait = "^0.1"` row to SS-deps-pin-manifest.md §Phase 1 Pin Manifest table. Caret pin; standard; no security concern.

**Affected artifact:** `.factory/specs/architecture/SS-deps-pin-manifest.md` §Phase 1 Pin Manifest.

---

## Cross-Reference Verification

| Check | Result |
|-------|--------|
| ADR-0004 → SS-core-types-and-abi (inputs + body) | PASS |
| ADR-0004 → SS-permissions-phase1 (inputs + body) | PASS |
| SS-core-types-and-abi → ADR-0004 (body citations) | PASS |
| SS-engine-module → SS-core-types-and-abi (cross-ref) | PASS |
| SS-engine-module → SS-daemon-lifecycle (cross-ref) | PASS |
| SS-engine-module → SS-deps-pin-manifest (cross-ref) | PARTIAL (async-trait noted but not added — G-R14-003) |
| Brief v1.4.8 → ADR-0004 (body reference) | PASS |
| Brief → SS-engine-module | MISSING (G-R14-001) |
| Brief → ADR-0004 in supplements | MISSING (G-R14-001) |
| SS-permissions-phase1 → ADR-0004 | MISSING (expected; ADR produced after; minor back-ref gap) |
| 5 hook paths consistent brief/SS-daemon/SS-engine | PASS |
| /healthz + /status + /shutdown present | PASS |
| FactoryState 7 fields | PASS |
| HookEvent 5 variants all defined | PASS |
| read_state field names match STATE.md | PASS |
| plugin-sdk-escape-hatch pattern (no unsafe impl) | PASS |
| Phase1Permission NOT in #[non_exhaustive] list | PASS |
| ClaudeCodeTool in exhaustive-exempt ADR-0004 | PASS |
| BC-AUTH-001/002 auth token format | PASS |
| BC-RING-001 format_version first key | PASS |
| BC-PROTO-001a/001b field split | PASS |

---

## Defer-Pattern Scan

**SS-engine-module.md:** Two `todo!()` macros in `ClaudeCodeModule::spawn` and `ClaudeCodeModule::preflight`. These are intentional spec markers for implementation-time bodies; the trait signatures are binding. Comment in artifact: "The `todo!()` markers are intentional: `ClaudeCodeModule` is a Phase 1 spec artifact. These signatures are binding — the implementer must not change them." This is NOT a defer pattern — it is a spec-time stub for implementation stories. PASS.

**ADR-0004:** Zero defer patterns. PASS.

---

## Supplement Count Summary

| Artifact | Status |
|----------|--------|
| Brief v1.4.8 `supplements:` lists 10 entries | Stale — should be 12 (missing SS-engine-module + ADR-0004) |
| SS-forward-compatibility.md: "11 pre-staged BC IDs" | Stale — BC-ENGINE-001/002 added in R13; not back-referenced in forward-compat. Informational only; forward-compat is a scan document not a running registry. |

---

## Validation Gate Result

**GAPS_FOUND — NOT BLOCKING for pre-Phase-1 gate entry, but two remediations required before Phase 1 PRD dispatch.**

| Finding | Severity | Blocks Gate? | Action |
|---------|----------|-------------|--------|
| G-R14-001: Brief supplements missing SS-engine-module + ADR-0004 | IMPORTANT | No (but must fix before PRD dispatch) | Add 2 entries to supplements:; bump to v1.4.9 |
| G-R14-002: SS-permissions-phase1 §Consequences overstates exhaustiveness | IMPORTANT | No (but must fix before PRD dispatch — PRD agent will read this doc) | Correct §Consequences bullet |
| G-R14-003: async-trait missing from SS-deps-pin-manifest | MINOR | No | Add 1 row to manifest table |

All 13 R13 defects: RESOLVED.
