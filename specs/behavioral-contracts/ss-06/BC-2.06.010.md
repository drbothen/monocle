---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.010: Permission Overlay: Diff Preview via `similar 3`

## Description

When the permission overlay renders a `PromptModal` whose `tool_payload` is
`ToolPayload::Edit { old_content, new_content, path }`, the overlay body displays a
unified diff computed via `similar::TextDiff::from_lines`. Lines representing deletions
(`ChangeTag::Delete`) are prefixed with `-` and rendered in red (`Color::Red`). Lines
representing insertions (`ChangeTag::Insert`) are prefixed with `+` and rendered in green
(`Color::Green`). Context lines (`ChangeTag::Equal`) are prefixed with a space and
rendered in the default terminal color. For all other `ToolPayload` variants (`Bash`,
`Read`, `Generic`), no diff is rendered; the overlay shows the raw payload fields only.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack` is non-empty with at least one `PromptModal`.
2. The front `PromptModal` (`App.overlay_stack.front()`) has `tool_payload == ToolPayload::Edit { old_content, new_content, path }` where both `old_content` and `new_content` are non-empty strings.
3. The `similar` crate (version 3.x) is a dependency of `monocle-tui` (NOT of `monocle-core`).
4. The overlay layout has sufficient height to render at least the header (2 rows), hint
   line (1 row), and at least 1 diff line. The diff area height is capped to
   `(overlay_height - 8)` rows.

## Postconditions

1. **Diff computation:** The renderer calls `similar::TextDiff::from_lines(&old_content, &new_content)` to produce the change set. This is a line-level diff; byte-level or word-level diff is not used.
2. **Delete lines rendered red:** For each `ChangeTag::Delete` change, the overlay renders
   a `Line` containing a `Span` styled with `Style::default().fg(Color::Red)`, with text
   formatted as `format!("-{}", change.value())`.
3. **Insert lines rendered green:** For each `ChangeTag::Insert` change, the overlay
   renders a `Line` containing a `Span` styled with `Style::default().fg(Color::Green)`,
   with text formatted as `format!("+{}", change.value())`.
4. **Equal lines rendered default:** For each `ChangeTag::Equal` change, the overlay
   renders a `Line` with default styling, formatted as `format!(" {}", change.value())`.
5. **Height cap enforced:** The diff area is capped to `(overlay_height - 8)` rows to
   preserve the overlay header (prompt metadata) and the action hint line. If the diff
   exceeds the available height, it is truncated. Truncated diffs do not raise an error;
   the user may scroll the diff area with `[↑]`/`[↓]` (if scrolling is implemented) or
   use `Action::OverlayCycleNext` to cycle to other prompts.
6. **Non-Edit payloads: no diff rendered:** When `tool_payload` is `Bash`, `Read`, or
   `Generic`, the diff renderer is NOT called. The overlay body renders the payload fields
   as plain text (e.g., `command: <cmd>` for Bash, `path: <path>` for Read).
7. **Empty `old_content`:** When `old_content` is an empty string (new file creation), the
   diff consists only of `Insert` lines (all of `new_content`). All lines render green.
8. **Empty `new_content`:** When `new_content` is an empty string (file deletion), the
   diff consists only of `Delete` lines (all of `old_content`). All lines render red.
9. **Identical content:** When `old_content == new_content`, the diff consists entirely of
   `Equal` lines. The overlay renders them in default color. No error. The action hint
   line remains active so the user can still accept or reject.
10. **`Wrap { trim: false }` applied:** The diff is rendered in a `Paragraph` widget with
    `Wrap { trim: false }` so long lines wrap without truncating leading spaces.

## Invariants

1. `similar::TextDiff` is computed at render time from `old_content` / `new_content` stored
   in the `PromptModal`. The diff is NOT cached between render frames. Recomputing on each
   frame is acceptable because the overlay is static until a decision is made; the cost is
   bounded by the size of the diff, which is bounded by the 256 KiB body size limit
   (BC-2.01.003) applied at hook ingestion.
2. The `similar` crate is ONLY a dependency of `monocle-tui`, never `monocle-core`. The
   purity boundary (BC-2.06.001 Postcondition 5) is not violated.
3. The diff renderer has no side effects: it does not write to disk, send IPC messages, or
   modify `AppMode`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-069 | `old_content` is empty string (new file creation) | All lines are `Insert`; rendered fully in green; no error |
| EC-070 | `new_content` is empty string (file deletion) | All lines are `Delete`; rendered fully in red; no error |
| EC-071 | `old_content == new_content` (no changes) | All lines are `Equal`; rendered in default color; overlay still accepts a decision |
| EC-072 | `old_content` / `new_content` contains non-UTF-8 bytes | `PromptModal.tool_payload` stores `String` (Rust guarantees UTF-8); non-UTF-8 bytes are rejected at IPC deserialization before reaching the renderer; no panic in renderer |
| EC-073 | Diff produces more lines than `(overlay_height - 8)` rows | Diff area is truncated at the height cap; no scroll indicator required in Phase 1 (P2 enhancement); no panic |
| EC-074 | `tool_payload` is `ToolPayload::Bash { command }` | Diff renderer is not called; overlay body shows `command: <cmd>` in default styling |
| EC-075 | `tool_payload` is `ToolPayload::Generic { tool_name, tool_input }` | Diff renderer is not called; overlay body shows `tool_name: <name>` and a JSON representation of `tool_input` or a truncated excerpt |
| EC-076 | `old_content` contains a trailing newline but `new_content` does not | `similar::TextDiff::from_lines` treats lines as ending at `\n`; the trailing newline difference appears as a one-line change; rendered per normal delete/insert rules |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ToolPayload::Edit { old_content: "fn foo() {}\n", new_content: "fn bar() {}\n", path: "src/lib.rs" }` | Diff shows `-fn foo() {}` in red, `+fn bar() {}` in green | happy-path |
| `ToolPayload::Edit { old_content: "", new_content: "fn new() {}\n", path: "src/new.rs" }` | All lines green (Insert only) | edge-case |
| `ToolPayload::Edit { old_content: "fn keep() {}\n", new_content: "fn keep() {}\n", path: "src/same.rs" }` | One Equal line in default color; no red/green | edge-case |
| `ToolPayload::Bash { command: "rm -rf /tmp/test" }` | No diff rendered; body shows `command: rm -rf /tmp/test` | edge-case |
| `ToolPayload::Edit` with diff exceeding `(overlay_height - 8)` rows | Truncated at height cap; no panic | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Delete lines are styled with `Color::Red` | unit test (render output inspection) |
| VP-TBD | Insert lines are styled with `Color::Green` | unit test |
| VP-TBD | Equal lines are styled with default color | unit test |
| VP-TBD | Non-Edit payloads do not invoke `similar::TextDiff` | unit test (mock renderer or direct test) |
| VP-TBD | Diff area is capped to `(overlay_height - 8)` rows | unit test with small terminal size |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the diff preview feature within the "permission overlay stack" component of CAP-006, which is the product's primary competitive differentiator over lazygit's single-popup and NikiforovAll's Option<Panel> patterns |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: diff rendering is a pure read-only computation producing ratatui `Line` values; no file writes occur) |
| Architecture Module | monocle-tui (overlay.rs render_diff function; `similar 3.x` dependency) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Permission Overlay §Diff Preview; §Dependency Graph (`similar 3.x`); §Purity Boundary (similar::TextDiff in monocle-tui only) |
| Cross-Ref | BC-2.06.008 (overlay push — PromptModal carrying ToolPayload::Edit arrives via this path), BC-2.06.001 (purity boundary — similar is monocle-tui only), BC-2.01.003 (256 KiB body limit bounds diff input size) |
| Test File | `monocle-tui/tests/overlay_diff_preview.rs` |
| Test Name | `test_BC_2_06_010_diff_preview_color_coding` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: purity boundary rule: `similar` is in `monocle-tui` only, not `monocle-core`
- [BC-2.06.008] — depends on: `PromptModal` with `ToolPayload::Edit` arrives via the push path defined in BC-2.06.008
- [BC-2.06.011] — composes with: after reviewing the diff preview, the user decides via Accept-Once; diff preview is informational only, does not alter the decision path

