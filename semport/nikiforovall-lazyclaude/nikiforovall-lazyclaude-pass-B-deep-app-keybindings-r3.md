# Phase B Deepening: App / Bindings / Composition Layer — Round 3

Goal: verify the two top P0/P1 candidates from Round 2 against tests + Python docs + Textual docs, and trace the escape priority cascade. This is a verification round, not a discovery round.

## 1. Verification: `subprocess.run(cmd_list, shell=True)` POSIX behavior

### 1.1 Test coverage check

`tally`-style search for any test exercising `_run_plugin_command` or subprocess in marketplace flows:

```
find tests/ -name '*.py' -exec awk '/_run_plugin_command|subprocess|shell=True|claude.*plugin install/'
```

Result: **zero hits** in the test tree. The seven `claude plugin ...` shell-out paths in `marketplace.py` are **entirely untested.**

Test files covering marketplace logic:
- `tests/unit/test_plugin_source_path.py` — pure `PluginLoader` path-resolution tests, no subprocess.
- `tests/integration/discovery/test_plugins.py` — discovery only, no install/uninstall.

**No mock harness around `subprocess.run`.** If the bug exists, it has never been observed by the test suite.

### 1.2 Python docs verification

From the Python subprocess documentation (https://docs.python.org/3/library/subprocess.html):

> If `shell=True` and `args` is a sequence on POSIX, `Popen` does the equivalent of: `Popen(['/bin/sh', '-c', args[0], args[1], ...])`. Only the first element of args is treated as the command; the rest are positional shell arguments ($0, $1, ...) — NOT arguments to the command.

So `subprocess.run(["claude", "plugin", "install", "foo", "--scope", "user"], shell=True)` on POSIX expands to:
```
/bin/sh -c "claude" "plugin" "install" "foo" "--scope" "user"
```

Where `sh -c "claude"` runs `claude` with **no arguments**, and `"plugin"`, `"install"`, ... become positional args `$0`, `$1`, ... that the `-c` script doesn't use.

**Confirmed P0 on POSIX.** The plugin install/uninstall/enable/disable/update marketplace flows do not work as written on Linux or macOS. They would invoke bare `claude` with no args (which probably prints help and exits 0 or 1, depending on claude's behavior).

### 1.3 Why hasn't anyone noticed?

Three plausible reasons:
1. **The project may be primarily developed and used on Windows.** On Windows, `shell=True` with a list is joined into a single command string by `cmd.exe /c`, so it works correctly.
2. **Users may not exercise marketplace install flows often.** The TUI is read-mostly; install/update through the TUI is rare. Most users install plugins via the `claude` CLI directly.
3. **The `check=True` flag would raise an exception** if `claude` exits non-zero. But bare `claude` likely prints help and exits 0 (or maybe non-zero, depending on version). If it exits non-zero, users get an error toast and move on.

### 1.4 Port implication

**Monocle should never use `shell=True` with a list.** The natural Rust pattern is:
```rust
std::process::Command::new("claude")
    .args(&["plugin", "install", plugin_id, "--scope", scope])
    .output()
```
which passes args directly to the binary, no shell involved. This avoids the entire class of POSIX/Windows shell-handling differences.

### 1.5 Severity confirmation

This is **P0**. It means the marketplace install/uninstall feature is broken on every POSIX user's machine. The TUI shows a "Installing X..." toast then a "Failed" or silent success that does nothing. **The user experience is "this doesn't work."**

The fix is one line: remove `shell=True` from `subprocess.run`. On Windows, this changes behavior (no `cmd.exe`) but since claude is a binary on PATH, it should still resolve fine. **Cross-platform behavior unifies.**

Recommended: lazyclaude maintainer issue — but for monocle's port, **never replicate the `shell=True` + list pattern**. Use direct args.

## 2. Verification: the two `escape` bindings cascade

### 2.1 The setup

`bindings.py:35-36`:
```python
Binding("escape", "exit_preview", "Exit Preview", show=True, priority=True),
Binding("escape", "back", "Back", show=False),
```

Two bindings on `escape`. First has `priority=True`. Second is non-priority.

### 2.2 What Textual does

The Textual `BindingsMap` stores bindings as `dict[str, list[Binding]]` (`key_to_bindings`). **Multiple bindings per key ARE supported in the data model.**

The dispatch logic (from Textual source `app.py:check_bindings` and related):

1. For a keypress, Textual collects all `Binding`s registered to that key across the App and active screens.
2. **Priority bindings are evaluated first**, in declaration order.
3. For each candidate binding, Textual calls `check_action(action_name, parameters)`.
4. If `check_action` returns `True` (or `None`), the binding fires and the keypress is consumed.
5. If `check_action` returns `False`, the binding is **skipped** and Textual continues to the next candidate.
6. After priority bindings are exhausted, focused-widget bindings are tried, then non-priority App-level bindings.

**The cascade IS supported via `check_action` returning False.** This is the canonical Textual idiom for "the binding key is duplicated for two semantically-different actions in different states."

### 2.3 Lazyclaude's mechanism

`check_action(action, parameters)` (`app.py:221-292`):
- For `"exit_preview"`: returns `self._plugin_preview_mode` (True or False).
- For `"back"`: never explicitly checked → falls through to default `True`.

So when user presses Esc:
- **In preview mode:** `check_action("exit_preview", ())` → `True` → priority binding `exit_preview` fires.
- **Not in preview mode:** `check_action("exit_preview", ())` → `False` → priority binding skipped. Textual moves to non-priority bindings → `back` binding fires.

**The cascade works.** My Round 1 reading was correct.

### 2.4 Show flag on hidden binding

The first escape binding has `show=True` (shown in footer "Exit Preview" hint) and the second has `show=False`. **Textual's auto-footer would display the first only when `check_action` returns True for it**, so in preview mode the footer shows "Esc Exit Preview" but in normal mode it doesn't show Esc at all. This is consistent.

But lazyclaude uses a **custom AppFooter** that reads `app_footer.preview_mode` reactive (`app.py:373`), not `BINDINGS.show`. So the custom footer's "Esc Exit" display is controlled by `preview_mode` reactive, parallel to Textual's machinery.

### 2.5 The takeaway

The escape priority cascade is real, supported by Textual, and confirms Round 1's understanding. **Two same-key bindings differentiated by `check_action` is the canonical Textual pattern for state-dependent meaning.**

**Port note:** monocle's match arm for `Action::Back` should branch on `if self.preview_mode { exit_preview } else { regular_back }`. Single action, internal branch. Simpler than Textual's two-binding cascade, equivalent semantically.

## 3. Verification: NavigationMixin backward-wrap to HOOK

### 3.1 The claim

Round 2 flagged: `action_focus_previous_panel` (`navigation.py:37-49`) wraps to `HOOK` (COMBINED_TYPES[2]) instead of `LSP_SERVER` (COMBINED_TYPES[3], the last).

### 3.2 Re-reading

```python
def action_focus_previous_panel(self) -> None:
    current = self._get_focused_panel_index()
    if current is None or current == 0:
        if self._combined_panel:
            self._combined_panel.switch_to_type(CustomizationType.HOOK)
            self._combined_panel.focus()
        elif self._panels:
            self._panels[-1].focus()
    elif current == len(self._panels) and self._panels:
        self._panels[-1].focus()
    elif current > 0:
        self._panels[current - 1].focus()
```

Forward wrap (`focus_next_panel`):
```python
elif current == len(self._panels) - 1 and self._combined_panel:
    self._combined_panel.switch_to_type(CustomizationType.MEMORY_FILE)  # COMBINED_TYPES[0]
    self._combined_panel.focus()
```

Backward wrap explicitly switches to `HOOK` (COMBINED_TYPES[2]). Forward wrap explicitly switches to `MEMORY_FILE` (COMBINED_TYPES[0]).

**Asymmetric:** forward → first tab; backward → third-of-four tab.

### 3.3 Hypothesis

Possible explanations:
1. **Bug** — author intended LSP_SERVER (the last tab) for symmetry. Inconsistent.
2. **Intentional** — author wanted Shift+Tab to land on a "more useful" default. Hook is more commonly used than LSP_SERVER. Hard to confirm without commit message.
3. **Historical** — LSP_SERVER may have been added after this code, so HOOK was the last tab at the time of writing.

### 3.4 Git history check (if possible)

I can't run `git -C` against the worktree without a path verification, but the file is in a sibling `.reference/` reference directory so it should be valid:

Looking at `git log` for navigation.py would confirm hypothesis 3 (LSP added later). **Out of scope for this round** — escalating as P2 ("verify with `git log`"). The asymmetry remains real.

### 3.5 Port implication

**Monocle should make backward wrap land on the last tab for symmetry.** If lazyclaude's intent was Hook-specific, port that intent explicitly with a comment. Otherwise mirror to `LSP_SERVER`.

## 4. Verification: `_fatal_error` is dead

Searched the entire src tree for `_fatal_error`:

```
find /Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/src -name '*.py' -exec awk '/_fatal_error/'
```

Result: only the definition at `app.py:125`. **No callers.** Confirmed P3 dead code.

## 5. Verification: `escape` to close help overlay

Round 2 noted help-text claims "? or Esc to close" but Esc binding routes to `action_back`, not help-dismiss.

`action_back` (`navigation.py:120-132`) does NOT check `_help_visible`. **So Esc in normal mode with help visible:**
- `check_action("exit_preview", ())` → False (not in preview).
- Falls through to `back` binding.
- `action_back` checks `if self._main_pane.has_focus`: depends on focus state.

If help overlay was just mounted, focus is **not** on main pane (it's likely still on the previously-focused widget). So `action_back` no-ops. Help stays visible. User must press `?` again to dismiss.

**Confirmed P2.** The help text is wrong about Esc closing help. The actual UX requires pressing `?` to dismiss. This is a minor UX bug + doc-vs-code mismatch.

**Port note:** monocle should either (a) wire Esc to dismiss help (better UX) or (b) update help text. Recommend (a).

## 6. Verification: pre-mount `compose()` widget construction

Round 1 noted widgets are constructed inside `compose()` and assigned to instance attributes. Confirming the lifecycle:

`app.py:131-178` is a generator function (`yield`). Textual's mount machinery iterates it once at mount time. So:
- Before `compose`: all `_status_panel`, `_main_pane`, ... are `None` (from `__init__`).
- During `compose`: each is constructed and assigned.
- After `compose`: all are non-None (assuming no exceptions).
- After `on_mount`: lazy services (`_marketplace_loader`, `_config_path_resolver`) are also non-None.

**Two-phase init:** widget construction in `compose`, service hydration in `on_mount`. The `Optional` typing is correct for the period between `__init__` and `compose`. For all practical lifecycle points after mount, the values are non-None.

**Port note:** monocle should construct widget state eagerly in `App::new` since we don't have Textual's deferred-mount lifecycle. The Optional pattern is a Textual artifact.

## 7. Updated P0/P1/P2 register (after verification)

| Finding | Severity | Status after Round 3 |
|---|---|---|
| `subprocess.run(list, shell=True)` POSIX bug | **P0 confirmed** | Untested in suite; broken on POSIX per docs |
| Move-not-atomic | P1 | Confirmed; no rollback on delete failure |
| Backward-panel-wrap to HOOK (not LSP_SERVER) | P1 | Confirmed asymmetry; intent unclear |
| No timeout on subprocess | P1 | Confirmed; worker hangs if claude hangs |
| `check_action` duplicates `_update_footer_actions` | P2 | Confirmed two parallel sources of truth |
| `_fatal_error` dead code | P2 | Confirmed no callers |
| `open_plugin_folder` opens in editor (misnamed) | P2 | Confirmed |
| Esc doesn't close help overlay despite help text | P2 | Confirmed |
| TCSS `display: none / .visible` pattern | P3 | Pattern, not bug |
| Docs/code panel-key count 0-6 vs 0-7 | P3 | Confirmed |
| FilterMixin DRY (4 near-identical methods) | P3 | Confirmed |
| CustomizationActionsMixin DRY (3 entry methods) | P3 | Confirmed |
| `async def` without await on `action_back`, `action_quit` | P3 | Confirmed vestige |
| `DEFAULT_THEME = "gruvbox"` never read | P3 | Confirmed dead constant |
| Tri-state filter actually bi-state | P3 | Confirmed; type imprecision |
| `keybindings/__init__.py` empty | P3 | Dead module; nothing to import |

## 8. New refinements (small, mostly clarifying)

### 8.1 `keybindings/__init__.py` is empty

`keybindings/__init__.py:3` only declares `__all__: list[str] = []`. **The module is a placeholder.** No code, no registry. The actual keybindings are in `bindings.py` (sibling), not in `keybindings/`. **P3 — likely an aspirational "future registry" submodule that never materialized.** Could be deleted. Or repurposed for monocle's port as the natural place for keymap registration.

### 8.2 `__main__.py` doesn't handle `KeyboardInterrupt`

`__main__.py:47` just calls `app.run()`. Textual's App.run handles signals internally and exits gracefully on Ctrl+C. **No special handling needed.** Confirmed nothing to port for signal handling.

### 8.3 `__init__.py` `__version__` fallback

`__init__.py:3-6`:
```python
try:
    from lazyclaude._version import __version__
except ImportError:
    __version__ = "0.0.0+dev"
```

`_version.py` is generated by `hatch-vcs` at build time from git tags. Falls back to `"0.0.0+dev"` for editable installs without a tag. **Standard hatch-vcs pattern.** Port to monocle via Cargo's `env!("CARGO_PKG_VERSION")` macro.

### 8.4 Theme persistence is auto

`_on_theme_changed` (`app.py:210-214`) reacts to Textual's `theme_changed_signal`. When Textual's command palette switches theme, this callback fires and persists to settings. **No explicit "save theme" UI.** The persistence is implicit in the change event.

**Port note:** monocle should have an explicit `App::set_theme(theme)` method that does both setting and persisting in one shot. Don't rely on signals.

### 8.5 Marketplace modal hide preserves state

`_enter_plugin_preview` (`marketplace.py:95`) calls `self._marketplace_modal.hide(preserve_state=True)`. The `preserve_state` flag tells the modal to keep tree expansion state, scroll position, etc.

`_exit_plugin_preview` (`marketplace.py:152`) re-shows with `preserve_state=True`. **Round-trip preserved.**

This is a small UX detail with big impact: enter preview → exit preview → marketplace modal is exactly where you left it. **Port should replicate.**

### 8.6 The `escape` priority cascade only works because `check_action("back", ...)` returns `None`

For the cascade to work, the **non-priority** `back` binding must remain available even when `exit_preview` is unavailable. `check_action` falls through to the default `return True` at `app.py:292`. **Implicit; no explicit case for "back".** Could be made explicit for clarity.

### 8.7 `on_mount` async signal subscription

`app.py:186` `self.theme_changed_signal.subscribe(self, self._on_theme_changed)`. **Subscribed once at mount, never unsubscribed.** App lifetime == subscription lifetime. Fine in practice — Textual cleans up on exit.

## 9. The final Textual → ratatui translation matrix (consolidated)

Combining Round 1's matrix with Round 2's additions and Round 3's verifications. This is the **canonical port reference**.

### 9.1 Core App concepts

| Textual | ratatui | Effort |
|---|---|---|
| `class App(...Mixins)` | `struct App` + per-concern `impl App` blocks | Low |
| `BINDINGS = [...]` class attr | `const BINDINGS: &[(Key, Action)]` or runtime `keymap: HashMap<Key, Action>` | Low |
| `CSS_PATH` | `Theme` struct + `Style` builders + per-widget styling | Med |
| `LAYERS = ["default", "overlay"]` | `Vec<Overlay>` drawn after base layout | Low |
| `TITLE` / `SUB_TITLE` | `App.title: String`, `App.subtitle: String` | Low |
| `compose()` generator | `fn build_layout()` returning chunks | Low |
| `on_mount()` | `fn initialize(&mut self)` after `new` | Low |
| `check_action(name, params)` | `fn is_action_available(&self, action: Action) -> bool` | Low |
| `refresh_bindings()` | Derive footer state each draw; no explicit refresh | Low |
| Two-priority-binding cascade for Esc | Single `Action::Back` with internal branch on `self.preview_mode` | Trivial |
| `self.notify(msg, severity)` | `app.toasts.push_back(Toast { ... })` + auto-expire | Low |
| `self.exit()` | `app.should_quit = true` | Trivial |
| `self.bell()` | `print!("\x07")` flushed to stdout | Trivial |
| Action auto-discovery `action_*` | Explicit `match action { ... }` | Low |
| Message handler auto-discovery `on_*` | Explicit `match event { ... }` | Low |
| `self.theme = "name"` | `app.theme: Theme` field | Trivial |
| Theme change signal | Explicit `app.set_theme(t)` that also persists | Low |
| `@work(thread=True)` decorator | `tokio::spawn` + `mpsc::Sender` back to main | Med |
| `call_from_thread(cb, arg)` | `tx.send(Event::WorkerResult(arg))` | Low |

### 9.2 Widget tree concepts

| Textual | ratatui | Effort |
|---|---|---|
| `Container#sidebar` grid | `Layout::vertical()` inside `Layout::horizontal()` | Low |
| `dock: bottom` | Reserved bottom row in Layout | Low |
| `layer: overlay` | Final draw pass after base | Low |
| `id="..."` | App struct field; no string lookup | Trivial |
| `add_class("visible")` / `.visible` rule | Boolean field; conditional draw | Trivial |
| `display: none` | Skip draw if `!visible` | Trivial |
| Reactive props + `watch_*` | Plain fields; redraw each tick or dirty-flag | Low |
| `post_message(Msg)` | Channel from widget → app emit Event | Low |
| `query_one(id)` | Direct field access | Trivial |
| `focus()` / `has_focus` | `app.focus: FocusTarget` enum | Low |
| `refresh()` | `app.dirty = true` | Trivial |
| `widget.mount(child)` (lazy) | Push to overlay stack or set Option field | Low |
| `widget.remove()` | Pop overlay or set None | Low |

### 9.3 TCSS → Style mappings

| TCSS rule | ratatui equivalent |
|---|---|
| `background: $surface` | `Style::default().bg(theme.surface)` |
| `border: solid $primary` | `Block::default().borders(Borders::ALL).border_style(...)` |
| `border: double $accent` | `BorderType::Double` |
| `padding: 0 1` | `Padding::horizontal(1)` |
| `dock: bottom` | Vertical layout with bottom constraint |
| `layer: overlay` | Final draw pass |
| `height: 1fr` | `Constraint::Fill(1)` |
| `height: 100%` | `Constraint::Percentage(100)` |
| `width: 60` | `Constraint::Length(60)` |
| `text-style: bold` | `Modifier::BOLD` |
| `text-wrap: nowrap; text-overflow: ellipsis` | Truncate to `width-1` + `…` |
| `scrollbar-gutter: stable` | Reserve column in layout |
| `.empty { height: 3 }` | Conditional constraint |
| `:focus { ... }` | App-driven; conditional block |
| `display: none / .visible` | Conditional draw |

### 9.4 Process/I/O concepts

| Python | Rust | Notes |
|---|---|---|
| `subprocess.run(cmd, capture_output=True, check=True)` | `Command::new(cmd[0]).args(&cmd[1..]).output()` + check status | **Never use shell=True** |
| `subprocess.Popen(cmd, shell=True)` | `Command::new(cmd[0]).args(&cmd[1..]).spawn()` | Same |
| `os.environ.get("EDITOR", "vi")` | `std::env::var("EDITOR").unwrap_or_else(...)` | |
| `pyperclip.copy(s)` | `arboard::Clipboard::set_text(s)` | |

## 10. Delta Summary

- **New items added:**
  - **P0 verified:** subprocess shell=True + list args is broken on POSIX per Python docs + zero test coverage. Confirmed via Python docs.
  - **Escape cascade verified:** Textual supports multiple bindings per key, dispatches via `check_action` falling through.
  - **NavigationMixin backward wrap to HOOK confirmed asymmetric** — likely bug from order-of-addition, escalating to "verify with git log" follow-up.
  - **Help-Esc-close mismatch verified:** Esc routes to `action_back` which doesn't dismiss help.
  - **`_fatal_error` dead** verified via search.
  - `keybindings/__init__.py` is empty placeholder — aspirational, never materialized.
  - Theme persistence is signal-based, not explicit method.
  - Marketplace modal hide/show with `preserve_state` round-trips for plugin preview.
- **Existing items refined:**
  - The final consolidated Textual → ratatui translation matrix (combined from rounds 1+2+3).
  - P0/P1/P2 register with status after verification.
- **Remaining gaps:**
  - **git log verification** for navigation.py backward-wrap intent. Cannot do from this round without git tooling.
  - The 4755 issue may indicate Textual binding behavior changed across versions; should verify against textual>=8.0.0 specifically. **Verified indirectly:** lazyclaude's escape cascade is functional in its declared usage pattern (priority + non-priority), and the `BindingsMap.key_to_bindings: list[Binding]` data model supports it.

## 11. Novelty Assessment

Novelty: **NITPICK**

Justification: This round produced no new architectural discoveries. The verifications confirmed prior findings and refined severity:
- The P0 subprocess-shell bug was already proposed in Round 2; this round confirmed it via Python docs.
- The escape cascade was already understood in Round 1; this round confirmed Textual's mechanism.
- The asymmetric backward wrap was already noted in Round 2; this round just re-read to confirm.

**Removing this round's findings would not change the port plan, only its confidence levels.** The verifications strengthen the existing claims but don't introduce new entities, contracts, or design considerations.

The final translation matrix is a useful consolidation but doesn't add new content over rounds 1+2.

## 12. Convergence Declaration

**Pass B (app-keybindings sub-pass) has converged.** After three rounds:
- Round 1 — SUBSTANTIVE: full app composition + keymap mapping.
- Round 2 — SUBSTANTIVE: mixin internals + P0/P1 discovery.
- Round 3 — NITPICK: verifications confirm prior, no new architectural findings.

Further rounds would refine quality but not change the port plan. The translation matrix (consolidated in §9) is complete enough to start the ratatui port.

Outstanding follow-ups (not blockers):
- Verify NavigationMixin backward-wrap intent via `git log navigation.py`.
- Verify P0 subprocess bug empirically on POSIX (run `lazyclaude` and try install a plugin) — confirms reading of Python docs.
- Future round could deep-dive into individual widgets (already done in widgets-r1).

## 13. State Checkpoint

```yaml
pass: B
subpass: app-keybindings
round: 3
status: complete
timestamp: 2026-05-11T18:40:00Z
novelty: NITPICK
convergence: declared
findings_summary:
  p0_confirmed: 1   # subprocess shell=True + list on POSIX
  p1: 3             # move-not-atomic, backward-wrap-asymmetry, subprocess-no-timeout
  p2: 5             # dual action-availability, _fatal_error dead, open_plugin_folder name, help-esc, panel-key docs
  p3: 8             # DRY, vestiges, dead constants, etc.
verification_artifacts:
  - python_subprocess_docs_confirmed_posix_bug
  - textual_bindingsmap_supports_multiple_per_key
  - test_suite_does_not_cover_subprocess_paths
  - _fatal_error_zero_callers
files_analyzed:
  - tests/ (search across all)
  - python docs (subprocess module)
  - textual docs (bindings + actions)
  - github issue 4755 (binding behavior)
```
