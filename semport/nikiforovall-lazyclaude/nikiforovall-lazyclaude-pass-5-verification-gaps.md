# Pass 5: Verification Gaps — nikiforovall/lazyclaude

This pass identifies behavioral claims in code/docs that are NOT covered by tests, and tests that exist but check trivial properties. Citations are file:line.

## Tests inventory (verified)

By `find /Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/tests -name '*.py' | wc -l` → **28 files**, 5,275 LOC.

### Integration tests (discovery)

| File | What it likely covers (inferred from name) |
|---|---|
| `tests/integration/discovery/test_skills.py` | SkillParser end-to-end via fake FS |
| `tests/integration/discovery/test_slash_commands.py` | SlashCommandParser end-to-end |
| `tests/integration/discovery/test_subagents.py` | SubagentParser end-to-end |
| `tests/integration/discovery/test_memory_files.py` | MemoryFileParser end-to-end |
| `tests/integration/discovery/test_auto_memory.py` | `_discover_auto_memory` slug + synth refs |
| `tests/integration/discovery/test_mcps.py` | MCPParser user + project + project-local |
| `tests/integration/discovery/test_hooks.py` | HookParser end-to-end |
| `tests/integration/discovery/test_plugins.py` | Plugin three-phase loader + plugin types |
| `tests/integration/discovery/test_gitignore.py` | GitignoreFilter pruning during discovery |
| `tests/integration/discovery/test_behavior.py` | Cross-feature integration behavior |

### Integration tests (writer)

| File | What it likely covers |
|---|---|
| `tests/integration/writer/test_mcp_writer.py` | `write_mcp_customization` / `delete_mcp_customization` |
| `tests/integration/writer/test_delete_writer.py` | Generic `delete_customization` + skill rmtree |

### Unit tests

| File | What it likely covers |
|---|---|
| `tests/unit/test_app_customization_actions.py` | Mixin behaviors at the App level |
| `tests/unit/test_combined_panel.py` | CombinedPanel tab/list behavior |
| `tests/unit/test_config_path_resolver.py` | ConfigPathResolver mappings |
| `tests/unit/test_customization_writer.py` | Writer copy paths |
| `tests/unit/test_filesystem_scanner.py` | GlobStrategy dispatch |
| `tests/unit/test_gitignore_filter.py` | Filter unit semantics |
| `tests/unit/test_level_selector.py` | LevelSelector widget |
| `tests/unit/test_memory_file_ref.py` | Memory ref resolution edge cases |
| `tests/unit/test_plugin_source_path.py` | PluginLoader source path resolution |
| `tests/unit/test_rules_discovery.py` | Rules discovery |
| `tests/unit/test_settings_service.py` | SettingsService load/save + migration |

### Tests likely MISSING (no corresponding file found by name)

| Missing area | Risk | Citation in source |
|---|---|---|
| LSPServerParser | LSP servers handled only since recent commits; no `test_lsp*.py` exists | `parsers/lsp_server.py:1-140`, `discovery.py:701-722` |
| MarketplaceLoader (joins + scope filtering) | Complex multi-scope state; no test file | `services/marketplace_loader.py:113-247` |
| MarketplaceModal (UX flow) | Most complex widget; 788 LOC; no `test_marketplace_modal.py` | `widgets/marketplace_modal.py:1-789` |
| Background plugin command failure paths | `_run_plugin_command` swallow; no test for non-zero exit | `mixins/marketplace.py:248-280` |
| Filter combinations (level × plugin_enabled × query) | Likely partial via behavior test, but not exhaustive | `services/filter.py:60-118` |
| Path-key fuzzing for Windows in MCP local discovery | Slash/backslash key lookup | `discovery.py:600-607` |
| `ConfigPathResolver` for non-directory plugin source | Unit test exists; cross-check whether github-source path still resolves | `services/config_path_resolver.py:30-71` |
| Concurrent writer + refresh races | None — single-threaded assumption | n/a |
| Theme switch persistence | Tested? not obvious from filename list | `app.py:210-214`, `services/settings.py:55-69` |
| Pyperclip failure paths | Will crash UI if pyperclip can't find a system clipboard. Not handled. | `app.py:622` |

## Behavioral claims (in code or docs) WITHOUT explicit test coverage

### G1: `description` heuristic in `SlashCommandParser`

The "first non-`#` body line truncated to 100" fallback (`slash_command.py:53-57`) is a behavior that depends on subtle whitespace handling. **No test name suggests body-fallback coverage** — likely tested in `test_slash_commands.py` for the happy path but not for edge cases (empty body, body starting with multiple blank lines, body of only `#` lines).

### G2: Memory file cycle detection at depth-5

`memory_file.py:99-117` claims cycle-safe up to depth 5. `tests/unit/test_memory_file_ref.py` exists but coverage of pathological cases (A→B→A, deep mutual recursion, broken `~` expansion) is uncertain without reading the test.

### G3: `_discover_local_mcps` Windows-path fuzzing

`discovery.py:600-607` tries both `/` and `\\` normalizations of the project path against `~/.claude.json` `projects` keys. **No `test_mcps.py` content was read** to confirm cross-platform path testing — risk for Monocle is that this is an under-tested Windows-only behavior.

### G4: `enabledPlugins` defaults to `True` if missing

`plugin_loader.py:301-306`:
```python
is_enabled = self._registry.user_enabled.get(plugin_id, True)
```
Implicit "missing key = enabled". This is a contract that affects every plugin scanned but not explicit in docs and tested only implicitly.

### G5: SUBDIR scan strategy's gitignore-dir-filter

`filesystem_scanner.py:96-115`: when iterating subdirs (for skills), the gitignore filter is applied to the **subdir name** AND **is_dir_ignored** check. Then a second filter pass on the matched `SKILL.md` file paths via `is_ignored`. This double-filter is subtle; one of the filter paths may be redundant. No clear coverage.

