---
document_type: architecture-section
level: L3
section: "forward-compatibility"
slug: "phase-2-3-4-impact-on-phase-1"
subsystem: cross-cutting
version: "1.2.19"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-18T12:00:00Z
inputs: [product-brief.md, research/domain-monocle-vision-synthesis.md, SS-deps-pin-manifest.md, SS-permissions-phase1.md, SS-daemon-lifecycle.md, SS-conventions-anti-patterns.md, adr/ADR-0001-wasmtime-vs-wasmi.md, adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md, adr/ADR-0003-license-selection.md, planning/oq-research.md]
input-hash: "432b9f7"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Phase 2/3/4 Forward-Compatibility Scan

## [Section Content]

### Scope

This artifact answers a single question: does any Phase 2, 3, or 4 requirement, per scan performed against brief v1.4.5 + vision v1.1.2 at scan time, constrain a Phase 1 spec decision that has NOT yet been locked?

Phase 1 spec decisions are locked by: brief v1.4.5 at scan time (product-owner), oq-research.md (OQ-01..OQ-11, SOQ-1..SOQ-4), SS-daemon-lifecycle.md, SS-permissions-phase1.md, SS-deps-pin-manifest.md, and the three ADRs. Anything already locked in those artifacts is not subject to this scan. The scan targets decisions that are still open at the Phase 1 architecture (PRD + behavioral contracts) level.

Phase 2-4 high-level objectives per brief v1.4.5 §Phase Plan Rationale (scan-time reference):

- **Phase 2:** customization-aware overlays, trigger-trace (`[t]` from permission overlay to defining settings.json line), expanded TUI (full AppMode state machine, 5-level binding precedence, telescope overlay), `monocle-static` crate, `redb 2.x` for transcript indexing.
- **Phase 3:** workflow plane (`monocle-workflow` crate, `VsddFactoryAdapter` promoted to WASM-loadable, `notify 8` FS watcher), `monocle-plugin-sdk` crate (WASM ABI via wasmtime 44), MSRV bump to Rust 1.92.
- **Phase 4:** `CodeMachineModule`, `russh 0.60` federation tunnel, `monocle-ipc` shared-memory ring transport variant, OTel cost/token panel, CCR integration, `rmcp 1.6` MCP bridge, `oauth2 5.x` federation auth, optional QUIC transport.

### Phase 2 Forward-Compatibility Analysis

#### Item P2-1: Trigger-trace data model and JSONL ring buffer

**Question:** Phase 2 indexes Claude Code session transcripts via `redb 2.x` to support `[t]` jump-to-source from the permission overlay. The Phase 1 daemon ingests hook events into a hybrid RAM ring + async JSONL flush (OQ-06: `<runtime_dir>/monocle-events.jsonl`, 100MB × 5 rotation). Does the Phase 1 JSONL ring format need additional metadata fields or a version field so Phase 2 can read Phase 1 history without a migration step?

**Analysis:** The JSONL ring format is defined by the Phase 1 architecture as a retention log of hook events fired against the 5 endpoints. Phase 2 trigger-trace requires correlating a permission prompt (`permission_prompt` Notification) with the settings.json rule that generated it. This correlation is done by reading the `Notification` body fields — `tool_name`, `tool_input`, `message` — and joining against the static customization tree parsed from `settings.json` by `monocle-static`. It does NOT require transcript IDs or parent message IDs from Claude Code's internal session model; monocle is explicitly observe-only and does NOT own session transcripts (brief §Out of Scope (Explicit Non-Goals): "hook events are ephemeral ingestion signals; full transcript storage belongs to each harness's own persistence layer").

The JSONL ring is a retention log for the event ribbon panel (Phase 1) and, secondarily, as trigger-trace source material for Phase 2. For Phase 2 trigger-trace to work, each JSONL record must carry:
- The hook type (already implicit: endpoint path or event `type` field)
- The timestamp (already required for the event ribbon latency display)
- The `session_id` (present in all 5 monocle-canonical hook body schemas per dtu-assessment.md v1.7.5 §monocle-canonical column; note: the gene-source BC-HOOK-007 matrix does NOT include session_id on PreToolUse and Notification — it is a monocle EX-2 addition verified in SS-core-types-and-abi.md v1.2.13 §Non-Exhaustive Inner Structs)
- The `tool_name` and `tool_input` (already in `PreToolUse` and `Notification` bodies)
- The `pid` (already present in all 5 monocle-canonical hook schemas per dtu-assessment.md v1.7.5 §monocle-canonical column)

No additional fields are required. The `HookArgs` struct in `monocle-core::permissions` (SS-permissions-phase1.md) already captures `tool_name`, `tool_input`, and `message`.

**Version field:** The JSONL ring needs a format version field so Phase 2 can detect Phase 1-origin records and apply the correct parser. Without this, Phase 2 `redb` indexing code must hard-code assumptions about the Phase 1 format, creating a hidden compatibility debt.

**Verdict: PHASE 1 MUST DO — add a `format_version: u32` field (value `1`) to every JSONL event record.** Cost: trivial (1 field in the serialization struct). This is a locked Phase 1 spec decision. The JSONL event record schema must be specified in the Phase 1 PRD behavioral contracts (BC-RING-NNN) with `format_version: 1` as an immutable field. Phase 2 indexing code checks this field; a Phase 1 ring with `format_version: 1` and a Phase 2 ring with (hypothetical) `format_version: 2` are distinguishable. **Severity: IMPORTANT. Owner: architect (Phase 1 PRD BC authoring).**

#### Item P2-2: Customization-context exposure for Phase 2 overlay

**Question:** Phase 2 surfaces which customization (Builtin, Global, PerContext, UserCustomCommand, SearchPrompt) is active for the session in focus. Does Phase 1 daemon need to expose a customization-context field to Phase 2 readers?

**Analysis:** The 5-level binding precedence (brief §Phase Plan Rationale (Phase 2 objectives), vision §Key Abstractions `BindingSource`) is managed entirely by `monocle-tui` in Phase 2 — it is a TUI-side concern, not a daemon-side concern. The daemon's job is hook ingestion; it has no visibility into which binding resolved a keystroke. The `monocle-static` crate (Phase 2) reads `settings.json`, `CLAUDE.md`, `keybindings.json`, and hook scripts directly from the filesystem — it does not query the daemon for customization state.

The Phase 1 daemon DOES need to produce `session_id` in hook event records (already covered under P2-1 analysis) so Phase 2 can join hook events to the session that owns the customization tree. This is already present in the Phase 1 hook schema.

**Verdict: NO IMPACT.** Phase 1 daemon does not need any new customization-context field. The static plane reads customization files directly; the daemon's role is limited to hook event forwarding. The join key (`session_id`) is already in all 5 monocle-canonical hook schemas (dtu-assessment.md v1.7.5 §monocle-canonical column).

#### Phase 2 Summary

| Item | Verdict | Severity | Owner |
|------|---------|----------|-------|
| P2-1: JSONL ring format version field | PHASE 1 MUST DO | IMPORTANT | architect |
| P2-2: Customization-context exposure | NO IMPACT | — | — |

### Phase 3 Forward-Compatibility Analysis

#### Item P3-1: `monocle-core` trait stability for WASM ABI

**Question:** Phase 3 ships `monocle-plugin-sdk` as a WASM ABI. Plugin authors implement `EngineModule` and `FactoryAdapter` traits from `monocle-core`. If Phase 1 defines these traits without ABI stability markers, plugin binaries compiled against Phase 1 `monocle-core` may break silently against Phase 3 ABI changes.

