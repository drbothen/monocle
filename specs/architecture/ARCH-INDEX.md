---
document_type: architecture-index
level: L3
version: "1.0.23"
status: active
producer: vsdd-factory:architect
timestamp: 2026-05-29T12:00:00Z
phase: phase-1-expansion
inputs: [product-brief.md, prd.md]
input-hash: "da60462"
traces_to: prd.md
deployment_topology: single-service
project: monocle
---

# Architecture Index: monocle

> **Context Engineering:** This is a lightweight index (~200-400 tokens). Agents load
> ONLY the section files they need, not the full architecture. See the Document Map
> for per-section consumer guidance.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| Daemon Lifecycle | SS-daemon-lifecycle.md | ~23,730 | orchestrator, implementer, test-writer | HTTP server, hooks, auth, locking, ring buffer, crash recovery |
| Core Types and ABI | SS-core-types-and-abi.md | ~10,072 | implementer, formal-verifier | Forward-compatible wire formats, factory abstractions, protocol versioning |
| Engine Module | SS-engine-module.md | ~15,013 | implementer, formal-verifier | EngineModule trait, ClaudeCodeModule adapter, harness abstraction |
| Daemon Wiring | SS-daemon-wiring.md | ~4,800 | orchestrator, implementer, test-writer | Composition root: CLI surface, daemon start sequence, event bus, hooks-settings.json |
| IPC | SS-ipc.md | ~3,200 | implementer, test-writer | UDS transport, framing protocol, message types, reconnection, SOQ-3 overlay clear |
| TUI | SS-tui.md | ~5,200 | implementer, test-writer, formal-verifier | AppMode state machine, Action dispatch, panels, permission overlay, Ctrl-\ integration |
| Dependency Manifest | SS-deps-pin-manifest.md | ~9,976 | implementer, devops-engineer | Version pins, MSRV policy, workspace dependency graph |
| Conventions & Anti-Patterns | SS-conventions-anti-patterns.md | ~25,794 | implementer, code-reviewer | Code conventions, forbidden patterns, clippy + semgrep enforcement |
| Forward Compatibility | SS-forward-compatibility.md | ~7,871 | architect, implementer | FC contracts P2-1..P3-N |
| Phase 1 Permissions | SS-permissions-phase1.md | ~2,661 | implementer, test-writer | Phase 1 permission enum |
| Config | SS-config.md | ~2,600 | implementer, test-writer | Config persistence, harness profiles, profile picker, CCR detection |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implementation plan for daemon | SS-daemon-lifecycle.md + SS-core-types-and-abi.md + SS-deps-pin-manifest.md |
| Harness abstraction implementation | SS-engine-module.md + SS-core-types-and-abi.md |
| Verification plan for a module | SS-core-types-and-abi.md + SS-engine-module.md |
| Phase 3+ upgrade impact | SS-forward-compatibility.md + SS-deps-pin-manifest.md |
| Code review enforcement rules | SS-conventions-anti-patterns.md |
| Daemon binary wiring (composition root) | SS-daemon-wiring.md + SS-daemon-lifecycle.md + SS-engine-module.md |
| IPC protocol (TUI ↔ daemon transport) | SS-ipc.md + SS-daemon-wiring.md |
| TUI implementation (panels + overlay) | SS-tui.md + SS-ipc.md + SS-core-types-and-abi.md + SS-deps-pin-manifest.md |
| Config crate implementation | SS-config.md + SS-deps-pin-manifest.md + SS-conventions-anti-patterns.md |

## Subsystem Registry

> **Source of truth** for subsystem names and IDs. BC frontmatter `subsystem:`,
> BC-INDEX subsystem column, story `subsystems:` fields, and PRD subsystem
> references MUST all use the exact Name from this table.

| SS ID | Name | Architecture Doc | Implementing Modules | Phase Introduced |
|-------|------|-----------------|---------------------|-----------------|
| SS-01 | Daemon Lifecycle | SS-daemon-lifecycle.md | monocle-runtime (daemon binary, HTTP server, ring buffer, lock file, auth) | Phase 1 |
| SS-02 | Core Types and ABI | SS-core-types-and-abi.md | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) | Phase 1 |
| SS-03 | Engine Module | SS-engine-module.md | monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — `monocle-runtime/src/engine/claude_code.rs`) | Phase 1 |
| SS-04 | Daemon Wiring | SS-daemon-wiring.md | monocle (binary crate — `main.rs`, `clap` CLI, daemon entrypoint, TUI entrypoint); monocle-runtime (hooks-settings.json generation, bounded event bus, MONOCLE_NO_AUTOSTART check) | Phase 1 |
| SS-05 | IPC | SS-ipc.md | monocle-ipc (UDS client + server, Transport trait, message types, framing, reconnection logic) | Phase 1 |
| SS-06 | TUI | SS-tui.md | monocle-core (AppMode, Action, FocusSnapshot, PanelId, PromptModal, BindingSource, Binding, transition() — pure types and transition function); monocle-tui (ratatui renderer, panel layout, crossterm event loop, IPC client, keybinding dispatcher) | Phase 1 |
| SS-07 | Config | SS-config.md | monocle-config (config.json reader/writer, harness profile schema, profile picker logic, CCR detection) | Phase 1 |

**ID format:** `SS-NN` (two-digit sequential, append-only).

**Naming rules:**
- Names are human-readable, title-case
- Names are stable — once assigned, a subsystem name does not change
- If a subsystem is retired, mark it `(retired)` in the Name column; do not remove the row

**Capability traceability:**

| SS ID | L2 Capability | Description |
|-------|--------------|-------------|
| SS-01 | CAP-001 | Daemon ingestion of Claude Code hook events; lifecycle management |
| SS-02 | CAP-002 | Forward-compatible ABI; wire format stability; factory-state abstraction |
| SS-03 | CAP-003 | Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter |
| SS-04 | CAP-004 | Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation |
| SS-05 | CAP-005 | Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear |
| SS-06 | CAP-006 | User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration |
| SS-07 | CAP-007 | Configuration persistence; harness profile management; profile picker; CCR detection |

## Cross-Cutting Files

The following architecture documents are not assigned an SS-NN runtime subsystem ID.
They define conventions, constraints, and cross-cutting concerns that apply to all subsystems.

| File | Purpose |
|------|---------|
| SS-conventions-anti-patterns.md | Code conventions, forbidden patterns, clippy + semgrep + PR-template + CI enforcement |
| SS-deps-pin-manifest.md | Canonical dependency pins, MSRV policy, security-advisory response, workspace graph |
| SS-forward-compatibility.md | FC contracts for Phase 2/3/4 forward-compatibility surface |
| SS-permissions-phase1.md | Phase 1 `Phase1Permission` enum definition and exhaustive-enum policy |

## ADR Registry