### G6: `_find_latest_version_dir` semver-tuple-or-string comparison

`plugin_loader.py:329-353` & duplicate at `marketplace_loader.py:267-283`. Behavior: if all subdir names parse as integers, compares as `tuple[int,...]`; otherwise as `tuple[str]`. **Mixing semver and non-semver subdirs produces a TypeError that's caught silently in `_find_latest_version_dir` but propagates if compared raw** — covered by `test_plugin_source_path.py` only for the happy semver path? Untested.

### G7: Auto-memory synth-refs ordering

`discovery.py:500-522`: sibling `*.md` files in `~/.claude/projects/<slug>/memory/` are sorted alphabetically and appended. There is a test_auto_memory.py — confirm it pins the ordering.

### G8: Plugin preview rolls back state correctly on Esc

`mixins/marketplace.py:134-153`: `_exit_plugin_preview` clears `_plugin_customizations`, restores subtitle, returns to marketplace modal. Critical UX flow. Test coverage unclear.

### G9: Marketplace install-with-scope state machine

`marketplace_modal.py:549-565, 711-723`: the scope-selection mode is a sub-state of the modal (no separate widget). If the user presses non-1/2/3 keys, behavior is governed by `check_action`-like gating. **No `test_marketplace_modal.py` present**, so the state machine is unverified.

### G10: `claude plugin <verb>` failure mode

`mixins/marketplace.py:263-267`: on `subprocess.CalledProcessError`, error message is captured. On `FileNotFoundError` (no `claude` CLI), shows `"Claude CLI not found"`. **Tested via mock subprocess?** No filename suggests so.

### G11: Empty `marketplace.json` plugins array

`marketplace_loader.py:104-111`: `data.get("plugins", [])` — empty list → no plugins → marketplace with `plugins=[]`. Renders as `"[0/0]"`. No bug, but never explicitly tested.

### G12: `enabledPlugins` aggregation across scopes

`marketplace_loader.py:167-238` builds `_enabled_plugin_ids` as union of user/project/local enabled minus union of explicitly-disabled. **The exact set algebra is complex** (`marketplace_loader.py:181-197`); regression risk if scopes change semantics.

### G13: Symlinks

Nothing in the codebase explicitly handles symlinks. `Path.resolve()` is used heavily; chained symlinks resolve correctly but the **walker may traverse into linked directories outside the intended scope** if `os.walk` follows them (default = no symlink follow for dirs — saving us).

### G14: Permission errors during discovery

A `~/.claude/plugins/` directory the user can't read produces a silent empty list (every `is_file()` / `is_dir()` returns False on permission error in Python? — actually they return False on permission error). Discovery proceeds without warning. UI shows nothing.

## Documentation drift

### D1: `CLAUDE.md` keybinding table vs `bindings.py`

- `CLAUDE.md:54-77` lists `0`-`6` for panels, but `bindings.py:25-32` registers `0`-`7` (panel 7 = LSP, added with LSP support).
- `CLAUDE.md:67` claims `D` Toggles disabled plugins — confirmed.
- `CLAUDE.md` does not mention `t` (toggle plugin enabled) but `bindings.py:11` has it.

### D2: Combined panel description in `CLAUDE.md`

`CLAUDE.md:121` says "[4]Mem [5]MCP [6]H" — three tabs — but `combined_panel.py:29-41` shows four (`MEMORY_FILE`, `MCP`, `HOOK`, `LSP_SERVER`). LSP_SERVER was added later and the README is stale.

### D3: `models/customization.py:14` comment claims `PROJECT_LOCAL = ~/.claude.json (for MCPs only)`

This is **wrong** — PROJECT_LOCAL is also used for `./.claude/settings.local.json` hooks (`writer.py:327-331`) and `./CLAUDE.local.md` memory files (`discovery.py:464-474`).

### D4: `services/parsers/slash_command.py:23`

Docstring says `File pattern: commands/**/*.md` — correct. But the comment doesn't note that command names use **colon-separated path components**, which is a non-obvious behavior worth documenting in a Rust port.

## Tests that exist but might be shallow

Without reading the actual test bodies (only filenames inspected), the following names suggest happy-path coverage only — verify against contract edge cases:

- `test_combined_panel.py` — likely tests tab switching + selection. Whether it tests **per-tab restored selection** (which is the key feature, `combined_panel.py:141-144`) is unclear.
- `test_level_selector.py` — likely show/hide; whether op="move" vs "copy" prompt difference is tested is unclear.
- `test_filesystem_scanner.py` — likely tests all three GlobStrategy values; the **`TypeError` fallback at `filesystem_scanner.py:66-68` (gitignore_filter kwarg duck-typing) is brittle** and worth a regression test.

## NFR gaps

The code does not explicitly handle (relevant when porting):

- **Memory cost of skill discovery**: every file in every skill dir is eagerly read into `SkillFile.content`. For a skill with 50 large reference files, memory usage spikes. Rust port should lazy-read.
- **Discovery latency**: full scan is synchronous, blocks UI thread on Mount. For users with 100+ plugins this is multi-second. **No spinner, no progressive load.** P1 UX for Rust port.
- **No file watcher**: refresh is manual (`r`). Changes to settings/customization files don't auto-reflect.
- **No locking**: `~/.claude.json` is shared with the `claude` CLI. Read-modify-write of this file races against concurrent CLI activity. Real-world bug surface. P0.

## State Checkpoint

```yaml
pass: 5
status: complete
timestamp: 2026-05-11T17:20:00Z
next_pass: 6
gaps_identified: 14_behavioral + 4_doc_drift + 4_nfr = 22
```