Sub-questions:
- Does `monocle-core` need `Sealed` trait pattern to prevent plugin authors from implementing internal traits?
- Do Phase 1 enums need `#[non_exhaustive]` so Phase 3 can extend them?
- Do Phase 1 message types need an explicit ABI version field?

**Analysis — Sealed trait:**

The `EngineModule` and `FactoryAdapter` traits are intended for third-party implementation (that is their purpose per vision §Key Abstractions). Sealing them would prevent the Phase 3 WASM SDK from exposing them to plugin authors, which defeats the entire point. The sealed-trait pattern is correct for INTERNAL traits that happen to be `pub` for technical reasons (e.g., supertrait bounds) but must not be implemented by downstream code. `EngineModule` and `FactoryAdapter` are NOT in this category; they are explicitly designed for external implementation.

**Verdict on Sealed:** NO IMPACT. Do not apply the Sealed pattern to `EngineModule` or `FactoryAdapter`.

**Analysis — `#[non_exhaustive]` on Phase 1 enums:**

The relevant Phase 1 enums are:
1. `Phase1Permission` (SS-permissions-phase1.md) — exhaustive by design; `#[non_exhaustive]` is EXPLICITLY FORBIDDEN per SS-permissions-phase1.md §Decision. Phase 3 introduces a categorically DISTINCT `monocle-plugin-sdk::PluginPermission` enum (not an extension of `Phase1Permission`). No conflict.
2. `AppMode` (vision §AppMode state machine) — Phase 3 does not add new `AppMode` variants; it adds a new panel (`Workflow`) rendered within the existing `Dashboard` variant. No extension needed.
3. `Action` (vision §Action enum) — Phase 3 may add new `Action` variants for plugin-management operations. However, `Action` is defined in `monocle-core` and used by the `monocle-tui` dispatcher; both are first-party monocle crates under the same workspace MSRV. The Phase 3 WASM plugin SDK does NOT expose `Action` to guest plugins — plugins produce `FactoryState` and `EnrichedSession` data; the host (monocle) decides what `Action` to dispatch. No `#[non_exhaustive]` needed on `Action`.
4. `HookEvent` / hook type enum — If Phase 1 defines an enum over hook types (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit), Phase 4 may add `PostToolUse` (brief §Scope (Phase 4 — PostToolUse revisit note): "revisit PostToolUse endpoint need at this point"). `#[non_exhaustive]` on a `HookType` enum would allow Phase 4 to add the variant without breaking match sites. **PHASE 1 MUST DO: apply `#[non_exhaustive]` to any `HookType` or `HookEvent` enum that Phase 1 defines, EXCEPT for the `Phase1Permission` enum which is explicitly exhaustive by design (different concern).** Severity: IMPORTANT.

**Analysis — ABI version field:**

The vision §Key Abstractions `EngineMetadata` struct already carries `hook_schema: &'static str` — a reference to the JSON schema for hook payloads. Phase 3 plugin SDK needs to know which version of `monocle-core` it was compiled against to refuse loading binaries built against an incompatible host. The standard WASM component model approach is to encode the interface version in the WIT (WebAssembly Interface Types) component interface definition, which wasmtime 44's component model supports. This is a Phase 3 architecture concern (the `monocle-plugin-sdk` crate design), not a Phase 1 concern.

However: Phase 1 `monocle-core` SHOULD declare a `MONOCLE_ABI_VERSION: u32 = 1` constant so Phase 3 can embed it in the WIT interface and refuse to load plugins compiled against a different ABI version. This is a one-line addition that has zero runtime cost and prevents a silent compatibility failure.

**Verdict on ABI version:** PHASE 1 MUST DO — declare `pub const MONOCLE_ABI_VERSION: u32 = 1;` in `monocle-core`. Severity: IMPORTANT. Owner: architect (Phase 1 PRD).

#### Item P3-2: Workflow plane evolution and factory detection data structures

**Question:** Phase 3 ingests `.factory/STATE.md` and watches for changes via `notify 8`. Phase 1 already includes factory project detection in its Success Criteria ("Detection succeeds on monocle's own `.factory/`"). Does Phase 1 factory detection need to expose data structures Phase 3 will consume?

**Analysis:** Phase 1 statically bundles `VsddFactoryAdapter` (OQ-03). The adapter is not yet a WASM module — it is compiled into the binary. Phase 3 promotes it to a WASM-loadable module implementing the same `FactoryAdapter` trait. The `FactoryAdapter` trait (vision §FactoryAdapter) is already defined in `monocle-core` (or `monocle-workflow` crate) with the full interface: `detect`, `read_state`, `on_change`. The `FactoryState` struct is already fully specified in the vision.

The Phase 1 static bundle must implement the same trait that Phase 3 will expose via WASM. If Phase 1 defines `VsddFactoryAdapter` as a struct that does NOT implement the `FactoryAdapter` trait (e.g., it is wired inline without a trait), Phase 3 has no clean extraction path.

**Verdict: PHASE 1 MUST DO — `VsddFactoryAdapter` must implement the `FactoryAdapter` trait from day one**, even though Phase 1 statically bundles it. The trait must be defined in `monocle-workflow` (or `monocle-core` if cross-crate visibility requires it) with the exact interface Phase 3 will use for WASM. This is already implied by vision §FactoryAdapter and brief §Phase Plan Rationale (Phase 3 objectives), but must be an explicit Phase 1 PRD behavioral contract. Severity: CRITICAL. Owner: architect.

#### Item P3-3: Permission enum forward path

**Question:** Phase 3 introduces `monocle-plugin-sdk::PluginPermission` (zellij-style 17-variant enum). SS-permissions-phase1.md states these enums are CATEGORICALLY DISTINCT and must not merge. Does this decision still hold?

**Analysis:** The decision holds. `Phase1Permission` models Claude Code session-permission dispatch (TUI overlay response to a `permission_prompt` Notification). `PluginPermission` models host-capability grants to untrusted WASM guest plugins (sandbox boundary). These are orthogonal concerns. Phase 3 adds `PluginPermission` in `monocle-plugin-sdk` alongside Phase 1's `Phase1Permission` in `monocle-core`; they share no variants and no inheritance relationship. The crate boundary enforces the separation at compile time.

**Verdict: NO IMPACT.** The categorical-distinction decision stands. Phase 1 does NOT need `Phase1Permission` to be extensible for Phase 3. Phase 3 designs `PluginPermission` entirely independently in `monocle-plugin-sdk`.

#### Phase 3 Summary

| Item | Verdict | Severity | Owner |
|------|---------|----------|-------|
| P3-1a: Sealed trait on `EngineModule`/`FactoryAdapter` | NO IMPACT | — | — |
| P3-1b: `#[non_exhaustive]` on `HookType`/`HookEvent` enum | PHASE 1 MUST DO | IMPORTANT | architect |
| P3-1c: `MONOCLE_ABI_VERSION` constant in `monocle-core` | PHASE 1 MUST DO | IMPORTANT | architect |
| P3-2: `VsddFactoryAdapter` implements `FactoryAdapter` trait | PHASE 1 MUST DO | CRITICAL | architect |
| P3-3: Permission enum categorical distinction | NO IMPACT | — | — |

### Phase 4 Forward-Compatibility Analysis

#### Item P4-1: Prost message version fields for federation

**Question:** Phase 4 uses prost-encoded cross-host events. Phase 1 pins prost 0.14 EXACT (SS-deps-pin-manifest.md §Patch-Pinning Policy). Does Phase 1 need version fields on prost message types, and do the 5 hook event types need stable wire-format contracts?