| ADR ID | Title | Status | File |
|--------|-------|--------|------|
| ADR-0001 | wasmtime vs wasmi for WASM Plugin Runtime | accepted | adr/ADR-0001-wasmtime-vs-wasmi.md |
| ADR-0002 | Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger | accepted | adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md |
| ADR-0003 | MIT OR Apache-2.0 Dual-License Selection | accepted | adr/ADR-0003-license-selection.md |
| ADR-0004 | Exhaustive Enums — `Phase1Permission` and `ClaudeCodeTool` | accepted | adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md |
| ADR-0005 | Auth Header Dual-Accept — Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias | accepted | adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md |
| ADR-0006 | Non-Exhaustive Structs with Public Positional Constructors | accepted | adr/ADR-0006-non-exhaustive-structs-with-public-constructors.md |
| ADR-0007 | Version-Pin Citation Discipline — Semantic Anchors + CI Registry Enforcement | accepted | adr/ADR-0007-version-pin-citation-discipline.md |
| ADR-0008 | Structural-Claim Discipline — Canonical Shape Anchors + POL-12 Detection | accepted | adr/ADR-0008-structural-claim-discipline.md |

**Note:** ADR-0001 covers Phase 3 wasmtime 44 adoption (not a Phase 1 runtime dependency).
ADR-0002 accepts nucleo 0.5 dormancy risk; re-eval trigger: if nucleo has no commit activity
for 6+ months by Phase 2 start, the architect must re-evaluate alternatives.
ADR-0005 resolves the auth header interop gap between monocle's canonical header and real
Claude Code's hardcoded `X-Claude-Code-Ide-Authorization` (BC-HOOK-016); dual-accept at the
router-level auth middleware.
ADR-0006 ratifies `pub fn new(...)` constructors on `#[non_exhaustive]` structs for internal
workspace crates anchored to external wire protocols (Phase 1 scope; S-022 cycle).
ADR-0007 resolves the 7-instance META-pattern version-pin staleness species (escalation ladder
passes 9/16/18/22/23/24/25); selects Option C-Refined (hybrid: semantic anchors for new
artifacts + CI registry gate for all, opportunistic legacy migration). Dispatches
devops-engineer POL-11-version-pin hook, story-writer and product-owner template updates.
v1.0.4 adds §Enforcement Scan Scope formalizing NORMATIVE vs EXEMPT document classes for
POL-11: adds `plans/`, `planning/`, `code-delivery/`, and `STATE.md` to the exempt list
alongside the existing `cycles/` exemption (ADV-29 scope ratification, human-approved).
v1.0.5 adds §inputs[] Provenance Classification: individual story inputs[] = HISTORICAL
provenance (exempt from POL-11); living index doc inputs[] = ACTIVE (scanned).
v1.0.6 closes the classification with a default-HISTORICAL + closed-active-set rule:
inputs[] pins are HISTORICAL by default for ALL document classes; ACTIVE only for files
whose basename matches `*-INDEX.md` (STORY-INDEX, BC-INDEX, ARCH-INDEX, VP-INDEX,
EVAL-INDEX, L2-INDEX) or equals `prd.md`. Active set is CLOSED — extension requires ADR
amendment. Eliminates classification recursion for unclassified doc classes (SS-*, BC-*,
ADR-*, dep-graph, prd-supplements); ~92 over-flagged inputs[] pins reclassified HISTORICAL.
v1.0.7 amends Pattern A to be REGISTRY-DRIVEN: the vocabulary of detectable artifact IDs
for prose/inline version-pin literals is derived from ALL keys in version-pin-registry.yaml
at runtime — not from a hardcoded prefix alternation (`SS-[a-z-]+\|BC-[0-9.]+`). The
hardcoded vocabulary omitted dtu-assessment, ADR-*, product-brief, nfr-catalog,
error-taxonomy, and *-INDEX artifact classes, causing ~59 stale `dtu-assessment.md v1.7.5` <!-- version-pin-historical: version that was stale at Pass 31; cited as a historical diagnostic record -->
citations to be invisible to Pass 31 detection. Matcher uses longest-match sort (descending
ID length) for prefix-disambiguation and word-boundary after version token. Together with
v1.0.6's closed-rule: complete enforcement surface (which docs are ACTIVE + which IDs are
detectable both fully specified with no open vocabulary tails).
v1.0.8 extends the EXEMPT set in §Enforcement Scan Scope with three new living-state file
entries, parallel to the STATE.md exemption: `factory_root/stories/sprint-state.yaml`
(continuously-rewritten sprint dashboard, every story transition), `factory_root/tech-debt-register.md`
(living human-directed debt register, historical chronicle not normative spec), and
`repo_root/CLAUDE.md` (living project instructions with continuously-rewritten Pipeline
State section). All three produce version-race CI false positives for zero correctness
value under pin-freshness enforcement. The EXEMPT set remains CLOSED and enumerated —
adding a file requires a further ADR amendment. Adjudication: dependency-graph-expansion.md
and holdout-scenarios.md are NORMATIVE (not exempt); their stale citations are legitimate
POL-11 targets handled by story-writer opportunistically. Devops dispatch updated with
exact-path exclusion specs including repo-root CLAUDE.md mechanism.
ADR-0008 resolves the structural-claim sub-species of the same authoring-time documentation
drift root (Task #9 m.6 tripwire; Passes 26/27 — module-doc column table + story-body
type-name); selects Option B (distinct ADR, POL-12 `monocle-structural-claim-check`). Governs
type identifiers, table column counts, variant lists — a categorically distinct detection
mechanism from ADR-0007's literal `vN.M.P` regex. Dispatches devops-engineer POL-12 CI script
and story-writer structural-claim sweep for in-flight Wave 6 stories.

## §Trace v1.0.2

**T-128e audit-trail reconciliation** (2026-05-17T17:00:00Z):
- NORMATIVE: §Trace v1.0.1 body corrected: hash citation `561ef4d` → `ee1f76a` to match
  frontmatter `input-hash: ee1f76a` (commit `0af206a`). No frontmatter change — frontmatter
  was always correct; only the §Trace narrative diverged.
- INFORMATIONAL: Version bump 1.0.1 → 1.0.2 records audit-trail correction; no content changes.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T16:30:00Z.
- Audit reference: `.factory/plans/adversary-cycle-001/R105-findings.md` F-R105-5 (HIGH).

**Audit R2 residual fix RES-04 + RES-01 fix-pass** (2026-05-17T16:30:00Z):
- RES-04: Added `Tokens` column to Document Map per architecture-index-template.md.
  Token counts computed as word_count × 1.3 (approximate), using `wc -w` per section file.
  All seven section files enumerated with `~N` token estimates.
- RES-01: Normalized `inputs:` field in ARCH-INDEX.md from absolute paths to relative
  paths (inline array format) resolvable by compute-input-hash. input-hash updated to
  `ee1f76a` (reflecting [product-brief.md, prd.md] content at fix-pass time). Path
  normalization also applied to 18 other [live-state] placeholder files in the same pass.
