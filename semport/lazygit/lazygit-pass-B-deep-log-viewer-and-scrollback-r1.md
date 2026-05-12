# Pass B (deep) — Log Viewer and Scrollback (round 1)

## The "extras" panel — lazygit's command log
`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/extras_panel.go` (119 LOC)
`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/command_log_panel.go` (189 LOC)
`pkg/gui/context/setup.go:108-116` — `CommandLog` context (`Kind: EXTRAS_CONTEXT`, `Focusable: true`, `WindowName: "extras"`).

## How it appears in the layout
- Sits at the bottom of the main column (`pkg/gui/controllers/helpers/window_arrangement_helper.go:187-193` — appended to `mainPanelChildren` when `args.ShowExtrasWindow`).
- Size logic (`window_arrangement_helper.go:391-404`):
  - If currently focused → `baseSize = 1000` (the lazygit idiom for "fill remaining space").
  - If terminal too short (`height < 40`) → `baseSize = 1`.
  - Otherwise → `userConfig.Gui.CommandLogSize` (default 8).
  - Plus `frameSize = 2` for the border.

So the extras panel is *small by default and grows to full-height when focused*. This is the docked-bottom + expand-on-focus pattern monocle should adopt for its log viewer.

## Autoscroll discipline
Already covered in BC-DRAFT-019. The key insight:
- **Writes** (via `LogAction` / `LogCommand`) set `Autoscroll = true`.
- **User-triggered scrolls** set `Autoscroll = false`.
- **`G` (`goToExtrasPanelBottom`)** re-enables autoscroll (line 89).

This means the *only* way to re-engage autoscroll once paused is to explicitly jump to bottom. Subtle but right: scrolling away should pause autoscroll permanently until the user signals intent to follow again.

## Action vs Command vs CmdWriter
Three write paths into the extras view:
1. `LogAction(action string)` — yellow (`style.FgYellow`), preceded by newline. Used to denote intent groups: "Stage file", "Push", "Rebase".
2. `LogCommand(cmdStr string, isCommandLine bool)` — default text colour if it's a CLI-equivalent command, magenta otherwise. Indented two spaces. Used to log the actual subprocess command.
3. `getCmdWriter() io.Writer` (line 96-118) — returns a `prefixWriter` that injects `"\n\nGit output:\n"` before its first write, then streams stdout/stderr from running subprocess. This is what shows command output below the command itself.

The Action/Command/Output layering is the **structural pattern** monocle should adopt for any TUI logging:
- Action = intent (high-level user verb).
- Command = mechanism (the actual API call or shell command).
- Output = effect (the result).

Each visually distinct, all in the same scrollback.

## The startup banner / random tip
`command_log_panel.go:54-69` — on first render, lazygit prints a coloured intro line referencing the `ExtrasMenu` keybinding, then (if `ShowRandomTip`) a random tip from `getRandomTip()`. The tips are keybinding-aware — they interpolate the user's actual configured keys (`config.Universal.Push` etc.) into the tip strings.

This is a quality-of-life nicety: every startup shows a different keystroke-aware hint. **Monocle should inherit this** with its own tip corpus.

## Vim-style scroll bindings on extras
Wired in `pkg/gui/keybindings.go:288-357`:
- `Universal.PrevItem` / `PrevItemAlt` → scroll up.
- `Universal.NextItem` / `NextItemAlt` → scroll down.
- `Universal.PrevPage` / `NextPage` → page up/down (uses `ViewTrait.PageDelta()` for the delta).
- `Universal.GotoTop` / `GotoTopAlt` → top.
- `Universal.GotoBottom` / `GotoBottomAlt` → bottom (re-enables autoscroll).
- Mouse wheel up/down.
- Left-click → `handleFocusCommandLog` to give focus.

So `gg` / `G` (or whatever keys are mapped) work naturally because the framework treats extras as a list-like view for navigation. `Tag: "navigation"` on these bindings (line 301-320) means they show in the `?` menu's navigation section.

## Focus model
`extras_panel.go:40-46` — `handleFocusCommandLog`:
1. Ensure ShowExtrasWindow is true (auto-show on focus).
2. Set parent context = current side context so Esc returns there.
3. Push the CommandLog context.

So pressing `<c-x>` (or whatever `ExtrasMenu`-then-focus invokes) jumps focus down. Esc returns. The parent-context pattern (line 43) is reusable: monocle's modal subviews that "drill down" should set their parent so escape backs out predictably.

## Visual line selection / clipboard
**Not present** in extras specifically. The visual-line-selection + clipboard-copy pattern (which the lazyclaude README claims is inherited) is implemented in *list contexts* (`pkg/gui/types/context.go:284-303` — `IListCursor` with `ToggleStickyRange`, `ExpandNonStickyRange`, `GetSelectionRange`). The extras view inherits range selection through being a list-like context, but visual-line copy specifically uses:

- `Universal.ToggleRangeSelect` (default `v`) → start range.
- `Universal.RangeSelectDown` / `Up` (default `V`+arrows) → expand range.
- `Universal.CopyToClipboard` (default `<c-o>`) → copy selected lines.

For a generic log viewer in monocle, the **lineage**: lazygit's list-cursor range-select + copy → lazyclaude inherits → monocle reproduces with ratatui list state extended with `range_start: Option<usize>`.

## Translation to monocle
```
gui.Views.Extras                         → LogViewer (ratatui Paragraph or List)
gui.LogAction(s)                         → log_event(LogKind::Action, s)
gui.LogCommand(s, isCli)                 → log_event(LogKind::Command { cli: bool }, s)
prefixWriter                             → impl io::Write for OutputCollector
Autoscroll on/off                        → AutoscrollState { Following | Paused }
goToExtrasPanelBottom resumes autoscroll → action::ScrollLogBottom does the same
handleFocusCommandLog with parent ctx    → push_context(LogContext { parent: current_side })
getRandomTip()                           → const TIPS: &[fn(&KeyConfig) -> String] table
window size logic                        → matching Constraint::Length(8) / Constraint::Min(1) / Constraint::Percentage(80)
```

## Delta summary
- New items: 5 (size formula, autoscroll discipline triggers, 3-layer Action/Command/Output, focus-with-parent-context pattern, lineage of visual-line-select).
- Refined: write paths into the extras view.
- Remaining gaps: how PTY allocation in logWithPty interacts with the writer (out of scope — pty.go is small).

## Round assessment
SUBSTANTIVE. Lane CONVERGED.
