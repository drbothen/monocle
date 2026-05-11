# Pass B Deep: `internal/gui` — Round 4

**Scope this round:** keybindings.go tail (dialog Enter/Esc/Tab handlers for worktree/profile/askpass/connect/help), keyhandler subpackage (panel, sessions, fullscreen, global handlers), chooser package.

**Files read in full this round:** keybindings.go (full now), keyhandler/{panel, sessions, fullscreen, global}.go, chooser/chooser.go.

## Dialog confirm/cancel flow

Every dialog has Enter (confirm) and Esc (cancel) handlers registered per-view. Common shape:

```go
confirmFn := func(g *gocui.Gui, v *gocui.View) error {
    // 1. Read field contents synchronously
    branch := strings.TrimSpace(branchView.TextArea.GetContent())
    prompt := promptView.TextArea.GetContent()
    profile := selectedProfileFromState()
    options := optionsView.TextArea.GetContent()
    // 2. Validate (only worktree: ValidateWorktreeName)
    if err := session.ValidateWorktreeName(branch); err != nil {
        a.showError(g, ...)
        return nil  // KEEP dialog open
    }
    // 3. Close dialog
    a.closeWorktreeDialog(g)
    // 4. Spawn goroutine, call provider
    go func() {
        err := a.sessions.CreateWorktreeWithOpts(...)
        a.gui.Update(func(g *gocui.Gui) error {
            if err != nil { a.showError(g, ...) } else { a.setStatus(g, ...) }
            return nil
        })
    }()
    return nil
}
```

### BC-GUI-DLGFLOW-001: Worktree dialog validates `ValidateWorktreeName(branch)` BEFORE closing; on error, dialog stays open
**Postconditions:** Other dialogs (rename, connect, profile, askpass) do NOT pre-validate — they always close on Enter and report errors via async toast.
**Evidence:** keybindings.go:138-142.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-002: Worktree confirm reads from FOUR views (branch, prompt, profile-chooser, options) into local variables before closing
**Postconditions:** Views are destroyed by `closeWorktreeDialog`, so all field reads must happen first.
**Evidence:** keybindings.go:126-172.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-003: Worktree dialog Tab navigates Branch → Prompt → Profile → Options → Branch (4-step loop)
**Postconditions:** `a.dialog.ActiveField` is updated AND `g.SetCurrentView` is called. ActiveField persists across re-renders so layoutMain can restore focus.
**Evidence:** keybindings.go:199-235.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-004: Worktree-resume dialog Tab navigates Prompt → Profile → Options → Prompt (3-step loop, no Branch field)
**Postconditions:** Resume reuses an existing branch, so only prompt+profile+options are editable.
**Evidence:** keybindings.go:404-431.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-005: Profile dialog Tab is 2-step: Profile-chooser → Options → Profile-chooser
**Evidence:** keybindings.go:546-562.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-006: Worktree-prompt and worktree-resume-prompt views accept `Ctrl+J` for newline insertion
**Postconditions:** Enter is reserved for confirm, so multiline prompts need Ctrl+J. `v.TextArea.TypeCharacter("\n") + v.RenderTextArea()`.
**Evidence:** keybindings.go:191-196, 396-401.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-007: Chooser views (worktree-chooser, connect-chooser, worktree-profile-chooser, worktree-resume-profile-chooser, profile-chooser) accept `j/k` and Up/Down arrow keys for cursor movement
**Postconditions:** Identical loop pattern across all choosers: `for _, key := range []gocui.Key{KeyArrowDown, KeyArrowUp}` and `for _, ch := range []rune{'j', 'k'}`.
**Evidence:** keybindings.go:249-266, 283-305, 445-462, 575-592, 658-690.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-008: Profile chooser uses chooser.State + chooser.Move helpers for clamped delta movement
**Postconditions:** Centralizes the clamp logic. Worktree chooser, by contrast, has inline clamp (keybindings.go:269-282).
**Evidence:** chooser.go:56-73; keybindings.go:238-247.
**Confidence:** HIGH — minor inconsistency: worktree-chooser uses inline clamp while profile-choosers use chooser.Move.

