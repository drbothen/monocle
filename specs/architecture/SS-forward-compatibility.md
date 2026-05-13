---
document_type: architecture-section
level: L3
section: "forward-compat"
slug: "phase-2-3-4-impact-on-phase-1"
subsystem: "forward-compat"
version: "1.2.1"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T18:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0003-license-selection.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
input-hash: "[live-state]"
traces_to: "human Q-4 forward-compat scan authorization; production-grade canonical principle CLAUDE.md; FC-01..FC-06 RESOLVED pre-Phase-1 per human authorization (v1.1); v1.2.1: N16-7 stale sealed-pattern prose swept: FC-04 Disposition + Verdict paragraph updated to reflect open-trait resolution (sealing removed round-15 per Q-15-1)"
project: monocle
---

# Architecture: Phase 2/3/4 Forward-Compatibility Scan

## [Section Content]

### Scope

This artifact answers a single question: does any Phase 2, 3, or 4 requirement, as currently specified in brief v1.4.5 and vision v1.1.2, constrain a Phase 1 spec decision that has NOT yet been locked?

Phase 1 spec decisions are locked by: brief v1.4.5 (product-owner), oq-research.md (OQ-01..OQ-11, SOQ-1..SOQ-4), SS-daemon-lifecycle.md, SS-permissions-phase1.md, SS-deps-pin-manifest.md, and the three ADRs. Anything already locked in those artifacts is not subject to this scan. The scan targets decisions that are still open at the Phase 1 architecture (PRD + behavioral contracts) level.

Phase 2-4 high-level objectives per brief v1.4.5 §Phase Plan:

- **Phase 2:** customization-aware overlays, trigger-trace (`[t]` from permission overlay to defining settings.json line), expanded TUI (full AppMode state machine, 5-level binding precedence, telescope overlay), `monocle-static` crate, `redb 2.x` for transcript indexing.
- **Phase 3:** workflow plane (`monocle-workflow` crate, `VsddFactoryAdapter` promoted to WASM-loadable, `notify 8` FS watcher), `monocle-plugin-sdk` crate (WASM ABI via wasmtime 44), MSRV bump to Rust 1.92.
- **Phase 4:** `CodeMachineModule`, `russh 0.60` federation tunnel, `monocle-ipc` shared-memory ring transport variant, OTel cost/token panel, CCR integration, `rmcp 1.6` MCP bridge, `oauth2 5.x` federation auth, optional QUIC transport.

### Phase 2 Forward-Compatibility Analysis

#### Item P2-1: Trigger-trace data model and JSONL ring buffer

**Question:** Phase 2 indexes Claude Code session transcripts via `redb 2.x` to support `[t]` jump-to-source from the permission overlay. The Phase 1 daemon ingests hook events into a hybrid RAM ring + async JSONL flush (OQ-06: `<runtime_dir>/monocle-events.jsonl`, 100MB × 5 rotation). Does the Phase 1 JSONL ring format need additional metadata fields or a version field so Phase 2 can read Phase 1 history without a migration step?

**Analysis:** The JSONL ring format is defined by the Phase 1 architecture as a retention log of hook events fired against the 5 endpoints. Phase 2 trigger-trace requires correlating a permission prompt (`permission_prompt` Notification) with the settings.json rule that generated it. This correlation is done by reading the `Notification` body fields — `tool_name`, `tool_input`, `message` — and joining against the static customization tree parsed from `settings.json` by `monocle-static`. It does NOT require transcript IDs or parent message IDs from Claude Code's internal session model; monocle is explicitly observe-only and does NOT own session transcripts (brief §Explicit Non-Goals: "hook events are ephemeral ingestion signals; full transcript storage belongs to each harness's own persistence layer").

