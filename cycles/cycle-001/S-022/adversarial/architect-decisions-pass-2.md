---
document_type: architect-decision
story: S-022
pass: 2
producer: vsdd-factory:architect
timestamp: 2026-05-27T00:00:00Z
findings_addressed:
  - F-S022-ADV2-HIGH-002
  - F-S022-ADV2-MED-003
---

# S-022 Architect Decisions — Adversarial Pass 2

## Context

Two adversarial findings from S-022 Pass 2 require architect adjudication before the
implementer can complete Round 3. This document records the decision for each finding
with full rationale, the chosen option, and the implementer directive.

---

## F-S022-ADV2-HIGH-002: ring_tail Fidelity Violation

### Issue Summary

`InitialState.ring_tail` is typed `Vec<HookEvent>` per BC-2.05.002 PC-2. The RAM ring
stores `HookEventRecord`. The S-022 Round 2 implementer wrote a conversion that silently
fabricates empty strings for missing fields (cwd, transcript_path, prompt, stop_reason,
notification_type, message). No WARN emitted on fidelity loss.

### Options Considered

**Option A — Extend HookEventRecord:** Add all missing fields to `HookEventRecord` so
reconstruction is lossless. Keeps BC-2.05.002 PC-2 type unchanged. Cons: ring storage
grows substantially; `prompt` and `message` fields can be up to 256 KiB each, turning
the 4096-entry RAM ring from a modest bounded structure into a potential multi-GB object;
JSONL lines bloat proportionally, approaching the 100 MiB rotation cap faster.

**Option B — Change ring_tail to Vec<HookEventRecord>:** Update BC-2.05.002 PC-2 and
SS-ipc.md `InitialState` to type `ring_tail` as `Vec<HookEventRecord>`. Ring storage
remains as designed; conversion is eliminated (pass-through). Push display logic to TUI
side (S-025). Cons: BC update required; SS-ipc.md update required; downstream TUI
rendering in S-025 targets `HookEventRecord` fields rather than `HookEvent` variants.

**Option C — Hybrid:** Add cheap fields only (`notification_type`, `message`, `stop_reason`,
`prompt`) with `Option<...>` for bulky content; keep `cwd`/`transcript_path` as
`Option<PathBuf>/Option<String>` documented-empty. Still silently drops cwd/transcript_path
on reconstruction; partial fidelity improvement only; the "no fidelity loss" invariant
remains violated for those fields.

### Decision: Option B

**Chosen:** `ring_tail: Vec<HookEventRecord>`

**Rationale:**

The RAM ring was designed as a persistence-layer cache (`HookEventRecord` is the JSONL
wire format), not a rich-event cache. The ring's contract per BC-2.04.012 PC-1 is
"zero-disk-read TUI queries" — the TUI needs a quick snapshot of recent event metadata
(hook type, session id, timestamp, tool name) for the event ribbon display (S-025).
It does not need the full event payload reconstruction for that use case.

Option A violates the ring storage contract: adding `prompt: Option<String>` and
`message: Option<String>` to `HookEventRecord` allows entries up to 256 KiB each,
turning a 4096-slot ring from a bounded ~16 MB structure (current: ~4 KB per record)
into potentially 4096 * 256 KiB = 1 GB. The 100 MiB JSONL hard cap was sized for the
current record format; adding large optional fields changes the sizing invariants. This
is not a "ring storage grows" problem; it is a "ring storage becomes unbounded" problem.

Option C provides no correctness guarantee — it remains a lossy conversion for
`cwd`/`transcript_path`, which are precisely the fields that matter for S-025's
session context display. A half-fidelity fix is not production-grade.

Option B aligns the IPC wire format with the actual storage type. The TUI in S-025
renders the event ribbon from `HookEventRecord` fields, which are sufficient:
`hook_type`, `session_id`, `timestamp_micros`, `tool_name`. For the future Phase 2
"full event detail view," the TUI can query the JSONL file directly (also `HookEventRecord`
format) — not the ring_tail, which is bounded to N events anyway.

This choice is justified by:
- **DI-001** (every hook event written to JSONL ring before ACK): the ring stores records;
  the wire format follows the ring's type.