- version: 1.0 → 1.0.1; timestamp: 2026-05-17T11:00:00Z → 2026-05-17T16:30:00Z.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T16:30:00Z >= chain high-water 2026-05-17T16:00:00Z.
- Audit references: `.factory/plans/template-compliance-audit-r2.md` RES-01, RES-04.
- **T-128e reconciliation** (2026-05-17T17:00:00Z): §Trace originally cited `561ef4d`; corrected
  to `ee1f76a` to match frontmatter line 10 (actual value written by commit `0af206a`). Root
  cause: §Trace narrative was authored with an intermediate hash computed before the final
  compute-input-hash write; frontmatter received the definitive `ee1f76a` in the same commit.
  SE-17c body-scope grep evidence: `grep "561ef4d\|ee1f76a"` in §Trace body (lines ≥ 96)
  returned 1 match (`561ef4d`) prior to this correction — confirming the divergence between
  audit trail and artifact state (defect F-R105-5). Frontmatter `input-hash: ee1f76a` is
  authoritative; §Trace narrative now aligned. No frontmatter change required.
  Audit reference: `.factory/plans/adversary-cycle-001/R105-findings.md` F-R105-5 (HIGH).

**Template compliance Dispatch 1 of 6+** (2026-05-17T11:00:00Z):
- Created as new artifact; no prior version.
- Populates Subsystem Registry with SS-01 (Daemon Lifecycle), SS-02 (Core Types and ABI),
  SS-03 (Engine Module) per audit §MISS-03 subsystem proposals.
- Cross-Cutting Files section covers SS-deps-pin-manifest, SS-conventions-anti-patterns,
  SS-forward-compatibility, SS-permissions-phase1 (not runtime subsystems).
- ADR Registry enumerates ADR-0001..ADR-0004 from `.factory/specs/architecture/adr/`.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §MISS-03.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.

## §Trace v1.0.3

**T-128h BC ID canonicalization — F-R105-8 closure** (2026-05-17T17:00:00Z):
- NORMATIVE: All stale pre-renumbering BC IDs propagated to canonical BC-2.SS.NNN forms
  across 3 SS architecture documents per BC-INDEX.md v1.1 §Renumbering Map (canonical
  at T-128h dispatch time 2026-05-17T17:00:00Z; current canonical advances over time
  per F-R107-8 historical-pin discipline).
  Scope: SS-daemon-lifecycle.md, SS-engine-module.md, SS-core-types-and-abi.md.
- SE-17g META AUDIT — final re-grep confirms zero stale IDs remaining across all 3 docs
  (grep pattern: old-form DAEMON/AUTH/RING/LOCK/ABI/TYPES/FACTORY/PROTO/ENGINE prefixes):
  SS-daemon-lifecycle.md: 0 lines match (was 95 lines / 102 occurrences)
  SS-engine-module.md: 0 lines match (was 31 lines / 33 occurrences)
  SS-core-types-and-abi.md: 0 lines match (was 39 lines / 46 occurrences)
  Grand total replaced: 181 occurrences across 165 lines. SE-17g PASS: 165 → 0.
- DISCOVERED: PROTO-001 (bare pre-split form, old-style) — 2 occurrences in historical §Trace
  prose in SS-core-types-and-abi.md. This ID is retired by split (F-FC-O004); it has no
  canonical new-form entry in BC-INDEX §Renumbering Map (only the a/b split variants are
  mapped to BC-2.02.006 and BC-2.02.007). Resolved: historical §Trace prose rewritten to
  descriptive form; stale ID removed from SS doc body. Record preserved in BC-INDEX §Renumbering
  Map per append-only policy.
- SS doc versions bumped: SS-daemon-lifecycle.md 1.0.27 → 1.0.28; SS-engine-module.md
  1.1.17 → 1.1.18; SS-core-types-and-abi.md 1.2.10 → 1.2.11.
- ARCH-INDEX does not carry per-SS doc version numbers — no Document Map changes required.
- INFORMATIONAL: Version bump 1.0.2 → 1.0.3 records SE-17g META audit; no content changes
  to ARCH-INDEX body.
- SE-16d PASS: 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T17:00:00Z (same burst;
  ARCH-INDEX and SS docs updated in the same T-128h dispatch — monotonicity satisfied).

## §Trace v1.0.4

**T-128m ADR-0005 auth header dual-accept — F-R105 closure chain Round 3** (2026-05-17T19:00:00Z):
- NORMATIVE: ADR-0005 authored and registered. Decision: dual-accept (option a).
  File: `adr/ADR-0005-auth-header-dual-accept-canonical-x-monocle-authorization.md`.
  Resolves interop gap surfaced by BA in T-128f: real Claude Code hook scripts have
  `X-Claude-Code-Ide-Authorization` hardcoded per BC-HOOK-016 deep ingest; they cannot
  send `X-Monocle-Authorization`. ADR-0005 directs the daemon auth middleware to
  dual-accept both headers with `X-Monocle-Authorization` as canonical priority.
- NORMATIVE: ADR Registry updated — ADR-0005 row added.
- NORMATIVE: SS-daemon-lifecycle.md v1.0.28 → v1.0.29 — auth middleware spec updated
  to dual-accept; Rust stub rewritten; BC-2.01.009 table expanded; §Trace added.
- NORMATIVE: dtu-assessment.md v1.7.2 → v1.7.3 — ADR-0005 auth header rationale block
  added to endpoint matrix preamble; 10 `X-Claude-Code-Ide-Authorization` occurrences
  confirmed correct (DTU tests compatibility alias path).
- NORMATIVE: BC-2.01.009 update surfaced to PO for Round 4 (postcondition 1 "missing"
  semantics; alias path postconditions 2-3 extension).
- NORMATIVE: CAP-001 §P2 step 1 compatibility alias note surfaced to BA for Round 4.
- INFORMATIONAL: product-brief.md occurrences (lines 116 and 239) out of scope for this
  dispatch; surfaced to PO for Round 4 as noted.
- SE-16d PASS: 2026-05-17T19:00:00Z > chain high-water 2026-05-17T17:00:00Z.

## §Trace v1.0.5

**F-R106 Round 5E — ADR-0005 path fix + SS-daemon-lifecycle F-FC-I005 removal** (2026-05-17T22:00:00Z):
- NORMATIVE: ADR-0005 v1.0.1 → v1.0.2 — frontmatter `inputs:` third entry normalized;
  spurious `specs/` prefix removed from `behavioral-contracts/ss-01/BC-2.01.009.md`.
  §Trace v1.0.2 added to ADR-0005.
- NORMATIVE: SS-daemon-lifecycle.md v1.0.29 → v1.0.30 — F-FC-I005 fabricated ID removed
  from two sites: §Start Sequence body (~line 298) and §Behavioral Contract Summary
  BC-2.01.009 table row (~line 800). Replaced with FC-06 alone (canonical reference).
  SE-17g META AUDIT PASS: zero F-FC-I005 occurrences remain in SS-daemon-lifecycle.md.
  §Trace v1.0.30 added.