## Architecture Anchors

- `architecture/SS-tui.md#diff-preview` — render_diff() sketch with similar::TextDiff usage
- `architecture/SS-tui.md#permission-overlay` — PromptModal type definition with ToolPayload::Edit variant
- `architecture/SS-tui.md#purity-boundary` — similar::TextDiff in monocle-tui only row

## Story Anchor

S-TBD — Implement diff preview for Edit ToolPayload using similar 3.x in overlay renderer (filled by story-writer)

## VP Anchors

- VP-TBD — Unit tests for diff color coding and height cap enforcement

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.010 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Diff Preview, §Dependency Graph,
  §Purity Boundary; prd-expansion-scope.md §3.3 BC-2.06.010 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: diff rendering is read-only; no file writes.
- Postcondition 10 documents `Wrap { trim: false }` per the SS-tui.md sketch.
- EC-072 documents the UTF-8 safety guarantee from Rust's `String` type.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**IPC sweep — fabricated `ToolPayload::Generic { raw }` replaced with canonical `{ tool_name, tool_input }`** (2026-05-26T14:30:00Z):
- EC-075: `ToolPayload::Generic { raw }` → `ToolPayload::Generic { tool_name, tool_input }`.
  The `Generic` variant has fields `tool_name: String` and `tool_input: serde_json::Value`,
  NOT a single `raw` field. This aligns with SS-tui.md §ToolPayload enum definition and the
  F-P1D4-001 correction already applied to SS-tui.md (CRITICAL finding — corrected variant definition).
  The edge case description is updated to reflect the correct rendering behavior.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.5

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Cosmetic precondition update only: `AppMode` is `Overlay { prior }` (not `Overlay { stack, prior }`); `App.overlay_stack.front()` replaces `stack.front()`.
- No postcondition or test vector changes required: this BC's scope is diff rendering logic, which is independent of where the VecDeque lives.
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-28T00:00:00Z > v1.0.4. PASS.