The JSONL ring is a retention log for the event ribbon panel (Phase 1) and, secondarily, as trigger-trace source material for Phase 2. For Phase 2 trigger-trace to work, each JSONL record must carry:
- The hook type (already implicit: endpoint path or event `type` field)
- The timestamp (already required for the event ribbon latency display)
- The `session_id` (already present in all 5 hook body schemas per DTU endpoint matrix)
- The `tool_name` and `tool_input` (already in `PreToolUse` and `Notification` bodies)
- The `pid` (already present in all 5 hook schemas)

No additional fields are required. The `HookArgs` struct in `monocle-core::permissions` (SS-permissions-phase1.md) already captures `tool_name`, `tool_input`, and `message`.

**Version field:** The JSONL ring needs a format version field so Phase 2 can detect Phase 1-origin records and apply the correct parser. Without this, Phase 2 `redb` indexing code must hard-code assumptions about the Phase 1 format, creating a hidden compatibility debt.

**Verdict: PHASE 1 MUST DO — add a `format_version: u32` field (value `1`) to every JSONL event record.** Cost: trivial (1 field in the serialization struct). This is a locked Phase 1 spec decision. The JSONL event record schema must be specified in the Phase 1 PRD behavioral contracts (BC-RING-NNN) with `format_version: 1` as an immutable field. Phase 2 indexing code checks this field; a Phase 1 ring with `format_version: 1` and a Phase 2 ring with (hypothetical) `format_version: 2` are distinguishable. **Severity: IMPORTANT. Owner: architect (Phase 1 PRD BC authoring).**

#### Item P2-2: Customization-context exposure for Phase 2 overlay

**Question:** Phase 2 surfaces which customization (Builtin, Global, PerContext, UserCustomCommand, SearchPrompt) is active for the session in focus. Does Phase 1 daemon need to expose a customization-context field to Phase 2 readers?

**Analysis:** The 5-level binding precedence (brief §Phase Plan Phase 2, vision §Key Abstractions `BindingSource`) is managed entirely by `monocle-tui` in Phase 2 — it is a TUI-side concern, not a daemon-side concern. The daemon's job is hook ingestion; it has no visibility into which binding resolved a keystroke. The `monocle-static` crate (Phase 2) reads `settings.json`, `CLAUDE.md`, `keybindings.json`, and hook scripts directly from the filesystem — it does not query the daemon for customization state.

The Phase 1 daemon DOES need to produce `session_id` in hook event records (already covered under P2-1 analysis) so Phase 2 can join hook events to the session that owns the customization tree. This is already present in the Phase 1 hook schema.

**Verdict: NO IMPACT.** Phase 1 daemon does not need any new customization-context field. The static plane reads customization files directly; the daemon's role is limited to hook event forwarding. The join key (`session_id`) is already in all 5 hook schemas.

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
4. `HookEvent` / hook type enum — If Phase 1 defines an enum over hook types (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit), Phase 4 may add `PostToolUse` (brief §Phase 4 notes: "revisit PostToolUse endpoint need at this point"). `#[non_exhaustive]` on a `HookType` enum would allow Phase 4 to add the variant without breaking match sites. **PHASE 1 MUST DO: apply `#[non_exhaustive]` to any `HookType` or `HookEvent` enum that Phase 1 defines, EXCEPT for the `Phase1Permission` enum which is explicitly exhaustive by design (different concern).** Severity: IMPORTANT.

**Analysis — ABI version field:**

The vision §Key Abstractions `EngineMetadata` struct already carries `hook_schema: &'static str` — a reference to the JSON schema for hook payloads. Phase 3 plugin SDK needs to know which version of `monocle-core` it was compiled against to refuse loading binaries built against an incompatible host. The standard WASM component model approach is to encode the interface version in the WIT (WebAssembly Interface Types) component interface definition, which wasmtime 44's component model supports. This is a Phase 3 architecture concern (the `monocle-plugin-sdk` crate design), not a Phase 1 concern.

However: Phase 1 `monocle-core` SHOULD declare a `MONOCLE_ABI_VERSION: u32 = 1` constant so Phase 3 can embed it in the WIT interface and refuse to load plugins compiled against a different ABI version. This is a one-line addition that has zero runtime cost and prevents a silent compatibility failure.