- INFORMATIONAL: ARCH-INDEX ADR Registry table does not carry per-ADR version numbers;
  no content change to ADR-0005 row required. ARCH-INDEX Document Map does not carry
  per-SS doc version numbers (confirmed §Trace v1.0.3). Only §Trace version bumped here.
- SE-16d PASS: 2026-05-17T22:00:00Z > chain high-water 2026-05-17T19:00:00Z (monotonic).

## §Trace v1.0.6

**F-R107 Round 6D — BC ID canonicalization + historical-pin clarification** (2026-05-17T23:00:00Z):
- NORMATIVE (F-R107-5 HIGH / SS-forward-compatibility.md v1.2.14 → v1.2.16): All stale
  pre-renumbering BC IDs in FC table and BC-mapping table canonicalized to BC-2.SS.NNN forms
  per BC-INDEX.md v1.4 §Renumbering Map. FC-04 body prose updated. BC-mapping table
  restructured: "Old-Form ID (retired)" column added; all 16 rows carry canonical new IDs as
  primary. Notes paragraph updated with old→new cross-references. SE-17g META AUDIT: zero
  normative stale BC IDs remain in SS-forward-compatibility.md. [Note: this entry originally
  cited "v1.2.15 → v1.2.16" — corrected to "v1.2.14 → v1.2.16" by F-R109-13 Round 8A.
  Version v1.2.15 was never a real intermediate state; git evidence confirms the file went
  directly from v1.2.14 to v1.2.16 in commit 98396fe. See SS-forward-compatibility.md
  §Trace v1.2.17-R109 for full disposition.]
- INFORMATIONAL (F-R107-8 / ARCH-INDEX §Trace v1.0.3): BC-INDEX cite `v1.1 §Renumbering Map`
  expanded to explicit historical-pin form: `v1.1 §Renumbering Map (canonical at T-128h
  dispatch time 2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`. Purpose: prevent future fresh-context audits from re-flagging
  the historical pin. [Note: original "current canonical BC-INDEX is v1.4" language corrected
  to historical-pin-only form by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-engine-module.md v1.1.18 → v1.1.19): Same historical-pin
  expansion applied to §Trace v1.1.18. `v1.1 §Renumbering Map (canonical at T-128h dispatch
  time 2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`. [Original live-version claim corrected by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-daemon-lifecycle.md v1.0.30 → v1.0.31): Same historical-pin
  expansion applied to §Trace v1.0.28. [Original live-version claim corrected by F-R108-1 Round 7C.]
- INFORMATIONAL (F-R107-8 / SS-core-types-and-abi.md v1.2.11 → v1.2.12): Same historical-pin
  expansion applied to §Trace v1.2.11. [Original live-version claim corrected by F-R108-1 Round 7C.]
- SE-16d PASS: 2026-05-17T23:00:00Z > chain high-water 2026-05-17T22:00:00Z (monotonic).

## §Trace v1.0.7

**F-R108 Round 7C — historical-pin live-version removal + frontmatter timestamp correction** (2026-05-18T01:00:00Z):
- NORMATIVE (F-R108-1 CRITICAL): All "current canonical BC-INDEX is v1.4 per F-R107-2 closure"
  live-version claims removed from 5 arch docs per O-R108-3 codification. Sites corrected:
  - SS-forward-compatibility.md: 1 occurrence in BC-mapping notes (pre-§Trace body).
  - SS-daemon-lifecycle.md: 2 occurrences — §Trace v1.0.28 body + §Trace v1.0.31 prose.
  - SS-engine-module.md: 2 occurrences — §Trace v1.1.18 body + §Trace v1.1.19 prose.
  - SS-core-types-and-abi.md: 2 occurrences — §Trace v1.2.11 body + §Trace v1.2.12 prose.
  - ARCH-INDEX.md (this file): 3 occurrences — §Trace v1.0.6 (2 occurrences, corrected below)
    + §Trace v1.0.3 (1 additional occurrence discovered during O-R108-3 corpus application).
  Replacement language: "current canonical advances over time per F-R107-8 historical-pin discipline".
- NORMATIVE (F-R108-9 HIGH): frontmatter `timestamp` corrected on all 5 files from stale values
  to 2026-05-18T01:00:00Z (matching chain high-water). SE-16b violation resolved on each.
  No version bumps applied to SS docs — content-change §Trace entries written; frontmatter bumps
  deferred to Round 8A per cross-dispatch coordination directive (F-R109-1 closure).
  SS docs that received new §Trace entries (content change): SS-daemon-lifecycle (v1.0.32 added),
  SS-engine-module (v1.1.20 added), SS-core-types-and-abi (v1.2.13 added),
  SS-forward-compatibility (v1.2.17 added).
- NORMATIVE (F-R108-10 HIGH): ADR-0002 v1.0.2 → v1.0.3. frontmatter `inputs:` path fix:
  `tech-debt-register.md` → `../tech-debt-register.md`; `plans/production-grade-reaudit.md`
  → `../plans/production-grade-reaudit.md`. Both paths now resolve from `.factory/specs/`
  context. §Trace v1.0.3 added to ADR-0002.
- INFORMATIONAL (F-R108-20 LOW): dtu-assessment.md inputs paths verified: all 4 resolve correctly
  from `.factory/specs/` context (product-brief.md, architecture/SS-deps-pin-manifest.md,
  architecture/SS-core-types-and-abi.md, semport/any-context-lazyclaude/…-final-synthesis-v2.md).
  No changes required. §Trace v1.7.4 added to dtu-assessment.md (verification record).
- INFORMATIONAL (F-R108-21 LOW): §Trace v1.0.6 above split into 4 sub-bullets per artifact,
  replacing the combined narrative. Content preserved; structure only.
- SE-16d PASS: 2026-05-18T01:00:00Z > chain high-water 2026-05-17T23:00:00Z (monotonic).

## §Trace v1.0.8

**F-R109 Round 8A — SS frontmatter version reconciliation + §Trace ordering + v1.2.15 gap disposition** (2026-05-18T05:00:00Z):
- NORMATIVE (F-R109-1 CRITICAL + F-R109-2 CRITICAL): frontmatter `version` bumped on 4 SS docs
  to reconcile with §Trace version numbers already written in Round 7C. The Round 7C cross-dispatch
  coordination directive withheld frontmatter bumps to avoid PO 7B pin staleness; this created
  fabrication-class defects where frontmatter claimed prior versions while §Trace bodies documented
  new versions. Bumps applied:
  - SS-daemon-lifecycle.md: frontmatter "1.0.31" → "1.0.32" (§Trace v1.0.32 already present).
  - SS-engine-module.md: frontmatter "1.1.19" → "1.1.20" (§Trace v1.1.20 already present).
  - SS-core-types-and-abi.md: frontmatter "1.2.12" → "1.2.13" (§Trace v1.2.13 already present).
  - SS-forward-compatibility.md: frontmatter "1.2.16" → "1.2.17" (§Trace v1.2.17 already present).
  - ARCH-INDEX.md (this file): "1.0.7" → "1.0.8" (cascade from SS doc changes + F-R109-2/9/13).
