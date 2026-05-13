---
document_type: consistency-report
level: ops
version: "1.0"
producer: consistency-validator
phase: pre-phase-1-final-gate-round-28
timestamp: 2026-05-13T23:55:00Z
input-hash: "[live-state]"
traces_to: "round-27 fix burst commits 9be1033 + 48d952a + a1c83a9; SS-engine-module.md v1.1.7; SS-conventions-anti-patterns.md v1.5; product-brief.md v1.4.13"
project: monocle
---

# Consistency Audit — Round 28 (Post-Round-27 Fix Burst)

**Scope:** SS-engine-module.md v1.1.7, SS-core-types-and-abi.md v1.2.3,
SS-daemon-lifecycle.md v1.0.4, SS-permissions-phase1.md v1.1,
SS-deps-pin-manifest.md v1.1.7, SS-conventions-anti-patterns.md v1.5,
SS-forward-compatibility.md v1.2.1, all 4 ADRs,
product-brief.md v1.4.13, domain-monocle-vision-synthesis.md v1.1.2,
dtu-assessment.md, STATE.md, CLAUDE.md (read-only).

**Overall result: CLEAN — 0 CRITICAL / 0 MEDIUM / 1 LOW (informational) / all targeted checks PASS**

---

## Summary Table

| Check | Criterion | Result |
|-------|-----------|--------|
| 1 | Constructor consistency — EngineMetadata::new (4 args, correct order) | PASS |
| 2 | Constructor consistency — ProcessSnapshot::new (4 args), ::with_full_context (7 args) | PASS |
| 3 | Constructor consistency — EnrichedSession::new (6 args, correct order) | PASS |
| 4 | Constructor consistency — HookResponse::new (1 arg, defaults correct) | PASS |
| 5 | No remaining struct literals for E0639-affected types in any spec | PASS |
| 6 | BC-ENGINE-002 test spec uses ProcessSnapshot::new with 4 complete args | PASS |
| 7 | BC-ENGINE-002-ERR async test spec uses ProcessSnapshot::new with 4 complete args | PASS |
| 8 | Version pointer consistency — STATE.md Critical Artifacts table | PASS |
| 9 | Version pointer consistency — product-brief inline citations (SS-engine-module v1.1.7) | PASS |
| 10 | Semgrep rule count — 4 rules present in .semgrep.yml block | PASS |
| 11 | Semgrep §Test Conventions cross-references §Semgrep Rules (no duplicate YAML) | PASS |
| 12 | §Semgrep Coverage Hardening subsection present with fixture table + CI step specs | PASS |
| 13 | pattern-either covers all 4 env-mutation variants | PASS |
| 14 | BC count = 16 consistent across STATE.md, SS-core-types-and-abi, SS-forward-compat, brief | PASS |
| 15 | BC-ENGINE-002-ERR appears in SS-engine-module pre-staging table | PASS |
| 16 | Engine BC count = 4 (ENGINE-001/002/002-ERR/003) in all enumerations | PASS |
| 17 | v1.1.4 trace block supersession annotation present and correct | PASS |
| 18 | v1.1.5 trace block supersession annotation present and correct | PASS |
| 19 | CLAUDE.md §Current Pipeline State stale text captured (report-only) | PASS |
| 20 | CLAUDE.md §Architectural Authority stale text captured (report-only) | PASS |
| 21 | Vision non-authoritative framing internally consistent post-round-27 | PASS |
| 22 | §Semgrep Coverage Hardening referenced from §Test Conventions | PASS |
| 23 | No rationalization phrases in round-27-modified files | PASS |
| 24 | STATE.md Immediate Next Action fresh-context-executable | PASS |
| 25 | STATE.md Phase 1 Gate Questions — 3 present (D-031, D-032, Q-3) | PASS |
| 26 | STATE.md required H2 sections present | PASS |
| 27 | HookEvent inner structs (5 event structs) — no cross-crate struct literal construction in any spec | PASS |
| 28 | D-032 — no spec doc makes premature assumption about routing-decision outcome | PASS |
| 29 | "minimum viable signal set" phrase — technical description, not Rule 1 rationalization | PASS (see note) |
| 30 | HookDecision enum variants — not subject to E0639 (enum variants, not struct literals) | PASS |

---

## Detailed Findings

### Check 1-4: Constructor Consistency

**EngineMetadata::new — 4 args, field-declaration order:**

