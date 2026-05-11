# Pass 7: Holdout Seeds — nikiforovall/lazyclaude

This pass identifies code areas that look "obviously simple" but harbor non-trivial behavior — areas worth dedicating extra deepening rounds to. Each seed is a hypothesis: **"this looks like a one-liner but isn't."**

## Seed 1: `_get_project_slug` — the slug must match Claude Code's exact convention

`discovery.py:478-484`:
```python
def _get_project_slug(self) -> str:
    return re.sub(r"[^a-zA-Z0-9\-]", "-", str(self.project_root))
```

Looks like a 1-line sanitization. But:
- The slug is used to find `~/.claude/projects/<slug>/memory/` (`discovery.py:490`).
- It **MUST** match the slug that Claude Code uses internally when it writes there.
- Behavior on Windows (`C:\Users\joe\dev\foo` → `C--Users-joe-dev-foo`) — explicitly noted in docstring; reproducing this in Rust requires the same regex.
- Edge: hyphens at start/end remain; consecutive non-alphanumerics produce consecutive `-` (the regex doesn't collapse runs). Match this exactly or memory-discovery silently breaks.

**Deepening priority:** P0 for Monocle's Rust port. The Rust impl must produce **byte-identical** output to this Python.

## Seed 2: `parse_frontmatter` regex with `re.DOTALL`

`parsers/__init__.py:55`:
```python
pattern = r"^---\s*\n(.*?)\n---\s*\n(.*)$"
```

Looks trivial. But:
- `\s*` after `---` allows trailing whitespace before newline. Important.
- `\n---\s*\n` requires the closing `---` to be on its own line with optional trailing whitespace before newline.
- A file ending with `---` (no trailing newline) **does not match** — frontmatter not recognized.
- A file with `\r\n` line endings: `\n` won't match `\r\n` line endings on Windows-authored files without normalization. The `read_text` call uses `encoding="utf-8"` (no newline translation), so **CRLF files silently appear to have no frontmatter**.

**Deepening priority:** P1 — Rust port must handle CRLF.

## Seed 3: `parse_tools_list` polymorphic input

`parsers/__init__.py:67-73`:
```python
def parse_tools_list(tools_value: str | list | None) -> list[str]:
    if tools_value is None:
        return []
    if isinstance(tools_value, list):
        return [str(t).strip() for t in tools_value]
    return [t.strip() for t in str(tools_value).split(",") if t.strip()]
```

Looks like a switch. But:
- `[str(t).strip() for t in list]` — preserves empty strings if `t = ""` (`str("").strip() == ""`). The CSV path filters empties; the list path doesn't. **Asymmetric.**
- A `dict` would pass `isinstance(list)` check as False, then `str(dict).split(",")` produces garbage. YAML might produce a dict if frontmatter writes `tools:\n  Read: yes`.
- `subagent.py:56-62` **duplicates** this logic for `skills` field with subtly different semantics (the CSV path uses `.strip()` differently — actually identical, but the duplication is the gene).

**Deepening priority:** P2 — minor.

## Seed 4: `GitignoreFilter.walk_filtered` depth tracking via `os.sep`

`gitignore_filter.py:131-149`:
```python
root_depth = str(root).count(os.sep)
for dirpath, dirnames, filenames in os.walk(root):
    current_depth = str(dirpath).count(os.sep) - root_depth
    if max_depth is not None and current_depth >= max_depth:
        dirnames.clear()
        continue
```

Looks like a depth-limit. But:
- `os.sep` is `\\` on Windows. `str(root).count("\\")` counts backslashes. **If `root` is normalized to forward slashes** (e.g., from `Path.as_posix()`), the count is wrong.
- Depth comparison `>=` is at the root — `max_depth=5` means **5 levels deep including root**. Off-by-one trap.
- `dirnames.clear()` then `continue` — this **doesn't yield the current directory's matching files at the depth-limit**. Files exactly AT max_depth are skipped. May or may not be intended.

**Deepening priority:** P1 for Rust port (Rust's `WalkDir` doesn't use `os.sep` of course, but the depth semantics must match).

## Seed 5: `_find_latest_version_dir` — three copies, semver-vs-string

`plugin_loader.py:329-353`, `marketplace_loader.py:267-283`, `marketplace_modal.py:425-437`.

Each is "find max by version". But:
- `_parse_version` returns `tuple[int,...]` on success or `tuple[str]` on `ValueError`.
- Mixing semver and non-semver dirs in the same `max()` call **compares `tuple[int]` against `tuple[str]`** which raises `TypeError` in Python 3.
- All three call sites wrap in `try/except OSError` — but **not `TypeError`**. The semver/string mix-up would crash discovery.

**Deepening priority:** P0 — latent crash. Verify with a test case of `plugins/foo/latest` and `plugins/foo/1.2.3` coexisting.

## Seed 6: Marketplace `_load_installed_plugins` set algebra

`marketplace_loader.py:181-197`:
```python
self._enabled_plugin_ids = (
    (
        self._installed_plugin_ids
        - {pid for pid, enabled in {
            **registry.user_enabled,
            **registry.project_enabled,
            **registry.local_enabled,
        }.items() if not enabled}
    )
    | enabled_in_user
    | enabled_in_project
    | enabled_in_local
)
```

Looks like a one-expression union-difference. But:
- Dict merge `{**a, **b, **c}` — keys in later dicts shadow earlier. If `user_enabled["foo"] = False` but `project_enabled["foo"] = True`, only `True` wins in the merged dict — but the `not enabled` filter then excludes nothing for "foo", so foo stays in the disabled-removal set. **Subtle.**
- Set difference is applied BEFORE the unions add explicit-enabled-anywhere. Net behavior: "enabled if explicitly enabled in any scope, OR installed and not explicitly disabled in any scope (with last-dict-wins shadowing)."
- The intended semantics seem to be "enabled in at least one scope OR no scope flags it disabled" — but the implementation conflates these. Worth a unit test that exercises:
  - User-disable + project-enable → enabled (correct: union)
  - User-enable + project-disable → ??? (depends on merge order — `project_enabled` overrides → disabled subtraction... if it's in `user_enabled` the union puts it back)

**Deepening priority:** P0 — verify intent vs implementation with concrete tests.

## Seed 7: Auto-collapse heuristic

`marketplace_modal.py:373-378`:
```python
should_collapse = len(marketplace.plugins) > 20 or installed_count == 0
if self._auto_collapse and should_collapse:
    mp_node.collapse()
else:
    mp_node.expand()
```

Looks trivial. But:
- The magic 20 is undocumented.
- "installed_count == 0" hides marketplaces the user hasn't engaged with — a nudge to keep the view focused.
- `_auto_collapse` itself is settings-driven (`AppSettings.marketplace_auto_collapse`, default `True`).
- For users with many marketplaces, the first thing they see on `M` is a collapsed list. **First impression hinges on this.**

**Deepening priority:** P1 — UX-load-bearing.

## Seed 8: Skill expansion key uses `skill.name`, memory uses `str(memory.path)`

Skills: `expanded_skills: set[str]` keyed by `skill.name` (`type_panel.py:546`).
Memory files: `expanded_memory_files: set[str]` keyed by `str(memory.path)` (`type_panel.py:612`, `combined_panel.py:489`).

Why the asymmetry? Because **two skills with the same name at different levels would collide on `skill.name`** — but skills are usually unique. **Two memory files almost always have the same name** (every project has `CLAUDE.md`), so they need a path-based key. Worth noting in port docs.

**Deepening priority:** P2.

## Seed 9: `_extract_frontmatter_text` re-implements `parse_frontmatter`

`detail_pane.py:142-148` re-extracts frontmatter independently:
```python
def _extract_frontmatter_text(self, content: str) -> tuple[str | None, str]:
    pattern = r"^---\s*\n(.*?)\n---\s*\n(.*)$"
    match = re.match(pattern, content, re.DOTALL)
    if match:
        return match.group(1), match.group(2)
    return None, content
```

Why duplicate `parse_frontmatter`? Because the renderer wants the **raw YAML text** (for syntax-highlighting as YAML), not the parsed dict. The pattern is duplicated, not refactored. **DRY violation, P2.**

## Seed 10: `_emit_selection_message` triggers on focus/blur

`type_panel.py:449-453`:
```python
def on_focus(self) -> None:
    self.is_active = True
    self._refresh_display()
    self._emit_selection_message()
```

Looks like routine focus handling. But:
- Every focus change emits a `SelectionChanged` to the app.
- The app `on_type_panel_selection_changed` (`app.py:414-422`) updates MainPane's customization, which then triggers `watch_customization` (a Textual reactive), which calls `_refresh_display`.
- This is fine for the active panel, but **also fires when you Tab away and Tab back** — and **focuses re-fire the message even if selection didn't change**. Net: MainPane content is re-rendered on every panel focus change. For large content, this may flicker.

**Deepening priority:** P2 perf observation.

## Seed 11: Plugin scope determination is name-substring-based

`mixins/marketplace.py:164-174`:
```python
def _resolve_plugin_scope(self, plugin: MarketplacePlugin) -> str:
    view_scope = (...)
    if view_scope == "project":
        return next(
            (s for s in plugin.installed_scopes if s in ("project", "local")),
            "project",
        )
    return "user" if "user" in plugin.installed_scopes else view_scope
```

The CLI scope argument is **picked from `installed_scopes`** — a list of strings populated by `MarketplaceLoader._load_installed_plugins`. Important because:
- A plugin installed only at `project` scope must NOT be uninstalled with `--scope user` — that would silently fail.
- If a plugin appears in `installed_scopes` with multiple entries (somehow), `next()` returns the first match — order matters.
- Default fallback to `view_scope` for the `else` branch — but `view_scope` is only ever `"user"` or `"project"` (per `display_scope`), so the default is sensible.

**Deepening priority:** P1 — scope mismatch is a real failure mode.

## Seed 12: `watch_active_type` saves old-index and restores new-index

`combined_panel.py:242-260`:
```python
def watch_active_type(self, old_type, new_type) -> None:
    self._selected_indices[old_type] = self.selected_index
    restored_index = self._selected_indices.get(new_type, 0)
    ...
```

Per-tab restored selection. Looks straightforward but:
- Initial state: `_selected_indices = dict.fromkeys(COMBINED_TYPES, 0)`.
- If the user filters and the new tab has fewer items than the saved index, **`restored_index` is clamped to `count - 1`**.
- Memory mode rebuild happens before clamping → flat items count is checked.
- The watch fires **before** the items are actually rebuilt visually (`call_later`).

**Deepening priority:** P1 for any port that mirrors the per-tab state machine.

## Hot-spot deepening recommendation

For Phase B convergence, allocate at least 2 rounds to:

1. **`services/discovery.py`** — every method has subtle ordering rules.
2. **`services/parsers/mcp.py` + `_discover_local_mcps`** — Windows path fuzzing, wrapped/unwrapped formats.
3. **`services/parsers/memory_file.py`** — cycle detection, depth limits, ref resolution rules.
4. **`services/plugin_loader.py`** — three-phase scope enumeration, semver dir resolution.
5. **`services/marketplace_loader.py`** — set algebra of enabled IDs.
6. **`widgets/marketplace_modal.py`** — sub-state (scope selection), wrapping tree, footer-state-driven hints.
7. **`widgets/type_panel.py`** — tree expansion for SKILL vs MEMORY (different keys, different rebuild paths).

Lower priority for deepening (these are simpler than they look but not the gene material):
- `widgets/marketplace_source_input.py` (suggestion sorting heuristic is interesting but UX-only)
- `services/opener.py`
- `themes.py`

## State Checkpoint

```yaml
pass: 7
status: complete
timestamp: 2026-05-11T17:30:00Z
next_pass: B-deepening
seeds_identified: 12
high_priority_seeds: [1, 5, 6, 11]  # crash/correctness risk
```