**Analysis:** Phase 1 `monocle-proto` crate declares `prost` as a dependency but no Phase 1 wire path uses protobuf encoding (SS-deps-pin-manifest.md: "Phase 1: zero runtime cost"). Hook POST bodies use `serde_json` (not prost). The prost pin is established now to lock the audit baseline before Phase 4 activation.

However, the Phase 1 `.proto` message definitions (if any are written in Phase 1) must be forward-compatible with Phase 4 extension. The key pattern is: every proto message type that Phase 4 federation will re-broadcast MUST include a `uint32 schema_version = 1;` field in its `.proto` definition from Phase 1. This allows Phase 4 federation nodes running different monocle versions to detect schema mismatches before attempting deserialization.

The 5 hook event types (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit) are currently deserialized from HTTP POST bodies via `serde_json`. Phase 4 re-broadcasts these as prost messages across federation nodes. Phase 1 MUST define stable `.proto` message types for these 5 event types (in `monocle-proto`) even though Phase 1 does not use them on the wire — this locks the message shape before Phase 4 must implement against it.

**Verdict: PHASE 1 MUST DO — define `.proto` message types for all 5 hook event types in `monocle-proto/src/*.proto` with `uint32 schema_version = 1;` fields.** Phase 1 generates the Rust types via `prost-build` in `build.rs` but does not use them on the wire. Phase 4 activates the wire path. This is already implied by OQ-07 ("protobuf seams v1 (zero cost)") but must be an explicit Phase 1 PRD behavioral contract specifying the message schemas. Severity: IMPORTANT. Owner: architect.

#### Item P4-2: rmcp stub crate

**Question:** rmcp 1.6 is pinned but Phase 1 workspace does not instantiate it (OQ-09). Does Phase 1 need a stub crate or trait that Phase 4 `monocle-mcp-bridge` will extend?

**Analysis:** Per OQ-09 resolution (brief §Phase 1 Constraints): "rmcp MCP bridge: OMITTED in v1; Phase 4 ships real impl (no stub in v1)." SS-deps-pin-manifest.md §Phase 1 vs Pinned-But-Unused Crates confirms: "the `monocle-mcp-bridge` crate — which will declare `rmcp` as a dependency — does not exist in the Phase 1 workspace." This decision is locked.

The Phase 4 `monocle-mcp-bridge` crate will be a new crate added at the Phase 4 workspace expansion (12 → 13 crates). It requires no Phase 1 trait stub because the `EngineModule` trait (which MCPs may eventually expose data through) is already defined in Phase 1 `monocle-core` with the full interface. The MCP bridge is an outbound service, not an extension of `EngineModule`; it does not require a Phase 1 seam.

**Verdict: NO IMPACT.** OQ-09 resolution stands. No stub crate or trait is required in Phase 1 for Phase 4 MCP bridge work. The rmcp pin in SS-deps-pin-manifest.md is sufficient.

#### Item P4-3: Auth model forward compatibility for federation

**Question:** Phase 1 uses `X-Monocle-Authorization: <token>` for local daemon access (SS-daemon-lifecycle.md §Body Size Limit: "auth_layer... X-Monocle-Authorization enforced"). Phase 4 federation requires cross-host auth (SS-deps-pin-manifest.md §Phase 4 — Federation, MCP Bridge: `oauth2 5.x`). Does Phase 1 auth design need forward-compatible structure — scope claims, token versioning — so Phase 4 OAuth2 federation does not break Phase 1 callers?

**Analysis:** The Phase 1 auth mechanism is a shared secret: 32-byte cryptographically random token (hex-encoded) generated at daemon start, written to the lock file at mode 0o600, presented by callers as `X-Monocle-Authorization: <token>`. This token is local-only: it is written to `<runtime_dir>/monocle.lock` (accessible only by the OS user who started the daemon) and consumed by the TUI client connecting via UDS.

Phase 4 federation auth uses `oauth2 5.x` for cross-host trust establishment. The Phase 4 federation tunnel (russh) requires authenticating monocle daemon A against monocle daemon B on a remote host. This is a DIFFERENT auth surface from the Phase 1 local daemon auth:
- Phase 1 auth: `TUI client → local daemon` (shared secret, single-host, OS file permissions as trust anchor)
- Phase 4 auth: `daemon-A → daemon-B` (OAuth2/PKCE or device flow, cross-host, certificate or token-based)

These two auth surfaces are categorically distinct and do not share a wire format. Phase 4 does not extend the Phase 1 `X-Monocle-Authorization` header; it adds a new auth layer on the russh federation tunnel, which operates on a different port and connection than the HTTP hook endpoint.

However, Phase 1 SHOULD version the token format so Phase 4 can distinguish Phase 1 tokens from future token types: the current format is a bare 64-char hex string. If Phase 4 needs to issue a different token type (e.g., a JWT with scope claims for federation) for the local daemon auth path, the bare hex string has no version prefix and Phase 4 cannot distinguish "Phase 1 shared secret" from "Phase 4 OAuth2-derived token."

**Verdict: PHASE 1 MUST DO — version the local auth token format.** The token written to the lock file should be prefixed with a version indicator: `"monocle-v1:<64-char hex>"` rather than a bare hex string. The daemon auth middleware strips and validates the prefix before comparing the secret. Cost: 10-15 LOC change. This is a concrete Phase 1 spec decision. Phase 4 can then introduce `"monocle-v4-oauth:<jwt>"` without ambiguity. **Severity: IMPORTANT. Owner: architect (SS-daemon-lifecycle.md must be updated to reflect the `monocle-v1:` prefix in the token format specification).**

#### Phase 4 Summary

| Item | Verdict | Severity | Owner |
|------|---------|----------|-------|
| P4-1: Proto message version fields + Phase 1 `.proto` definitions | PHASE 1 MUST DO | IMPORTANT | architect |
| P4-2: rmcp stub crate | NO IMPACT | — | — |
| P4-3: Auth token version prefix | PHASE 1 MUST DO | IMPORTANT | architect |

### Cross-Phase Decisions Required

All "PHASE 1 MUST DO" findings synthesized. v1.1 adds Disposition column — all 6
items resolved pre-Phase-1 per human authorization (commit in same burst as v1.1).