SS-engine-module.md lines 175-182: `pub fn new(display_name: &'static str, icon: char, config_paths: Vec<PathBuf>, hook_schema_version: u32) -> Self`. Matches struct field order (display_name, icon, config_paths, hook_schema_version). ClaudeCodeModule::metadata() call site (lines 510-519) passes: `"Claude Code"`, `'●'`, `vec![...]`, `1`. Arg count = 4. PASS.

**ProcessSnapshot::new — 4 args (minimal), field-declaration order:**

SS-engine-module.md lines 262-277: `pub fn new(pid: u32, exe_path: Option<PathBuf>, cmdline: Vec<String>, start_time_secs: i64) -> Self`. Sets ppid=None, working_dir=None, env=HashMap::new(). Note: struct field declaration order is pid, ppid, exe_path, cmdline, working_dir, env, start_time_secs. The constructor arg order is pid, exe_path, cmdline, start_time_secs — this **skips the optional fields** intentionally (they are set to None/empty in the body). This is a deliberate two-tier design choice where the constructor takes only the four detection-relevant args. The rustdoc explains this. PASS — the field-declaration-order criterion for new() applies only to the fields that are args; the skipped fields have documented defaults.

**ProcessSnapshot::with_full_context — 7 args, all 7 fields in declaration order:**

Lines 284-295: `pub fn with_full_context(pid, ppid, exe_path, cmdline, working_dir, env, start_time_secs)`. Matches struct field order exactly. PASS.

**EnrichedSession::new — 6 args, field-declaration order:**

Lines 346-355: `pub fn new(session_id: String, harness_type: String, transcript_path: Option<PathBuf>, config_path: Option<PathBuf>, status: SessionStatus, last_event_micros: i64) -> Self`. Matches struct field order exactly. ClaudeCodeModule::enrich() call site (lines 561-568) passes: session_id, self.id().to_string(), transcript_path, Some(claude_config_root), SessionStatus::Active, 0. Arg count = 6. PASS.

**HookResponse::new — 1 required arg (decision), 2 fields default to None:**

Lines 414-416: `pub fn new(decision: HookDecision) -> Self { Self { decision, redirect_url: None, diagnostic: None } }`. ClaudeCodeModule::on_hook call site: `HookResponse::new(HookDecision::Allow)`. The `redirect_url` and `diagnostic` fields are set to None per the Phase 1 production-correct defaults. PASS.

### Check 5: No Remaining E0639-Affected Struct Literals

Full-text search across all spec files for `EngineMetadata {`, `ProcessSnapshot {`, `EnrichedSession {`, `HookResponse {` — all hits are either struct _declarations_ (`pub struct EngineMetadata {`) or explicit rustdoc illustration of the _forbidden_ pattern in the constructor rationale ("construction (`EngineMetadata { display_name: ..., ... }`) is forbidden outside the defining crate"). No live construction sites use struct literal syntax. PASS.

### Check 6-7: BC-ENGINE-002 and BC-ENGINE-002-ERR Test Spec Call Sites

BC-ENGINE-002 test spec (SS-engine-module.md lines 776-779): all three ProcessSnapshot::new() calls have exactly 4 positional arguments with complete values. PASS.

BC-ENGINE-002-ERR async test spec (lines 854-859): `ProcessSnapshot::new(12345, Some(PathBuf::from("/usr/local/bin/claude")), vec![], 1_700_000_000)` — 4 positional arguments, all specified. Comment confirms which 3 fields (ppid, working_dir, env) are defaulted by the constructor. PASS.

### Check 8: STATE.md Version Pointer Consistency

STATE.md Critical Artifacts table (lines 113-123):
- SS-engine-module.md v1.1.7 — matches actual frontmatter PASS
- SS-core-types-and-abi.md v1.2.3 — matches actual frontmatter PASS
- SS-daemon-lifecycle.md v1.0.4 — matches actual frontmatter PASS
- SS-permissions-phase1.md v1.1 — matches actual frontmatter PASS
- SS-deps-pin-manifest.md v1.1.7 — matches actual frontmatter PASS
- SS-conventions-anti-patterns.md v1.5 — matches actual frontmatter PASS
- SS-forward-compatibility.md v1.2.1 — matches actual frontmatter PASS

### Check 9: Product-Brief Inline Citations

