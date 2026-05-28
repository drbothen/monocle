---
document_type: architectural-decision-record
adr_id: "ADR-0006"
status: accepted
producer: vsdd-factory:architect
timestamp: 2026-05-27T00:00:00Z
traces_to: architecture/SS-conventions-anti-patterns.md
decision_date: 2026-05-27
---

# ADR-0006: Non-Exhaustive Structs with Public Positional Constructors

## Context

S-022 Round 2 added `pub fn new(...)` positional constructors to 5 `#[non_exhaustive]`
hook event inner structs in `monocle-core`:

- `SessionStartEvent`
- `UserPromptSubmitEvent`
- `PreToolUseEvent`
- `NotificationEvent`
- `StopEvent`

ADV-W5GATE-MED-003 (durable task register, Wave 5 gate) flagged this as requiring
architect adjudication. The adversarial Pass 2 finding F-S022-ADV2-MED-003 routed the
question here per CLAUDE.md §Correct Agent Routing ("monocle-core ABI changes route to
architect").

The core tension: `#[non_exhaustive]` is intended to preserve future additivity — external
crate consumers cannot construct the struct via struct-literal syntax, so adding new fields
is not a breaking change for them. Adding a `pub fn new(...)` with positional parameters
partially restores that breaking-change surface: any caller of `new()` is broken if a new
required positional parameter is added.

## Decision

Accept `pub fn new(...)` positional constructors on `#[non_exhaustive]` structs that meet
ALL THREE of the following criteria:

1. **Internal workspace scope:** The struct is defined in a workspace crate that is not
   published to crates.io and is not consumed by external downstream crates outside the
   monocle workspace. All `monocle-*` crates in Phase 1 are internal-only.

2. **External protocol anchor:** The struct models an external wire protocol whose field
   additions require coordinated, intentional changes across multiple layers (Claude Code
   version bump, monocle BC revision, monocle story, and architect approval). This is not
   a struct whose fields evolve organically via refactoring.

3. **All required fields present as positional parameters.** No `Default` substitution
   for required fields in the constructor body.

The 5 hook event inner structs and `HookEventRecord` meet all three criteria. The
`pub fn new(...)` constructors are ratified as-is.

## Consequences

### Positive

- HTTP hook handler code is readable: `HookEvent::SessionStart(SessionStartEvent::new(cwd, transcript_path, session_id, pid))` is unambiguous.
- No builder boilerplate for 2-4-field structs. Builder overhead is disproportionate for
  small structs whose fields are all required.
- Compile-time breakage on new required field addition is caught in the same PR that adds
  the field — all call sites are in the same workspace, so `cargo check` catches every gap.

### Negative / Mitigated

- A future required field addition is a breaking change to `new()` callers. **Mitigation:**
  All callers are internal workspace crates. New required fields arise only from protocol
  changes (Claude Code version bump) that require monocle story scope and BC revision by
  design. The risk of an unnoticed breaking change is effectively zero in this context.
- External Phase 4 publication: if any of these types are ever exposed in a published SDK
  crate, this decision must be revisited. **Mitigation:** A Phase 4 SDK crate will require
  a new architecture review; this ADR explicitly scopes the decision to Phase 1 internal
  workspace crates.

## Alternatives Rejected

**Builder pattern:** `SessionStartEventBuilder::new().cwd(...).session_id(...).build()`.
Rejected: 5 builder types + 5*N methods for structs with 2-4 fields is engineering overhead
disproportionate to the additivity risk. All construction sites are within the workspace;
compile-time errors surface immediately.

**`FromRecord` conversion API:** `SessionStartEvent::from_record(record: &HookEventRecord) -> Self`.
Rejected: The use case that motivated this (ring_tail reconstruction) was eliminated by
ADR companion decision F-S022-ADV2-HIGH-002 (ring_tail type changed to `Vec<HookEventRecord>`,
removing the need to reconstruct `HookEvent` from ring storage). No other construction
use case benefits from a record-based API over a field-based `new()`.

**Revert constructors, use struct literal with `#[allow(clippy::exhaustive_structs)]`:**
Rejected: `#[non_exhaustive]` on hook event inner structs is correct — these types WILL
gain new optional fields over time as the Claude Code protocol evolves. Removing the
attribute to allow struct-literal construction is architecturally incorrect.

## Breaking-Change Discipline (operative rule)

When adding a new **required field** to any struct covered by this ADR:

1. Add the field as a new positional parameter to `new()`.
2. Update all call sites in the same PR.
3. Update the §Trace in the owning architecture spec (`SS-core-types-and-abi.md` for hook
   event types; `ring.rs` inline doc for `HookEventRecord`).
4. Add a BC revision if the field addition changes the wire behavior.

Adding a new **optional field** (`Option<T>`) does not break `new()` — initialize to
`None` in the constructor body. Optional fields do not need to appear as constructor
parameters unless there is a compelling reason (e.g., always populated on construction).

## Audit Table Obligation

Every struct covered by this ADR MUST appear in the Cross-Crate Constructor Audit Table
in `SS-engine-module.md`. The semgrep rule `monocle-non-exhaustive-struct-audit-completeness`
enforces this at CI time. When a new `new()` constructor is added to any
`#[non_exhaustive] pub struct`, the audit table entry MUST be updated in the same PR.

## Structs Covered at Time of Ratification

| Struct | Crate | Required fields | Constructor |
|--------|-------|----------------|-------------|
| `SessionStartEvent` | `monocle-core` | cwd, transcript_path, session_id, pid | `new(cwd, transcript_path, session_id, pid)` |
| `UserPromptSubmitEvent` | `monocle-core` | prompt, session_id, pid | `new(prompt, session_id, pid)` |
| `PreToolUseEvent` | `monocle-core` | tool_name, tool_input, session_id, pid | `new(tool_name, tool_input, session_id, pid)` |
| `NotificationEvent` | `monocle-core` | notification_type, tool_name, tool_input, message, session_id, pid | `new(notification_type, tool_name, tool_input, message, session_id, pid)` |
| `StopEvent` | `monocle-core` | stop_reason, session_id, pid | `new(stop_reason, session_id, pid)` |
| `HookEventRecord` | `monocle-runtime` | session_id, timestamp_micros, pid, hook_type, tool_name, tool_input | `new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input)` |

## References

- ADV-W5GATE-MED-003 — durable task register entry (Wave 5 gate), states "architect + implementer follow-up"
- F-S022-ADV2-MED-003 — adversarial Pass 2 finding that routed to architect
- SS-conventions-anti-patterns.md §Non-Exhaustive Structs with Public Constructors — operative convention rule derived from this ADR
- SS-engine-module.md §Cross-Crate Constructor Audit Table — enforcement mechanism
- BC-2.03.001 — `HookEvent` definition invariants
- BC-2.04.012 PC-1 — RAM ring type (`HookEventRecord`)