| ID | Finding | Severity | Phase 1 Spec Change | Owner | Disposition |
|----|---------|----------|---------------------|-------|-------------|
| FC-01 | Add `format_version: u32 = 1` field to every JSONL ring event record | IMPORTANT | BC-2.01.007: JSONL event record schema includes `format_version: 1` as first key; specified in SS-daemon-lifecycle.md v1.0.32 §Drain | architect | RESOLVED PRE-PHASE-1 — locked in SS-daemon-lifecycle.md v1.0.6 per human authorization |
| FC-02 | Apply `#[non_exhaustive]` to `HookType` / `HookEvent` enum in `monocle-core` | IMPORTANT | BC-2.02.003: `#[non_exhaustive]` default for all pub enums; exemption policy documented; specified in SS-core-types-and-abi.md §Enum Extensibility | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-03 | Declare `pub const MONOCLE_ABI_VERSION: u32 = 1;` in `monocle-core` | IMPORTANT | BC-2.02.001 + BC-2.02.002: constant declared in `monocle-core::abi`; exposed via `/status`; specified in SS-core-types-and-abi.md §ABI Version Constant | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-04 | `VsddFactoryAdapter` MUST implement `FactoryAdapter` trait from Phase 1 | CRITICAL | BC-2.02.004 + BC-2.02.005: trait defined in `monocle-core::factory`; full open-trait signature (no sealed bound; see §Item P3-1 Sealed trait analysis above), self-referential test specified in SS-core-types-and-abi.md §FactoryAdapter Trait | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-05 | Define `.proto` message types for all 5 hook event types in `monocle-proto` with `schema_version` field | IMPORTANT | BC-2.02.006 + BC-2.02.007 + BC-2.02.008: full HookEnvelope proto schema with field-number reservation convention; BC-2.02.006 = wire field number contract (old BC-PROTO-001a), BC-2.02.007 = Rust struct schema_version field (old BC-PROTO-001b), BC-2.02.008 = Phase 4 validation requirement (old BC-PROTO-002); specified in SS-core-types-and-abi.md §Prost Wire Schemas | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-06 | Version the local auth token: `monocle-v1:<64-char-hex>` format | IMPORTANT | BC-2.01.008 + BC-2.01.009: token format, constant-time comparison, 401 rejection rule; specified in SS-daemon-lifecycle.md v1.0.32 §Start Sequence | architect | RESOLVED PRE-PHASE-1 — locked in SS-daemon-lifecycle.md v1.0.6 per human authorization |

No findings require REWORK-level severity. All 6 findings are resolved with
complete, production-grade spec text — not deferred, not advisory, not TODO.

### Verdict

**PHASE 1 READY** — all 6 forward-compat items resolved pre-Phase-1 per human
authorization; spec package self-contained for fresh Phase 1 context.

All six patches (FC-01 through FC-06) are locked into binding architecture
artifacts BEFORE Phase 1 PRD dispatch. Phase 1 agents operating from a fresh
context will find complete, unambiguous specs in:

- `SS-core-types-and-abi.md` — FC-02, FC-03, FC-04 (CRITICAL), FC-05
- `SS-daemon-lifecycle.md v1.0.32` — FC-01, FC-06

None of the patches changes Phase 1 delivery scope, crate count, or external
behavior. They are additions that prevent silent forward-compatibility failures
at Phase 2, 3, and 4 boundaries.

FC-04 (`VsddFactoryAdapter implements FactoryAdapter trait`) was the only CRITICAL
finding. It is resolved with a complete, open-trait specification (Round 15:
sealing removed entirely per human Q-15-1 honoring vision authority; trait now open
for plugin SDK consumption) including the full trait signature, the
`VsddFactoryAdapter` implementation skeleton, the Phase 3 extension path, and two
behavioral contracts (BC-2.02.004 + BC-2.02.005).

The product-owner (`/vsdd-factory:create-prd`) MUST load this document as an input.
The following 16 pre-staged BC IDs are RESERVED — the PRD must use these exact IDs
when formalizing the contracts with postconditions and verification harness stubs:

| BC ID (canonical) | Old-Form ID (retired; BC-INDEX §Renumbering Map) | Source Artifact |
|-------------------|--------------------------------------------------|----------------|
| BC-2.01.007 | BC-RING-001 | SS-daemon-lifecycle.md |
| BC-2.02.001 | BC-ABI-001 | SS-core-types-and-abi.md |
| BC-2.02.002 | BC-ABI-002 | SS-core-types-and-abi.md |
| BC-2.02.003 | BC-TYPES-001 | SS-core-types-and-abi.md |
| BC-2.02.004 | BC-FACTORY-001 | SS-core-types-and-abi.md |
| BC-2.02.005 | BC-FACTORY-002 | SS-core-types-and-abi.md |
| BC-2.02.006 | BC-PROTO-001a | SS-core-types-and-abi.md |
| BC-2.02.007 | BC-PROTO-001b | SS-core-types-and-abi.md |
| BC-2.02.008 | BC-PROTO-002 | SS-core-types-and-abi.md |
| BC-2.01.008 | BC-AUTH-001 | SS-daemon-lifecycle.md |
| BC-2.01.009 | BC-AUTH-002 | SS-daemon-lifecycle.md |
| BC-2.01.010 | BC-LOCK-001 | SS-daemon-lifecycle.md |
| BC-2.03.001 | BC-ENGINE-001 | SS-engine-module.md |
| BC-2.03.002 | BC-ENGINE-002 | SS-engine-module.md |
| BC-2.03.003 | BC-ENGINE-002-ERR | SS-engine-module.md |
| BC-2.03.004 | BC-ENGINE-003 | SS-engine-module.md |

Notes: BC-PROTO-001 was split into BC-PROTO-001a (wire field number, now BC-2.02.006)
and BC-PROTO-001b (Rust struct surface, now BC-2.02.007) per F-FC-O004.
BC-LOCK-001 (now BC-2.01.010) added per F-FC-O001 (lock-file `contract_version` field).
BC-ENGINE-001/002/003 (now BC-2.03.001/002/004) added per round-14 fix burst
(SS-engine-module.md v1.1; N5 BC count propagation). BC-ENGINE-002-ERR (now BC-2.03.003)
added in SS-engine-module.md v1.1.4 (commit 563b573); pre-staging table updated in v1.1.5
(round-23 micro-fix burst). All old-form IDs retired per BC-INDEX.md v1.1 §Renumbering
Map (canonical at T-128h dispatch time 2026-05-17T17:00:00Z; current canonical advances
over time per F-R107-8 historical-pin discipline).

## §Trace

**§Trace v1.2.19** (2026-05-18T12:00:00Z) — F-R115-1 HIGH back-cascade miss closure (SS-daemon-lifecycle v1.0.7 → v1.0.32):
- NORMATIVE (F-R115-1 HIGH): Three active current-pointer citations to SS-daemon-lifecycle.md refreshed from stale v1.0.7 to canonical v1.0.32:
  (1) FC-01 Phase 1 Spec Change column: `SS-daemon-lifecycle.md v1.0.7 §Drain` → `v1.0.32 §Drain`.
  (2) FC-06 Phase 1 Spec Change column: `SS-daemon-lifecycle.md v1.0.7 §Start Sequence` → `v1.0.32 §Start Sequence`.
  (3) §Verdict bullet list: `SS-daemon-lifecycle.md v1.0.7` → `v1.0.32`.
  Historical §Trace pinpoints and Disposition column citations (`v1.0.6`) are immutable audit records — preserved unchanged per PG-D042-WITHIN-FILE historical-pinpoint carve-out.
- PG-D042-BACK-CASCADE APPLIED: This file is itself a D-042 back-cascade dispatch target (per PG-D042-BACK-CASCADE as codified in SS-conventions-anti-patterns.md §D-042 CANONICAL SCOPE in Round 13). The irony is noted: the document that codified back-cascade obligation in §Trace v1.2.18 had its own stale SS-daemon-lifecycle citations (v1.0.7, unrefreshed since FC table authoring) that required a back-cascade correction. F-R115-1 closes this residual.
- SE-17c BEFORE: `SS-daemon-lifecycle.md v1.0.7` (3 active sites — FC-01 column 4, FC-06 column 4, §Verdict bullet).
- SE-17c AFTER: `SS-daemon-lifecycle.md v1.0.32` (3 active sites corrected).
- SE-16d PASS: 2026-05-18T12:00:00Z > chain high-water 2026-05-18T11:00:00Z (monotonic).