Forward-compatibility Success Criteria row (product-brief.md line 245): "Per `SS-core-types-and-abi.md`, `SS-daemon-lifecycle.md` v1.0.4, and `SS-engine-module.md` v1.1.7." SS-engine-module.md citation is v1.1.7 (current). SS-daemon-lifecycle.md is v1.0.4 (current). SS-core-types-and-abi.md citation is unversioned (the 1.4.13 revision history note at line 77 confirms "SS-conventions-anti-patterns.md and SS-core-types-and-abi.md have no versioned inline body citations"). PASS.

### Check 10-13: Semgrep Rule Visibility

SS-conventions-anti-patterns.md v1.5 §Semgrep Rules block contains exactly 4 rules:
1. `monocle-no-shell-injection` (pattern-either: sh, bash)
2. `monocle-no-naked-fs-write` (pattern-either: std::fs::write, tokio::fs::write)
3. `monocle-no-unbounded-channel` (single pattern)
4. `monocle-no-raw-env-mutation-in-tests` (pattern-either: 4 patterns)

Rule 4 `pattern-either` covers: `std::env::set_var($X, $Y)`, `std::env::remove_var($X)`, `env::set_var($X, $Y)`, `env::remove_var($X)`. All four env-mutation variants present. PASS.

§Test Conventions §CI enforcement paragraph (lines 499-504) references "§Semgrep Rules above" as the canonical single-source-of-truth location. No duplicate YAML block in §Test Conventions. PASS.

§Semgrep Coverage Hardening subsection (lines 125-190) is present between §Semgrep Rules and §PR Template Checklist. Contains:
- Fixture corpus table (4 rows: one per rule, with fixture file path and violation pattern)
- CI Step 1 specification (fixture corpus scan, expected finding counts, log line format, failure behavior)
- CI Step 2 specification (production scan, zero-findings assertion, log line format)
- Step ordering (fixture first; skip production scan if fixture fails)
PASS.

§Test Conventions (line 504) cross-references "§Semgrep Coverage Hardening for the positive-coverage fixture corpus requirement (POL-11)". PASS.

### Check 14-16: BC Count and Enumeration

SS-core-types-and-abi.md (lines 1035-1037): "Combined with SS-engine-module.md (BC-ENGINE-001, BC-ENGINE-002, BC-ENGINE-002-ERR, BC-ENGINE-003 = 4 BCs) and SS-daemon-lifecycle.md (BC-RING-001, BC-AUTH-001, BC-AUTH-002, BC-LOCK-001 = 4 BCs), the pre-Phase-1 pre-staged total is 16 BCs across all architecture artifacts." Engine BC count = 4, correctly includes BC-ENGINE-002-ERR. PASS.

SS-forward-compatibility.md (line 232 + BC table): "The following 16 pre-staged BC IDs are RESERVED" with BC-ENGINE-002-ERR listed at line 251. PASS.

product-brief.md Success Criteria (line 245): "16 behavioral contracts pre-staged for Phase 1 PRD: BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001a/001b/002, BC-RING-001, BC-AUTH-001/002, BC-ENGINE-001/002/002-ERR/003, BC-LOCK-001." Count: ABI-001+002=2, TYPES-001=1, FACTORY-001+002=2, PROTO-001a+001b+002=3, RING-001=1, AUTH-001+002=2, ENGINE-001+002+002-ERR+003=4, LOCK-001=1 → total=16. PASS.

STATE.md brief v1.4.13 (line 48). PASS.

### Check 17-18: Supersession Annotations

SS-engine-module.md v1.1.4 trace block (lines 982-987):
"NOTE: Superseded by v1.1.5 (BC-ENGINE-002-ERR added to Pre-Staging table; cross-ref consistency fix) and v1.1.6 (test-spec async/sync split; temp-env ^0.2 → ^0.3; env-var list HOME+USERPROFILE+HOMEDRIVE+HOMEPATH; XDG_* removed). The v1.1.4 temp-env pin (^0.2) and XDG_* env-var list in this entry are SUPERSEDED — implementers MUST follow the v1.1.6 (and later v1.1.7) specifications."
Clear, actionable, correctly identifies both superseding versions. PASS.

SS-engine-module.md v1.1.5 trace block (lines 968-971):
"NOTE: Superseded by v1.1.6 (F-R24-adv-1: test spec async/sync split; temp-env ^0.2 → ^0.3; env-var list corrected to HOME+USERPROFILE+HOMEDRIVE+HOMEPATH; XDG_* removed; commits captured in v1.1.6 trace) and v1.1.7 (F-R26-adv-1: constructors added; F-R26-adv-5: ProcessSnapshot args fully specified in test; F-R26-2: these annotations added)."
Clear, identifies both superseding versions and specific findings. PASS.

