---
document_type: adversary-report
version: "1.0"
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.20 9371348 + VP v1.27 202e15c + arch v1.0.20 8533ea2 + manifest v1.1.12 8005075; D-047 strict pass 1 attempt 27 (R94); post-F-R93 serial fix-burst snapshot; CONTENT-CENTRIC LENS — implementation example correctness recursive + content-centric BC semantic gap"
level: ops
producer: adversary
project: monocle
status: complete
phase: phase-1-spec-crystallization
timestamp: 2026-05-15T22:05:11Z
pass_number: 1
attempt: 27
policy: D-047-strict
verdict: FINDINGS
counter_before: 0/3
counter_after: 0/3
findings_count: 2 HIGH + 3 MEDIUM + 2 LOW observations
lens_class: CONTENT-CENTRIC (implementation example correctness + BC semantic gap)
---

# Adversary Pass R94 — Phase 1 Spec Review

**Policy:** D-047 strict (0 findings of any severity for 3 consecutive passes required)
**Attempt:** 27 (pass 1 of current cycle)
**Lens:** CONTENT-CENTRIC — implementation example correctness recursive + content-centric BC semantic gap
**Input artifacts:** PRD v1.20 (9371348) + VP v1.27 (202e15c) + arch v1.0.20 (8533ea2) + manifest v1.1.12 (8005075)
**Counter before:** 0/3
**Counter after:** 0/3 (FINDINGS — counter stays)

---

## Verdict: FINDINGS

2 HIGH + 3 MEDIUM + 2 LOW observations. Counter stays at 0/3.

---

## Critical Findings (HIGH)

### C-R94-1 HIGH — Orchestrator-induced defect: imprecise doc-comment rationale in arch (paths (b) and (c) lumped)

**Location:** arch SS-daemon-lifecycle.md lines 232-234 doc-comment rationale for `resolve_runtime_dir`.

**Finding:** The F-R93 architect dispatch prompt authored an imprecise doc-comment rationale that lumps paths (b) and (c) together: "paths (b) and (c) can only return None/empty IF ProjectDirs::new() returned None."

This is factually wrong for path (b). `ProjectDirs::runtime_dir()` returns `Option<&Path>` and returns `None` on macOS and Windows **by platform-ABI design** — not because `ProjectDirs::new()` failed. On macOS there is no conventional "runtime" directory (XDG_RUNTIME_DIR is Linux-only); on Windows there is no equivalent. The method returning `None` is normal, expected, documented behavior for those platforms regardless of whether `ProjectDirs::new()` succeeded.

The inline comment at arch line 250 correctly states "data_local_dir() returns &Path (never Option) — this branch is infallible," which is accurate for path (c). But the doc-comment block (lines 232-234) groups paths (b) and (c) together under the "only if ProjectDirs::new() returned None" rationale, which incorrectly implies path (b) failure is also conditional on `new()` failure only.

**Root cause:** Orchestrator-induced — the F-R93 architect dispatch prompt included the rationale text that was copied verbatim into the arch doc-comment. The orchestrator's dispatch language was spec content in this case.

**Correct rationale distinction:**
- Path (a): `MONOCLE_RUNTIME_DIR` env override — only fails if value is empty string (EC-060).
- Path (b): `ProjectDirs::runtime_dir()` — returns `None` on macOS/Windows by platform-ABI design (XDG_RUNTIME_DIR Linux-only); NOT dependent on `ProjectDirs::new()` success.
- Path (c): `ProjectDirs::data_local_dir()` — returns `&Path` (never Option); infallible path when `ProjectDirs::new()` succeeded; only construction failure is `new()` returning `None` upstream.

The doc-comment must distinguish (b) from (c): path (b) can return None on macOS/Windows regardless of `new()` outcome; path (c) is infallible once `new()` succeeds.

**Severity:** HIGH — the doc-comment misrepresents a platform-ABI behavior difference between two resolution paths. This would mislead implementers writing the runtime-dir resolution chain on macOS/Windows.

**Fix route:** architect arch v1.0.21 — doc-comment lines 232-234 rewrite distinguishing path (b) macOS/Windows platform-ABI None from path (c) infallible fallback.

---

### C-R94-2 HIGH — VP-RING-001 §Post-condition 4 + Counter-example sketch 5 incorrectly list `Notification` as no-tool-surface

**Location:** VP-RING-001 §Post-condition 4 and Counter-example sketch 5 in verification-properties.md.

**Finding:** VP-RING-001 §Post-condition 4 states (paraphrasing): "Events with no tool surface (SessionStart, UserPromptSubmit, Notification, Stop) MUST serialize with `tool_name: null` and `tool_input: {}`."

This is factually wrong. `Notification` carries `tool_name` and `tool_input` fields. The gene source (BC-HOOK-019 wire schema line 329), arch lines 534-561, and PRD line 476 all confirm that `Notification` events include:
- `tool_name: String` — the tool that produced the notification
- `tool_input: serde_json::Value` — the tool's input payload

