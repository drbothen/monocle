---
document_type: adr
adr_id: ADR-0004
status: accepted
date: 2026-05-13
subsystems_affected: ["core"]
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.4"
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-17T16:30:00Z
inputs: [SS-core-types-and-abi.md, SS-permissions-phase1.md, product-brief.md]
input-hash: "67d17bb"
traces_to: "F-FC-C001 adversary finding; F-FC-I004 adversary finding; BC-TYPES-001 exemption mechanism; SS-permissions-phase1.md §Decision; brief v1.4.7 §Scope (Public enum extensibility forward-compatibility contract)"
project: monocle
---

# ADR-0004: Exhaustive Enums — `Phase1Permission` and `ClaudeCodeTool`

## Status

Accepted

## Date

2026-05-13

## Context

`SS-core-types-and-abi.md` §Enum Extensibility establishes `#[non_exhaustive]` as the
default for all public enums in `monocle-core` (BC-TYPES-001). The general rule requires
an ADR to document any exemption from this default.

Two enums in `monocle-core` require exhaustive matching and must NOT carry
`#[non_exhaustive]`:

**`Phase1Permission`** (defined in `monocle-core::permissions`, specified in
`SS-permissions-phase1.md`): the complete set of permission decisions monocle's TUI can
produce in response to a Claude Code `permission_prompt` Notification event. Five
variants: `AllowOnce`, `AllowAlways`, `DenyOnce`, `DenyAlways`, `AskUser`.

**`ClaudeCodeTool`** (defined in `monocle-core::permissions`, specified in
`SS-permissions-phase1.md`): the set of Claude Code tools that monocle can receive
`permission_prompt` Notifications for. Fifteen named variants plus `Unknown(String)`:
`Bash`, `Read`, `Write`, `Edit`, `MultiEdit`, `Glob`, `Grep`, `LS`, `WebFetch`,
`WebSearch`, `TodoRead`, `TodoWrite`, `NotebookRead`, `NotebookEdit`, `Task`.

BC-TYPES-001 states that exemptions require an ADR. `SS-permissions-phase1.md` documents
the exhaustiveness decision for both enums but is not an ADR — it is an architecture
section document. This ADR formalizes the exemption per the BC-TYPES-001 mechanism.

Adversary finding F-FC-C001 identified that the SS-core-types-and-abi.md §Enum
Extensibility section mentioned only `Phase1Permission` in the exempt list and omitted
`ClaudeCodeTool`. Adversary finding F-FC-I004 identified that `SS-permissions-phase1.md`
is cited as the exemption source but is not an ADR. Both findings are resolved here.

## Decision

`Phase1Permission` and `ClaudeCodeTool` are **exhaustive** and `#[non_exhaustive]` is
**forbidden** on both.

This ADR is the authoritative exemption record for BC-TYPES-001. Any future exemption
from the `#[non_exhaustive]` default requires a new ADR — not a change to an
architecture section document.

## Rationale

### `Phase1Permission` — exhaustive by correctness requirement

The TUI permission dispatcher (`monocle-runtime::hook_handler::dispatch_permission`)
uses a `match` statement over `Phase1Permission` variants. Exhaustive matching is a
compile-time correctness invariant: if a new variant is added without adding match arms
at every dispatch site, the Rust compiler rejects the build. This is the desired
behavior — adding a permission decision type is an explicit architectural act with
behavioral consequences that affect Claude Code's hook-response semantics.

The five variants (`AllowOnce`, `AllowAlways`, `DenyOnce`, `DenyAlways`, `AskUser`)
directly model the Claude Code session-permission decision space. This is a closed,
well-defined set derived from the gene-source (any-context-lazyclaude hook protocol).
Claude Code's permission model has been stable since the hook protocol launched; new
permission decision types would require a corresponding Claude Code protocol change,
which is an Anthropic product decision requiring explicit monocle architectural response.

Phase 3 adds a categorically distinct `monocle-plugin-sdk::PluginPermission` enum for
WASM sandbox capabilities. These enums are orthogonal concerns and must not merge.

### `ClaudeCodeTool` — exhaustive by explicit tool-set tracking requirement

`ClaudeCodeTool` models Claude Code's tool set as monocle knows it. The `Unknown(String)`
catch-all variant handles tools introduced between monocle releases at runtime, but the
enum itself is not `#[non_exhaustive]` because adding a new named variant is a deliberate
monocle product decision that requires specifying the intended permission dispatch behavior
for that tool.

If Claude Code ships a new tool and monocle adds it to `ClaudeCodeTool` non-exhaustively,
existing `match` sites would compile with wildcard arms silently routing the new tool to
the `Unknown` handler. This is functionally correct — the `Unknown` arm exists for exactly
this case — but architecturally incorrect: the explicit named variant means monocle has
made a deliberate decision about that tool's permission semantics, which should be visible
to code reviewers and compile-time verifiable.

The `Unknown(String)` variant is the runtime safety net. The exhaustive named variants
are the specification of known tools.

### Extension strategy (Phase 2 and beyond)

When Claude Code ships a new tool:

1. Produce an ADR citing this ADR, documenting the new tool and monocle's intended
   permission dispatch behavior for it.
2. Add the variant to `ClaudeCodeTool` in `monocle-core::permissions`.
3. Add match arms at every `match tool { ... }` site in the codebase.
4. Dispatch a security-reviewer agent: new tool variants in the `AllowOnce`/`DenyOnce`
   paths may affect Claude Code's hook-abort semantics.
5. Remove (or narrow) any existing `Unknown(String)` match arm that was previously
   handling the tool at runtime.