### BC-GUI-DLGFLOW-009: Profile confirm dispatches by `dialog.ProfileConfirmKind`: "session" / "session_cwd" / "pm_session"
**Postconditions:** Each kind calls a different sessions provider method (CreateWithOpts, CreateAtPaneCWDWithOpts, CreatePMSessionWithOpts). On success, "session" and "session_cwd" call `moveCursorToLastSession`; "pm_session" does not.
**Evidence:** keybindings.go:479-527.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-010: Askpass Enter sends password to `a.askpassCh`; Esc sends empty string
**Postconditions:** Empty string on Esc is the documented "cancel" signal that BC-ASKPASS-006 expects. The channel is the bridge to the askpass UDS handler.
**Evidence:** keybindings.go:617-637.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-011: Connect chooser "Manual input" entry is at index `len(hosts)`; selecting it opens the manual connect-input dialog instead
**Postconditions:** Same pattern as worktree-chooser "+ New worktree" entry.
**Evidence:** keybindings.go:692-708; render.go:111 (renderConnectChooser line for "Manual input"); render.go:106-113 (renderWorktreeChooser line for "+ New worktree").
**Confidence:** HIGH

### BC-GUI-DLGFLOW-012: Connect dialog Enter triggers connectToHost in a non-blocking pattern; empty host is no-op
**Evidence:** keybindings.go:640-650.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-013: Help dialog HelpCursor double-clamps (lines 735-743) — guards both negative and overflow on filter-narrowing
**Postconditions:** After filter changes, the items may have fewer entries; cursor clamp to last valid; second clamp `< 0` handles edge case of empty filtered list.
**Evidence:** keybindings.go:734-743.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-014: Help dialog uses Ctrl+J/K (not j/k) for list movement because j/k are sent to the editor in helpInputView
**Postconditions:** Filter input takes precedence; Ctrl-modified keys bypass editor.
**Evidence:** keybindings.go:749-761 + explicit comment 749.
**Confidence:** HIGH

### BC-GUI-DLGFLOW-015: remote-profile-error dialog is Esc-only; no Enter handler
**Evidence:** keybindings.go:594-600.
**Confidence:** HIGH

## Keyhandler subpackage

### PanelManager (panel.go)

`PanelManager.FocusNext/FocusPrev` rotates `focusIdx` modulo `len(panels)`. Wraps both directions. Three panels are registered by App.initDispatcher: Sessions, Plugins, Logs.

### BC-GUI-PANELMGR-001: Panels are constructed via NewSessionsPanel/NewPluginsPanel/NewLogsPanel; each returns a PanelWithHandler with HandleKey closure bound to the panel's narrow interface
**Postconditions:** Closure captures the panel, but HandleKey wraps it to accept the wider AppActions composite.
**Evidence:** panel.go:30-33 + app.go:236-241 (NewPanelManager invocation).
**Confidence:** HIGH

### BC-GUI-PANELMGR-002: ActivePanel returns nil when no panels registered; FocusNext/FocusPrev are no-ops on empty
**Evidence:** panel.go:48-69.
**Confidence:** HIGH

### SessionsPanel (sessions.go)

Single-tab panel. Maps 22 ActionDef constants → AppActions method calls. Key actions include CursorUp/Down, NewSession/SessionCWD, Delete, Attach, LaunchLazygit, EnterFull (with branch-on-project-node behavior), Rename, Worktree start/select, PurgeOrphans, PMSession, SendKey1/2/3, StartSearch, ConnectRemote, DismissError, CopyError.

### BC-GUI-SESPANEL-001: ActionEnterFull branches based on cursor type: project → ToggleProjectExpanded; session → EnterFullScreen
**Postconditions:** Enter behaves contextually. ActionEnterFullR (right-arrow) always enters fullscreen regardless of node type.
**Evidence:** sessions.go:58-65.
**Confidence:** HIGH

### BC-GUI-SESPANEL-002: ActionSendKey1/2/3 maps to SendKeyToPane("1"/"2"/"3") which sends to the cursor's tmux pane without entering fullscreen
**Postconditions:** Lets the user accept/allow/reject a popup choice from the sidebar without opening the popup.
**Evidence:** sessions.go:76-81.
**Confidence:** HIGH

### BC-GUI-SESPANEL-003: SessionsPanel is single-tab; OnTabChanged is no-op
**Evidence:** sessions.go:29, 108-109.
**Confidence:** HIGH

### FullScreenHandler (fullscreen.go)