The correct no-tool-surface set is `(SessionStart, UserPromptSubmit, Stop)` — three events, not four. `Notification` is a tool-surface event.

**History:** This defect was introduced in F-R89-3 VP v1.23 when VP-RING-001 absence-of-field probe 1.d was added. The `Notification` mis-listing has survived 5 subsequent fix-bursts (v1.23 → v1.24 → v1.25 → v1.26 → v1.27) without detection because:
1. The CONTENT-CENTRIC lens was not applied to VP-RING-001 §Post-condition 4 specifically.
2. The §Post-condition 4 was not in the fix-burst diff for R89-R93 (it was pre-existing prose not modified by those bursts).
3. The adversary passes R90-R93 focused on other areas.

**Correct statement:** VP-RING-001 §Post-condition 4 must read: "Events with no tool surface **`(SessionStart, UserPromptSubmit, Stop)`** MUST serialize with `tool_name: null` and `tool_input: {}`." Counter-example sketch 5 must be updated to match (remove `Notification` from the no-tool-surface set, or rewrite to describe a `Notification` event WITH a tool name/input as the positive case).

**Severity:** HIGH — content defect in a behavioral invariant that defines the wire serialization contract for the ring buffer. Implementers writing the serializer for `Notification` events would apply the wrong contract (null tool_name/input vs populated tool_name/input).

**Fix route:** formal-verifier VP v1.28 — §Post-condition 4 + Counter-example sketch 5 correction. Also requires arch cross-check: arch lines 534-561 HookEventRecord struct annotation for `tool_name`/`tool_input` must confirm the Notification event fields are documented as required (not optional-with-None). PRD line 476 should be verified for consistency.

---

## Informational Findings (MEDIUM)

### I-R94-1 MED — arch HookEventRecord field docstring "JSON-encoded tool input" misleading

**Location:** arch SS-daemon-lifecycle.md HookEventRecord struct field `tool_input` docstring.

**Finding:** The docstring reads: `/// JSON-encoded tool input` (or similar). The actual Rust type is `serde_json::Value` — an in-memory parsed JSON tree, not an encoded string. A `serde_json::Value` is NOT "JSON-encoded"; it is a deserialized, in-memory representation. The encoded (string) form would be `String` with a `serde(with = ...)` directive.

**Impact:** Misleads implementers into believing the field holds a `String` containing JSON text (e.g., `r#"{"key":"value"}"#`) rather than a structured `serde_json::Value`. The implementation would be correct (the type annotation is right) but the docstring creates confusion during code review and may cause incorrect assertions in test code.

**Severity:** MEDIUM — docstring is semantically incorrect about the type's form. The type annotation is correct so runtime behavior is unaffected, but implementation guidance is wrong.

**Fix route:** architect arch v1.0.21 — HookEventRecord `tool_input` field docstring update: "In-memory parsed JSON tree (`serde_json::Value`); deserialized from the hook payload at intake."

---

### I-R94-2 MED — VP-RING-001 §Counter-example sketch 3 has implicit dependency on serde_json `preserve_order` feature being OFF

**Location:** VP-RING-001 §Counter-example sketch 3 in verification-properties.md.

**Finding:** Counter-example sketch 3 tests field ordering in the serialized JSON output (e.g., asserting that `tool_name` appears before `tool_input` or vice versa, or that canonical field order is preserved). This test implicitly assumes that serde_json's `preserve_order` feature is NOT enabled (default behavior serializes struct fields in definition order, not insertion order).

However, the §Pre-conditions section of VP-RING-001 does NOT document this assumption. If a future build or test environment enables `serde_json`'s `preserve_order` feature (which causes HashMap/BTreeMap-backed serialization with arbitrary key ordering), the counter-example's field-order assertions would become fragile.

**Correct fix:** VP-RING-001 §Pre-conditions must add: "serde_json `preserve_order` feature MUST be disabled (default); the ring-buffer serialization contract assumes struct-definition field order in JSON output."

**Severity:** MEDIUM — the §Pre-conditions omission could cause a future false-positive failure in the counter-example sketch if the feature is accidentally enabled. Documenting it makes the assumption explicit and auditable.

**Fix route:** formal-verifier VP v1.28 — VP-RING-001 §Pre-conditions new bullet: `serde_json` feature `preserve_order` MUST be absent from `Cargo.toml` workspace dependency (canonical: `serde_json = "=1.0.149"` with no features); if feature is present, ring-buffer field-order counter-examples are invalid.

---

### I-R94-3 MED — arch `enum AuthError` not `pub`; VP declares `pub enum AuthError`

**Location:** arch SS-daemon-lifecycle.md `enum AuthError` definition; VP verification-properties.md VP-AUTH-001/VP-AUTH-002 §Pre-conditions.