- **BC-2.04.012 PC-1** (RAM ring = zero-disk-read queries): the RAM ring type is
  `HookEventRecord`; the IPC type should match, not require reconstruction.
- **CLAUDE.md Production-Grade Principle Rule 1** (no rationalization): Option A is an
  inversion of the ring's bounded storage contract rationalized as "keeps BC unchanged."
  The correct answer is to update the BC to match the correct architecture, not bloat the
  architecture to avoid a spec update.

**Downstream impact:**

- BC-2.05.002 PC-2 updated (see below): `ring_tail: Vec<HookEventRecord>`.
- SS-ipc.md `InitialState` variant updated to `ring_tail: Vec<HookEventRecord>`.
- `monocle-ipc` crate `types.rs` `InitialState` field type updated.
- S-025 TUI rendering targets `HookEventRecord` fields, not `HookEvent` variants.
- No ring.rs changes required.

---

## F-S022-ADV2-MED-003: monocle-core ABI Change (non_exhaustive constructors)

### Issue Summary

S-022 Round 2 added `pub fn new(...)` positional constructors to 5 `#[non_exhaustive]`
hook event inner structs: `SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`,
`NotificationEvent`, `StopEvent`. Positional `new()` constructors partially undermine the
`#[non_exhaustive]` future-additivity guarantee — adding a new field would be a breaking
change to `new()` callers. ADV-W5GATE-MED-003 tracks this as architect+implementer scope.

Note: Finding F-S022-ADV2-HIGH-002 Option B decision (ring_tail = Vec<HookEventRecord>)
eliminates the ring_tail reconstruction use case that may have motivated these constructors.
The remaining use case is hook ingestion: HTTP handler parses JSON body and constructs
`HookEvent::SessionStart(SessionStartEvent::new(...))`. This is a legitimate use case.

### Options Considered

**Option A — Ratify new() constructors + codify breaking-change discipline:** Document in
SS-conventions-anti-patterns.md that `#[non_exhaustive]` structs with `pub fn new()`
constructors require a major version bump when a new positional field is added.
Acceptability depends on the "how frequently do these structs gain new required fields?"
question. These 5 structs model the Claude Code hook protocol's wire format — new required
fields require a new Claude Code version and a monocle MSRV-equivalent bump. The protocol
is external and stable; additions are rare and intentional.

**Option B — Builder pattern:** Replace `new()` with `SessionStartEventBuilder::new()
.cwd(...).session_id(...).build()`. Restores additivity: new fields can be optional in the
builder without breaking callers. Cons: adds 5 builder types + 5*N methods to monocle-core's
public API surface; every construction site becomes multi-line; the structs have 2-4 fields
each — builder overhead is disproportionate for this size.

**Option C — `*FromRecord` conversion API:** `SessionStartEvent::from_record(record: &HookEventRecord) -> Self`. Scoped to ring_tail conversion. But with F-S022-ADV2-HIGH-002
resolved as Option B (ring_tail = Vec<HookEventRecord>), no conversion back to `HookEvent` is
needed. Option C would be added solely as a construction alternative, duplicating the
HTTP handler's JSON parsing path with a record-based path. No use case survives.

### Decision: Option A

**Chosen:** Ratify `new()` constructors + codify discipline in SS-conventions-anti-patterns.md

**Rationale:**

Option B's builder overhead is engineering theater for 2-4-field structs. The additivity
concern is real but bounded: `SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`,
`NotificationEvent`, and `StopEvent` model a specific external wire protocol (Claude Code
hook POST bodies). New fields in the protocol require:
1. A Claude Code version bump on the Claude Code side.
2. A monocle story to add handling on the monocle side.
3. A BC update authorizing the new field.

This workflow is always a two-party coordination, not a silent internal change. The
"breaking change to new() callers" risk in practice means: when we add a new required field
to `SessionStartEvent`, we also update the HTTP handler's construction site. These are the
SAME commit scope. There is no external caller of `new()` from a downstream crate —
`monocle-core` is not published to crates.io; all callers are internal workspace crates.

For internal workspace crates, a `new()` positional breakage is caught at compile time,
identified immediately, fixed in the same PR. It is not a hidden runtime failure. The
`#[non_exhaustive]` guarantee is about external crate consumers; for internal workspace
crates with workspace-level coordinated changes, the risk is adequately managed by
documenting the discipline.

