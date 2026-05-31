---
document_type: test-fixture
purpose: "POL-12 regression fixture — Form (c) multi-line split App claim with stale type must FAIL"
expected_pol12_result: FAIL
target_subpath: specs/behavioral-contracts/ss-99/
---

# POL-12 Regression Fixture: Multi-Line Split App Claim — Stale Type (Must FAIL)

## Regression context (F-S025-ADV33-MED-001)

This fixture covers the cross-line Path A blind spot surfaced in Adversarial Pass 33:
when "App.<field>:" ends line N and the type starts line N+1, the pair was invisible
to POL-12 (per-line regex cannot match). POL-12 now implements cross-line detection
(pending_app_split buffer in scan_file()): if "App.<field>:" appears at EOL with no
type following on the same line, and the NEXT line opens with a Rust container type,
the pair is treated as a split App-qualified structural claim and checked against
canonical. Historical-anchor exemption applies if EITHER line carries a marker.

## Description

When the TUI receives a `PermissionPromptQueued` IPC message, it pushes to
`App.overlay_stack:
Vec<SessionState>` — INTENTIONALLY WRONG type (stale).

(The canonical type is VecDeque<PromptModal>. This cross-line form MUST be detected
and flagged by POL-12. Expected result: FAIL.)