- NORMATIVE (F-R109-8 HIGH): §Trace v1.0.32/v1.1.20/v1.2.13/v1.2.17 bodies in all 4 SS docs
  rewritten to remove "No version bump — content unchanged; timestamp-only correction"
  self-contradiction. The F-R108-1 removals of live-version claim strings ARE normative content
  changes. All 4 §Trace bodies now accurately describe the version bumps as applied in Round 8A.
- NORMATIVE (F-R109-9 HIGH): §Trace ordering in ARCH-INDEX reordered from descending to ascending
  (v1.0.2 → v1.0.3 → … → v1.0.8) to match BC-INDEX pattern per F-R109-9.
- NORMATIVE (F-R109-13 MED): §Trace v1.0.6 above corrected: "v1.2.15 → v1.2.16" changed to
  "v1.2.14 → v1.2.16". Version v1.2.15 of SS-forward-compatibility.md never existed; the file
  transitioned directly from v1.2.14 to v1.2.16 in Round 6D (git commit 98396fe confirms). The
  ARCH-INDEX v1.0.6 narrative fabricated the v1.2.15 intermediate. SS-forward-compatibility.md
  §Trace v1.2.17-R109 carries the full disposition record.
- NORMATIVE (F-R109-8 / ARCH-INDEX): §Trace v1.0.7 above updated to remove "No version bumps
  applied — content unchanged" language for the SS doc frontmatter side; corrected to note that
  frontmatter bumps were deferred to Round 8A per cross-dispatch coordination directive.
- SE-17c BEFORE (F-R109-13): "SS-forward-compatibility.md v1.2.15 → v1.2.16" in §Trace v1.0.6.
- SE-17c AFTER (F-R109-13): "SS-forward-compatibility.md v1.2.14 → v1.2.16" in §Trace v1.0.6.
- SE-16d PASS: 2026-05-18T05:00:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic; corrected from erroneous 2026-05-17T04:30:00Z per F-R110-1).

## §Trace v1.0.9

**F-R110 Round 9A — timestamp correction + ADR-0002 citation fix + ARCH-INDEX input-hash verification** (2026-05-18T05:30:00Z):
- NORMATIVE (F-R110-1 CRITICAL): §Trace v1.0.8 header and frontmatter `timestamp` corrected from
  "2026-05-17T04:30:00Z" to "2026-05-18T05:00:00Z". Round 8A dispatch used a date in the past
  relative to Round 7C output (2026-05-18T01:00:00Z), breaking the SE-16d monotonic chain across
  all 5 affected files (4 SS docs + this ARCH-INDEX). The erroneous SE-16d PASS claim
  "2026-05-17T04:30:00Z satisfies chain monotonicity" was arithmetically false: 04:30 on May 17
  precedes 01:00 on May 18 by nearly 21 hours. Corrected timestamps restore the chain:
  01:00:00Z (Round 7C) < 05:00:00Z (Round 8A corrected) < 05:30:00Z (this entry).
- NORMATIVE (F-R110-6 HIGH): ADR-0002 §Source / Origin section corrected absolute machine-local
  path to relative path for SS-deps-pin-manifest.md. §Trace v1.0.4 added to ADR-0002.
- NORMATIVE (F-R110-11 MED): ARCH-INDEX input-hash refreshed. Declared hash "ee1f76a" was stale;
  compute-input-hash recomputed to "da60462" reflecting current state of inputs [product-brief.md,
  prd.md] (both updated in Round 8B by PO scope; hash not refreshed in Round 8A arch scope).
  Frontmatter `input-hash` updated from "ee1f76a" to "da60462".
- ARCH-INDEX version: "1.0.8" → "1.0.9" (cascade from F-R110-1 normative timestamp correction).
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T05:00:00Z (monotonic).

## §Trace v1.0.10

**R16B F-R117-2 ADR-0002 + full ADR Registry H1 sibling sweep** (2026-05-18T15:30:00Z):
- NORMATIVE (F-R117-2 HIGH — ADR-0002 row "for Phase 1" qualifier restoration):
  ADR-0002 INDEX row title corrected from `Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger`
  to `Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`.
  BEFORE: `Accept nucleo 0.5 Dormancy Risk with Explicit Re-eval Trigger`
  AFTER:  `Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`
  Source authority: ADR-0002 H1 (line 21): `# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`.
  "for Phase 1" is normatively load-bearing — it scopes the acceptance decision to Phase 1 only,
  which drives the re-eval trigger semantics described in the ADR Registry Note below the table.
  Defect class: H1↔INDEX-row title drift (same class as F-R116-1 closed for VP-INDEX).
- NORMATIVE (sibling-sweep ADR-0004 backtick restoration):
  ADR-0004 INDEX row title corrected from `Exhaustive Enums — Phase1Permission and ClaudeCodeTool`
  to `Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``.
  BEFORE: `Exhaustive Enums — Phase1Permission and ClaudeCodeTool`
  AFTER:  `Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``
  Source authority: ADR-0004 H1 (line 21): `# ADR-0004: Exhaustive Enums — \`Phase1Permission\` and \`ClaudeCodeTool\``.
  Backtick code spans in markdown table cells are valid syntax and must be preserved verbatim.
- NORMATIVE (sibling-sweep ADR-0005 backtick restoration):
  ADR-0005 INDEX row title corrected from
  `Auth Header Dual-Accept — Canonical X-Monocle-Authorization with X-Claude-Code-Ide-Authorization Compatibility Alias`
  to
  `Auth Header Dual-Accept — Canonical \`X-Monocle-Authorization\` with \`X-Claude-Code-Ide-Authorization\` Compatibility Alias`.
  BEFORE: `Auth Header Dual-Accept — Canonical X-Monocle-Authorization with X-Claude-Code-Ide-Authorization Compatibility Alias`
  AFTER:  `Auth Header Dual-Accept — Canonical \`X-Monocle-Authorization\` with \`X-Claude-Code-Ide-Authorization\` Compatibility Alias`
  Source authority: ADR-0005 H1 (line 21): full verbatim with backtick code spans around both header names.
