---
document_type: test-fixture
purpose: "POL-12 regression fixture — Form (c) multi-line split App claim with canonical type must PASS"
expected_pol12_result: PASS
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Regression Fixture: Multi-Line Split App Claim — Canonical Type (Must PASS)

## Regression context (F-S025-ADV33-MED-001)

Companion to pol12_test_multiline_split_app_claim_stale.md. Verifies that cross-line
Path A detection correctly classifies a split claim where the cited type MATCHES the
canonical type (VecDeque<PromptModal> for App.overlay_stack).

The cross-line form at BC-2.06.008:32-33 and BC-2.06.009:34-35 in the live corpus
uses this exact form — it must remain PASS after the multi-line detection is added.
If POL-12 incorrectly flags a canonical split claim, it would block CI on correct artifacts.

## Description

When the TUI receives a `PermissionPromptQueued` IPC message, it constructs
a `PromptModal` and pushes it to the back of `App.overlay_stack:
VecDeque<PromptModal>` — the single source of truth for the modal stack.

(CANONICAL — matches SS-tui.md §App struct App.overlay_stack declaration exactly.
Cross-line form detected by pending_app_split buffer; type resolves to canonical. PASS.)