**§Trace v1.2.18** (2026-05-18T11:00:00Z) — F-R114-1 D-042 back-cascade: citation refresh (active sites only):
- NORMATIVE (F-R114-1 MED): Three active current-pointer citations refreshed to canonical versions:
  (1) §P2-1 session_id bullet: `dtu-assessment.md v1.7` → `v1.7.5`; `SS-core-types-and-abi.md v1.2.8` → `v1.2.13`.
  (2) §P2-1 pid bullet: `dtu-assessment.md v1.7` → `v1.7.5`.
  (3) §P2-2 verdict: `dtu-assessment.md v1.7` → `v1.7.5`.
  Historical §Trace pinpoints (rounds 52–60) that cite earlier dtu-assessment and SS-core-types-and-abi
  versions are immutable audit records — preserved unchanged per PG-D042-WITHIN-FILE historical-pinpoint
  carve-out.
- D-042 BACK-CASCADE OBLIGATION TRIGGER: this bump was itself triggered by a back-cascade from
  dtu-assessment.md v1.7 → v1.7.5 and SS-core-types-and-abi.md v1.2.8 → v1.2.13. The explicit
  back-cascade obligation is now codified in SS-conventions-anti-patterns.md §D-042 CANONICAL SCOPE
  (v1.29.4) to prevent future recurrence.
- SE-17c BEFORE: `dtu-assessment.md v1.7` (3 sites), `SS-core-types-and-abi.md v1.2.8` (1 site).
- SE-17c AFTER: `dtu-assessment.md v1.7.5` (3 sites), `SS-core-types-and-abi.md v1.2.13` (1 site).
- SE-16d PASS: 2026-05-18T11:00:00Z > chain high-water 2026-05-18T05:30:00Z (monotonic).

**§Trace v1.2.17** (2026-05-18T01:00:00Z) — F-R108-1 + F-R108-9 frontmatter + historical-pin correction (Round 7C):
- NORMATIVE (F-R108-1 CRITICAL): §BC-mapping notes (pre-§Trace body) "current canonical BC-INDEX is v1.4
  per F-R107-2 closure" removed. O-R108-3 pattern codification: live-version claims in §Trace historical-pin
  notes are structurally fragile. Replaced with "current canonical advances over time per F-R107-8
  historical-pin discipline" — historical-pin-only language. Site: BC-mapping notes paragraph (1 occurrence).
  This is a normative content change: one live-version claim string was removed from the document body.
- NORMATIVE (F-R108-9 HIGH): frontmatter `timestamp` corrected from 2026-05-17T16:30:00Z to
  2026-05-18T01:00:00Z. Prior timestamp lagged the latest §Trace entry (v1.2.16 at 23:00:00Z);
  SE-16b violation. Version bump v1.2.16 → v1.2.17 applied in Round 8A (F-R109-1) to
  reconcile frontmatter with the §Trace version number this entry already claimed.
- SE-17c BEFORE: "current canonical BC-INDEX is v1.4 per F-R107-2 closure" (1 occurrence).
- SE-17c AFTER: "current canonical advances over time per F-R107-8 historical-pin discipline".
- SE-16d PASS: 2026-05-18T01:00:00Z > chain high-water 2026-05-17T23:00:00Z (monotonic).

**§Trace v1.2.17-R109** (2026-05-18T05:00:00Z) — F-R109-1 + F-R109-8 + F-R109-13 reconciliation (Round 8A):
- NORMATIVE (F-R109-1 CRITICAL): frontmatter `version` bumped from "1.2.16" to "1.2.17" to
  match the §Trace v1.2.17 entry already present in the document body since Round 7C. The Round
  7C dispatch wrote §Trace v1.2.17 but withheld the frontmatter bump per cross-dispatch
  coordination directive (targeting current PO 7B SS pin). This created a fabrication-class
  defect: frontmatter claimed v1.2.16 while §Trace body documented v1.2.17 as the current state.
- NORMATIVE (F-R109-8 HIGH): §Trace v1.2.17 body rewritten to remove "No version bump —
  content unchanged; timestamp-only correction" self-contradiction. The F-R108-1 removal of one
  live-version claim string IS normative content change. The body now accurately states that the
  version bump v1.2.16 → v1.2.17 was applied in Round 8A to reconcile frontmatter with §Trace.
- INFORMATIONAL (F-R109-13 MED — §Trace v1.2.15 gap): Version v1.2.15 was never a real
  intermediate state of this document. ARCH-INDEX §Trace v1.0.6 erroneously claimed "SS-forward-
  compatibility.md v1.2.15 → v1.2.16" for the Round 6D F-R107-5 BC ID canonicalization work.
  Git evidence (commit 98396fe): this file was already at v1.2.16 in that commit; no v1.2.15
  intermediate commit exists. The actual Round 6D transition was v1.2.14 → v1.2.16 (template-
  compliance rename in v1.2.14; BC ID canonicalization directly to v1.2.16). The "v1.2.15"
  reference in ARCH-INDEX §Trace v1.0.6 is a fabricated intermediate version; the gap in this
  file's §Trace history between v1.2.14 and v1.2.16 is correct — v1.2.15 simply never existed.
  ARCH-INDEX §Trace v1.0.8 (Round 8A) corrects the ARCH-INDEX narrative to "v1.2.14 → v1.2.16".
- SE-17c BEFORE: "No version bump — content unchanged; timestamp-only correction".
- SE-17c AFTER: "Version bump v1.2.16 → v1.2.17 applied in Round 8A (F-R109-1) to reconcile
  frontmatter with the §Trace version number this entry already claimed."
- SE-16d PASS: 2026-05-18T05:00:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic; corrected from erroneous 2026-05-17T04:30:00Z per F-R110-1).

**§Trace v1.2.17-R110** (2026-05-18T05:30:00Z) — F-R110-1 timestamp correction (Round 9A):
- NORMATIVE (F-R110-1 CRITICAL): frontmatter `timestamp` and §Trace v1.2.17-R109 header corrected
  from "2026-05-17T04:30:00Z" to "2026-05-18T05:00:00Z". Round 8A used wrong date; SE-16d chain
  regressed below prior §Trace v1.2.17 entry at 2026-05-18T01:00:00Z. Arithmetic correction
  applied across all 5 affected files in parallel Round 9A burst.
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T05:00:00Z (monotonic).

**§Trace v1.2.16** (2026-05-17T23:00:00Z) — F-R107-5 BC ID canonicalization (Round 6D):
- NORMATIVE: All stale pre-renumbering BC IDs in FC table + BC-mapping table replaced with
  canonical BC-2.SS.NNN forms per BC-INDEX.md v1.4 §Renumbering Map. Finding: F-R107-5 HIGH.
- SE-17f BEFORE (FC table, lines 188-193): FC-01 cited `BC-RING-001`, FC-02 cited
  `BC-TYPES-001`, FC-03 cited `BC-ABI-001 + BC-ABI-002`, FC-04 cited
  `BC-FACTORY-001 + BC-FACTORY-002`, FC-05 cited `BC-PROTO-001 + BC-PROTO-002`,
  FC-06 cited `BC-AUTH-001 + BC-AUTH-002`.
- SE-17f BEFORE (FC-04 body prose ~line 219): `BC-FACTORY-001 + BC-FACTORY-002`.
- SE-17f BEFORE (BC-mapping table ~lines 225-242): all 16 rows used old-form IDs;
  table had no "Old-Form ID" column; the Notes paragraph lacked old→new cross-references.