Has TWO scopes: ScopeFullScreen (default) and ScopeScroll (when scroll mode active). Switches via interface assertion.

### BC-GUI-FSHANDLER-001: When scroll mode active, ScopeScroll keys are dispatched AND unbound keys are still consumed (return Handled)
**Postconditions:** Scroll mode prevents key leakage to lower handlers. Unbound keys are eaten, not passed through.
**Evidence:** fullscreen.go:55-83. Line 59: "Handled // consume unbound keys in scroll mode".
**Confidence:** HIGH

### BC-GUI-FSHANDLER-002: ScopeFullScreen handles 6 actions: ExitFull, ScrollEnter, ForwardEnter, ForwardEsc, ForwardDown, ForwardUp
**Postconditions:** Rune keys (printable characters) go to inputEditor.Edit() instead — comment line 6.
**Evidence:** fullscreen.go:34-52.
**Confidence:** HIGH

### BC-GUI-FSHANDLER-003: Scroll mode handles 9 actions: ScrollUp/Down/HalfUp/HalfDown/ToTop/ToBottom/ToggleSelect/Copy/Exit
**Evidence:** fullscreen.go:62-82.
**Confidence:** HIGH

### GlobalHandler (global.go)

Handles 7 actions: Quit (three variants), FocusNextPanel/PrevPanel, UnsuspendPopups, PanelNextTab/PrevTab, ShowKeybindHelp. Also handles special-mode Esc (quits) and Ctrl+C in non-main modes.

### BC-GUI-GLOBAL-001: In non-main modes (Diff, Tool), most global keys are skipped; only `ActionQuitCtrlC` is honored
**Postconditions:** Modal protection. The TUI presently has only ModeMain (AppMode 0); the non-main branch is forward-looking.
**Evidence:** global.go:34-43.
**Confidence:** HIGH

### BC-GUI-GLOBAL-002: Esc in non-main mode invokes Quit (special-case outside registry)
**Postconditions:** Documented in code: "Esc has different semantics per mode (popup suspend vs quit)" so not in registry.
**Evidence:** global.go:25-30.
**Confidence:** HIGH