**Finding:** The arch spec shows `enum AuthError` without a `pub` visibility qualifier. The VP §Pre-conditions for VP-AUTH-001 and VP-AUTH-002 reference `pub enum AuthError` (or implicitly require public access to match/assert on variants in integration tests). Integration tests in the `tests/` directory cannot access non-`pub` enum variants for exhaustive matching; the tests would fail to compile.

**Severity:** MEDIUM — visibility mismatch between arch spec and VP test expectations. Implementers following the arch spec (no `pub`) would produce code that fails to compile against the VP-mandated integration tests.

**Fix route:** architect arch v1.0.21 — add `pub` visibility to `enum AuthError` declaration. Alternatively, if the intent is pub(crate), arch must explicitly state `pub(crate)` AND VP §Pre-conditions must document the test access pattern (e.g., use `#[cfg(test)]` re-export or `pub(crate)` with integration test access via `monocle_runtime::auth::AuthError` path). Production-grade default: `pub enum AuthError` (library error type, accessible to callers).

---

## Observations (LOW)

### O-R94-1 LOW — manifest chrono pin row mentions `shutdown_utc` without `(BC-DAEMON-006)` paren attribution

**Location:** SS-deps-pin-manifest.md manifest table chrono 0.4 row, "Used by" / attribution column.

**Finding:** The chrono 0.4 pin row attribution column lists `shutdown_utc` as a field that requires chrono, but lacks the `(BC-DAEMON-006)` parenthetical attribution that other field entries in the same column carry. For example, other fields in the same or adjacent rows have the form `fieldname (BC-NNN-NNN)`. The `shutdown_utc` field omits this paren.

**Severity:** LOW — stylistic inconsistency; does not affect correctness. Other rows with BC attributions may be verifiable via grep; `shutdown_utc` attribution is only recoverable by cross-referencing PRD.

**Fix route:** architect (manifest update v1.1.13) — add `(BC-DAEMON-006)` paren attribution to `shutdown_utc` in the chrono row, matching the convention of sibling rows.

---

### O-R94-2 LOW [process-gap] — Doc-comment semantic-correctness audit axis codification candidate

**Finding:** C-R94-1 reveals a new defect axis: doc-comment text (Rust `///` and `/** */` blocks) can contain factually incorrect semantic claims that are structurally valid prose. Prior adversary passes have not explicitly included doc-comment semantic-correctness as a lens axis. The adversary reviewed architecture spec files (markdown), but when arch spec files include Rust-like code sketches with doc-comment text, those doc-comments must be semantically verified against the actual type/method behavior, not merely syntax-checked.

**Codification candidate:** Add to adversary dispatch prompt checklist: "For any code sketch in arch spec files, verify doc-comment claims against the actual crate's documented behavior (crates.io / docs.rs / spec). In particular: (a) Option vs infallible return type claims; (b) 'encoded' vs 'in-memory' form claims for JSON/binary types; (c) 'platform-specific' behavior claims for directories/file-system crates."

**Process note:** This is an ORCHESTRATOR-induced defect class (C-R94-1 root cause: dispatch prompt authored the imprecise rationale). Future codification should address both the adversary lens AND the orchestrator dispatch-prompt content review discipline.

**Severity:** LOW — process-gap observation. Recommend codification in a future SE (SE-18 candidate) if recurrence observed. Current instance is C-R94-1 (HIGH) being the trigger.

---

## Convergence Counter

- Counter before: 0/3
- This pass: FINDINGS
- Counter after: **0/3** (stays — FINDINGS resets counter)

---

## Next Step

SERIAL fix-burst required:
1. **architect arch v1.0.21** — C-R94-1 doc-comment rationale (distinguish path (b) macOS/Windows platform-ABI from path (c) infallible) + I-R94-1 HookEventRecord docstring + I-R94-3 `pub enum AuthError` + O-R94-1 manifest chrono shutdown_utc BC paren.
2. **manifest v1.1.13** — O-R94-1 chrono row `shutdown_utc (BC-DAEMON-006)` paren. (Can be combined with arch commit if architect owns manifest.)
3. **product-owner PRD v1.21** — arch v1.0.21 pin propagation (SE-15e pre-dispatch grep required); verify PRD line 476 Notification tool_name/tool_input consistency.
4. **formal-verifier VP v1.28** — C-R94-2 §Post-condition 4 + Counter-example 5 correction (Notification is tool-surface; no-tool set = SessionStart+UserPromptSubmit+Stop) + I-R94-2 VP-RING-001 §Pre-conditions preserve_order clause + PRD v1.21 + arch v1.0.21 pin propagation.

Disciplines in force: 27 (unchanged — no new codification classes; O-R94-2 is a process-gap candidate for SE-18 if recurrence).