- INFORMATIONAL (sibling-sweep — rows confirmed MATCH, no fix required):
  ADR-0001: INDEX title `wasmtime vs wasmi for WASM Plugin Runtime` matches ADR-0001 H1 title portion. PASS.
  ADR-0003: INDEX title `MIT OR Apache-2.0 Dual-License Selection` matches ADR-0003 H1 title portion. PASS.
  Document Map "Section" column: assessed as navigation labels (deliberate short forms), not verbatim H1 matches.
  The "Architecture: " prefix is consistently omitted across all seven Document Map rows by established ARCH-INDEX convention;
  no §Trace entry has previously flagged this as drift. Convention confirmed valid; no fixes required.
  Subsystem Registry rows: Names (SS-01 Daemon Lifecycle, SS-02 Core Types and ABI, SS-03 Engine Module) are
  defined in the Subsystem Registry itself as the source of truth — they are NOT H1 copies from SS-*.md files.
  Confirmed correct per ARCH-INDEX.md §"Naming rules" (stable, title-case, human-readable). PASS.
- SE-17a LITERAL-GREP EVIDENCE (F-R117-2):
  `grep -nE "^# " .factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md`
  → `21:# ADR-0002: Accept nucleo 0.5 Dormancy Risk for Phase 1 with Explicit Re-eval Trigger`
  Confirms "for Phase 1" present in H1; absent from prior INDEX row — defect confirmed, now closed.
- SE-17c BEFORE/AFTER (F-R117-2): captured inline in NORMATIVE bullets above.
- SE-17f SCOPED VERIFICATION (sibling sweep): all 5 ADR H1s extracted via `grep -nE "^# "` per ADR file;
  all 7 SS-*.md H1s extracted via single grep invocation. Five ADR INDEX rows cross-checked.
  Three drifted rows found; three fixed. Two matched; confirmed PASS with no change.
- SE-17g NORMATIVE vs INFORMATIONAL:
  NORMATIVE: ADR-0002 "for Phase 1" restoration (F-R117-2 HIGH); ADR-0004 backtick restoration (sibling);
  ADR-0005 backtick restoration (sibling). All three alter INDEX row content.
  INFORMATIONAL: ADR-0001, ADR-0003 match confirmations; Document Map Section label convention assessment;
  Subsystem Registry name source-of-truth confirmation. No file content changed.
- SE-17e SIBLING-PROPAGATION: F-R116-1 established H1↔INDEX-row verbatim-match discipline for VP-INDEX;
  SE-17e directs this discipline to propagate to all INDEX files. This burst (R16B) applies the discipline
  to ARCH-INDEX ADR Registry as the first sibling application. Occurrence count per SE-22:
  this is occurrence #2 of the H1↔INDEX-row drift pattern (F-R116-1 in VP-INDEX = #1; F-R117-2
  in ARCH-INDEX ADR Registry = #2). D-114 observation status maintained; SE-22 3+ threshold not yet
  reached. Pattern: agents authoring INDEX rows strip qualifiers ("for Phase 1") and code spans
  (backticks) when transcribing H1 titles — likely caused by informal paraphrasing during index creation.
- SE-16d PASS: 2026-05-18T15:30:00Z > chain high-water 2026-05-18T05:30:00Z (monotonic).

## §Trace v1.0.11

**F-PHASE2-R05-04 — SS-03 Subsystem Registry trait-vs-impl split clarification** (2026-05-19T10:00:00Z):
- NORMATIVE (F-PHASE2-R05-04 HIGH — SS-03 Implementing Modules column corrected):
  Subsystem Registry SS-03 row "Implementing Modules" column updated to reflect the
  trait-vs-implementation split that governs `EngineModule` / `ClaudeCodeModule` placement.
  - SE-17c BEFORE: `monocle-core (EngineModule trait, ClaudeCodeModule adapter)`
  - SE-17c AFTER: `monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — \`monocle-runtime/src/engine/claude_code.rs\`)`
  - Source authority: `SS-engine-module.md` line 546 (`monocle-runtime/src/engine/claude_code.rs`)
    and `BC-2.03.002.md` PRE-2 (`monocle-runtime::engine::claude_code`). Both already correctly
    place `ClaudeCodeModule` in `monocle-runtime`. The ARCH-INDEX SS-03 row was incomplete:
    it described the trait location correctly (`monocle-core`) but conflated it with
    `ClaudeCodeModule adapter` — implying the impl also lives in monocle-core. It does not.
    Standard layering: traits in core (pure, no side effects), implementations in runtime
    (spawns processes, reads env vars, interacts with filesystem). This entry corrects the
    Subsystem Registry to capture both halves of the split.
  - Sibling-sweep (SS-01, SS-02): Both rows examined for trait-vs-impl conflation.
    SS-01: `monocle-runtime (daemon binary, HTTP server, ring buffer, lock file, auth)` —
    SS-01 has no trait/impl split; all items listed are runtime implementations. PASS.
    SS-02: `monocle-core (FactoryAdapter trait, wire format types, protocol versioning)` —
    `FactoryAdapter` trait and its types are pure-core, no runtime impl in a different crate.
    This is structurally correct for SS-02; no split ambiguity. PASS.
    No sibling fixes required.
- INFORMATIONAL: Phase 2 adversary r05 finding F-PHASE2-R05-04 is the trigger for this
  correction. This does not affect SS-engine-module.md (already correct at v1.1.20) or
  BC-2.03.002 (already correct). BC-2.03.001 is untouched (PO domain, concurrent work).
- SE-16d PASS: 2026-05-19T10:00:00Z > chain high-water 2026-05-18T15:30:00Z (monotonic).

## §Trace v1.0.12

**SS-07 Config subsystem registration** (2026-05-26T00:00:00Z):
- NORMATIVE: SS-07 (Config) row added to Subsystem Registry. Implementing crate:
  `monocle-config`. Architecture doc: `SS-config.md` (new artifact, same burst).
- NORMATIVE: SS-07 row added to Capability Traceability table: CAP-007 (config
  persistence, harness profiles, profile picker, CCR detection).
- NORMATIVE: SS-config.md added to Document Map with ~2,600 token estimate and
  primary consumers (implementer, test-writer).
- NORMATIVE: Cross-References row added: "Config crate implementation" →
  `SS-config.md + SS-deps-pin-manifest.md + SS-conventions-anti-patterns.md`.
- INFORMATIONAL: SS-04 (Daemon Wiring), SS-05 (IPC), and SS-06 (TUI) are proposed
  in `prd-expansion-scope.md` §Section 2 but their architecture documents do not
  exist yet at this write time. They are NOT registered here; they will be registered when their
  respective SS-NN.md files are produced. Append-only registry discipline preserved.
  [Correction: SS-04 was registered in §Trace v1.0.13 in the same session, after
  SS-daemon-wiring.md was produced. SS-05 was registered in §Trace v1.0.14 after
  SS-ipc.md was produced. SS-06 was registered in §Trace v1.0.15 after SS-tui.md
  was produced.]
- version: 1.0.11 → 1.0.12; timestamp: 2026-05-19T10:00:00Z → 2026-05-26T00:00:00Z.
- SE-16d PASS: 2026-05-26T00:00:00Z > chain high-water 2026-05-19T10:00:00Z (monotonic).

## §Trace v1.0.13

