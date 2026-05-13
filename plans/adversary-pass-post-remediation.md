---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, production-grade lens, post-remediation) — transcribed by orchestrator
phase: pre-phase-1-final-gate-pre-human-approval
timestamp: 2026-05-12T09:30:00Z
inputs:
  - /Users/jmagady/Dev/monocle/CLAUDE.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md
  - /Users/jmagady/Dev/monocle/.factory/specs/dtu-assessment.md
  - /Users/jmagady/Dev/monocle/.factory/tech-debt-register.md
  - /Users/jmagady/Dev/monocle/.factory/planning/market-intelligence.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
  - /Users/jmagady/Dev/monocle/.factory/plans/production-grade-reaudit.md
input-hash: "[live-state]"
traces_to: "prior adversary pass 0bd4ba9; convergence trajectory through rounds 2-4; canonical principle CLAUDE.md b69c09f + 3366d58; round-4 consistency c2bf9e2"
project: monocle
verdict: MULTIPLE_DEFER_PATTERNS
---

# Adversarial Pass — Post-Remediation (Production-Grade Lens)

## 1. Top-line verdict

**MULTIPLE_DEFER_PATTERNS** — 4 CRITICAL, 6 IMPORTANT, 4 ADVISORY. The remediation burst resolved the textual defer-patterns (TODOs, "Pending architect," "Placeholder for architect"), but a fresh-context read reveals (a) a wire-protocol path divergence between vision and brief that the consistency validators missed, (b) a security-rationale that misidentifies the threat-bearing crate, (c) a copy-from-gene-source enum ("17 variants permission token") that ships dead scaffolding for Phase 1, (d) missing wire-contract timeouts that determine whether the daemon meets Claude Code's silent-drop deadline, and (e) genuine missing topics (health endpoint, payload size limits, license/SBOM) that production-grade requires before Phase 1 begins.

The convergence trajectory is real for *textual* defer-patterns, but novelty remains MEDIUM-HIGH for substantive defects. This is exactly the class the prior validation chain could not see.

## 2. Disposition of prior 14 violations

| # | Prior Finding | Disposition |
|---|---|---|
| 1 | dependencies.md "Placeholder for architect" Phase 2/3/4 | RESOLVED |
| 2 | dependencies.md 6 Architect-TODO items | RESOLVED |
| 3 | conventions.md Placeholder + 5 TODOs | RESOLVED |
| 4 | OQ-M1/OQ-M3 "Pending architect review" | RESOLVED |
| 5 | TD-001 unanchored deferral | RESOLVED |
| 6 | DTU deferred until architecture | RESOLVED |
| 7 | Brief supersession-notes pattern | RESOLVED but **NEW DEFECT INTRODUCED** (F-NEW-01) |
| 8 | "13 crates total" typo | RESOLVED |
| 9 | brief-validation-v2 stub paths | RESOLVED |
| 10 | oq-research.md frontmatter brief_version "1.1" | RESOLVED |
| 11 | R-001 "ship Phase 1 fast" mitigation | RESOLVED |
| 12 | reference-ingest pass-naming inconsistency `[process-gap]` | NOT RESOLVED (upstream) |
| 13 | tech-debt-register.md frontmatter fields | RESOLVED |
| 14 | oq-research.md missing traces_to | RESOLVED |

**Summary: 12 RESOLVED / 1 NEW DEFECT INTRODUCED / 1 NOT RESOLVED (upstream).**

## 3. New findings (this pass)

### CRITICAL

**F-NEW-01 [CRITICAL] — Vision/brief/DTU hook path divergence on UserPromptSubmit.**
- Location: vision-synthesis.md §Process Topology says `POST .../hooks/user-prompt-submit`; brief.md says `POST /hooks/prompt-submit`; dtu-assessment.md says `/hooks/prompt-submit`.
- Three artifacts (vision, brief, DTU) reference the same wire-protocol endpoint with two different paths. The DTU clone is built to `/hooks/prompt-submit`; an implementer who reads the vision diagram and wires `/hooks/user-prompt-submit` will fail clone validation. The vision v1.1.1 patch in the prose says it "refreshed endpoint set (canonical 5)" but the diagram path was not synced.
- Severity rationale: wire contract divergence in a canonical L1+L3 set. Any implementer of either artifact alone builds a non-compliant daemon.
- Routing: **business-analyst** — sync vision diagram to brief canonical paths.