When a new `Phase1Permission` decision type is needed (e.g., a new Claude Code permission
interaction pattern emerges):

1. Produce an ADR documenting the new decision type and its Claude Code hook-response
   semantics.
2. Add the variant to `Phase1Permission`.
3. Add match arms at every dispatch site.
4. Security-reviewer sign-off required (same as for new `ClaudeCodeTool` variants).

## Alternatives Considered

| Alternative | Rejection Rationale |
|-------------|---------------------|
| Apply `#[non_exhaustive]` to `Phase1Permission` | Breaks the compile-time correctness guarantee that every permission decision type has an explicit match arm. The `Unknown(String)` escape hatch on `ClaudeCodeTool` already handles runtime unknowns; `Phase1Permission` has no equivalent escape hatch because every permission decision type must be deliberately handled. |
| Apply `#[non_exhaustive]` to `ClaudeCodeTool` | Silently routes newly named variants through wildcard arms. Degrades the explicit intent tracking that is the point of having named variants. The `Unknown(String)` catch-all already handles the unknown-at-compile-time case. |
| Use `SS-permissions-phase1.md` as the exemption record (no ADR) | BC-TYPES-001 explicitly requires an ADR. Architecture section documents are not ADRs and do not satisfy the BC-TYPES-001 exemption mechanism. This ADR corrects that gap. |
| Add a third `#[non_exhaustive]`-exempt enum for `AppMode` | `AppMode` is first-party (defined and consumed only by `monocle-core` and `monocle-tui`, both in the same workspace). Phase 3 does not add new `AppMode` variants. No exemption needed; `AppMode` is not subject to external-consumer exhaustion concerns. If Phase 3 or 4 requires a new `AppMode` variant, that is an in-workspace change with full match-arm coverage enforced by the compiler — a new ADR would be produced at that time if an exemption becomes relevant. |

## Consequences

### Immediate (Phase 1)

- `monocle-core::permissions::Phase1Permission` is declared WITHOUT `#[non_exhaustive]`.
- `monocle-core::permissions::ClaudeCodeTool` is declared WITHOUT `#[non_exhaustive]`.
- `SS-core-types-and-abi.md` §Enum Extensibility exhaustive-enum forbidden list lists
  both enums by name with a reference to this ADR.
- BC-TYPES-001 exemption list: `Phase1Permission` and `ClaudeCodeTool` per ADR-0004.
- `cargo clippy` lint that checks public enums for `#[non_exhaustive]` must exclude
  these two enums from its scope (or the lint must be structured to check for absence
  of exemption ADR citations rather than blanket presence of the attribute).

### Phase 2+

Any new exemption from BC-TYPES-001 requires a new ADR. This ADR does not grant
blanket authority for future exemptions — each must be independently justified.

### Phase 3 interaction

`Phase1Permission` and `ClaudeCodeTool` are in `monocle-core`. Phase 3's
`monocle-plugin-sdk::PluginPermission` is a separate enum in a separate crate.
No merge, no inheritance, no shared variants. This ADR does not constrain Phase 3's
design of `PluginPermission`.

## Source / Origin

- **Adversary finding F-FC-C001:** Identified omission of `ClaudeCodeTool` from the
  exhaustive-enum forbidden list in SS-core-types-and-abi.md.
- **Adversary finding F-FC-I004:** Identified that `SS-permissions-phase1.md` is cited
  as the ADR-exemption source but is not an ADR; BC-TYPES-001 exemption requires an ADR.
- **SS-permissions-phase1.md §Decision:** Canonical definition of both enums; documents
  the exhaustiveness invariant and the extension protocol.
- **Brief v1.4.7 at time of ADR authoring §Scope (Public enum extensibility forward-compatibility contract):** Lists
  both enums as exhaustive-by-design and refers to this ADR.
- **BC-TYPES-001:** The behavioral contract requiring ADR documentation for all
  `#[non_exhaustive]` exemptions.
- **Canonical principle (CLAUDE.md):** Production-grade correctness; compile-time
  exhaustiveness enforcement is production-grade; silent wildcard routing is not.

## Amendment History

v1.0.3 changes (round-57.1 F-R57-1 PG-5 historical-anchor fix):
- F-R57-1 RESOLVED (MEDIUM content — adversary finding R57): §Source / Origin body at
  `Brief v1.4.7 §Scope (Public enum extensibility forward-compatibility contract)` failed
  PG-5 — bare version, neither current (brief at v1.4.23) nor explicitly qualified as
  historical. Fix: Form 2 historical-anchor applied. Now reads `Brief v1.4.7 at time of
  ADR authoring §Scope (...)`. The R56.1 "comprehensive PG-5 sweep" missed the ADR class
  entirely; this burst adds the explicit ADR-N sweep to the PG-5 sweep-evidence checklist.
  Note: `traces_to` frontmatter citation remains unchanged — frontmatter is exempt per
  PG-5 Option B carve-out (codified in SS-conventions-anti-patterns.md v1.25).

v1.0.2 changes (round-53.1 F-R53-adv-5 brief §-anchor fix):
- F-R53-adv-5 RESOLVED (LOW — adversary finding R53): `traces_to` frontmatter and
  §Source / Origin body cited `brief v1.4.7 §Public enum extensibility`. Brief has no
  heading named "Public enum extensibility"; the text appears only as a bold-label
  sub-bullet (`**Public enum extensibility:**`) within `## Scope`. Per PG-4
  §-heading-existence convention, corrected to `brief §Scope (Public enum extensibility
  forward-compatibility contract)` using the parenthetical-descriptor form. Two sites
  corrected: `traces_to` frontmatter and §Source / Origin body.