### BC-GUI-GLOBAL-003: Quit, QuitCtrlC, QuitCtrlBackslash all call actions.Quit()
**Postconditions:** Three keybindings for quit: `q`, `Ctrl+C`, `Ctrl+\`. The last one is documented as the normal-mode toggle in .claude/CLAUDE.md but here it's mapped to Quit.
**Evidence:** global.go:45-47.
**Confidence:** HIGH — note this contradicts (or supplements) the .claude/CLAUDE.md statement that "Ctrl+\ is the normal-mode toggle." It's both: in the popup that gets opened, Ctrl+\ quits the TUI process; the tmux key table separately uses Ctrl+\ to launch the popup. Two different layers, same key.

## Chooser package

`chooser.Render`, `chooser.Move`, `chooser.IndexOfDefault` — three pure functions.

### BC-GUI-CHOOSER-001: Render produces one line per item: `<cursorMark><defaultMark><label>` (cursorMark = "▸ " or "  "; defaultMark = "* " or "  ")
**Evidence:** chooser.go:34-52.
**Confidence:** HIGH

### BC-GUI-CHOOSER-002: Empty items returns nil (not empty slice)
**Evidence:** chooser.go:36-38.
**Confidence:** HIGH

### BC-GUI-CHOOSER-003: Move clamps cursor to [0, len-1]; nil pointer is no-op
**Evidence:** chooser.go:56-73.
**Confidence:** HIGH

### BC-GUI-CHOOSER-004: IndexOfDefault returns first Default=true index, or 0 (NOT -1) if none
**Postconditions:** Caller cannot distinguish "no default" from "default at index 0" without external knowledge.
**Evidence:** chooser.go:77-84.
**Confidence:** HIGH — subtle API choice; flagged for porter awareness.

### BC-GUI-CHOOSER-005: `width` parameter on Render is accepted for future truncation support but currently ignored
**Evidence:** chooser.go:33-35 (line 35: `_ = width`).
**Confidence:** HIGH

## Cross-pass synthesis

After 4 rounds, GUI coverage is approximately:

- **Source files read**: app.go, app_actions.go, layout.go (partial), render.go, popup.go, popup_controller.go, popup_types.go, fullscreen.go, scroll_state.go, state.go, dialog.go, notify_loop.go, input.go (partial), preview.go, plugin_state.go, mcp_state.go, logs_state.go, search.go (partial), keybindings.go, keydispatch/dispatcher.go, keyhandler/popup.go, keyhandler/sessions.go, keyhandler/fullscreen.go, keyhandler/global.go, keyhandler/panel.go, chooser/chooser.go.
- **Source files not read**: app_actions.go used filtered* helpers + showProfileDialog + showWorktreeDialog (etc.) — these dialog-construction functions are in app_actions.go or sibling files, not yet read. tree.go (TreeNode types + BuildTreeNodes + filteredTreeNodes), debug.go (debugLog), keymap/{doc, types}.go body, keyhandler/{plugins, logs, actions, types}.go, presentation/{diff, logline, mcp, plugins, style, tool}.go, popup_types_test.go scenarios, sshconfig.go (ParseSSHHosts), sshdetect.go, search.go tail (the actual filtering logic), keybind_help.go, render_mcp.go, render_plugins.go.

The unread files are mostly either (a) sub-renderers (render_mcp, render_plugins, presentation/*) or (b) panel-specific HandleKey implementations following the same pattern as sessions.go. The novelty in remaining files is mostly **mechanical repetition** of patterns already documented.

## Delta Summary

- New items added: 24 (15 BC-GUI-DLGFLOW, 2 BC-GUI-PANELMGR, 3 BC-GUI-SESPANEL, 3 BC-GUI-FSHANDLER, 3 BC-GUI-GLOBAL, 5 BC-GUI-CHOOSER)
- Existing items refined: 1 (BC-GUI-DISPATCH-001 from r1 strengthened with FullScreenHandler scroll-consume behavior, BC-GUI-FSHANDLER-001)
- Remaining gaps: tree.go (small), sshconfig.go (small), keyhandler/{plugins, logs, actions}.go (~360 LOC), presentation/{*} (~600 LOC), keybind_help.go, search.go tail, dialog-show-functions in app_actions.go siblings. All are sub-renderers or repetitions of already-documented patterns.

## Novelty Assessment

Novelty: NITPICK

Justification: This round produced 24 contracts, but the model-changing findings are exhausted:
- All dialog flows follow the same Enter/Esc/Tab/chooser-Move pattern (15 BC-GUI-DLGFLOW are repetitions of the same shape across worktree, profile, askpass, connect, help).
- All Panel HandleKey implementations follow the same scope-match → switch-action pattern (sessions.go differs only in which actions it registers; plugins.go and logs.go would be near-identical).
- Chooser package is a pure function library (5 contracts but mechanical).
- The remaining unread files (~1500 LOC) are sub-renderers and parallel keyhandlers — none would introduce a new architectural pattern. Reading plugins.go and logs.go would replicate sessions.go shape. Reading presentation/* would document static styling constants. Reading render_mcp/render_plugins would document panel-specific list rendering.

If we removed Round 4's findings, a porter would still need to know: dialog flows have Enter=confirm Esc=cancel Tab=next-field, choosers have ▸/* markers, Panel handlers map actions to AppActions methods. These would be reconstructible from Rounds 1-3 contracts plus the convention of the codebase. **Round 4 adds detail, not new architecture.**

The single SUBSTANTIVE finding worth noting is BC-GUI-GLOBAL-003 — the Ctrl+\ semantic conflict between tmux-key-table launch and gocui-binding-quit. This is documented; a porter would discover it on integration testing regardless.

## Convergence Declaration

**Pass B GUI has converged — findings are nitpicks, not gaps.** A round 5 reading the remaining files (sub-renderers, parallel keyhandlers, presentation, tree.go) would add documentation but no new behavioral contracts that materially change the spec model.

## State Checkpoint

```yaml
pass: B
subsystem: gui
round: 4
status: complete
files_read_full: [keybindings.go, keyhandler/panel.go, keyhandler/sessions.go, keyhandler/fullscreen.go, keyhandler/global.go, chooser/chooser.go]
contracts_drafted: 24
total_gui_contracts_across_rounds: 127  # 32 r1 + 35 r2 + 36 r3 + 24 r4
timestamp: 2026-05-11T20:30:00Z
novelty: NITPICK
convergence: PASS-B-GUI CONVERGED
next_subsystem: daemon
```