**SS-04 Daemon Wiring subsystem registration** (2026-05-26T01:00:00Z):
- NORMATIVE: SS-04 (Daemon Wiring) row added to Subsystem Registry. Implementing crates:
  `monocle` (binary crate — `main.rs`, `clap` CLI, daemon entrypoint, TUI entrypoint);
  `monocle-runtime` (hooks-settings.json generation, bounded event bus,
  `MONOCLE_NO_AUTOSTART` check). Architecture doc: `SS-daemon-wiring.md` (new artifact,
  this burst).
- NORMATIVE: SS-04 row added to Capability Traceability table: CAP-004 (binary
  composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile
  generation).
- NORMATIVE: `SS-daemon-wiring.md` added to Document Map with ~4,800 token estimate and
  primary consumers (orchestrator, implementer, test-writer).
- NORMATIVE: Cross-References row added: "Daemon binary wiring (composition root)" →
  `SS-daemon-wiring.md + SS-daemon-lifecycle.md + SS-engine-module.md`.
- NORMATIVE: §Trace v1.0.12 informational note corrected to record that SS-04 was
  subsequently registered in this (v1.0.13) burst.
- version: 1.0.12 → 1.0.13; timestamp: 2026-05-26T00:00:00Z → 2026-05-26T01:00:00Z.
- SE-16d PASS: 2026-05-26T01:00:00Z > chain high-water 2026-05-26T00:00:00Z (monotonic).

## §Trace v1.0.14

**SS-05 IPC subsystem registration** (2026-05-26T02:00:00Z):
- NORMATIVE: SS-05 (IPC) row added to Subsystem Registry. Implementing crate:
  `monocle-ipc` (UDS client + server, Transport trait, message types, framing,
  reconnection logic). Architecture doc: `SS-ipc.md` (new artifact, this burst).
- NORMATIVE: SS-05 row added to Capability Traceability table: CAP-005 (internal
  TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision
  routing; SOQ-3 overlay clear).
- NORMATIVE: `SS-ipc.md` added to Document Map with ~3,200 token estimate and primary
  consumers (implementer, test-writer).
- NORMATIVE: Cross-References row added: "IPC protocol (TUI ↔ daemon transport)" →
  `SS-ipc.md + SS-daemon-wiring.md`.
- version: 1.0.13 → 1.0.14; timestamp: 2026-05-26T01:00:00Z → 2026-05-26T02:00:00Z.
- SE-16d PASS: 2026-05-26T02:00:00Z > chain high-water 2026-05-26T01:00:00Z (monotonic).

## §Trace v1.0.15

**SS-06 TUI subsystem registration** (2026-05-26T03:00:00Z):
- NORMATIVE: SS-06 (TUI) row added to Subsystem Registry. Implementing crates:
  `monocle-core` (AppMode, Action, FocusSnapshot, PanelId, PromptModal, BindingSource,
  Binding, transition() — pure types and transition function);
  `monocle-tui` (ratatui renderer, panel layout, crossterm event loop, IPC client,
  keybinding dispatcher). Architecture doc: `SS-tui.md` (new artifact, this burst).
- NORMATIVE: SS-06 row added to Capability Traceability table: CAP-006 (user-facing
  TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon;
  permission overlay stack; Ctrl-\ popup integration).
- NORMATIVE: `SS-tui.md` added to Document Map with ~5,200 token estimate and primary
  consumers (implementer, test-writer, formal-verifier).
- NORMATIVE: Cross-References row added: "TUI implementation (panels + overlay)" →
  `SS-tui.md + SS-ipc.md + SS-core-types-and-abi.md + SS-deps-pin-manifest.md`.
- NORMATIVE: §Trace v1.0.12 informational note updated to record that SS-06 was
  subsequently registered in this (v1.0.15) burst.
- version: 1.0.14 → 1.0.15; timestamp: 2026-05-26T02:00:00Z → 2026-05-26T03:00:00Z.
- SE-16d PASS: 2026-05-26T03:00:00Z > chain high-water 2026-05-26T02:00:00Z (monotonic).

## §Trace v1.0.16

**SS-tui.md keybinding canonicalization and IPC-driven pop semantics** (2026-05-27T00:00:00Z):
- NORMATIVE: SS-tui.md bumped 1.6.0 → 1.7.0. No Subsystem Registry or Document Map
  structural changes; the doc token estimate is unchanged.
- INFORMATIONAL: BC-2.06.011, BC-2.06.012, BC-2.06.013 updated to v1.1.0 by PO with
  mnemonic keybindings (`y`/`Enter` = Accept-Once; `A` = Accept-Always; `n`/`r` = Reject)
  replacing the numeric set (`[1]`/`[2]`/`[3]`). SS-tui.md v1.7.0 propagates these changes
  to all affected locations: §Dispatcher Logic comment, §Status Bar keybinding hint line,
  §Overlay Stack Lifecycle Step 3, §Killer Scenario table.
- INFORMATIONAL: SS-tui.md v1.7.0 also corrects overlay pop semantics: per BC-2.06.023,
  the TUI does NOT pop the front `PromptModal` on decision keypress. The `transition()`
  decision arms now leave `AppMode::Overlay` unchanged; removal is exclusively IPC-driven
  via `ServerToClient::PermissionPromptResolved { prompt_id }` → `handle_ipc_message()`
  → `stack.retain()`. Key Invariant 2 rewritten to reflect this.
- version: 1.0.15 → 1.0.16; timestamp: 2026-05-26T03:00:00Z → 2026-05-27T00:00:00Z.
- SE-16d PASS: 2026-05-27T00:00:00Z > chain high-water 2026-05-26T03:00:00Z (monotonic).

## §Trace v1.0.18

**ADR-0008 ADR Registry registration — D-206 structural-spec drift tripwire closure** (2026-05-29T12:00:00Z):
- NORMATIVE: ADR-0008 row added to ADR Registry. Title: `Structural-Claim Discipline —
  Canonical Shape Anchors + POL-12 Detection`. File:
  `adr/ADR-0008-structural-claim-discipline.md`. Status: accepted.
