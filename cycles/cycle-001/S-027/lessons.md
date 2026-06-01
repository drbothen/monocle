# S-027 Lessons Learned + Residual Follow-Up Register

## Residual Non-Blocking Follow-Ups

These items are doc/prose-only with zero behavioral impact. They are tracked here — NOT silently
deferred — with an explicit resolution target of the wave-7-gate sweep.

### F-S027-DOC-001

**ID:** F-S027-DOC-001
**Target:** wave-7-gate sweep / BC-2.06.021 v1.0.7 (PO one-line clarification)
**Owner:** product-owner
**Severity:** doc-only, non-blocking
**Description:** BC-2.06.021 PC-3 prose — "no AppMode in which the hint is hidden or replaced"
— is stale relative to BC-2.06.019 v1.1.0 PC-7, which establishes the supersede model (hint
visibility is governed by BC-2.06.019; BC-2.06.021 PC-3 should cross-reference rather than
assert independence). PO one-line clarification needed in BC-2.06.021 v1.0.7.

### F-S027-DOC-002

**ID:** F-S027-DOC-002
**Target:** wave-7-gate sweep / implementer doc touch-up
**Owner:** implementer
**Severity:** doc-only, non-blocking
**Description:** Stale doc-comment in `monocle-tui/src/app.rs` `render_frame` function
(# Drop counter section, ~line 1659). The comment describes the legacy `[dropped: N] monocle`
path that S-027 replaced with `render_status_bar`. The legacy path description should be
updated to reference the new status bar delegation.

### F-S027-DOC-003

**ID:** F-S027-DOC-003
**Target:** wave-7-gate sweep / doc-comment touch-ups
**Owner:** implementer
**Severity:** doc-only, non-blocking
**Description:** Two stale doc-comment locations:
1. Module doc in `monocle-tui/src/ui/overlay.rs` — references `render_overlay` in this module;
   actual implementation is in `overlay_widget.rs` / `status_bar.rs`. Module doc should be
   updated to redirect readers to the correct source files.
2. Stale "SearchPrompt arm" docstrings in `tests/overlay_stub.rs` — binding is `per_context`,
   not `SearchPrompt`. Doc strings should reflect the actual binding layer.

## Lessons Learned

### L-S027-001: 18-pass convergence on rendering stories is expected

Rendering stories (overlay widget, diff preview, status bar) generate higher adversarial
churn than IPC/logic stories because:
- Visual correctness claims (layout, color, hint text) are easier to find gaps in
- BC prose for "display" acceptance criteria tends to be underspecified relative to behavior ACs
- NITPICK_ONLY oscillations (passes 6/7/8, 10/11, 15/16/17/18) are normal for rendering;
  convergence window of 3 consecutive NITPICK_ONLY is the correct stopping criterion

**Recommendation for Wave 7:** S-028 (Sessions Panel + Event Ribbon) is also a rendering
story; budget 12–18 passes for convergence. Do not declare convergence at first CLEAN pass.

### L-S027-002: Drop-counter coexistence is a recurring regression surface

Pass-14 found a drop-counter coexistence MAJOR (4da27b3 fix). The drop-counter render path
is shared between the legacy status line and the new render_status_bar. Any future change to
the status bar must verify coexistence with the existing drop-counter increment path.

**Recommendation:** add a coexistence smoke test to the permanent test suite (not just
adversarial cycle tests) before wave-7-gate.

### L-S027-003: [t]-stub compile-gate finding pattern

Pass-9 found a [t]-stub compile-gate MAJOR (28e3ad5). This pattern — where a test helper stub
diverges from the production interface during implementation — should be caught earlier. The
`[t]` prefix convention flags test-only stubs; a pre-commit lint check on `[t]-` stubs vs
their production counterparts would catch this before adversarial review.

### L-S027-004: [process-gap] PROCESS-GAP-REGISTRY-ATOMICITY

**ID:** PROCESS-GAP-REGISTRY-ATOMICITY
**Severity:** process gap (CI breakage)
**Description:** BC version bumps during S-027 adversarial convergence (BC-2.06.015 v1.0.7 at
commit 2e544fd; BC-2.06.016/019/020 v1.1.0 at commit f8cf600) did not update
`.factory/specs/version-pin-registry.yaml` atomically in the same factory-artifacts commit.
The registry still listed the pre-S-027 versions, so POL-11 (check_version_pins.py) failed
in CI on PR #32 when story frontmatter and test prose cited the new versions (ADR-0007
§"Registry Update Obligation").

pr-manager authored the fix commit (ffaf406) locally on factory-artifacts but did not push;
state-manager pushed it as part of the S-027 bookkeeping repair on 2026-06-01.

**Fix:** the PO or state-manager MUST update version-pin-registry.yaml in the SAME burst as
any BC version bump. The registry update and the BC file bump are a single atomic unit.
Committing the BC file without updating the registry is a partial commit that breaks CI.

**Target:** codify this as a mandatory checklist item in the convergence-fix burst checklist:
"[ ] version-pin-registry.yaml updated for every BC whose version changed in this burst."