Option C has no surviving use case post F-S022-ADV2-HIGH-002 Option B decision.

**Justification:**

- **ADV-W5GATE-MED-003** (durable task register): explicitly tracks as architect+implementer
  follow-up; this decision closes the architect obligation.
- **CLAUDE.md Production-Grade Principle Rule 1**: Builders for 2-4-field internal structs
  are over-engineering. The correct production-grade answer is: ratify with explicit discipline.
- **CLAUDE.md Production-Grade Principle Rule 4**: The constructors were added without architect
  routing; the architect's job is to adjudicate, not to demand revert. These constructors are
  correct; they needed a discipline document, not removal.

---

## Implementer Directives

### Directive 1 — ring_tail type change (HIGH-002)

In the S-022 worktree, make the following changes:

**`crates/monocle-ipc/src/types.rs`**
- Change `ring_tail: Vec<HookEvent>` to `ring_tail: Vec<HookEventRecord>` in the
  `ServerToClient::InitialState` variant.
- Add import: `use monocle_runtime::ring::HookEventRecord;` (or re-export path — check
  actual crate boundary; `HookEventRecord` lives in `monocle-runtime::ring`).
- Remove any `HookEvent` conversion code that fabricated fields.

**`crates/monocle-runtime/src/` (wherever InitialState is constructed — ipc_server.rs or state.rs)**
- When constructing `ServerToClient::InitialState`, set `ring_tail` to
  `ring_buffer.latest_events(N)` (returns `Vec<HookEventRecord>` directly).
- No conversion step needed.

**`crates/monocle-ipc/src/lib.rs` or wherever HookEvent was previously imported for ring_tail**
- Remove `HookEvent` import if it was only needed for the ring_tail conversion.

**Test files referencing ring_tail as Vec<HookEvent>**
- Update to construct `HookEventRecord` values via `HookEventRecord::new(...)` in test setup.
- Remove any conversion helper tests.

**CRITICAL:** Ensure no `tracing::warn!` or fabricated-fields logic survives. The fix is
type-correct pass-through; zero fabrication paths.

### Directive 2 — SS-conventions-anti-patterns.md addition (MED-003)

The architect writes the convention rule below (see §Spec Updates). The implementer's
obligation is:

- Verify the 5 `new()` constructors in `monocle-core/src/hook_events.rs` match the
  production-grade form: all required fields present, no `Default` shortcutting.
- Ensure `HookEventRecord::new()` (already in `monocle-runtime/src/ring.rs`) also
  appears in the audit table in SS-engine-module.md §Cross-Crate Constructor Audit Table.
- Do NOT add builders or change the constructor API.

---

## Spec Updates Required

### 1. BC-2.05.002 v1.0.4 — ring_tail type change

Update `.factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md`:

**Postcondition 2 change:**
- Old: `ring_tail: Vec<HookEvent>` — the last N events from the RAM ring
- New: `ring_tail: Vec<HookEventRecord>` — the last N events from the RAM ring

**Version:** `1.0.3` → `1.0.4`

**Trace entry:** document this change as F-S022-ADV2-HIGH-002 resolution.

The full PC-2 sentence becomes:
```
- `ring_tail: Vec<HookEventRecord>` — the last N events from the RAM ring
  (N defined by daemon configuration; may be empty if no events have been received
  yet). `HookEventRecord` is the canonical ring storage type (BC-2.04.012 PC-1);
  the TUI renders event ribbon display from `hook_type`, `session_id`,
  `timestamp_micros`, and `tool_name` fields.
```

The §Architecture Anchors entry should add a reference to BC-2.04.012 PC-1.

### 2. SS-ipc.md v1.7.0 — ring_tail type change

Update `.factory/specs/architecture/SS-ipc.md`:

In §Message Types §Server-to-Client Messages, change the `InitialState` variant:
```rust
InitialState {
    sessions: Vec<EnrichedSession>,
    ring_tail: Vec<HookEventRecord>,   // was Vec<HookEvent>
    overlay_stack: Vec<PermissionPromptPayload>,
    drop_counter: u64,
},
```

Add import context comment: `HookEventRecord` is defined in `monocle-runtime::ring`.