### Check 19-20: CLAUDE.md Stale Text (Read-Only — Reporting for Human)

CLAUDE.md was read only. It was NOT modified. Stale text confirmed:

**§Current Pipeline State (CLAUDE.md line 22):**
Current text: `Brief: \`v1.4.2\` at \`.factory/specs/product-brief.md\`, \`validate-brief\` verdict: v5 VALID.`
Correct value: `Brief: \`v1.4.13\`; validate-brief verdict: v5 VALID.`

**§Architectural Authority item 6 (CLAUDE.md line 47):**
Current text: `` `.factory/specs/product-brief.md` v1.4.2 ``
Correct value: `` `.factory/specs/product-brief.md` v1.4.13 ``

**§Architectural Authority item 7 (CLAUDE.md line 48):**
Current text: `` `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.1 ``
Correct value: `` `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 ``

ACTION FOR HUMAN at Phase 1 gate: update CLAUDE.md lines 22, 47, and 48 to the current values listed above. Routing: human (CLAUDE.md is human-authored authority; AI agents do not edit it). This is consistent with STATE.md Phase 1 Gate Question 3.

### Check 21: Vision Non-Authoritative Framing Internal Consistency

SS-engine-module.md v1.1.7 §EngineModule Trait Signature opening section explicitly states: "the vision is non-authoritative for this surface per CLAUDE.md §Architectural Authority ('the LATER, MORE-SPECIFIC artifact wins'); SS-engine-module.md is both later and more specific. Implementers MUST use the Result forms defined here, not the infallible vision sketch." No other spec doc has reverted or contradicted this framing post-round-27. SS-forward-compatibility.md v1.2.1 (N16-7) confirms sealed-trait analysis reflects open-trait (no stale sealed-pattern references). PASS.

### Check 22: §Semgrep Coverage Hardening Anchor Cross-Reference

§Test Conventions (line 504) contains explicit reference: "See §Semgrep Rules for the full rule definition and §Semgrep Coverage Hardening for the positive-coverage fixture corpus requirement (POL-11)." §Semgrep Coverage Hardening is not referenced from any other doc that depends on it — it is a self-contained devops-engineer specification. No broken anchors. PASS.

### Check 23: Production-Grade Compliance — No Rationalization Phrases

Full-text search of round-27-modified files (SS-engine-module.md v1.1.7, SS-conventions-anti-patterns.md v1.5, product-brief.md v1.4.13) for: "for now", "good enough", "can fix later", "minimum viable", "MVP", "ship fast", "placeholder".

Results: "minimum viable signal set" appears in ProcessSnapshot::new rustdoc (line 230). This is a technical description — it describes the minimal OS-observable signal set sufficient for the detection use case, distinguishing the two-tier constructor design. It is not a rationalization for incomplete work; the full production-grade implementation of both tiers is specified in the same document. This phrase is NOT a CLAUDE.md Rule 1 rationalization because: (a) no work is deferred, (b) both constructors are fully specified, (c) the rationale explicitly argues against a builder pattern as unnecessary complexity for a type with a natural two-tier access pattern. LOW informational note; not a finding.

No MVP, "good enough", "for now", or "ship fast" phrases found in the three files. PASS.

### Check 24-26: STATE.md Structure

**Immediate Next Action:** Describes round-28 validation in complete detail — lists the exact checks (a)-(g), names specific files and versions, identifies the HookEvent inner struct question as a specific verification target. Fresh-context-executable: a new agent can read this and begin work immediately. PASS.

**Phase 1 Gate Questions:** Three questions present at STATE.md lines 180-184:
1. D-031 — Vision-vs-architecture authority
2. D-032 — Architect-brief-routing precedent
3. Q-3 — CLAUDE.md operational pointer refresh (includes exact stale values and correct replacements)
All three have clear answer spaces. PASS.

**Required H2 sections:** READ THIS FIRST, Project Metadata, Phase Progress, Current Phase Steps, Decisions Log, Skip Log, Blocking Issues, Session Resume Checkpoint, Historical Content — all present. PASS.

### Check 27: HookEvent Inner Structs — No Cross-Crate Construction

