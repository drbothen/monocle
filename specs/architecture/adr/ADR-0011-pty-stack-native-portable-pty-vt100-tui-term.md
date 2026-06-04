---
document_type: adr
level: L3
adr_id: "ADR-0011"
title: "PTY Stack: Native portable-pty + vt100 + tui-term (Q-7 tui-term posture)"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.2.0"
timestamp: 2026-06-04T00:00:00Z
inputs:
  - research/embedded-pty-evaluation.md
  - specs/product-brief.md
  - specs/architecture/SS-deps-pin-manifest.md
  - semport/DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md
input-hash: "e2b7eab"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# ADR-0011: PTY Stack — Native portable-pty + vt100 + tui-term

## Status

Accepted — 2026-06-03 (PTY-stack selection + Q-7 tui-term vendoring posture resolution)

## Context

The embedded-pty-evaluation.md (v1.0) recommended `portable-pty 0.9.0 + vt100 0.16.2 +
tui-term 0.3.4` as the native embedded-PTY stack for monocle. The brief v2.0 and vision v2.1
ratified this stack. This ADR formalizes the decision as an ADR record and resolves the
remaining Q-7 open question: whether to vendor tui-term immediately or defer vendoring
to on-need.

### Evaluated options

**(A) Native: portable-pty + vt100 + tui-term — ADOPTED**

Evaluated in detail in `embedded-pty-evaluation.md` §3. Summary:
- `portable-pty 0.9.0`: MIT, no RUSTSEC, MSRV <1.88, actively maintained (wezterm project,
  6.7M+ downloads). Provides cross-platform PTY pair creation, child spawn, master read/write.
- `vt100 0.16.2`: MIT, no RUSTSEC, MSRV <1.88, last release July 2025 (actively maintained).
  ANSI/VT100 parser → in-memory screen state (cursor, colors, scrollback, alt-screen).
- `tui-term 0.3.4`: MIT, no RUSTSEC, MSRV 1.86 < monocle's 1.88 floor. ratatui widget
  rendering `vt100::Screen`. Depends on `ratatui-core ^0.1.0` and `ratatui-widgets ^0.3.0` —
  exactly what `ratatui 0.30.0` pins. Unifies to a single copy in the dependency graph
  (no duplicate Buffer type). Actively maintained (0.3.2 Mar 2026, 0.3.3 Mar 2026, 0.3.4
  Apr 2026).

**(B) tmux control mode — REJECTED as primary**

Documented in embedded-pty-evaluation.md §4. Adds external runtime dependency (tmux must
be installed), inherits tmux fragility (server crash → launcher failure), and pushes
terminal fidelity through a text-scraping seam. Remains a documented fallback (see
DISPOSITION-V2-CONTROL-CENTER-ROLLUP.md).

**(C) Zellij-as-library — REJECTED**

Documented in embedded-pty-evaluation.md §5. Zellij is a binary, not a consumable library;
its PTY code uses async-std + crossbeam + nix direct calls incompatible with monocle's tokio
stack. Adopt as architecture model only.

## Decision

**Adopt option (A):**

```toml
# crates/monocle-session-host/Cargo.toml (spawner; owns PTY master)
portable-pty = "0.9"
vt100        = "0.16"

# crates/monocle-tui/Cargo.toml (renderer; renders vt100::Screen)
tui-term     = "0.3"
vt100        = "0.16"
```

**Important:** Do NOT enable the `unstable` feature flag on `tui-term`. The `unstable` feature
gates a `portable-pty` spawn helper inside tui-term. monocle spawns via `portable-pty` in
`monocle-session-host`; the TUI is a display client only. Coupling the TUI to `portable-pty`
via tui-term's unstable feature would create an unwanted TUI → PTY spawn dependency.

### Q-7 Resolution: tui-term vendoring posture

tui-term 0.3.4 self-describes as "work in progress." This is a maturity caveat, not a
maintenance-risk flag (the crate is actively maintained at monthly cadence as of 2026-Q2).

**Decision: exact-pin + deferred-vendoring-on-need.**

Rationale:
- The core capability (render a `vt100::Screen` as a ratatui widget) is stable in tui-term
  0.3.x. The "work in progress" label applies to advanced features (custom scrollback rendering,
  selection highlighting) that monocle does not use in v1A.
- tui-term's surface that monocle uses is `PseudoTerminal::new(parser.screen()).render(...)`.
  This API has been stable across 0.3.2/0.3.3/0.3.4. Upgrading through review is low-cost.
- Vendoring adds maintenance overhead (tracking upstream patches, re-integrating security fixes).
  Deferred-on-need is the production-grade default when the crate is actively maintained.
- If monocle requires custom scrollback rendering, cell selection, or other features absent
  from the stable API: vendor at that point. The small surface (one widget file) makes vendoring
  trivial.

**Exact-pin contract:** `tui-term = "=0.3.4"` in `monocle-tui/Cargo.toml`. Every upgrade
requires a PR with:
1. Review of tui-term CHANGELOG for breaking changes.
2. `cargo tree -d` confirming zero duplicate `ratatui-core`/`ratatui-widgets`/`vt100` versions.
3. Full test suite green (including PTY rendering integration tests added in v1A).

**O2 — tui-term WIP risk: explicit human risk-acceptance required.**

tui-term 0.3.4 self-describes as "work in progress." This ADR accepts that risk under the
deferred-vendoring-on-need strategy and the exact-pin contract. However, production-grade
principle requires that this risk be surface explicitly to the human product owner for sign-off
before the v1A story wave begins:

> **Risk statement:** tui-term 0.3.x is self-labeled WIP. The specific monocle usage
> (`PseudoTerminal::new(parser.screen()).render(...)`) has been stable across 0.3.2–0.3.4.
> If tui-term's rendering has an unfound defect in an edge case (e.g., complex ANSI sequences,
> double-width chars, alternate screen handling), the primary mitigation is our test suite —
> we do not own the renderer. Fallback: vendor tui-term at the v1A implementation point and
> patch defects locally.
>
> **Human action:** Acknowledge this risk before v1A story wave begins. If accepted,
> sign off with comment in NEXT-SESSION-PIVOT.md or equivalent durable checkpoint.
> If not accepted, direct architect to vendor tui-term immediately as a pre-wave task.

**Cargo-init spike required before v1A implementation begins:**
Run `cargo tree -d` in the workspace after adding the three crates. Confirm:
- Single resolved version of `ratatui-core` (must be `^0.1.0`).
- Single resolved version of `ratatui-widgets` (must be `^0.3.0`).
- Single resolved version of `vt100` (must be `^0.16.2`).
Zero duplicates is a hard gate before the v1A story wave begins.

## MSRV and compatibility summary

| Crate | Version pinned | MSRV | License | RUSTSEC |
|-------|---------------|------|---------|---------|
| `portable-pty` | `"0.9"` (caret) | <1.88 | MIT | none (2026-06-03) |
| `vt100` | `"0.16"` (caret) | <1.88 | MIT | none (2026-06-03) |
| `tui-term` | `"=0.3.4"` (exact) | 1.86 | MIT | none (2026-06-03) |

Phase-1 MSRV (1.88) is unchanged. No MSRV floor increase.

## Pin policy justification

`portable-pty` and `vt100` receive caret pins (not exact) because:
- They are not on the untrusted-input deserialization path (no network bytes pass through them).
- They are not security-sensitive protocol boundaries (no TLS, auth, WASM sandbox).
- They do not meet the 9-crate exact-pin criteria in SS-deps-pin-manifest §Patch-Pinning Policy.

`tui-term` receives an exact pin because of its WIP-label maturity caveat — the exact pin
ensures monocle does not silently absorb a tui-term API change that breaks rendering. The
review gate on each upgrade upgrade compensates for the WIP status.

## ADR Cross-References

- Extends: SS-deps-pin-manifest.md (adds three crates to Phase 1 Pin Manifest; see SS-deps-pin-manifest delta in this architecture delta batch).
- Requires: SS-08 Session Manager — monocle-session-host binary (uses `portable-pty` + `vt100`; see SS-session-manager.md).
- Requires: SS-09 Embedded PTY — monocle-tui renderer (uses `tui-term` + `vt100`; see SS-embedded-pty.md).

## §Trace v1.1.0

**O2 — tui-term WIP risk: human risk-acceptance requirement added** (2026-06-03):
- The existing "WIP" label acceptance was documented but not surfaced as a human action item.
  Adversarial Pass 1 finding O2 requires the risk be explicitly presented to the human
  before the v1A story wave begins. Added §O2 risk statement + human action block.
  The architectural decision (deferred vendoring) is unchanged; only the disclosure is new.

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- ADR-0011 authored to formalize the PTY stack selection ratified at D-237 and resolve Q-7.
- Native portable-pty + vt100 + tui-term adopted; tmux and zellij-as-library rejected.
- Q-7 VERDICT: deferred-vendoring-on-need; exact-pin on tui-term; Cargo-init spike required.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).

## §Trace v1.2.0

**I13-002 sweep — ADR Cross-References SS-08/SS-09 inversion corrected** (2026-06-04T00:00:00Z):
- NORMATIVE (I13-002 sweep — IMPORTANT): §ADR Cross-References "Requires:" lines were
  inverted: SS-09 was labeled "Session Host" and SS-08 was labeled "Embedded PTY" — the
  reverse of the ARCH-INDEX Subsystem Registry authoritative names.
  Per ARCH-INDEX Subsystem Registry: SS-08 = "Session Manager" (monocle-session-host binary,
  portable-pty/vt100 usage); SS-09 = "Embedded PTY" (monocle-tui renderer, tui-term/vt100 usage).
  The original incorrect lines:
  - `Requires: SS-09 Session Host (uses portable-pty + vt100).`
  - `Requires: SS-08 Embedded PTY (uses tui-term + vt100 in monocle-tui).`
  Corrected to:
  - `Requires: SS-08 Session Manager — monocle-session-host binary (uses portable-pty + vt100; see SS-session-manager.md).`
  - `Requires: SS-09 Embedded PTY — monocle-tui renderer (uses tui-term + vt100; see SS-embedded-pty.md).`
  - SE-17c BEFORE: `Requires: SS-09 Session Host (uses portable-pty + vt100).`
  - SE-17c AFTER:  `Requires: SS-08 Session Manager — monocle-session-host binary (uses portable-pty + vt100; see SS-session-manager.md).`
  - SE-17c BEFORE: `Requires: SS-08 Embedded PTY (uses tui-term + vt100 in monocle-tui).`
  - SE-17c AFTER:  `Requires: SS-09 Embedded PTY — monocle-tui renderer (uses tui-term + vt100; see SS-embedded-pty.md).`
  Note: the code-comment inside the IPC message type `KeyInput` variant in ADR-0010 (line ~237)
  references "SS-09 §Keyboard Encoding" — this is a cross-reference to the SS-09 Embedded PTY
  subsystem doc and is CORRECT per ARCH-INDEX (SS-09 = Embedded PTY owns keyboard encoding);
  it is not a mis-anchor and requires no fix.
- SE-16d PASS: 2026-06-04T00:00:00Z > chain high-water 2026-06-03T23:00:00Z (monotonic).