- NORMATIVE: ADR Registry **Note** paragraph updated to add ADR-0008 notes (passes 26/27
  structural-spec drift tripwire, Task #9 m.6, POL-12 dispatch).
- NORMATIVE: version: 1.0.17 → 1.0.18; timestamp: 2026-05-29T08:00:00Z → 2026-05-29T12:00:00Z.
- INFORMATIONAL: ADR-0008 ratifies a distinct sub-species of the ADR-0007 META-pattern root.
  ADR-0007 is unchanged at v1.0.1. Both ADRs now apply; see ADR-0008 §Relationship to ADR-0007.
- SE-16d PASS: 2026-05-29T12:00:00Z > chain high-water 2026-05-29T08:00:00Z (monotonic).

## §Trace v1.0.22

**ADR-0007 v1.0.8 — LIVING-STATE exempt set extension** (2026-05-30):

- NORMATIVE: ADR-0007 Note in ADR Registry updated with v1.0.8 summary. EXEMPT set
  extended with three new living-state files: `stories/sprint-state.yaml` (continuously-
  rewritten sprint dashboard), `tech-debt-register.md` (living human-directed debt
  register), and `repo_root/CLAUDE.md` (living project instructions). All three apply
  the STATE.md rationale (continuous rewriting, transient version refs, version-race
  false positives). EXEMPT set remains CLOSED. Adjudication: dependency-graph-expansion.md
  and holdout-scenarios.md are NORMATIVE (not exempt). Devops dispatch updated with
  exact-path exclusion specs for the three new exempt files, including CLAUDE.md
  repo-root resolution mechanism.
- NORMATIVE: version-pin-registry.yaml: ADR-0007 → v1.0.8; ARCH-INDEX → v1.0.22.
- NORMATIVE: ARCH-INDEX version 1.0.21 → 1.0.22.
- SE-16d PASS: 2026-05-30 >= chain high-water 2026-05-30 (sequential same-day patch;
  v1.0.21 and v1.0.22 are distinct bursts on the same calendar day).

## §Trace v1.0.21

**ADR-0007 v1.0.7 — Pattern A registry-driven amendment (F-S025-ADV31-MED-001)** (2026-05-30):

- NORMATIVE: ADR-0007 Note in ADR Registry updated with v1.0.7 summary. Pattern A amended
  from hardcoded prefix alternation to registry-key-driven matcher. Vocabulary of detectable
  artifact IDs = all keys in version-pin-registry.yaml at runtime. Longest-match sort
  (descending ID length) and word-boundary requirement specified. Hardcoded alternation
  `(SS-[a-z-]+\|BC-[0-9.]+\|...)` explicitly forbidden in devops implementation.
- NORMATIVE: version-pin-registry.yaml: ADR-0007 → v1.0.7; ARCH-INDEX → v1.0.21.
- NORMATIVE: ARCH-INDEX version 1.0.20 → 1.0.21.
- SE-16d PASS: 2026-05-30 >= chain high-water 2026-05-30 (sequential same-day patch;
  v1.0.20 and v1.0.21 are distinct bursts on the same calendar day).

## §Trace v1.0.20

**ADR-0007 v1.0.6 — inputs[] closed-rule ratification** (2026-05-30):

- NORMATIVE: ADR-0007 Note in ADR Registry updated: v1.0.5 and v1.0.6 additions documented.
  v1.0.5: §inputs[] Provenance Classification (story inputs[] = HISTORICAL; living index doc
  inputs[] = ACTIVE). v1.0.6: closed-rule ratification — default HISTORICAL for all doc classes;
  ACTIVE set CLOSED to `*-INDEX.md` basename OR `prd.md`; no doc class ever "unclassified."
- NORMATIVE: version-pin-registry.yaml: ADR-0007 → v1.0.6; ARCH-INDEX → v1.0.20.
- NORMATIVE: ARCH-INDEX version 1.0.19 → 1.0.20.
- SE-16d PASS: 2026-05-30 > chain high-water 2026-05-30T01:00:00Z (sequential same-day patch).

## §Trace v1.0.19

**ADV-29 scope ratification — ADR-0007 v1.0.4 §Enforcement Scan Scope, ADR-0008 v1.0.3 cross-reference** (2026-05-30T01:00:00Z):

- NORMATIVE: ADR-0007 bumped v1.0.3 → v1.0.4. §Enforcement Scan Scope section added,
  formally defining NORMATIVE (scanned) vs EXEMPT (not scanned) document classes for
  POL-11. NORMATIVE: `factory_root/stories/`, `factory_root/specs/` (all subdirs),
  `crates/`, `.github/`, `scripts/` (excl. `scripts/tests/`), root config files.
  EXEMPT (new additions alongside the pre-existing `cycles/` exemption):
  `factory_root/plans/`, `factory_root/planning/`, `factory_root/code-delivery/`,
  `factory_root/STATE.md`. Rationale: frozen historical records and living-state
  dashboard; pin-freshness is semantically wrong for these classes. Human-approved.
- NORMATIVE: ADR-0008 bumped v1.0.2 → v1.0.3. §CI enforcement gate scope note added
  confirming POL-12's narrower scan (stories + BCs + crate doc-comments) does not need
  the same exemptions as POL-11; cross-references ADR-0007 v1.0.4 §Enforcement Scan Scope.
- NORMATIVE: ADR Registry Note for ADR-0007 updated to document v1.0.4 scope addition.
- NORMATIVE: version-pin-registry.yaml updated: ADR-0007 → v1.0.4; ADR-0008 → v1.0.3;
  ARCH-INDEX → v1.0.19.
- NORMATIVE: ARCH-INDEX version 1.0.18 → 1.0.19.
- SE-16d PASS: 2026-05-30T01:00:00Z > chain high-water 2026-05-29T12:00:00Z (monotonic).

## §Trace v1.0.17

**ADR-0006 + ADR-0007 ADR Registry registration — D-204 architect-escalation tripwire closure** (2026-05-29T08:00:00Z):
- NORMATIVE: ADR-0006 row added to ADR Registry (was previously absent; introduced S-022 cycle
  but not registered in ARCH-INDEX). Title: `Non-Exhaustive Structs with Public Positional
  Constructors`. File: `adr/ADR-0006-non-exhaustive-structs-with-public-constructors.md`.
  Status: accepted. Registration is architectural bookkeeping; no content change to the ADR.
- NORMATIVE: ADR-0007 row added to ADR Registry. Title: `Version-Pin Citation Discipline —
  Semantic Anchors + CI Registry Enforcement`. File:
  `adr/ADR-0007-version-pin-citation-discipline.md`. Status: accepted.
- NORMATIVE: ADR Registry **Note** paragraph updated to add ADR-0006 and ADR-0007 notes.
- NORMATIVE: version: 1.0.16 → 1.0.17; timestamp: 2026-05-27T00:00:00Z → 2026-05-29T08:00:00Z.
- INFORMATIONAL: ADR-0006 omission from prior registry was a registration gap — the ADR was
  authored and its content cross-referenced in SS-conventions-anti-patterns.md v1.31.x but
  the ARCH-INDEX table row was never added. This entry closes the gap.
- SE-16d PASS: 2026-05-29T08:00:00Z > chain high-water 2026-05-27T00:00:00Z (monotonic).
## §Trace v1.0.23 — POL-11 version-pin remediation (2026-05-30)

**Bump:** 1.0.22 → 1.0.23.
**Scope:** ADR-0007 Note row: added `<!-- version-pin-historical -->` to "`dtu-assessment.md v1.7.5`" reference (Option 3 per ADR-0007 §Historical Anchor Classification — this text documents what version was stale at Pass 31 as a historical diagnostic record; it is not a live navigation pointer).
**SE-16d PASS:** 2026-05-30 >= 2026-05-30 (same-day patch; no normative content change).