**F-NEW-02 [CRITICAL] — Security rationale misidentifies untrusted-input crate.**
- Location: SS-deps-pin-manifest.md three places state `prost` "deserializes untrusted hook POST bodies at the network boundary."
- Per BC-HOOK-023 hook POST bodies carry `Content-Type: application/json`. The brief describes `monocle-proto: prost protobuf seam in monocle-core — zero runtime cost in v1, enables cross-host events in Phase 4`. In Phase 1, `prost` is NOT on the untrusted-input path — `serde_json` is. The Patch-Pinning Policy "threat surface" justification (which mandates security-reviewer agent dispatch on every bump) is applied to the wrong crate. `serde_json` is caret-pinned with no special handling.
- Routing: **architect** — promote `serde_json` to EXACT pin OR explicitly re-cast `prost`'s pin justification as "future-Phase-4 untrusted federation wire-format deserializer; pinned now to lock audit baseline."

**F-NEW-03 [CRITICAL] — 17-variant permission token enum is unjustified Phase-1 scaffolding.**
- Location: brief.md `Permission token enum: 17 variants in monocle-core::permissions; dispatcher no-op until Phase 3 (SOQ-4)`.
- The 17-variant enum is sourced from zellij's `PermissionType` enum (zellij's WASM plugin sandbox permission model). monocle is **not** a WASM sandbox host until Phase 3. Shipping a 17-variant enum with a no-op dispatcher in Phase 1 is exactly the "dead-code scaffolding for later" pattern that Rule 1 forbids. Either the enum is justified in Phase 1's actual scope (permission overlay) — in which case the brief must spell out which Phase 1 BCs each variant maps to — or the enum belongs in Phase 3 alongside the WASM SDK that uses it.
- Routing: **architect** — re-derive the Phase 1 permission-token surface from Claude Code hook semantics, not from zellij. HIGH confidence the answer is materially smaller than 17 for Phase 1.

**F-NEW-04 [CRITICAL] — Hook-receiver inbound timeouts not specified.**
- Location: brief.md Success Criteria — `≤100ms from hook POST receipt to TUI overlay render on localhost`.
- BC-HOOK-022 (gene-source) documents that Claude Code's hook commands give the daemon **300ms** for PreToolUse/Stop/SessionStart/UserPromptSubmit and **2000ms** for Notification before the client gives up and silently drops. The brief's 100ms render target is consistent but does not encode the 300ms/2000ms deadlines. The architect cannot derive the inbound SLO from the brief alone.
- Routing: **product-owner** — add Success Criteria row "Hook ingestion timeout budget: daemon responds within 300ms (PreToolUse/Stop/SessionStart/UserPromptSubmit) and 2000ms (Notification) — gene-source BC-HOOK-022 verbatim."

### IMPORTANT

**F-NEW-05 [IMPORTANT] — No health-check / liveness endpoint specified.**
- Daemon binds axum HTTP with 5 hook endpoints; no `/healthz`, no `/readyz`, no daemon-status query endpoint.
- Production-grade default: one axum route with five LOC. Multi-host federation (Phase 4) cannot work without it.
- Routing: **architect** — add `GET /healthz` and `GET /status` to Phase 1 endpoint set.

**F-NEW-06 [IMPORTANT] — No hook-receiver payload size limit / DoS surface protection.**
- axum 0.8 by default does NOT enforce body size limits unless `DefaultBodyLimit` is configured. The Notification body includes a `message` string with no upper bound. A misconfigured or malicious hook script could POST 1GB JSON bodies and exhaust daemon memory.
- Routing: **architect** — add BC: `daemon enforces ≤256KiB body limit on hook POST endpoints (RFC 7230 §3.3.2 compliant; reject with 413 Payload Too Large)`.