- SE-17c REPLACEMENTS by canonical new ID:
  BC-2.01.007 [old: BC-RING-001]: 1 FC table cell (FC-01)
  BC-2.02.003 [old: BC-TYPES-001]: 1 FC table cell (FC-02)
  BC-2.02.001 + BC-2.02.002 [old: BC-ABI-001 + BC-ABI-002]: 1 FC table cell (FC-03)
  BC-2.02.004 + BC-2.02.005 [old: BC-FACTORY-001 + BC-FACTORY-002]: 1 FC table cell (FC-04) + 1 body prose site
  BC-2.02.006 + BC-2.02.007 + BC-2.02.008 [old: BC-PROTO-001 + BC-PROTO-002]: 1 FC table cell (FC-05);
    note BC-PROTO-001 was split into BC-2.02.006 (wire field, old BC-PROTO-001a) +
    BC-2.02.007 (Rust struct, old BC-PROTO-001b); BC-PROTO-002 → BC-2.02.008
  BC-2.01.008 + BC-2.01.009 [old: BC-AUTH-001 + BC-AUTH-002]: 1 FC table cell (FC-06)
  BC-mapping table: all 16 rows canonicalized; "Old-Form ID (retired)" column added;
    Notes paragraph updated with old→new cross-references.
- SE-17d AFTER: BC-mapping table BC-2.01.007..BC-2.03.004 (all 16 rows); FC table
  BC-2.01.007, BC-2.02.001-008, BC-2.01.008-009; body prose BC-2.02.004 + BC-2.02.005.
- SE-17g META AUDIT: zero normative stale BC IDs remain in SS-forward-compatibility.md.
  Remaining old-form pattern appearances are INFORMATIONAL only:
  (a) line 53: `BC-RING-NNN` is a placeholder template in pre-formalization analysis
      prose (NNN = not a specific ID; historical analysis proposing the BC before naming);
  (b) FC-05 cell: `old BC-PROTO-001a/b/002` explicitly labeled as old-form context;
  (c) BC-mapping table "Old-Form ID (retired)" column: historical record by design.
  SE-17g PASS: 0 normative stale IDs.
- SE-17f PASS: sampled mapping verified — FC-01 → BC-2.01.007, FC-03 → BC-2.02.001+002,
  FC-04 → BC-2.02.004+005, FC-05 → BC-2.02.006+007+008, FC-06 → BC-2.01.008+009.
- SE-16d PASS: 2026-05-17T23:00:00Z > chain high-water 2026-05-17T22:00:00Z (monotonic).

v1.2.13 changes (round-60.1 F-R60-1 sibling-propagation fix):

- F-R60-1 RESOLVED (MED — sibling-propagation gap from R59.1 narrow sweep): §Trace v1.2.9
  entry for the comprehensive PG-4 sweep (round-53.1 F-R53-adv-2 companion) read "verified
  across all 8 architecture spec files" — actual SS-*.md count is 7 (confirmed by ls SS-*.md).
  Corrected "8" → "7". Root cause: R59.1 fixed 2 stale-count sites in SS-conventions-anti-patterns
  §Trace but performed only a narrow 2-site fix without corpus-wide grep, missing this sibling.
  F-R60-corpus-sweep META rule (§Corpus-Wide-Sweep Convention) codified in SS-conventions-anti-patterns
  v1.28 closes the partial-sweep recurrence pattern structurally.

  PG-3 self-audit: no directional qualifiers added; no current-state L-numbers used.
  PG-4 self-audit: "§Corpus-Wide-Sweep Convention" — confirmed heading exists in
  SS-conventions-anti-patterns.md v1.28. "§Trace v1.2.9" — versioned historical reference, exempt.
  PG-5 self-audit: no bare version citations added; no new false-currency claims.

v1.2.12 changes (round-56.1 F-R56-1 §Trace factual correction):

- F-R56-1 RESOLVED (MEDIUM content — adversary finding R56): §Trace v1.2.11 entry contained
  factually-false claim "vision v1.1.2 does not exist; canonical is v1.1". Vision IS at
  v1.1.2 per domain-monocle-vision-synthesis.md frontmatter (version: "1.1.2", approved
  2026-05-12). The false claim was introduced without verifying the vision file frontmatter.
  The R55.1 fix correctly identified the brief side as false-currency (brief v1.4.5 vs
  current v1.4.23) but incorrectly characterized the vision side; vision v1.1.2 was and
  remains the canonical version at scan time and at present. §Scope §Trace v1.2.11
  description corrected to reflect that only the brief side was a false-currency claim.
  §Scope opening sentence restored to historical-anchor form with the correct vision
  version: "per scan performed against brief v1.4.5 + vision v1.1.2 at scan time".
  PG-5 §Historical-Anchor Framing Convention codified in SS-conventions-anti-patterns.md
  v1.24 as part of same burst. D-042 cascade: dtu-assessment.md v1.6 → v1.7 in §P2-1
  session_id prose, §P2-2 pid prose, §P2-2 join-key sentence (3 sites; both SS-core-types
  bump and dtu bump triggered by this burst); SS-core-types-and-abi.md v1.2.7 → v1.2.8 in
  §P2-1 session_id prose (1 site). D-053 option (b) bounded residuals F-R55-adv-1 and
  F-R55-adv-3 unchanged.

v1.2.11 changes (round-55.1 F-R55-adv-2 historical-anchor rewrite under D-053 option b):

- F-R55-adv-2 RESOLVED (MEDIUM content — adversary finding R55): §Scope opening sentence
  contained "as currently specified in brief v1.4.5 and vision v1.1.2" — a false currency
  claim on the brief side (brief is at v1.4.23; vision v1.1.2 is and was the canonical
  version). §Scope second sentence cited "brief v1.4.5 (product-owner)" without
  historical-anchor framing. §Scope Phase 2-4 objectives list header cited
  "per brief v1.4.5 §Phase Plan Rationale" without scan-time qualifier. Three sites
  rewritten as explicit historical anchors: opening sentence → historical-anchor form
  (vision version erroneously changed to v1.1; corrected back to v1.1.2 in v1.2.12);
  second sentence → "brief v1.4.5 at scan time (product-owner)"; objectives header →
  "per brief v1.4.5 §Phase Plan Rationale (scan-time reference)". Provenance preserved;
  false "currently" claim on brief side removed. Under D-053 (human-ratified 2026-05-14,
  pre-Phase-1 scope only) option (b) convergence relaxation: F-R55-adv-1 (PG-4 em-dash
  separator codification gap) and F-R55-adv-3 (PG-4 intra-document scope hole) are bounded
  META residuals per option (b) — not fixed in this burst; state-manager catalogs in
  close-out.

v1.2.10 changes (round-54.1 F-R54-adv-1 D-042 same-file partial cascade fix):

- F-R54-adv-1 RESOLVED (MEDIUM — adversary finding R54): D-042 within-file partial cascade.
  §Cross-Phase Decisions FC table col 4 (Phase 1 Spec Change) for FC-01 and FC-06 cited
  `SS-daemon-lifecycle.md v1.0.6` while §Verdict (same file) already correctly cited
  `SS-daemon-lifecycle.md v1.0.7` since R53.1. Root cause: R53.1 burst updated §Verdict
  but applied the D-042 within-file grep selectively (by section) rather than at full-file
  scope, so the FC-01 and FC-06 table cells were missed. This is the 10th recurrence of the
  D-042 citation-staleness META-pattern and the first within-file-partial cascade variant.
  Fix: FC-01 col 4 `SS-daemon-lifecycle.md v1.0.6 §Drain` → `v1.0.7`; FC-06 col 4
  `SS-daemon-lifecycle.md v1.0.6 §Start Sequence` → `v1.0.7`. Disposition column citations
  in both rows left at `v1.0.6` intentionally — that column records the lock-in version at
  which the decision was made (historical pinpoint), not the current spec version.
  PG-D042-WITHIN-FILE sub-rule codified in SS-conventions-anti-patterns.md v1.23 as the
  structural closure of this recurrence pattern.

