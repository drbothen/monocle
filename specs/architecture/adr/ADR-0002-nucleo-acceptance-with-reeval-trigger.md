---
document_type: adr
adr_id: ADR-0002
status: accepted
date: 2026-05-12
subsystems_affected: []
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.4"
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-18T05:30:00Z
inputs: [SS-deps-pin-manifest.md, ../tech-debt-register.md, ../plans/production-grade-reaudit.md]
input-hash: "df65a05"
traces_to: "TD-001 retirement; adversary re-audit 0bd4ba9 §Top 8 CRITICAL/IMPORTANT item 5; canonical principle CLAUDE.md commit 3366d58 §Rule 3"
project: monocle
---

# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger

## Status

Accepted

## Context

monocle's session/customization filter panel requires a fuzzy matcher. `nucleo 0.5` is
the helix-editor team's fuzzy matcher, implemented in Rust with SIMD acceleration.
It is the most capable fuzzy matcher in the Rust ecosystem at this quality tier.

However, nucleo's upstream repository has been dormant since 2024-04-02 — the
helix-editor team shifted focus away from active nucleo maintenance. As of
2026-05-12, there are no open RUSTSEC advisories against nucleo 0.5 or any
of its transitive dependencies. The crate is functionally complete for Phase 1
requirements (session list filtering, customization filter panel).

TD-001 in the tech-debt-register captured this as a P1 deferred item with a
"Phase 2 re-eval" due date and no story anchor. Under the canonical principle
(CLAUDE.md §Rule 3), that registration pattern is forbidden — the tech-debt
register is for human-directed deferrals only, not AI-discovered deferrals without
a concrete story anchor. This ADR is the production-grade resolution: an explicit
architectural acceptance decision with a concrete re-eval trigger, retiring TD-001.

## Decision

**Accept nucleo 0.5 for the Phase 1 session/customization filter panel. Do NOT
migrate proactively.**

The `monocle-static` crate (Phase 2) and `monocle-tui` crate (Phase 1) use
`nucleo 0.5` as the fuzzy matcher. The dependency is pinned via caret pin
(`nucleo = "^0.5"`) in the workspace `Cargo.toml`.

## Rationale

- **Functionality intact**: nucleo 0.5 meets Phase 1 filter requirements without
  modification. No feature gap exists against the Phase 1 session-list and
  customization-filter panel behavioral contracts.
- **No security advisories**: as of 2026-05-12, `cargo audit` reports no RUSTSEC
  advisories against nucleo 0.5 or its transitive dependency chain. The weekly CI
  `cargo audit --json` scheduled run will surface any future advisory immediately.
- **Migration cost non-zero**: the leading alternatives have meaningfully different
  API surfaces. `nucleo-picker 0.11` is a TUI-focused fork with a different API.
  `frizbee 0.9` is SIMD-based with a distinct concurrency model and experimental
  status. Proactive migration to either alternative introduces API churn and
  integration risk without offsetting functional gain.
- **Proactive migration is over-engineering**: migrating a functionally correct,
  advisory-free dependency solely on the basis of upstream inactivity — without a
  concrete defect, advisory, or feature gap — is an MVP-shaped risk rationalization
  running in reverse. The production-grade default is to ship with what works and
  trigger migration on concrete evidence.

## Re-eval Trigger

**If ANY of the following occurs, a fresh architecture review is mandatory and a
new ADR-NNNN-nucleo-migration must be produced before any code change:**

(a) A RUSTSEC advisory is filed against `nucleo 0.5` or any of its transitive
    dependencies (detected by weekly `cargo audit --json` CI scheduled run).

(b) Phase 2 or later maintenance requires a bug fix or new feature in the fuzzy
    matcher AND the nucleo upstream remains inactive (no commits within 90 days of
    the need arising).

(c) `cargo audit` reports a new advisory affecting the nucleo dependency chain in
    the weekly scheduled CI run (superset of condition (a); included for clarity).

(d) The helix-editor project drops nucleo as a dependency, migrates to an
    alternative, or explicitly deprecates the crate with a migration guide.

None of these conditions are currently true as of 2026-05-12.

## Alternatives Considered