**F-NEW-07 [IMPORTANT] — Cross-IDE lock-file collision risk (BC-HOOK-024) not addressed.**
- any-context-lazyclaude-pass-B-deep-hooks-r1.md documents BC-HOOK-024: hook discovery JS picks "highest port wins" across all `~/.claude/ide/*.lock` files without filtering by `lock.App` field. monocle uses a different lock-file directory (`directories::ProjectDirs::runtime_dir()`) per OQ-10, so the same risk does not apply identically — but the spec must explicitly state whether monocle's hook commands scan only `monocle.lock` or scan a directory. If a directory, the same P2 collision risk recurs.
- Routing: **architect** — pin the lock-discovery mechanism: single-file (`<runtime_dir>/monocle.lock`) NOT directory scan.

**F-NEW-08 [IMPORTANT] — No license, no SBOM, no OSS compliance strategy.**
- 24+ pinned dependencies, several with non-trivial licenses (russh = Apache+MIT, wasmtime = Apache-2.0, nucleo = MPL-2.0 — weak copyleft). Greenfield Rust project. Vision/brief silent on monocle's own license. Architect inherits these crates with no compliance gate.
- Enterprise/production-grade default REQUIRES license declaration before code ships. SBOM is now standard (CISA + EU CRA both require by 2027).
- Routing: **architect** (license selection ADR-0003) + **devops-engineer** (cargo-deny + SBOM in CI pipeline).

**F-NEW-09 [IMPORTANT] — No daemon graceful shutdown spec.**
- `monocle daemon stop` semantics undefined. A long-lived daemon receiving hook POSTs needs: drain in-flight requests, flush JSONL buffer (per OQ-06 RAM ring), close UDS gracefully, persist last state, write `shutdown` field to lock file. None specified.
- Routing: **architect** — add §"Daemon Lifecycle Protocol" with start, drain-shutdown, force-kill, and crash-recovery sequences.

**F-NEW-10 [IMPORTANT] — DTU fidelity target 0.95 has no derivation/measurement spec.**
- The 0.95 fidelity floor is a number without a measurement protocol. What is the test fixture? How is fidelity computed (byte-equality? schema-equality? field-by-field? statistical sampling?)? Update cadence is "monthly schema check; quarterly recorded session re-validation" — but no procedure for "recorded session."
- Routing: **architect** (DTU author) — add "DTU Fidelity Measurement Procedure" §.

### ADVISORY

**F-NEW-11 [ADVISORY] — CLAUDE.md "Key Tech Stack" misses pulldown-cmark, serde_json, bytes, semver.**
- Routing: **state-manager** (CLAUDE.md sync from SS-deps-pin-manifest).

**F-NEW-12 [ADVISORY] — Brief v1.4.2 §Phase Plan Rationale phrasing.** No fix required.

**F-NEW-13 [ADVISORY] — STATE.md `awaiting:` field names round-3 adversary as outstanding.**
- Routing: **state-manager** — flip after this commits.

**F-NEW-14 [ADVISORY] [process-gap] — Consistency-validator skill cannot see URL/path divergence across diagrams + tables + body prose.**
- F-NEW-01 above survived 5 validation passes because the validators look at IDs/anchors/counts/version-pins, not URL-path coherence across artifact types.
- Routing: **upstream issue** (drbothen/vsdd-factory) — add a "URL/path/endpoint cross-document coherence" axis to consistency-validator.

## 4. Semantic coherence assessment

Brief, vision, architecture stubs are 90% coherent. Remaining gaps: F-NEW-01 (wire paths), F-NEW-02 (threat model), F-NEW-03 (Phase 1 scaffolding). Brief and vision agree on five-plane architecture, EngineModule trait, FactoryAdapter trait, observe-only-for-state principle. Architecture stubs (SS-deps, SS-conventions, ADR-0001, ADR-0002) are internally consistent.

## 5. Risk acceptance soundness — R-001 at <10%