In §Connection Lifecycle §Phase 1: Connect step 4, change:
- Old: "The last N events from the RAM ring (ring tail) as `Vec<HookEvent>`."
- New: "The last N events from the RAM ring as `Vec<HookEventRecord>`."

**Version:** `1.6.0` → `1.7.0`

**Trace entry:** document this change as F-S022-ADV2-HIGH-002 resolution.

### 3. SS-conventions-anti-patterns.md addition (MED-003)

Add the following section to `.factory/specs/architecture/SS-conventions-anti-patterns.md`
after the existing §Anti-Patterns section, before §Test-Time Enforcement:

---

**Section to add:**

```markdown
## Non-Exhaustive Structs with Public Constructors

`#[non_exhaustive]` on a `pub struct` prevents external crate consumers from constructing
the struct via struct-literal syntax (`StructName { field: value, ... }`). For internal
workspace crates, `pub fn new(...)` positional constructors are an acceptable alternative.

### When public constructors are permitted

A `#[non_exhaustive] pub struct` MAY carry a `pub fn new(...)` positional constructor when:

1. The struct is an **internal workspace type** — not published to crates.io and not
   consumed by external downstream crates outside this workspace.
2. The struct models an **external wire protocol** (e.g., Claude Code hook POST body fields)
   where new required fields are a coordinated, intentional change requiring explicit BC
   revision.
3. All existing required fields are included as positional parameters. No `Default`
   substitution for required fields.

The 5 hook event inner structs (`SessionStartEvent`, `UserPromptSubmitEvent`,
`PreToolUseEvent`, `NotificationEvent`, `StopEvent`) and `HookEventRecord` meet all three
criteria and carry `new()` constructors as ratified in ADR-0006.

### Breaking-change discipline

Adding a new **required field** to a `#[non_exhaustive]` struct that carries a `pub fn new()`
constructor is a **breaking change** to the `new()` API. The following discipline applies:

- A new required field MUST be added as a positional parameter to `new()`.
- All internal call sites MUST be updated in the same PR.
- The PR MUST include a §Trace entry in the affected architecture spec documenting the
  new field, its source (Claude Code wire protocol version or monocle BC revision), and
  the rationale.
- The crate version bump follows standard semver: if the struct is in a workspace crate
  without an independent published version (all Phase 1 monocle crates), update the
  §Trace version in the owning architecture spec.

Adding a new **optional field** (`Option<T>`) to a `#[non_exhaustive]` struct with a
`new()` constructor is NOT a breaking change — the field can be initialized to `None`
in the existing `new()` body without changing the constructor signature.

### Enforcement

The `monocle-non-exhaustive-struct-audit-completeness` semgrep rule in §Semgrep Rules
above catches every `#[non_exhaustive] pub struct` and verifies it appears in the
Cross-Crate Constructor Audit Table in SS-engine-module.md. When a `new()` constructor
is added to a struct, the audit table entry MUST be updated to record the constructor
presence.

Code review MUST reject a `new()` constructor addition that:
- Is missing from the audit table.
- Has positional parameters that do not match all required fields.
- Is applied to a struct that does not meet the three criteria above (e.g., a type
  that may be published externally in Phase 4).
```

**Version:** `1.30.2` → `1.31.0`

**Trace entry:** document as ADR-0006 ratification of hook event inner struct constructors.

### 4. ADR-0006 — non_exhaustive structs with public constructors

Create `.factory/specs/architecture/adr/ADR-0006-non-exhaustive-structs-with-public-constructors.md`
(see separate file produced by this decision).

---

## Files Produced / Modified

| File | Action | Version Change |
|------|--------|---------------|
| `.factory/cycles/cycle-001/S-022/adversarial/architect-decisions-pass-2.md` | Created | N/A |
| `.factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md` | Updated | 1.0.3 → 1.0.4 |
| `.factory/specs/architecture/SS-ipc.md` | Updated | 1.6.0 → 1.7.0 |
| `.factory/specs/architecture/SS-conventions-anti-patterns.md` | Updated | 1.30.2 → 1.31.0 |
| `.factory/specs/architecture/adr/ADR-0006-non-exhaustive-structs-with-public-constructors.md` | Created | N/A |