| Alternative | Version | Status | Rejection Rationale |
|------------|---------|--------|---------------------|
| `nucleo-picker` | 0.11 | Active | TUI-specific fork; different API surface from nucleo 0.5; small community; migration cost non-zero; no functional gain for Phase 1 |
| `frizbee` | 0.9 | Experimental | SIMD-based; concurrency model distinct from nucleo; experimental status; no production monocle usage data |
| `fuzzy-matcher` | 0.3 | Maintained | Older, simpler; no SIMD acceleration; adequate for Phase 1 but no functional gain over nucleo to justify switching from an already-pinned dep |
| Rolling our own | — | Out of scope | No implementation budget in Phase 1; no gene-source evidence that custom matcher is needed |

## Consequences

### Positive

- Phase 1 ships with a proven, functionally complete fuzzy matcher.
- No migration churn in Phase 1 or Phase 2 baseline.
- Concrete re-eval trigger prevents the "nucleo inactivity" concern from becoming
  either a forgotten deferral or an unnecessary migration distraction.

### Negative / Trade-offs

- If nucleo upstream remains inactive and a Phase 2+ bug surfaces, migration cost
  is paid at that point. This is the correct trade-off: pay migration cost only
  when there is a concrete reason, not preemptively.
- If a RUSTSEC advisory lands on nucleo, the weekly CI run detects it within 7 days
  and blocks merge until either a patch is available or risk-acceptance is filed.

### Status as of 2026-05-12

Pre-implementation (Phase 1 deliverable). Decision accepted during pre-phase-1
architecture remediation burst. nucleo 0.5 pinned in `SS-deps-pin-manifest.md`;
no code shipped yet.

## Supersedes

**Tech-debt-register entry TD-001** — retired by this ADR. TD-001 was introduced as
an AI-discovered deferral without a concrete story anchor, violating the canonical
principle §Rule 3. The production-grade resolution is this architectural acceptance
decision, not an open tech-debt item.

## Source / Origin

- **Tech-debt-register**: `/Users/jmagady/Dev/monocle/.factory/tech-debt-register.md` TD-001 (retired)
- **Adversary re-audit**: `/Users/jmagady/Dev/monocle/.factory/plans/production-grade-reaudit.md` §Top 8 CRITICAL/IMPORTANT item 5
- **Canonical principle**: `CLAUDE.md` §Rule 3 (AI-discovered deferrals must be fixed in-scope, not registered)
- **Dependency manifest**: `SS-deps-pin-manifest.md` nucleo row (v1.1.17; relative to `.factory/specs/architecture/`)

## §Trace

**§Trace v1.0.3** (2026-05-18T01:00:00Z) — F-R108-10 inputs path fix (Round 7C):
- NORMATIVE (F-R108-10 HIGH): frontmatter `inputs:` entries for `tech-debt-register.md`
  and `plans/production-grade-reaudit.md` did not resolve from the ADR's location context
  (`.factory/specs/architecture/adr/`). Both files live one level above `specs/`:
  `tech-debt-register.md` → `.factory/tech-debt-register.md`;
  `plans/production-grade-reaudit.md` → `.factory/plans/production-grade-reaudit.md`.
  Fix: prepended `../` to each path so they resolve correctly relative to `specs/` anchor.
  This was a non-closure of F-R107-9 — the path defect was flagged but not corrected in Round 6.
- SE-17c BEFORE: `inputs: [SS-deps-pin-manifest.md, tech-debt-register.md, plans/production-grade-reaudit.md]`
- SE-17c AFTER:  `inputs: [SS-deps-pin-manifest.md, ../tech-debt-register.md, ../plans/production-grade-reaudit.md]`
- SE-17f PASS: verified both corrected paths resolve to existing files on disk.
- SE-16d PASS: 2026-05-18T01:00:00Z > prior frontmatter timestamp 2026-05-17T16:30:00Z (monotonic).

**§Trace v1.0.4** (2026-05-18T05:30:00Z) — F-R110-6 stale absolute-path citation fix (Round 9A):
- NORMATIVE (F-R110-6 HIGH): §Source / Origin "Dependency manifest" entry replaced machine-local
  absolute path `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md`
  with repository-relative reference `SS-deps-pin-manifest.md` plus version pin annotation
  `v1.1.17`. Absolute machine paths are stale by construction (break on any other machine or
  directory layout). F-R109-21 (LOW) flagged the citation integrity concern; this Round 9A fix
  resolves it at the correct severity level (HIGH per production-grade principle).
- SE-17c BEFORE: `- **Dependency manifest**: \`/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md\` nucleo row`
- SE-17c AFTER:  `- **Dependency manifest**: \`SS-deps-pin-manifest.md\` nucleo row (v1.1.17; relative to \`.factory/specs/architecture/\`)`
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic).