The <10% framing is defensible given specific assumptions (agent-view stays research-preview, single-harness, no announced hook-protocol direction). Brief captures this rationale. The qualifier "based on additional context about agent view's roadmap and scope" is the human's judgment, not the AI's.

But the brief stops short of stating what *triggers a re-evaluation*. Compare to ADR-0002 which has a 4-condition explicit re-eval gate. R-001 should have an explicit re-eval trigger matching ADR-0002's pattern.

Routing: **product-owner** — add R-001 re-eval trigger paragraph to brief Competitive Positioning section.

## 6. Architecture stub policy soundness

| Stub | Verdict |
|---|---|
| SS-deps-pin-manifest.md | MOSTLY SOUND with one critical hole (F-NEW-02). Otherwise: 24-pin matrix concrete, RUSTSEC justifications evidence-bearing, MSRV policy + Patch-Pinning Policy well-reasoned. |
| SS-conventions-anti-patterns.md | SOUND. Clippy + semgrep + PR-template + CI wiring is the right shape. |
| ADR-0001 wasmtime-vs-wasmi | SOUND. Rationale concrete. |
| ADR-0002 nucleo acceptance | SOUND. Gold-standard pattern: concrete acceptance, four explicit re-eval triggers, TD-001 retirement noted. Use as template for future risk-acceptance ADRs. |

## 7. DTU assessment quality

Mostly sound. Six-category inventory respects DTU methodology. Issues: F-NEW-10 (fidelity floor 0.95 lacks measurement procedure), F-NEW-01 wire-path divergence invisible to DTU author.

## 8. Missing topics

1. Health endpoint / liveness probe (F-NEW-05)
2. Request size limits / DoS hardening (F-NEW-06)
3. OSS license + SBOM strategy (F-NEW-08)
4. Daemon graceful shutdown protocol (F-NEW-09)
5. Hook receiver timeout SLO as a Success Criterion / BC (F-NEW-04)
6. Cross-IDE lock-file scan policy (F-NEW-07)
7. DTU fidelity measurement procedure (F-NEW-10)
8. R-001 re-eval trigger spec (§5)

Lower-priority:
- Secrets-management policy for the `authToken`
- Configuration-validation strategy for `~/.monocle/config.json`
- Error budget / SLO for Phase 1

## 9. Verdict and recommendation

**Verdict: MULTIPLE_DEFER_PATTERNS** (per production-grade lens — same verdict class as prior pass, but on different defects).

**Recommendation: FIX FIRST.**

The textual defer-patterns have been remediated, and the trajectory shows the validation harness is working. But the package retains four CRITICAL defects (wire-protocol path divergence, wrong threat-crate identified, zellij-borrowed enum shipped as Phase-1 scaffold, missing inbound-timeout BC) and six IMPORTANT gaps (health endpoint, DoS limits, license/SBOM, graceful shutdown, BC-HOOK-024 cross-IDE risk, DTU fidelity measurement). None of these require new architecture decisions outside current scope — all are answerable in-scope by the correct specialist.

**Pre-Phase-1 burst plan (4 specialist dispatches):**

1. **business-analyst** — Sync vision diagram `/hooks/user-prompt-submit` → `/hooks/prompt-submit` to match brief and DTU.
2. **product-owner** — Add Success Criteria row for hook inbound-timeout SLO (300ms/2000ms per gene-source BC-HOOK-022); add R-001 re-eval trigger paragraph.
3. **architect** — (a) Correct SS-deps prost-vs-serde_json threat rationale; (b) re-derive Phase-1 permission token surface from monocle's actual scope; (c) add `/healthz` + `/status` + body-limit + graceful-shutdown to Phase 1 BC scope; (d) add lock-file scan policy (single-file, not directory); (e) add DTU fidelity measurement procedure; (f) ADR-0003 license selection.
4. **devops-engineer** — Add `cargo-deny` license/advisory gate to CI matrix (the SS-conventions CI wiring already has cargo audit; license check is one more step).

After this burst, the package should clear at PRODUCTION_READY. Estimated effort: 60-90 minutes of specialist work.
