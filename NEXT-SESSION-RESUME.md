# monocle — Resume From Here (D-251, 2026-06-03)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block for the full checkpoint).

---

## Status at Pause

The D-236 control-center pivot has been executed through:

- Vision revised and approved (domain-monocle-vision-synthesis.md v2.2, D-238, Joshua Magady).
- Brief delta complete (product-brief.md v2.0.1, validate-brief VALID).
- Architecture delta complete — 3 ADRs + 5 subsystem specs (D-239, D-240).
- 25 new v1A behavioral contracts authored + PRD v1.28.1 + 5 holdout scenarios (D-241).
- 9 adversarial passes complete on the full spec package (D-242 through D-250).

develop branch is UNCHANGED at 8bc22a5. No production code has been written for v1A.
All pivot work lives on the factory-artifacts branch under `.factory/`.

---

## Next Action: Adversarial Pass 10

The Phase-1d adversarial spec convergence is in progress. The human's directive is
strict 3-consecutive-clean passes (zero Critical + zero Important).

- Finding trajectory: Critical 5,5,4,1,2,2,0,0,0 / Important 8,6,9,4,4,2,4,2,1
- Passes 7/8/9 are all zero-Critical. Pass 9 had 1 Important (resolved, D-250).
- Consecutive-clean counter = 0. Need 3 to declare convergence.
- Pass 10 is the next dispatch (candidate 1-of-3).

Do NOT accept fewer than 3 consecutive clean passes. Do NOT resume Phase 4-7 of
the old observe-only scope. See `.factory/STATE.md next_session_resume_protocol`
for the complete convergence-loop procedure.

---

## Spec Package (feed to adversary for Pass 10+)

| Document | Version |
|----------|---------|
| domain-monocle-vision-synthesis.md | v2.2 (APPROVED) |
| product-brief.md | v2.0.1 |
| prd.md | v1.28.1 |
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.15.0 |
| SS-session-manager | v1.7.0 |
| SS-embedded-pty | v1.3.0 |
| SS-engine-module-v2-delta | v1.1.0 |
| SS-daemon-wiring-v2-delta | v1.5.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.0 |
| ADR-0009 | v1.0.0 |
| ADR-0010 | v1.5.0 |
| ADR-0011 | v1.1.0 |
| BC-INDEX | v1.39 (138 BCs; 25 new v1A) |
| EVAL-INDEX | v1.15 (29 scenarios; HS-EXP-011..015 new) |
| version-pin-registry.yaml | source of truth for all pins |

---

## Remaining Tasks (in order)

1. Finish Phase-1d convergence: Pass 10/11/12+ until 3 consecutive clean (counter = 0 now).
2. Human spec-package approval gate (run check-input-drift first; present review questions;
   gate items: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE).
3. Phase 2 story decomposition (vsdd-factory:story-writer): v1A delta into stories + waves;
   resolve all S-TBD anchors in 25 BCs + holdout stories_tested fields.
4. VP authoring (vsdd-factory:architect) — deferred to formal-hardening (VP-TBD pattern).
5. Pre-Phase-3: DTU clone check (S-DTU-001 fidelity 1.0 — D-234); CI/CD verification.
6. Phase 3 TDD implementation of v1A stories (wave gates). v1B stories authored later.

---

## Parked Human Items (required before v1A story wave, not before convergence)

- CC-TUITERM-WIP-SIGNOFF: tui-term 0.3.4 WIP-upstream risk acceptance (ADR-0011 §O2).
- CC-GLOBAL-MOUSE-CAPTURE: approval if a future story needs clickable monocle panels.
- v1B: embedded-terminal→overlay pre-emption needs human ratification before BC authoring.

---

## Full Checkpoint

See `.factory/STATE.md` block `next_session_resume_protocol` (version 7.02, D-251) for:

- The complete convergence-loop procedure (Steps A/B/C + commit rules + cycle checklist).
- Ratified decisions (persistence model, keyboard fidelity, PTY stack, schema_version 3, etc.).
- All codified lessons (registry atomicity, propagation-closure, anchor-resolution, etc.).
- The durable_task_register for non-blocking open items.
- Already-built substrate inventory (1514 tests, 9 workspace crates, daemon, TUI, IPC).