**Verdict on ABI version:** PHASE 1 MUST DO — declare `pub const MONOCLE_ABI_VERSION: u32 = 1;` in `monocle-core`. Severity: IMPORTANT. Owner: architect (Phase 1 PRD).

#### Item P3-2: Workflow plane evolution and factory detection data structures

**Question:** Phase 3 ingests `.factory/STATE.md` and watches for changes via `notify 8`. Phase 1 already includes factory project detection in its Success Criteria ("Detection succeeds on monocle's own `.factory/`"). Does Phase 1 factory detection need to expose data structures Phase 3 will consume?

**Analysis:** Phase 1 statically bundles `VsddFactoryAdapter` (OQ-03). The adapter is not yet a WASM module — it is compiled into the binary. Phase 3 promotes it to a WASM-loadable module implementing the same `FactoryAdapter` trait. The `FactoryAdapter` trait (vision §FactoryAdapter) is already defined in `monocle-core` (or `monocle-workflow` crate) with the full interface: `detect`, `read_state`, `on_change`. The `FactoryState` struct is already fully specified in the vision.

The Phase 1 static bundle must implement the same trait that Phase 3 will expose via WASM. If Phase 1 defines `VsddFactoryAdapter` as a struct that does NOT implement the `FactoryAdapter` trait (e.g., it is wired inline without a trait), Phase 3 has no clean extraction path.

**Verdict: PHASE 1 MUST DO — `VsddFactoryAdapter` must implement the `FactoryAdapter` trait from day one**, even though Phase 1 statically bundles it. The trait must be defined in `monocle-workflow` (or `monocle-core` if cross-crate visibility requires it) with the exact interface Phase 3 will use for WASM. This is already implied by vision §FactoryAdapter and brief §Phase Plan Phase 3, but must be an explicit Phase 1 PRD behavioral contract. Severity: CRITICAL. Owner: architect.

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

**Question:** Phase 1 uses `X-Monocle-Authorization: <token>` for local daemon access (SS-daemon-lifecycle.md §Body Size Limit: "auth_layer... X-Monocle-Authorization enforced"). Phase 4 federation requires cross-host auth (SS-deps-pin-manifest.md §Phase 4 Additions: `oauth2 5.x`). Does Phase 1 auth design need forward-compatible structure — scope claims, token versioning — so Phase 4 OAuth2 federation does not break Phase 1 callers?

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
| FC-01 | Add `format_version: u32 = 1` field to every JSONL ring event record | IMPORTANT | BC-RING-001: JSONL event record schema includes `format_version: 1` as first key; specified in SS-daemon-lifecycle.md v1.0.3 §Drain | architect | RESOLVED PRE-PHASE-1 — locked in SS-daemon-lifecycle.md v1.0.3 per human authorization |
| FC-02 | Apply `#[non_exhaustive]` to `HookType` / `HookEvent` enum in `monocle-core` | IMPORTANT | BC-TYPES-001: `#[non_exhaustive]` default for all pub enums; exemption policy documented; specified in SS-core-types-and-abi.md §Enum Extensibility | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-03 | Declare `pub const MONOCLE_ABI_VERSION: u32 = 1;` in `monocle-core` | IMPORTANT | BC-ABI-001 + BC-ABI-002: constant declared in `monocle-core::abi`; exposed via `/status`; specified in SS-core-types-and-abi.md §ABI Version Constant | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-04 | `VsddFactoryAdapter` MUST implement `FactoryAdapter` trait from Phase 1 | CRITICAL | BC-FACTORY-001 + BC-FACTORY-002: trait defined in `monocle-core::factory`; full open-trait signature (no sealed bound; see §Analysis — Sealed trait above), self-referential test specified in SS-core-types-and-abi.md §FactoryAdapter Trait | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-05 | Define `.proto` message types for all 5 hook event types in `monocle-proto` with `schema_version` field | IMPORTANT | BC-PROTO-001 + BC-PROTO-002: full HookEnvelope proto schema with field-number reservation convention; specified in SS-core-types-and-abi.md §Prost Wire Schemas | architect | RESOLVED PRE-PHASE-1 — locked in SS-core-types-and-abi.md per human authorization |
| FC-06 | Version the local auth token: `monocle-v1:<64-char-hex>` format | IMPORTANT | BC-AUTH-001 + BC-AUTH-002: token format, constant-time comparison, 401 rejection rule; specified in SS-daemon-lifecycle.md v1.0.3 §Start Sequence | architect | RESOLVED PRE-PHASE-1 — locked in SS-daemon-lifecycle.md v1.0.3 per human authorization |

