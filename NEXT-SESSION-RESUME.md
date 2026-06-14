# monocle — Resume From Here (D-271..D-277, 2026-06-13)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.28, for the full checkpoint).

---

## Status at Pause

**Phase-1d adversarial spec convergence is in progress.** 35 passes complete.
Consecutive-clean counter = 0. Pass-36 is the next dispatch (clean candidate 1 of 3).

develop branch has had docs/version-pin/CI-wiring commits this session (POL-13 anchor-lint
CI wiring). No v1A production code written. All pivot spec work on factory-artifacts.

---

## This Session's Key Changes (Passes 29-35)

Pass-29 (D-271): C29-001 harness_id missing from SpawnOptions struct — errata fix (no
version bump).

Pass-30 (D-272): ADR-0006 constructor gap — SpawnOptions for_spawn_request()+
with_daemon_fields(); SpawnRecipe/SessionSnapshot/SerializedCell/PermissionPromptPayload
new() constructors; E0639 ..opts workaround documented.

Pass-31 (D-273): EngineError declared as NEW canonical v1A #[non_exhaustive] enum
(UnsupportedOperation/BinaryNotFound/InvalidPath) in SS-engine-module-v2-delta §EngineError
(new in v1A). Independent of SpawnError/PreflightError/EngineMetadataError. Inner cross-
crate match REQUIRES _=> (forward-compat, not a swallow). SS-session-manager SpawnRecipe
parity updated.

Pass-32 (D-274): session_error_to_code _=> arm documented as forward-compat (not panic);
dead anchor #engineerror-additions fixed to #engineerror-new-in-v1a; EngineError framing
changed from 'extension' to new independent type.

Pass-33 (D-275): WHOLE-CLASS dead-anchor remediation. Built scripts/check_cross_ref_anchors.py
(POL-13), wired CI + pre-commit hook. 70 dead anchors found; 42 explicit <a id> navigational
anchors added across 12 docs (no version bumps). 2 defective citations fixed. ANCHOR-LINT-TOOL
durable item CLOSED.

Pass-34 (D-276): InvalidPath null-byte detection — two-pronged: to_str().is_none() for
non-UTF-8 AND as_bytes().contains(&0) for null bytes (null is valid UTF-8, so to_str()
alone misses it). BC-2.03.005/006/007/008 updated.

Pass-35 (D-277): FIRST CLEAN (0C/0I). 3 Suggestions fixed in-scope per production-grade
principle: S35-001 split-pair rationale arithmetic; S35-002 KeyInput SessionHostDead→
attach_failed error path undocumented; S35-003 mouse Drag(MouseButton) missing arm +
Moved Ps 32→35 + full Ps/modifier table. Suggestions changed the package → counter RESET
to 0. Pass-35 CLEAN does NOT count toward the 3-clean streak.

---

## Next Action: Adversarial Pass 36

Dispatch `vsdd-factory:adversary` in a fresh context for Pass 36.
Feed the full spec package (section D of `next_session_resume_protocol` in STATE.md v7.28).

The human's directive: **strict 3 consecutive clean passes** (zero Critical + zero Important).
Do NOT accept fewer than 3. Do NOT resume Phase 4-7 of the old observe-only scope.

Tell the adversary these are CLOSED and should NOT be re-litigated:
- EngineError = NEW canonical #[non_exhaustive] enum (3 variants), independent
- ADR-0006 constructors fully documented for all 5 v1A wire structs
- Dead-anchor class = whole-class remediated + POL-13 enforced
- Two-pronged InvalidPath null-byte detection
- Mouse SGR: Drag 32/33/34; Moved Ps=35; modifier Shift=4/Alt=8/Ctrl=16
- Ordered-pair-split → immediate disconnect independent of slow-client counter

Finding trajectory summary (C/I counts per pass):
- Passes 1-6: Critical present (5/8, 5/6, 4/9, 1/4, 2/4, 2/2)
- Passes 7-28: ALL zero-Critical (22 consecutive)
- Pass 20: 0C/0I — FIRST CLEAN; Pass 21: 0C/0I — SECOND CLEAN
- Pass 22: 0C/3I — RESET counter 2→0 (sibling-BC cluster caught)
- Passes 23-28: 0C/1I each — counter stays 0
- Pass 29: 1C/0I  Pass 30: 2C/1I  Pass 31: 1C/1I  Pass 32: 0C/3I
- Pass 33: 0C/2I  Pass 34: 1C/1I  Pass 35: 0C/0I — CLEAN but S-fixes reset counter

---

## Current Spec Package Headline Versions

| Document | Version |
|----------|---------|
| domain-monocle-vision-synthesis.md | v2.2.3 (APPROVED) |
| product-brief.md | v2.0.4 |
| prd.md | v1.28.3 |
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.20.1 |
| SS-session-manager | v2.2.1 |
| SS-embedded-pty | v1.5.2 |
| SS-engine-module-v2-delta | v1.4.1 |
| SS-daemon-wiring-v2-delta | v1.9.1 |
| SS-deps-pin-manifest-v2-delta | v1.0.1 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-INDEX | v1.40.0 (138 BCs) |
| EVAL-INDEX | v1.15 |
| version-pin-registry.yaml | source of truth |

---

## Remaining Tasks (in order)

1. Finish Phase-1d convergence: Pass 36/37/38+ until 3 consecutive clean (counter = 0 now).
2. Human spec-package approval gate (run check-input-drift first; present review questions;
   gate items: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE).
3. Phase 2 story decomposition (vsdd-factory:story-writer): v1A delta into stories + waves;
   resolve all S-TBD anchors in 25 BCs + holdout stories_tested fields.
4. VP authoring (vsdd-factory:architect) — deferred to formal-hardening (VP-TBD pattern).
5. Pre-Phase-3: DTU clone check (S-DTU-001 fidelity 1.0 — D-234; UNBLOCKED); CI/CD verify.
6. Phase 3 TDD implementation of v1A stories (wave gates). v1B stories authored later.

---

## Full Checkpoint

See `.factory/STATE.md` block `next_session_resume_protocol` (version 7.28, D-271..D-277) for:

- The complete convergence-loop procedure (Steps A/B/C + commit rules + cycle checklist).
- The full 35-pass finding trajectory with per-pass detail.
- The full spec package list with all current versions (derived from registry).
- Ratified decisions: EngineError canonical enum, ADR-0006 constructors, dead-anchor
  whole-class remediation, two-pronged InvalidPath, mouse SGR encoding, ordered-pair-split.
- All codified lessons (registry atomicity, propagation-closure, anchor-resolution,
  POL-13 anchor-lint, no-version-bump for navigational anchors, etc.).
- The durable_task_register for all non-blocking open items.
- Already-built substrate inventory (1514 tests, 9 workspace crates, daemon, TUI, IPC).
