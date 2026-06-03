---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: nikiforovall-lazyclaude
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: NikiforovAll/lazyclaude (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — Interactive Tune is now v1 (Static plane
becomes interactive: edit/apply bindings, profiles, model routing). The Static plane observer
(which was already in scope) must become interactive. This is the primary relevance lens for
this repo.

## Original Disposition Summary

The original v1.1.2 vision adopted NikiforovAll/lazyclaude as the primary source for:
- 7-customization-type schema (slash commands, subagents, skills, memory files, MCP servers,
  hooks, LSP servers) — ADOPT (for monocle's Static plane)
- AppMode state machine (compile-time mutual exclusion) — ADOPT (built in original Phase 1)
- FocusSnapshot enum (explicit focus restoration) — ADOPT (built)
- 5-layer `check_action` gate precedence — ADOPT (partially built)
- Discovery walker / multi-scope resolver — ADOPT (for monocle-static, Phase 2 scope)
- Atomic file writes (`tempfile::persist`) — ADOPT (already in CLAUDE.md conventions)
- Shell=True P0 bug — ADOPT fix (use `Command::new(binary).args(...)` in Rust)

What was NOT in scope: the Static plane was observe-only for Phase 2. Interactive editing
(edit/apply bindings, enable/disable plugins) was Phase 3+.

The pivot makes the Tune plane interactive in v1. This re-scopes the writer/CRUD genes
from "later phase" to "v1 required".

## Disposition by Capability Area

### 1. AppMode State Machine + FocusSnapshot (originally ADOPT, built)

**New verdict: ADOPT (already built, no change).** The compile-time AppMode state machine
(Dashboard / Filtering / Overlay / Fullscreen) is implemented. The pivot adds a new AppMode
variant: `EmbeddedTerminal { session_id, prior: FocusSnapshot }` for the PTY pane.

New AppMode variant needed:
```rust
pub enum AppMode {
    Dashboard { focused: FocusSnapshot },
    Filtering { panel: PanelId, query: String, prior: FocusSnapshot },
    Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot },
    Fullscreen { panel: PanelId, prior: FocusSnapshot },
    // NEW: embedded terminal occupies full PTY pane area; keys forwarded to PTY master
    EmbeddedTerminal { session_id: SessionId, prior: FocusSnapshot },
    // NEW: session creation wizard (profile picker → worktree creation → launch)
    SessionCreation { step: CreationStep, prior: FocusSnapshot },
}
```

The `EmbeddedTerminal` mode captures all keystrokes and forwards them to the PTY master
(except the designated "detach from terminal" key, e.g., `Ctrl-\` or a configurable escape).
This is the "pass-through mode" that makes the embedded terminal interactive.

The `SessionCreation` mode drives the new session launch wizard:
1. Profile picker (already built in S-031 as Ctrl-P)
2. Project picker (new: list of known projects / recent dirs)
3. Worktree option (new or existing cwd)
4. Launch confirmation

### 2. 5-Layer `check_action` Gate (originally ADOPT, partially built)

**New verdict: ADOPT + EXTEND.** New actions require gate rules:

New actions in control-center:
- `Action::SessionLaunch` — allowed in Dashboard mode; blocked in EmbeddedTerminal.
- `Action::SessionKill` — allowed when a session is selected; gated by confirmation overlay.
- `Action::EmbedTerminal` / `Action::DetachTerminal` — mode transitions.
- `Action::ForwardKey(KeyEvent)` — only allowed in EmbeddedTerminal mode; all other modes block.
- `Action::TuneApply(BindingChange)` — allowed in Tune panel; blocked in read-only view mode.

The existing 5-layer precedence (SearchPrompt > UserCustomCommand > PerContext > Global >
Builtin) handles all new actions within the existing framework. No new precedence levels.

### 3. Discovery Walker / Multi-Scope Schema (for monocle-static) (originally ADOPT, Phase 2)

**New verdict: ADOPT (confirm), scope note: monocle-static is NOT v1 launch scope.**

The D-236 pivot's "Interactive Tune" requirement means monocle-static (the customizations
explorer) must be interactive — edit/apply rather than browse-only. However, the pivot's v1
scope is primarily LAUNCH + MANAGE + EMBEDDED PTY. The Static plane interaction
(edit customizations, apply bindings) is important but secondary to the PTY launch capability.

Decision:
- Phase v1A (re-baselined launch capability): deliver LAUNCH + MANAGE + EMBEDDED PTY.
  The Static plane remains the observer it is today (Phase 1 built it as read-only).
- Phase v1B (Tune plane activation): activate the writer/CRUD genes from this repo to make
  the Static plane interactive.

This is a feature-ordering decision per CLAUDE.md Rule 2, NOT an MVP shortcut. The Tune
plane is v1 scope; it ships in the second wave of the re-baselined v1 delivery.

The specific genes needed for Tune plane activation:
- **Writer / CRUD pipeline** (BC-11, `services/writer.py` → Rust `CustomizationWriter`) —
  atomic writes, type-dispatched copy/move/delete.
- **Atomic-write-three-sites fix** — all three sites use naked `write_text` in the reference;
  monocle's Rust port uses `tempfile::persist()` (already in CLAUDE.md conventions).
- **TOCTOU on shared `~/.claude.json`** — advisory file lock via `fs2` or atomic-merge-with-
  retry for concurrent access.
- **Modal-Confirm-Callback 3-phase pattern** (from mixins) — every destructive action follows
  phase A (initiate), phase B (modal), phase C (resolve). Already modeled in the Overlay
  AppMode.

These are deferred to the Tune plane wave, not the launch capability wave.

### 4. Interactive Keybinding Editor (new capability from this repo)

**New verdict: MODEL (Tune plane).** The reference's `configuration` plugin (3,801 LOC) in
zellij edits running config via `Reconfigure` or `RebindKeys`. NikiforovAll's customization
writer provides the file-level mutation primitives.

monocle's Tune plane interactive keybinding editor (v1B wave) should:
- Display current bindings grouped by precedence level (5 levels).
- Allow editing a binding in-line (key combo input + action selection).
- Write changes to `~/.monocle/config.json` via atomic tempfile+rename.
- Hot-reload the binding dispatcher (already planned via config hot-reload, zellij gene).
- Show which level each binding resolves from (the "trace to source" capability already in
  the Static plane for permission prompts — same pattern for keybindings).

This is a new capability the original disposition did not include. It is v1B scope.

### 5. Customization Type Schema (7 types) (originally ADOPT, Phase 2)

**New verdict: ADOPT (confirmed, Phase 2/v1B).** The 7-type schema
(SlashCommand/Subagent/Skill/MemoryFile/Mcp/Hook/LspServer) and the discovery walker are
the foundation of the monocle-static crate. Disposition unchanged; scope is v1B (Tune wave).

The declaration-order sort key, the PluginScope serde literals, the Tagged Metadata enum,
and the 12-case filter truth table from the v2 synthesis remain the definitive specification
for the Rust port.

### 6. Textual-Specific Patterns (originally Leave-behind)

**Original verdict: Leave-behind** — Textual is Python; ratatui is Rust.

**CONFIRMED Leave-behind.** The Textual→ratatui translation matrix (akb-r1 §12) in the v2
synthesis documents the mapping but the framework-specific code is not imported.

### 7. `shell=True` P0 Bug (originally ADOPT fix)

**Original verdict: ADOPT fix** — use `Command::new(binary).args(...)` in Rust, never shell=True.

**CONFIRMED ADOPT fix.** This is already part of CLAUDE.md conventions (no shell injection).
Directly relevant to the spawn path: `portable-pty CommandBuilder` with explicit args, never
shell-interpolated command strings.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| AppMode state machine (built) | ADOPT | ADOPT + EXTEND (EmbeddedTerminal + SessionCreation variants) | Extended |
| FocusSnapshot (built) | ADOPT | ADOPT (confirmed) | Confirmed |
| 5-layer check_action gate | ADOPT | ADOPT + EXTEND (new actions gated) | Extended |
| Discovery walker / 7-type schema | ADOPT (Phase 2) | ADOPT (confirmed, v1B Tune wave) | Scoped to v1B |
| Writer / CRUD pipeline | ADOPT (Phase 3+) | ADOPT (v1B Tune wave; REQUIRED for interactive tune) | Pulled into v1B |
| Modal-Confirm-Callback 3-phase | ADOPT | ADOPT (confirmed) | Confirmed |
| Interactive keybinding editor | N/A (not in original) | MODEL (v1B Tune wave) | NEW |
| Textual-specific patterns | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| shell=True bug fix | ADOPT | ADOPT (already in conventions; applies to spawn path) | Confirmed |
| Atomic write 3-site fix | ADOPT | ADOPT (tempfile::persist already in conventions) | Confirmed |

## Net Assessment

NikiforovAll/lazyclaude remains the definitive source for the Static plane (monocle-static)
and the interactive Tune capability. The pivot makes the CRUD/writer genes a v1B requirement
rather than a later-phase concern.

The primary addition this disposition introduces:
- `EmbeddedTerminal` and `SessionCreation` AppMode variants (new, driven by the pivot).
- Interactive keybinding editor as a v1B Tune plane capability (new scope from the
  "Interactive Tune" requirement in NEXT-SESSION-PIVOT.md §4.5).

The writer/CRUD genes move from "Phase 3+" to "v1B (Tune wave)" — this is not a
production-grade shortcut, it is correct feature ordering per CLAUDE.md Rule 2.
