---
document_type: adr
adr_id: ADR-0006
status: accepted
date: 2026-05-27
subsystems_affected: ["SS-06"]
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.0"
producer: implementer
phase: phase-3-wave-4
timestamp: 2026-05-27T00:00:00Z
inputs: [SS-tui.md, SS-core-types-and-abi.md, behavioral-contracts/BC-2.06.001.md, stories/S-024.md]
input-hash: "s024-pr20"
traces_to: "ADR-0004 §Alternatives Considered; BC-2.06.001 INV-1; AC-013; PR #20 review finding HIGH"
project: monocle
---

# ADR-0006: `AppMode` Exhaustive by Design — BC-TYPES-001 Exemption

## Status

Accepted

## Date

2026-05-27

## Context

`BC-2.02.003` (BC-TYPES-001) requires all public enums in `monocle-core` to carry
`#[non_exhaustive]` unless exempted by an ADR. ADR-0004 documents two exemptions:
`Phase1Permission` and `ClaudeCodeTool`.

ADR-0004 §Alternatives Considered explicitly evaluated `AppMode` and concluded:
> "no exemption needed; `AppMode` is not subject to external-consumer exhaustion
> concerns. If Phase 3 or 4 requires a new `AppMode` variant, that is an in-workspace
> change with full match-arm coverage enforced by the compiler — a new ADR would be
> produced at that time if an exemption becomes relevant."

S-024 (BC-2.06.001, AC-013) introduces the four-variant `AppMode` enum as part of
the TUI core types story. `monocle-tui` consumes `AppMode` via exhaustive match
statements to dispatch rendering and keybinding logic. This is the "Phase 3 scenario"
described by ADR-0004: an in-workspace consumer of `AppMode` now relies on compile-time
exhaustiveness enforcement.

BC-2.06.001 INV-1 explicitly states:
> "AppMode is NOT #[non_exhaustive]. Match arms must be exhaustive without `_`."

This ADR formalizes the exemption as required by BC-TYPES-001 extension protocol.

## Decision

`AppMode` is **exhaustive** and `#[non_exhaustive]` is **forbidden** on it.

This ADR extends the ADR-0004 EXEMPT list from 2 entries to 3. The `enum_audit.rs`
harness (VP-013 Test Vector 13.d) tracks the count at `EXEMPT_COUNT_FROM_ADR_0004 = 3`.

## Rationale

### `AppMode` — exhaustive by compile-time safety mechanism

`AppMode` is defined in `monocle-core` and consumed exclusively by first-party crates
(`monocle-tui`, `monocle-runtime`) within the same Cargo workspace. It is never exposed
to external consumers or third-party plugin code.

`monocle-tui` renders different UI layouts depending on the active `AppMode`. Key
dispatch in `resolve_binding()` uses `AppModeTag::from_mode()` with an exhaustive match
over `AppMode` variants. If a new variant is added without updating every match arm,
the Rust compiler rejects the workspace build — this is the intended safety mechanism.

Marking `AppMode` `#[non_exhaustive]` would allow downstream crates to use wildcard
arms (`_ => ...`), silently routing unhandled variants to a default case. This would
hide unhandled `AppMode` variants at compile time, defeating the safety mechanism that
exhaustive matching provides.

The four variants — `Dashboard`, `Filtering`, `Overlay`, `Fullscreen` — model monocle's
complete application mode space. Future variants are in-workspace architectural decisions
that require matching logic in both the renderer and the keybinding resolver. Compile
errors on new variants are features, not bugs.

### Distinction from `Phase1Permission` and `ClaudeCodeTool`

All three exempted enums are exhaustive for first-party correctness reasons:
- `Phase1Permission` — exhaustive because every permission decision type must have an
  explicit handler in the hook-response path.
- `ClaudeCodeTool` — exhaustive because named variants encode deliberate permission
  semantics that should be visible to code reviewers.
- `AppMode` — exhaustive because every UI rendering path and keybinding dispatch path
  must explicitly handle each mode; silent wildcard routing is a rendering bug.

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Apply `#[non_exhaustive]` to `AppMode` | Allows `monocle-tui` to use wildcard arms in mode-dispatching matches. Unhandled `AppMode` variants would silently fall through to the wildcard, producing incorrect rendering or keybinding behavior at runtime instead of a compile error. This is the opposite of the intended safety property. |
| Keep `AppMode` `#[non_exhaustive]` and use a mode-tag enum for all dispatch | `AppModeTag` (a hash-compatible discriminant enum) already exists for `HashMap` keying. But the TUI renderer needs full `AppMode` values, not tags. A `#[non_exhaustive]` `AppMode` with a mandatory wildcard arm in the renderer would silently produce incorrect UI for any new variant added. |
| Amend ADR-0004 instead of producing a new ADR | ADR-0004 §Consequences states: "Any new exemption from BC-TYPES-001 requires a new ADR." Amending ADR-0004 would conflate Phase 1 permission enums with the S-024 TUI enum, reducing traceability. ADR-0006 is the correct separation-of-concerns approach. |

## Consequences

### Immediate (S-024 / Wave 4)

- `monocle-core::tui::state::AppMode` is declared WITHOUT `#[non_exhaustive]`.
- `AppMode` is added to the `EXEMPT` list in `enum_audit.rs` (VP-013).
- `EXEMPT_COUNT_FROM_ADR_0004` updated to 3.
- `AppModeTag` (the discriminant-only HashMap key enum) retains `#[non_exhaustive]`
  per BC-2.02.003 default — it is a different type with different extensibility semantics.

### Future waves

If a new `AppMode` variant is required (e.g., `AppMode::Settings` in Wave 6):
1. Produce an ADR amending or superseding ADR-0006, documenting the new variant.
2. Add the variant to `AppMode`.
3. Update all match arms in `monocle-tui` and `monocle-runtime`.
4. The compiler will reject the build until all match arms are updated — no silent routing.

## Source / Origin

- **ADR-0004 §Alternatives Considered:** Anticipated this exemption and directed that
  a new ADR be produced when the scenario materializes.
- **BC-2.06.001 INV-1:** Explicit behavioral contract requiring `AppMode` to be exhaustive.
- **S-024 AC-013:** Story acceptance criterion requiring no `#[non_exhaustive]` on `AppMode`.
- **PR #20 review finding (HIGH):** Reviewer identified that the `EXEMPT` extension
  lacked an ADR per ADR-0004 §Extension strategy protocol.