The five inner event structs (SessionStartEvent, UserPromptSubmitEvent, PreToolUseEvent, NotificationEvent, StopEvent) are defined in `monocle-core/src/hook_events.rs` and carry `#[non_exhaustive]`. All five are deserialized by the daemon's axum handlers via `serde_json::from_str` — that is the only spec-defined construction path. No spec document shows a struct literal construction (`PreToolUseEvent { tool_name: ..., ... }`) outside the defining crate.

The test suite spec in SS-core-types-and-abi.md (BC-PROTO-001b) constructs a `HookEnvelope` with `schema_version: 1` — this is a prost-generated struct, not a `#[non_exhaustive]` Rust struct, so E0639 does not apply. PASS.

The BC-ENGINE-003 test spec constructs no HookEvent inner structs. The on_hook test in BC-ENGINE-001 would receive a HookEvent from the daemon's deserialization path, not from test-side construction. No E0639 gap exists in the inner structs. PASS.

**Note for adversary:** The inner structs are `#[non_exhaustive]` and lack constructors. If any integration test eventually needs to construct a HookEvent for testing on_hook dispatch (e.g., to verify ClaudeCodeModule::on_hook returns HookResponse::Allow for a PreToolUseEvent), that test code will hit E0639 when constructing `PreToolUseEvent { ... }` outside monocle-core. The spec does not currently include such a test — on_hook is spec'd with `todo!()` in Phase 1 and its test is deferred to Phase 1 stories. This is a pre-existing structural risk, not a round-27 regression. It should be flagged for the Phase 1 story writer: monocle-core should export constructors or a test-builder for HookEvent inner structs before Phase 1 test implementation begins.

**Severity: LOW / informational.** No current spec is broken. A future implementer following the spec will encounter this at Phase 1 test-writing time, not before.

### Check 28: D-032 — No Premature Routing-Decision Commitment

product-brief.md v1.4.12 revision history (line 78) frames the routing precedent as a gate question with two explicitly undecided outcomes ("should architects be permitted to mechanically propagate counts... or should every cross-boundary edit route through the destination owner?"). No current spec doc assumes a specific answer. STATE.md gate question 2 (line 182) is equally neutral. PASS.

### Check 29: "minimum viable signal set" Phrase

See Check 23. Technical usage, not Rule 1 rationalization. Informational note only.

### Check 30: HookDecision — No E0639 Issue

`HookDecision::Allow` is an enum variant, not a struct field. Rust E0639 applies to struct literal construction (`MyStruct { field: value }`), not to enum variant construction. `HookDecision::Deny { reason: ... }` and `HookDecision::Defer { until: ... }` use named-field variant syntax — these ARE potentially affected if constructed outside monocle-core. However, the only spec-defined construction is `HookDecision::Allow` in `ClaudeCodeModule::on_hook`, which is a unit variant with no fields. No current spec shows cross-crate construction of `HookDecision::Deny` or `HookDecision::Defer` — those would also hit E0639 since HookDecision is `#[non_exhaustive]`. The same Phase 1 story concern applies: if Phase 1 tests need to construct Deny or Defer responses, they will need constructors. Not a current spec defect; informational for Phase 1. PASS.

---

## Frontmatter Input-Hash Status

All spec files use `input-hash: "[live-state]"`. No stale computed hashes present. PASS.

---

## Convergence Assessment

| Round | CRITICAL | MEDIUM | LOW |
|-------|----------|--------|-----|
| R20 | 0 | 2 | 1 |
| R22 | 0 | 3 | 0 |
| R24 | 0 | 3 | 2 |
| R26 | 1 | 2 | 3 |
| **R28** | **0** | **0** | **1** |

**Trend: Strong convergence.** The R26 CRITICAL (F-R26-adv-1, E0639) was a long-latent pre-existing defect, not a new regression — the fix burst addressed it comprehensively in 4 structs plus HookResponse. R28 finds no new defects introduced by the round-27 burst. The single LOW finding (HookEvent inner struct E0639 risk at Phase 1 test-writing time) is a pre-existing structural observation, not a new gap.

The round-27 fixes are internally consistent, correctly applied, and do not introduce new inconsistencies. All 30 targeted checks pass.

**Gate recommendation: READY for adversary pass. On adversary passing (0 CRITICAL + 0 MEDIUM), ready for Phase 1 gate presentation to human.**

---

## CLAUDE.md Edit Confirmation

CLAUDE.md was NOT modified. It was read-only. The stale text is captured above in Check 19-20 for the human to refresh at Phase 1 gate review. This is consistent with Q-3 in STATE.md Phase 1 Gate Questions.