v1.2.9 changes (round-53.1 F-R53-adv-3 brief §-anchor mis-anchors + comprehensive expanded-scope PG-4 sweep):

- F-R53-adv-3 RESOLVED (LOW — adversary finding R53): Five brief §-anchor mis-anchors
  corrected. Brief has no `## Phase Plan` heading; the actual heading is `### Phase Plan
  Rationale`. Brief has no `## Phase 4` or `## Explicit Non-Goals` headings; the bold
  labels for these appear within `## Scope` and `### Out of Scope` respectively. Per
  PG-4 §-heading-existence convention, parenthetical-descriptor form used where the
  enclosing heading is broad. Option (b) `§Phase Plan Rationale` chosen for Phase Plan
  sites (most specific actual heading). Option (a) `§Out of Scope (Explicit Non-Goals)`
  chosen for the non-goals site (brief §Out of Scope is the actual heading; parenthetical
  descriptor preserves the original semantic intent). Option parenthetical-descriptor
  `§Scope (Phase 4 — PostToolUse revisit note)` chosen for the Phase 4 note site;
  `§Phase Plan Rationale (Phase 2/3 objectives)` for the two Phase-Plan-Phase-N sites.
  Five corrections in this file: §Scope header, §P2-1 analysis, §P2-2 analysis,
  §P3-1 analysis, §P3-2 Verdict.

- Comprehensive expanded-scope PG-4 sweep (F-R53-adv-2 companion): brief.md §-anchors
  verified across all 7 architecture spec files. All brief §-anchor citations now resolve
  to actual headings in product-brief.md. No additional SS-*.md mis-anchors found beyond
  those captured in this burst.

- D-042 CASCADE: SS-core-types-and-abi.md bumped from v1.2.6 to v1.2.7 in this burst
  (F-R53-adv-3/4 co-edits). One body citation in this file updated at §P2-1 analysis
  session_id prose. dtu-assessment.md bumped from v1.5 to v1.6 (D-042 cascade from
  SS-core-types-and-abi.md v1.2.7 bump). Three body citations in this file updated:
  §P2-1 analysis session_id, §P2-2 Verdict pid, §P2-2 Verdict join-key.

v1.2.8 changes (round-52.2 F-R52R-1 D-042 cascade + F-R52R-2 §Trace reorder + PG-D042-DTU-SCOPE companion):

- F-R52R-1 RESOLVED (LOW NEW — D-042 incomplete cascade): Three body citations of
  `dtu-assessment.md v1.4` updated to `v1.5` in §P2-1 analysis session_id prose,
  §P2-2 Verdict pid prose, and §P2-2 Verdict join-key sentence.
  dtu-assessment.md was bumped to v1.5 in round-52.1 (D-042 cascade from
  SS-core-types-and-abi.md v1.2.5 → v1.2.6). The R52.1 D-042 primary grep used
  `grep -rn "SS-[a-z-]*\.md v"` — matching only SS-prefixed filenames — and did not
  catch `dtu-assessment.md` citations. Root cause is insufficient D-042 grep scope:
  the sibling pattern `grep -rn "dtu-assessment\.md v" .factory/specs/` was missing
  from the D-042 recipe. PG-D042-DTU-SCOPE codified in SS-conventions-anti-patterns.md
  v1.21 §Schema-Fact Citation Convention (this burst) adds the two missing sibling
  patterns to the canonical D-042 recipe.

- F-R52R-2 RESOLVED (LOW PRE-EXISTING — §Trace ordering anomaly): §Trace entries were
  in order v1.2.7, v1.2.6, v1.2.5, v1.2.2, v1.2.4, v1.2.3 — v1.2.2 appeared before
  v1.2.4 and v1.2.3, violating strict descending semantic-version order. No content
  within entries changed; only relative position. Corrected order: v1.2.7, v1.2.6,
  v1.2.5, v1.2.4, v1.2.3, v1.2.2.

- M-TRACE-ORDERING sweep result (round-52.2): all other spec files CLEAN. Only
  this file had a §Trace ordering anomaly; corrected by F-R52R-2 above.

v1.2.7 changes (round-52.1 PG-3-TRACE-NEW-ENTRY sweep + D-042 cascade):

- F-R52-cons-1 COMPANION (PG-3-TRACE-NEW-ENTRY sweep): §Trace v1.2.6 D-042 entry contained
  bare positional L-number tokens `(L55)`, `(L57)`, `(L73)` identifying where in this file's
  body the version citations were updated. PG-3 §Trace-prose sub-rule forbids current-state
  L-number pinpoints in §Trace prose. Tokens dropped; the section-name descriptions `§P2-1
  analysis session_id prose`, `§P2-2 Verdict pid prose`, and `§P2-2 Verdict join-key sentence`
  are sufficient for navigation.

- D-042 CITATION REFRESH (round-52.1 cascade from SS-core-types-and-abi.md v1.2.5 → v1.2.6
  bump): `SS-core-types-and-abi.md v1.2.5` → `v1.2.6` in §P2-1 analysis session_id prose.
  SS-core-types-and-abi.md was bumped from v1.2.5 to v1.2.6 in round-52.1 PG-3-TRACE-NEW-ENTRY
  sweep (bare L-number in §Trace v1.2.5 entry removed). D-042 full-scope grep surfaced this
  body citation site.

v1.2.6 changes (round-51.1 PG-4 §-heading-existence sweep + D-042 citation refresh):

- F-R51-adv-1 COMPANION (PG-4 sweep): §Item P4-3 body cited
  `SS-deps-pin-manifest.md §Phase 4 Additions` — no heading "Phase 4 Additions" exists
  in SS-deps-pin-manifest.md. Actual heading is `### Phase 4 — Federation, MCP Bridge`.
  Corrected in place. Context preserved: the oauth2 5.x reference is accurate; only the
  §-anchor was wrong.

- D-042 CITATION REFRESH (round-51.1 cascades): Two live body citations updated:
  (1) `dtu-assessment.md v1.3` → `v1.4` in §P2-1 analysis session_id prose,
  §P2-2 Verdict pid prose, and §P2-2 Verdict join-key sentence.
  dtu-assessment.md was bumped from v1.3 to v1.4 in this same burst (D-042 cascade from
  SS-core-types-and-abi.md v1.2.4 → v1.2.5 bump).
  (2) `SS-core-types-and-abi.md v1.2.4` → `v1.2.5` in §P2-1 analysis session_id prose.
  SS-core-types-and-abi.md was bumped in the PG-4 sweep companion edit.

v1.2.5 changes (round-49 D-042 citation refresh):