No findings require REWORK-level severity. All 6 findings are resolved with
complete, production-grade spec text — not deferred, not advisory, not TODO.

### Verdict

**PHASE 1 READY** — all 6 forward-compat items resolved pre-Phase-1 per human
authorization; spec package self-contained for fresh Phase 1 context.

All six patches (FC-01 through FC-06) are locked into binding architecture
artifacts BEFORE Phase 1 PRD dispatch. Phase 1 agents operating from a fresh
context will find complete, unambiguous specs in:

- `SS-core-types-and-abi.md` — FC-02, FC-03, FC-04 (CRITICAL), FC-05
- `SS-daemon-lifecycle.md v1.0.3` — FC-01, FC-06

None of the patches changes Phase 1 delivery scope, crate count, or external
behavior. They are additions that prevent silent forward-compatibility failures
at Phase 2, 3, and 4 boundaries.

FC-04 (`VsddFactoryAdapter implements FactoryAdapter trait`) was the only CRITICAL
finding. It is resolved with a complete, open-trait specification (Round 15:
sealing removed entirely per human Q-15-1 honoring vision authority; trait now open
for plugin SDK consumption) including the full trait signature, the
`VsddFactoryAdapter` implementation skeleton, the Phase 3 extension path, and two
behavioral contracts (BC-FACTORY-001 + BC-FACTORY-002).

The product-owner (`/vsdd-factory:create-prd`) MUST load this document as an input.
The following 16 pre-staged BC IDs are RESERVED — the PRD must use these exact IDs
when formalizing the contracts with postconditions and verification harness stubs:

| BC ID | Source Artifact |
|-------|----------------|
| BC-RING-001 | SS-daemon-lifecycle.md |
| BC-ABI-001 | SS-core-types-and-abi.md |
| BC-ABI-002 | SS-core-types-and-abi.md |
| BC-TYPES-001 | SS-core-types-and-abi.md |
| BC-FACTORY-001 | SS-core-types-and-abi.md |
| BC-FACTORY-002 | SS-core-types-and-abi.md |
| BC-PROTO-001a | SS-core-types-and-abi.md |
| BC-PROTO-001b | SS-core-types-and-abi.md |
| BC-PROTO-002 | SS-core-types-and-abi.md |
| BC-AUTH-001 | SS-daemon-lifecycle.md |
| BC-AUTH-002 | SS-daemon-lifecycle.md |
| BC-LOCK-001 | SS-daemon-lifecycle.md |
| BC-ENGINE-001 | SS-engine-module.md |
| BC-ENGINE-002 | SS-engine-module.md |
| BC-ENGINE-002-ERR | SS-engine-module.md |
| BC-ENGINE-003 | SS-engine-module.md |

Notes: BC-PROTO-001 was split into BC-PROTO-001a (wire field number) and BC-PROTO-001b
(Rust struct surface) per F-FC-O004. BC-LOCK-001 added per F-FC-O001 (lock-file
`contract_version` field). BC-ENGINE-001/002/003 added per round-14 fix burst
(SS-engine-module.md v1.1; N5 BC count propagation). BC-ENGINE-002-ERR added in
SS-engine-module.md v1.1.4 (commit 563b573); pre-staging table updated in v1.1.5
(round-23 micro-fix burst).