- D-042 CITATION REFRESH: Two sets of stale version citations updated in this file:
  (1) SS-core-types-and-abi.md v1.2.3 → v1.2.4 in §P2-1 analysis session_id prose.
  SS-core-types-and-abi.md was bumped from v1.2.3 to v1.2.4 in round-49 F-R48-adv-2
  fix; this file's §P2-1 analysis lagged by one version. The §Trace v1.2.4 entry's
  co-authoritative-sources reference ("dtu-assessment.md v1.2 and SS-core-types-and-abi.md
  v1.2.3") is left as a historical record of what was cited when the fix was made, with an
  inline parenthetical noting the subsequent bump.
  (2) dtu-assessment.md v1.2 → v1.3 in §P2-1 analysis session_id prose (line 55),
  §P2-2 Verdict pid prose (line 57), and §P2-2 Verdict join-key sentence (line 73).
  dtu-assessment.md was bumped from v1.2 to v1.3 in this same round-49 burst (D-042
  citation refresh for SS-core-types-and-abi.md version); this file lagged.
  D-042 full-scope grep (`.factory/specs/` recursive) surfaced all five sites.

v1.2.4 changes (round-47 fix F-R46-1 HIGH — schema-fact citation correction):

- F-R46-1 RESOLVED (HIGH — adversary finding): P2-1 analysis at the `session_id` bullet
  contained a factually-false claim: "already present in all 5 hook body schemas per DTU
  endpoint matrix." The gene-source BC-HOOK-007 endpoint matrix (the original DTU matrix
  in dtu-assessment.md v1.1) did NOT include session_id on PreToolUse or Notification.
  The claim was simultaneously true for monocle's canonical schema (session_id IS in all 5
  monocle-canonical structs per SS-core-types-and-abi.md v1.2.4 §Non-Exhaustive Inner Structs)
  and false as stated (the DTU matrix it cited was the gene-source-only form, not the
  monocle-canonical form). Fix: (1) dtu-assessment.md v1.2 splits the endpoint matrix into
  two columns — gene-source canonical vs monocle-canonical; (2) P2-1 analysis citation
  updated to reference the monocle-canonical column with explicit attribution to
  dtu-assessment.md v1.2 and SS-core-types-and-abi.md (v1.2.3 at time of this fix;
  subsequently bumped to v1.2.4 in round-49; subsequently bumped to v1.2.5 in round-51.1)
  as co-authoritative sources.
  PG-1 propagation: the PG-1 schema-fact citation convention (SS-conventions-anti-patterns.md
  v1.14 §Schema-Fact Citation Convention) was applied to two additional uncited schema-fact
  claims in this document discovered during the PG-1 re-validation grep:
  - P2-1 analysis bullet "The pid (already present in all 5 hook schemas)": updated to
    cite monocle-canonical column of dtu-assessment.md v1.2.
  - P2-2 verdict "join key (session_id) is already in all 5 hook schemas": updated to
    cite monocle-canonical column of dtu-assessment.md v1.2.
  Root-cause class: schema-fact citation drift — a factual claim in one doc about another
  doc's matrix content became false when the matrix was the gene-source-only form but
  monocle had extended the schema (EX-2). Covered by D-042 PG-1 extension (new grep anchor
  in dtu-assessment.md v1.2 §Schema-fact grep anchor). Re-validation grep:
  ```
  grep -rn "session_id.*all.*hook\|all.*hook.*session_id\|present in all 5\|in all 5 hook" .factory/specs/
  ```

v1.2.3 changes (round-43 fix D-042 scope correction + O-R42-1 mitigation):

- F-R42-cons-1 ROOT-CAUSE CLOSURE (6th recurrence of cross-artifact version-citation
  staleness META-pattern): product-brief.md line 250 cited SS-engine-module.md v1.1.10
  (stale; current v1.1.11). Root cause traced to D-042 documented grep scope:
  `grep -rn "SS-[name].md v" .factory/specs/architecture/` — this pattern excluded
  `.factory/specs/product-brief.md`, `.factory/specs/dtu-assessment.md`,
  `.factory/specs/research/domain-monocle-vision-synthesis.md`, and any other
  artifacts one directory level above the `architecture/` subtree. Confirmed two
  recurrences with brief as the stale artifact: round-32 (SS-engine-module.md v1.1.5
  in brief) and round-42 (SS-engine-module.md v1.1.10 in brief). The architecture/
  scope gave false confidence that all citation sites had been swept.

  D-042 WORKFLOW RULE (corrected): The D-042 manual workflow rule documented in this
  §Trace now uses the broader `.factory/specs/` scope. Two complementary grep patterns
  are defined:

  **Primary pattern (fast — strict form, exact doc-name + version):**
  ```
  grep -rn "SS-[a-z-]*\.md v" .factory/specs/
  ```
  Use before every SS-* version bump to enumerate all citation sites across the full
  spec tree. This pattern matches "SS-foo-bar.md v1.2.3" forms directly. Produces
  few false positives. Run this first.

  **Secondary pattern (thorough — permissive form, anchor-tolerant):**
  ```
  grep -rn "SS-[a-z-]*\.md.*v[0-9]" .factory/specs/
  ```
  Use for pre-gate sweeps and after version bumps as a complementary check. Allows
  arbitrary text between the document name and the version reference, catching forms
  like "SS-daemon-lifecycle.md §HookEventRecord at v1.0.5" where a section anchor
  intervenes before the version number. Produces more false positives (matches any
  line with an SS-*.md reference followed by any text containing v+digit); filter
  results manually. O-R42-1 mitigation: this pattern closes the case where intervening
  anchor text defeats the strict form.

  Recurrence history (6 confirmed): R26, R32, R36, R38, R40, R42. All six involved
  a citation in one artifact becoming stale when the referenced SS-* document was
  bumped in another burst without running the sweep. Correct sweep cadence:
  (1) run PRIMARY pattern before each SS-* version bump, (2) run SECONDARY pattern
  before each gate (pre-Phase-1 final gate, wave gates). Sweep must cover
  `.factory/specs/` recursively — not `.factory/specs/architecture/` only.

v1.2.2 changes (round-39 fix F-R38-2 MEDIUM — 4th recurrence META-pattern):

- F-R38-2 RESOLVED (MEDIUM — adversary finding): Three citations of
  `SS-daemon-lifecycle.md v1.0.3` were stale; current version is v1.0.6. Sites: FC-01
  table row (Phase 1 Spec Change column + Disposition column), FC-06 table row (same
  columns), and Verdict bullet listing the two FC items locked into that document.
  All three replaced with `SS-daemon-lifecycle.md v1.0.6`. Full sweep performed:
  all other SS-* version citations in this file (SS-engine-module.md v1.1, v1.1.4,
  v1.1.5 at lines 257–259) are historical §Trace narrative pinpoints (the version at
  which BCs were added), not cross-artifact current-version pointers; they are
  correct as written. No other stale citations found.
  META-pattern note (4th recurrence): cross-artifact version-citation staleness
  continues to recur across bursts. Root cause: SS-* docs are updated incrementally
  but citation sites in other docs are not enumerated at update time. Mitigation:
  `grep -rn "SS-daemon-lifecycle.md v" .factory/specs/architecture/` before any
  SS-daemon-lifecycle.md version bump to enumerate all citation sites in one pass.
  NOTE: the grep scope above (.factory/specs/architecture/) was defective — see
  v1.2.3 above for the corrected D-042 workflow rule.

**§Trace v1.2.14** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `section` field corrected from `"forward-compat"` → `"forward-compatibility"`
  (full, unambiguous section name matching filename per template convention).
- NORMATIVE: `subsystem` corrected from `"forward-compat"` → `cross-cutting` (canonical
  designation per ARCH-INDEX.md §Cross-Cutting Files; forward-compatibility is not a runtime
  subsystem, it is a cross-cutting constraint layer over all subsystems).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history
  string; ARCH-INDEX.md created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: `document_type` already `architecture-section` — no change required (audit §10
  confirmed PASS for forward-compatibility document_type).
- INFORMATIONAL: Version bump 1.2.13 → 1.2.14 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §10 (SS-forward-compat).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.
