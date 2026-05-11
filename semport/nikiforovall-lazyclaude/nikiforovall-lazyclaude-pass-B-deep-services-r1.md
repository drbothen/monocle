# Phase B Deepening: Non-Parser Services Layer — Round 1

Goal: extract per-file canonical semantics for the 10 supporting services in `src/lazyclaude/services/` that aren't parsers. The parsers layer was deepened in r1/r2; these supporting services are the **discovery walker, path resolver, scanner, filters, opener, settings, writer, marketplace loader, plugin loader** — the "frame" that the parser layer plugs into. The Rust port has to replicate these semantics verbatim because they encode the boundary contract with Claude Code's on-disk shapes.

## Files in scope (re-cited)

| File | Lines | Tests cited |
|---|---|---|
| `services/discovery.py` | 723 | `tests/integration/discovery/test_behavior.py`, `test_plugins.py`, `test_rules_discovery.py`, indirect via `test_*_files.py` |
| `services/config_path_resolver.py` | 72 | `tests/unit/test_config_path_resolver.py:1-213` (8 tests, well-covered) |
| `services/filesystem_scanner.py` | 117 | `tests/unit/test_filesystem_scanner.py:1-221` (7 tests, well-covered) |
| `services/filter.py` | 127 | **NONE** — confirmed via filename search |
| `services/gitignore_filter.py` | 150 | `tests/unit/test_gitignore_filter.py:1-227` (13 tests, well-covered) |
| `services/opener.py` | 42 | **NONE** — confirmed via filename search |
| `services/settings.py` | 111 | `tests/unit/test_settings_service.py:1-119` (covers theme load/save; **missing** marketplace migration, `marketplace_auto_collapse`) |
| `services/writer.py` | 519 | `tests/unit/test_customization_writer.py:1-522`, `tests/integration/writer/test_mcp_writer.py:1-385`, `test_delete_writer.py:1-345` |
| `services/marketplace_loader.py` | 307 | **NONE** — no `test_marketplace_loader*` file |
| `services/plugin_loader.py` | 354 | `tests/unit/test_plugin_source_path.py:1-301` (7 tests, **only `get_plugin_source_path` covered**) |

Test-coverage matrix at a glance: **5/10 files have direct unit tests; 2/10 (`filter`, `opener`) have ZERO tests; 1/10 (`marketplace_loader`) has zero direct tests; 2/10 are sparsely tested** (`settings` misses migration; `plugin_loader` misses registry).

## Canonical semantics — per-file tables for the Rust port

### 1. `discovery.py` — `ConfigDiscoveryService`

The discovery walker — the **single most important service** for monocle's static plane.

#### Entry-point matrix

| Method | Inputs | Returns | Cached? | Source |
|---|---|---|---|---|
| `discover_all()` | none | `list[Customization]`, sorted | YES (`self._cache`) | `discovery.py:158-186` |
| `discover_by_level(level)` | `ConfigLevel` | filtered list | implicit (delegates) | `discovery.py:188-190` |
| `discover_by_type(ctype)` | `CustomizationType` | filtered list | implicit | `discovery.py:192-194` |
| `refresh()` | none | fresh `list[Customization]` | clears self + `_plugin_loader.refresh()` | `discovery.py:196-200` |
| `get_active_config_path()` | none | `Path` (project if exists else user) | no | `discovery.py:202-206`; uses `project_config_path.is_dir()` |
| `discover_from_directory(plugin_dir, plugin_info?, marketplace_plugin?)` | preview entry | `list[Customization]` | NO (preview, fresh each call) | `discovery.py:208-241` |

#### Multi-source orchestration order in `discover_all()`

The order is **non-significant for correctness** (results are sorted at the end), but **is significant for test debugging and for parallel-port verification**:

1. For each of `SCAN_CONFIGS.values()` (slash_commands, subagents, skills), scan USER then PROJECT (`discovery.py:165-175`)
2. `_discover_memory_files()` — `discovery.py:177` → `:415-476`
3. `_discover_auto_memory()` — `discovery.py:178` → `:486-529`
4. `_discover_rules()` — `discovery.py:179` → `:531-569`
5. `_discover_mcps()` — `discovery.py:180` → `:571-586`
6. `_discover_hooks()` — `discovery.py:181` → `:622-641`
7. `_discover_plugins()` — `discovery.py:182` → `:643-665` (iterates `_plugin_loader.get_all_plugins()`; for each plugin, scans SCAN_CONFIGS dirs + mcps + hooks + lsp)
8. `_sort_customizations()` — `discovery.py:184` → `:243-251` with `key=(type_order[c.type], c.name.lower())`

#### Sort algorithm pinned

```python
type_order = {t: i for i, t in enumerate(CustomizationType)}
return sorted(
    customizations,
    key=lambda c: (type_order[c.type], c.name.lower()),
)
```
`discovery.py:243-251`. Variant index order from `models/customization.py:37-46`: `SLASH_COMMAND=1, SUBAGENT=2, SKILL=3, MEMORY_FILE=4, MCP=5, HOOK=6, LSP_SERVER=7`. Note: `auto()` starts at 1 by default in Python. Test confirms ordering by walking `list(CustomizationType)` (`test_behavior.py:29-33`).

**Conflict resolution: NONE.** If a slash command named `foo` exists at both USER and PROJECT, both customizations end up in the returned list — they are NOT deduplicated. The TUI's panel layer is expected to display both with a level indicator. Verified by reading `_discover_memory_files` (which DOES dedup, but only via `seen_paths.add(resolved)` for the same on-disk file, not for same-named files at different levels).

#### Memory-file dedup invariant (`_discover_memory_files`)

The only dedup is for **resolved path equality** (`discovery.py:419, 427, 434, 446, 469`), preventing the same file from being parsed twice if both user-config-path-relative and absolute paths resolve identically. The `seen_paths` set is **NOT** shared across the 5 branches of the function — it's a single set, and applies to (a) user CLAUDE.md/AGENTS.md → (b) user CLAUDE.local.md → (c) project CLAUDE.md/AGENTS.md → (d) nested project CLAUDE.md via `walk_filtered` → (e) project CLAUDE.local.md.

Critical for Rust port: the order matters because earlier branches register paths into `seen_paths`. If user_config_path and project_root happen to overlap (e.g., a symlink), the user path wins.

#### Auto-memory algorithm pinned

`_discover_auto_memory()` at `discovery.py:486-529`:

1. Compute slug: `_get_project_slug()` (`:478-484`) = `re.sub(r"[^a-zA-Z0-9\-]", "-", str(self.project_root))`. **Each match is replaced with `-`** — consecutive non-allowed chars become consecutive `-` (no collapsing). Example: `/home/user/dev/project` → `-home-user-dev-project`. Example: `C:\Users\user` → `C--Users-user`. Tests in `test_auto_memory.py` confirm.
2. Build `memory_dir = user_config_path / "projects" / slug / "memory"`
3. If `memory_dir` is not a dir → return empty
4. Look for entrypoint `MEMORY.md`:
   - If exists: parse it (`MemoryFileParser`), level=`PROJECT_LOCAL`. Then enumerate sibling `.md` files (excluding `MEMORY.md`) sorted by name (`f.name`), and for each that's NOT already in `customization.metadata["imports"]` (matched by basename via `split("/")[-1]`), synthesize a `MemoryFileRef` reading the file content and append to `customization.metadata["refs"]`. **Each topic file's content is eagerly read; OSError sets `content=None`.** (`:498-522`)
   - If no `MEMORY.md`: for each `*.md` file under `memory_dir` (non-recursive, `glob`, sorted by full Path repr), parse as PROJECT_LOCAL with `c.name = md_file.name` (`:524-527`)

The fall-through invariant: **no MEMORY.md → individual files become separate customizations**. This is tested.

#### Rules discovery (`_discover_rules`)

`discovery.py:531-569`. Dedup by resolved path across user + project; each rule file becomes a `MEMORY_FILE` customization with `name` = relative path from `rules_dir`. **Recursive via `walk_filtered`** (no `max_depth` cap, see gitignore_filter section). Test `test_rules_discovery.py` exercises happy paths but **does not test gitignore filtering of rules**.

#### MCP discovery — three locations

`_discover_mcps()` (`:571-586`):
1. `~/.claude.json` at USER level → `MCPParser.parse(...)` (which reads `data["mcpServers"]` at root)
2. `_discover_local_mcps()` (`:588-620`) — reads `~/.claude.json`, looks up `data["projects"][project_path_with_forward_slashes]` OR `data["projects"][project_path_with_backslashes]` (the **P0 fuzzing**, see below)
3. `./.mcp.json` at PROJECT level

The **P0 path fuzzing** in `_discover_local_mcps:600-606`:
```python
project_path = str(self.project_root).replace("\\", "/")
mcp_servers = None
for key in [project_path, project_path.replace("/", "\\")]:
    if key in projects:
        mcp_servers = projects[key].get("mcpServers", {})
        break
```
Forward-slash form is tried first, then backslash. Tested in `tests/integration/discovery/test_mcps.py:187-217`. **Order matters:** if both keys exist (pathological), forward-slash wins.

#### Hook discovery — three locations

`_discover_hooks()` (`:622-641`): USER → `~/.claude/settings.json`; PROJECT → `./.claude/settings.json`; PROJECT_LOCAL → `./.claude/settings.local.json`. All wrapped in `hooks` key. Plugin hooks at `<install_path>/hooks/hooks.json` (unwrapped, must NOT be in `hooks` key — this is a known schema divergence from Pass B-r1).

#### Plugin enumeration (`_discover_plugins`)

`discovery.py:643-665`. Iterates `self._plugin_loader.get_all_plugins()` (which is the three-phase user/project/local enumeration in `plugin_loader.py:108-157`). For each plugin:
- Scan all three SCAN_CONFIGS at `install_path` with level=PLUGIN
- `_discover_plugin_mcps(install_path, plugin_info)` (`:667-682`) reads `<install_path>/.mcp.json` and parses normally (wrapped in `mcpServers` key)
- `_discover_plugin_hooks(install_path, plugin_info)` (`:684-699`) reads `<install_path>/hooks/hooks.json` (un-wrapped per Pass B-r1)
- `_discover_plugin_lsp_servers(install_path, plugin_info)` (`:701-722`) reads BOTH `<install_path>/.lsp.json` AND `<install_path>/.claude-plugin/plugin.json[lspServers]` — **two distinct sources**

#### Preview entry (`discover_from_directory`)

`discovery.py:208-241`. Used for marketplace plugin preview. Differences from `_discover_plugins`:
- Creates fresh `GitignoreFilter` rooted at `plugin_dir` (not `project_root`) — `:218`
- Creates fresh `FilesystemScanner` — `:219`
- Iterates SCAN_CONFIGS for level=PLUGIN — `:221-224`
- If `marketplace_plugin` was passed, calls `_discover_marketplace_components(...)` (`:253-302`) which honors `extra_metadata` keys for `commands`, `agents`, `skills`, `mcpServers`, `hooks` — custom path overrides defined in `marketplace.json`. **Dedup via `seen_paths` set of resolved paths.**
- If `plugin_info` was passed, also discovers mcps/hooks/lsp from the standard plugin shapes (`:235-239`)

This entire path is **untested** in the reference (per Pass B-r2 finding 1).

#### Marketplace-extras normalization

`_normalize_paths(value)` (`:304-311`): None → `[]`; str → `[value]`; list → unchanged. So `marketplace.json` can specify `"commands": "path/to/dir"` OR `"commands": ["path1", "path2"]`. Used for `commands`, `agents`, `skills`. The `mcpServers` and `hooks` extras are str-only (`:290-300`).

#### Caching invariant

`self._cache: list[Customization] | None = None`. `discover_all()` returns it identity-equal on subsequent calls (`first_call is second_call` per `test_behavior.py:170-173`). `refresh()` sets to None and re-discovers; it also calls `self._plugin_loader.refresh()` (`:198-199`). **Subtle:** caller mutation of the returned list mutates the cached value — Rust port should use `Arc<Vec<...>>` or return a clone.

### 2. `config_path_resolver.py` — `ConfigPathResolver`

`config_path_resolver.py:9-71`. Translates PLUGIN-level paths from cache-install-paths to source-directory-paths.

#### Resolution table

| Input level | Has `plugin_info`? | `get_plugin_source_path` returns | `source_root == install_path`? | `file_path.relative_to(install_path)` raises? | Output |
|---|---|---|---|---|---|
| Non-PLUGIN | n/a | n/a | n/a | n/a | `file_path` as-is |
| PLUGIN | None | n/a | n/a | n/a | `file_path` as-is |
| PLUGIN | Some(info) | None | n/a | n/a | `file_path` as-is |
| PLUGIN | Some(info) | source_root | true | n/a | `file_path` as-is |
| PLUGIN | Some(info) | source_root | false | yes (path outside install_path) | `file_path` as-is |
| PLUGIN | Some(info) | source_root | false | no | `source_root / relative_path` |

8 cases, 5 distinct outputs. All explicitly tested in `test_config_path_resolver.py:18-212`.

#### File-path None handling

`resolve_file(c)` delegates to `resolve_path(c, c.path)`. `resolve_path` early-returns None when `file_path is None` (`:46-47`). This is the only public divergence between the two methods. Tested at `:139-154`.

**Used by:** any code that opens a plugin file in $EDITOR — without this, opens would resolve to ephemeral cache paths instead of the actual source repo.

### 3. `filesystem_scanner.py` — `FilesystemScanner`

`filesystem_scanner.py:34-116`. Generic scanner for the three SCAN_CONFIGS.

#### Strategy table

| Strategy | Files yielded | Gitignore application |
|---|---|---|
| `GlobStrategy.RGLOB` | `target_dir.rglob(pattern)` | If filter present: `filter.walk_filtered(target_dir, pattern, max_depth=config.max_depth)` (`:84-87`). **Otherwise raw rglob** — no skip-dir pruning, no gitignore. |
| `GlobStrategy.GLOB` | `target_dir.glob(pattern)` (flat) | Post-filter: `[f for f in files if not filter.is_ignored(f)]` (`:92-94`). |
| `GlobStrategy.SUBDIR` | `[subdir/pattern for subdir in iterdir() if is_dir]` | Pre-filter: skip_dir + is_dir_ignored (`:99-106`); then post-filter on file (`:114`). |

#### Parser factory dual-signature

`filesystem_scanner.py:65-68`:
```python
try:
    parser = config.parser_factory(target_dir, gitignore_filter=self._filter)
except TypeError:
    parser = config.parser_factory(target_dir)
```
This is the **anti-pattern flagged in Pass 8 D7** — the SlashCommandParser accepts a gitignore_filter kwarg, the others don't. Rust port should unify constructor signatures.

#### SUBDIR strategy details

`:95-115`:
- Enumerate `subdir for subdir in target_dir.iterdir() if subdir.is_dir()`
- For each, check (filter is None) OR (not should_skip_dir(subdir.name) AND not is_dir_ignored(subdir))
- Build `subdir / pattern` (the SKILL.md path)
- Keep only paths where `.is_file()`
- Apply final `is_ignored` check on the file

Tested in `test_filesystem_scanner.py:89-220` — including filtering both at directory and file level (`:177-220`).

**Race:** filesystem state can change between `iterdir()` and the `.is_dir()` / `.is_file()` checks. No locking, no retries. Acceptable for a single-user TUI.

### 4. `filter.py` — `FilterService`

`filter.py:57-126`. **ZERO unit tests.** Worth documenting carefully because filter semantics drive the UI.

#### Filter algorithm

`filter(customizations, query="", level=None, plugin_enabled=None)` (`:60-84`):

1. Start with `result = customizations`
2. If `level is not None`: filter to those matching `_matches_level(c, level)`
3. If `plugin_enabled is not None`: filter to those where `c.plugin_info is None OR c.plugin_info.is_enabled == plugin_enabled`
4. If `query` is truthy: lowercase query, filter to those matching `_matches_query(c, query_lower)`

**Composition is AND.** No OR, no negation. Order maintained from input.

#### Level matching (`_matches_level`)

`:86-107`:
- Exact match: `c.level == level` → true
- If `level == PROJECT`:
  - `c.level == PROJECT_LOCAL` → true (project_local items appear in project filter)
  - `c.plugin_info != None AND c.plugin_info.scope in (PROJECT, PROJECT_LOCAL)` → true (project-scoped plugins also appear in project filter)
- Otherwise → false

**Asymmetric:** PROJECT_LOCAL items match both PROJECT_LOCAL and PROJECT filters. But PROJECT items do NOT match PROJECT_LOCAL filter. **No reverse promotion.** This is intentional — the UI "Project" tab is broader than "Project-Local" sub-view.

#### Query matching (`_matches_query`)

`:109-118`:
- Direct substring match on `c.name.lower()`
- If plugin: also try `f"{plugin.short_name}:".lower()` as prefix AND `f"{plugin.short_name}:{name}".lower()` as full string. Match if query is a substring of either prefix or full.

**Plugin-prefix match is permissive:** typing `:` matches every plugin item.

#### `by_type` helper

`:120-126`. Pure filter `c.type == ctype`. Used for panel-by-type display.

**Coverage gap:** No tests for any of these. The behavior is encoded only in code, not pinned by tests. **Risk for Monocle port:** small refactors could change behavior without test failure.

### 5. `gitignore_filter.py` — `GitignoreFilter`

`gitignore_filter.py:57-149`. Two filters in one: hard-coded skip-dirs + soft pathspec-based gitignore.

#### Skip-dir set (DEFAULT_SKIP_DIRS)

`gitignore_filter.py:10-30`. 20 entries: `.git`, `node_modules`, `.venv`, `venv`, `__pycache__`, `.mypy_cache`, `.pytest_cache`, `build`, `dist`, `.eggs`, `.tox`, `.nox`, `htmlcov`, `.idea`, `.vscode`, `bin`, `obj`, `.vs`, `packages`. **Name-only match** (no path). Applied by `should_skip_dir(dirname)` (`:92-94`).

#### Default pattern set (DEFAULT_IGNORE_PATTERNS)

`:32-54`. 22 patterns mostly mirroring the skip-dirs but with trailing `/` plus `.coverage` and `*.egg-info/`. Loaded into pathspec at init **always** (not gated by `use_gitignore`).

#### Init behavior

`:60-74`:
- If `use_gitignore=True AND project_root != None`: read `<root>/.gitignore`, strip comments and blanks, append to patterns
- pathspec built from combined list, using `"gitignore"` style

If `use_gitignore=False`: only DEFAULT_IGNORE_PATTERNS applied.

#### `is_ignored` and `is_dir_ignored`

`:96-125`. Both:
- Returns False if no spec
- Computes `rel_path = path.relative_to(project_root)` if `project_root` set, else uses absolute path
- For `is_dir_ignored`: appends `"/"` to the str repr before matching (matches directory-only patterns like `temp/`)

#### `walk_filtered` algorithm

`:127-149`. The walker used by recursive discovery (slash commands rglob, memory files, rules):

1. Compute `root_depth = str(root).count(os.sep)`
2. For each `(dirpath, dirnames, filenames)` from `os.walk(root)`:
   a. `current_depth = str(dirpath).count(os.sep) - root_depth`
   b. If `max_depth is not None AND current_depth >= max_depth`: clear dirnames (prune) and skip processing
   c. Filter dirnames in-place: remove names matching `should_skip_dir` OR `is_dir_ignored`
   d. For each filename matching fnmatch.fnmatch(pattern): if not `is_ignored`, yield
3. **`fnmatch.fnmatch` is case-insensitive on Windows, case-sensitive on Unix.** This is a platform-dependent gotcha for the Rust port.

`DEFAULT_MAX_WALK_DEPTH = 5` from `discovery.py:31`. Applied only in nested CLAUDE.md walk (`discovery.py:450-451`), not in rules or skill scanning.

**Tests:** `test_gitignore_filter.py:1-227` exhaustively covers `should_skip_dir`, `is_ignored`, `walk_filtered` with/without project_root, with/without gitignore, prune nested dirs, max_depth.

**Symlink behavior:** `os.walk` does NOT follow symlinks by default (`followlinks=False`). Untested but inherited from Python stdlib. Rust port using `walkdir` should set `follow_links(false)` to match.

### 6. `opener.py` — system opener

`opener.py:1-42`. **ZERO tests.** Two functions.

#### `open_in_file_explorer(path) -> tuple[bool, str | None]`

`:9-28`:
- If path doesn't exist → `(False, f"Path not found: {path}")`
- Platform dispatch:
  - Windows → `subprocess.run(["explorer", str(path)], check=False)`
  - Darwin → `subprocess.run(["open", str(path)], check=False)`
  - else → `subprocess.run(["xdg-open", str(path)], check=False)`
- On OSError → `(False, f"Failed to open explorer: {e}")`
- Otherwise → `(True, None)`

**Note:** `check=False` means non-zero exit codes are NOT raised. So `explorer.exe` returning failure exit code is silently ignored. Verified by reading.

**No shell=True.** Safe wrt argument injection because args is a list.

#### `open_github_source(repo, sub_path=None) -> None`

`:31-42`:
- `url = f"https://github.com/{repo}"`
- If sub_path: `url = f"{url}/tree/main/{sub_path}"`
- `webbrowser.open(url)`

**P1 finding (new!):** The URL hardcodes the `main` branch. If a marketplace plugin's source dir is on a `master` branch or any other branch, the link 404s. This is **not** the same as the install_location — it's the in-repo source path. Worse, no validation on `sub_path` — could be `../../malicious` though the impact is minimal (just a 404).

**Used by:** `mixins/marketplace.py:309, 315, 329, 335`. Both invocations pass the marketplace's `source_type` discriminator — `directory` uses file explorer, `github` uses github URL. Validated by reading.

#### Monocle implication

The "open in OS file manager / browser" is a leaf-level platform-dispatch utility. Rust port:
- Use the `opener` crate or hand-roll `Command::new("xdg-open" | "open" | "explorer")` with platform cfg
- Hardcoded `main` branch should become a configurable default with optional `?branch=` override, or use the GitHub API to determine the default branch

### 7. `settings.py` — `SettingsService`

`settings.py:24-110`. Loads/saves `~/.lazyclaude/settings.json`.

#### Settings file shape

```json
{
  "theme": "<theme_name>",
  "marketplace_auto_collapse": true|false,
  "suggested_marketplaces": {
    "<owner/repo>": {"tags": [...], "stars": <int>}
  }
}
```

#### Default location

`~/.lazyclaude/settings.json` (`settings.py:28-30`). Overridable via constructor.

#### Load semantics

`load()` (`:37-53`):
- File not exists → return `AppSettings()` (defaults)
- Load JSON; on `json.JSONDecodeError` OR `OSError` → return `AppSettings()`
- Construct `AppSettings(theme=..., marketplace_auto_collapse=..., suggested_marketplaces=...)`
- **Subtle:** `data.get("theme", AppSettings.theme)` — this uses the **class attribute** as default. Since `AppSettings.theme` is `DEFAULT_THEME` constant, this works, but if `AppSettings` becomes a non-dataclass or `theme` becomes a field without class-level default, this breaks.
- `data.get("suggested_marketplaces", {})` — defaults to empty dict, not `DEFAULT_SUGGESTED_MARKETPLACES`. The migration runs separately via `ensure_suggested_marketplaces`.

#### Save semantics — **P0 atomic-write gap confirmed**

`save(settings)` (`:55-69`):
```python
self._settings_path.parent.mkdir(parents=True, exist_ok=True)
data = {...}
self._settings_path.write_text(
    json.dumps(data, indent=2) + "\n",
    encoding="utf-8",
)
```
**Naked `write_text`. No tempfile, no rename.** If the process is killed mid-write, the file can be left truncated. On power loss, the same. Rust port should use `tempfile::NamedTempFile + persist()`.

`OSError` is swallowed (`:68-69`). Silently fails to save. No notification to user, no logging.

#### `ensure_suggested_marketplaces(settings) -> AppSettings`

`:71-96`. Migration helper:
- For each `(repo, default_data)` in `DEFAULT_SUGGESTED_MARKETPLACES`:
  - If repo not in existing OR existing != default_data: assign default, mark updated
- If updated: call `save()` (and re-suffer the atomic-write gap)
- Return settings

#### Deep equality migration

`_marketplace_needs_update(existing, default)` (`:98-110`) just returns `existing != default`. Python dict `!=` does recursive comparison. Trade-off documented in docstring.

#### Default suggested marketplaces

`DEFAULT_SUGGESTED_MARKETPLACES` at `:9-21`. 8 entries: `anthropics/skills`, `anthropics/knowledge-work-plugins`, `anthropics/claude-plugins-official`, `NikiforovAll/claude-code-rules`, `Piebald-AI/claude-code-lsps`, `wshobson/agents`, `davila7/claude-code-templates`, `ComposioHQ/awesome-claude-skills`. Each with `tags: list[str]` and `stars: int`. **Stars values will become stale fast** — the migration logic specifically handles this: any field change triggers re-write on next startup.

#### Test coverage

`test_settings_service.py` covers:
- load defaults when file missing
- load defaults when invalid JSON
- load theme from valid JSON
- save creates dir and file
- save overwrites existing
- round-trip preserves theme
- default path = `~/.lazyclaude/settings.json`

**Gaps:** marketplace_auto_collapse not tested; ensure_suggested_marketplaces not tested; partial / malformed `suggested_marketplaces` structure not tested.

### 8. `writer.py` — `CustomizationWriter`

`writer.py:17-518`. The mutation surface. 519 LOC, the largest service file. Tests are extensive but **all of them write naked, non-atomic.**

#### Public method matrix

| Method | What it writes | Conflict semantics |
|---|---|---|
| `write_customization(c, target_level, user_cfg, project_cfg)` | Standalone files: slash commands, subagents, skills (entire dir), memory files | Returns `(False, ...)` if target exists |
| `delete_customization(c)` | n/a | Returns `(False, ...)` only on PermissionError or OSError |
| `write_hook_customization(c, target_level, user_cfg, project_cfg)` | settings.json under `hooks` key, merging | `(False, ...)` if no source hooks |
| `delete_hook_customization(c)` | settings.json — removes only `hooks` key, deletes file if empty | `(False, ...)` if no hooks key |
| `write_mcp_customization(c, target_level, project_cfg)` | `.claude.json` (user/local), `.mcp.json` (project) | `(False, ...)` if name already in mcpServers |
| `delete_mcp_customization(c, project_cfg)` | Removes specific server entry, deletes file if empty (PROJECT only) | `(False, ...)` if name not in mcpServers |
| `toggle_plugin_enabled(plugin_info, user_cfg, project_cfg)` | settings.json `enabledPlugins[plugin_id]` | Always succeeds (toggles current state) |

#### Target path table

| Type | Target path |
|---|---|
| SLASH_COMMAND `name=foo:bar:baz` | `<base>/commands/foo/bar/baz.md` — nested via `:` split |
| SLASH_COMMAND `name=foo` | `<base>/commands/foo.md` |
| SUBAGENT `name=foo` | `<base>/agents/foo.md` |
| SKILL `name=foo` | `<base>/skills/foo` (directory) |
| MEMORY_FILE | `<base>/<original_basename>` (preserves filename: CLAUDE.md / AGENTS.md / etc.) |
| Other | raises `ValueError` |

#### Hook target path table

| Level | Target path |
|---|---|
| USER | `<user_cfg>/settings.json` |
| PROJECT | `<project_cfg>/settings.json` |
| PROJECT_LOCAL | `<project_cfg>/settings.local.json` |

#### MCP target path table

| Level | Target path |
|---|---|
| USER | `~/.claude.json` (root-level `mcpServers`) |
| PROJECT | `<project_cfg.parent>/.mcp.json` (root-level `mcpServers`) |
| PROJECT_LOCAL | `~/.claude.json` (`projects[<path>][mcpServers]`) |

**Note:** PROJECT_LOCAL writes use `str(project_config_path.parent).replace("\\", "/")` — forward-slash form is **only what's written**. The discovery side fuzzy-matches both forms but this writer only ever produces forward-slash keys. **Inconsistency** — if a user has a backslash-keyed entry and adds a local MCP, both will exist, both will resolve, but only forward-slash is canonical.

#### Hook merge semantics

`_merge_hooks(existing, source)` (`:343-359`):
- For each `event_name` in source:
  - If event_name in existing: `merged[event_name] = existing[event_name] + source[event_name]` (list concat)
  - Else: `merged[event_name] = source[event_name]`
- **No dedup** — if existing has `{matcher: "Bash"}` and source has `{matcher: "Bash"}`, you get two identical entries.

Tested at `test_customization_writer.py:324-371`.

#### Skill copy

`_copy_skill_directory(source_dir, target_dir)` (`:420-432`):
```python
shutil.copytree(source_dir, target_dir, dirs_exist_ok=False)
```
- Source is `c.path.parent` (the directory containing SKILL.md)
- Target is the new skill directory location
- `dirs_exist_ok=False` — raises if target exists. Conflict check (`_check_conflict`) catches this first.
- Symlinks: by default `copytree` does not follow them. The Rust port using `std::fs::copy` or `fs_extra::dir::copy` should set the same.

#### File copy

`_write_file(source_path, target_path)` (`:415-418`):
```python
content = source_path.read_text(encoding="utf-8")
target_path.write_text(content, encoding="utf-8")
```
**Naked read+write.** No tempfile, no rename. **P0 confirmed.** If process is killed between `read_text` and `write_text`, target is unchanged (good); if killed after `write_text` starts and before completion, target may be truncated (BAD).

Rust port: `std::fs::copy(src, dst)` is atomic on most filesystems if src and dst are on same volume; cross-volume requires `tempfile::NamedTempFile::persist`.

#### Conflict detection

`_check_conflict(c, target_path)` (`:404-409`):
- For SKILL: `target_path.exists() AND target_path.is_dir()`
- Otherwise: `target_path.exists() AND target_path.is_file()`

**Race:** between conflict check and write, target could be created externally. Not protected.

#### JSON helpers

`_read_settings_json(path) -> dict` (`:505-513`):
- If not file → `{}`
- On `json.JSONDecodeError` → `{}` (silent)
- OSError → propagates (NOT swallowed; caller catches)

`_write_settings_json(path, data)` (`:515-518`):
- `path.parent.mkdir(parents=True, exist_ok=True)`
- `path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")`
- **Naked write.** **P0 confirmed.**

Used by hooks, mcps, toggle_plugin — all 3 mutation surfaces are non-atomic.

#### `toggle_plugin_enabled`

`writer.py:442-484`. Reads settings, sets `enabledPlugins[plugin_id] = not enabled_plugins.get(plugin_id, True)`. **Default True if missing** — toggling a never-seen plugin makes it `False`.

Scope dispatch via `_get_settings_path(scope, user_cfg, project_cfg)` (`:486-503`):
- USER → `<user_cfg>/settings.json`
- PROJECT → `<project_cfg>/settings.json`
- PROJECT_LOCAL → `<project_cfg>/settings.local.json`

**Transactional semantics: NONE.** Each write is independent. If a copy succeeds and the corresponding delete (for move) fails, both copies exist. The `move` operation is implemented in `mixins/customization_actions.py:165-212` as `write_*` then `delete_*` with no rollback. **P1 confirmed.**

#### Test coverage

| Surface | Tested |
|---|---|
| write_customization (4 types, conflict, parent dirs, permission) | YES (8 tests in `test_customization_writer.py`) |
| delete_customization (4 types, missing, project-level) | YES (8 tests in `test_delete_writer.py`) |
| write_hook_customization (new, merge, local) | YES (4 tests) |
| delete_hook_customization (preserve, empty-file, no-hooks) | YES (3 tests) |
| write_mcp_customization (3 levels, conflict) | YES (`test_mcp_writer.py`) |
| delete_mcp_customization (3 levels, last-entry-deletes-file, not-found) | YES |
| toggle_plugin_enabled | **NO** — confirmed gap |
| Permission error paths | YES (Unix only, `pytest.skipif win32`) |
| Atomic-write under crash | **NO** — not tested anywhere |

### 9. `marketplace_loader.py` — `MarketplaceLoader`

`marketplace_loader.py:15-307`. Loads marketplaces, joins install state, computes scoped views. **ZERO direct tests** — confirmed.

#### Cached state

Eleven cached members (lines `:25-34`):
- `_installed_plugin_ids: set[str] | None`
- `_enabled_plugin_ids: set[str] | None`
- `_install_paths: dict[str, Path] | None`
- `_installed_versions: dict[str, str] | None`
- `_marketplaces_cache: list[Marketplace] | None`
- `_installed_scopes: dict[str, list[str]] | None`
- `_user_installed_ids: set[str] | None`
- `_project_installed_ids: set[str] | None`
- `_scope_install_paths: dict[str, dict[str, Path]] | None`
- `_scope_versions: dict[str, dict[str, str]] | None`
- `display_scope: str = "user"` — **NOT** cleared by `refresh()`; user-facing toggle

All caches reset to None by `refresh()` (`:293-306`); except `display_scope`.

#### Load orchestration

`load_marketplaces()` (`:37-62`):
1. If cache present → return cached
2. Else read `<user_cfg>/plugins/known_marketplaces.json`
3. If file missing → return empty
4. On JSON / OS error → return empty (silent)
5. Call `_load_installed_plugins()` to populate the 11 fields
6. For each entry, parse → `MarketplaceEntry`, then `_load_marketplace(entry)` → `Marketplace` with plugins
7. Cache and return

#### Marketplace entry parsing

`_parse_marketplace_entry(name, data)` (`:64-86`):
- Returns None if `installLocation` is falsy
- Source: `MarketplaceSource(source_type=data["source"]["source"] or "unknown", repo=..., path=...)`
- The dual usage of `"source"` key inside `source` dict is **confusing** but matches Claude's on-disk format. Field path: `data["source"]["source"]` is the type discriminator.

#### Per-marketplace plugin loading

`_load_marketplace(entry)` (`:88-111`):
- Read `<install_location>/.claude-plugin/marketplace.json`
- File missing → `Marketplace(entry=..., error="marketplace.json not found at <path>")`
- JSON/OSError → `Marketplace(entry=..., error=str(e))`
- For each plugin in `data["plugins"]`, parse

#### Plugin parsing

`_parse_plugin(data, marketplace_name)` (`:113-165`):
- Skip if no `name`
- `full_id = f"{name}@{marketplace_name}"`
- Look up install state from cached fields, with scope-specific lookup:
  - `display_scope == "project"`: use `_project_installed_ids` + scope paths/versions["project" or "local"]
  - Otherwise: use `_user_installed_ids` + scope paths/versions["user"]
- Fallback to `_install_paths[full_id]` and `_installed_versions[full_id]` (non-scoped) if scope-specific not set
- `is_enabled = full_id in _enabled_plugin_ids if is_installed else True`
- Source field: if dict, use `source.get("url", str(source))`; else use as-is
- `extra_metadata`: all keys NOT in `("name", "description", "source")`

**Subtle:** disabled non-installed plugins show `is_enabled=True` by default (`:155`). Display logic must check `is_installed` first.

#### Installed-plugin enumeration

`_load_installed_plugins()` (`:167-248`): the most complex method. Reads `plugin_loader.load_registry()` and computes derived sets:

1. `installed_plugin_ids = set(registry.installed.keys())` — all known plugin IDs
2. `enabled_plugin_ids` — the **set algebra**:
   ```python
   enabled = (installed_plugin_ids - {pid for pid, e in merged.items() if not e}) 
             | enabled_in_user | enabled_in_project | enabled_in_local
   ```
   Where `merged = {**user_enabled, **project_enabled, **local_enabled}` (precedence local > project > user via dict merge order).
   
   **Reading:** start from "all installed", remove any that are explicitly disabled anywhere, then re-add any that are explicitly enabled in any scope.
   
   This means: a plugin that's `false` in user_enabled but `true` in project_enabled ends up enabled. Last-wins-via-OR.

3. For each `(pid, installations)` in `registry.installed.items()`:
   - `_install_paths[pid] = Path(installations[0].install_path)` — **first installation wins** for the non-scoped fallback. Order-dependent.
   - `_installed_versions[pid] = installations[0].version`
   - For each installation: dispatch by scope:
     - `"user"`: add to `_user_installed_ids`, append `"user"` to scopes
     - `"project"` or `"local"`: if `resolved_root` matches `installation.project_path`, add to `_project_installed_ids` and append the scope name
   - Always record scope-specific install_path and version in `_scope_install_paths[pid][scope]` and `_scope_versions[pid][scope]`
   - `_installed_scopes[pid] = list(dict.fromkeys(scopes))` — dedup preserving order

4. If `_plugin_loader` is None → all fields set to empty defaults

#### Source-dir resolution

`get_plugin_source_dir(plugin)` (`:250-265`):
1. If `plugin.install_path` exists → return it
2. Else if `install_path.parent` exists → return `_find_latest_version_dir(parent_dir)`
3. Else find the marketplace and compute `(install_location / plugin.source).resolve()`
4. Return None if all fail

**Difference from `PluginLoader.get_plugin_source_path`:** that one resolves source-of-truth for directory-source plugins (the actual repo); this one resolves the **install** location for opening in editor / file explorer.

#### Latest-version directory

`_find_latest_version_dir(parent_dir)` (`:267-275`) + `_parse_version` (`:277-283`): same algorithm as PluginLoader's. `tuple(int(part) for part in name.split("."))` falls back to `(name,)` for non-numeric.

**Numeric tuples sort before string tuples in Python 2-but-not-3:** `(1,2,3) < ("foo",)` raises in Python 3. **Risk:** if a plugin parent dir has BOTH semver and non-semver subdirs, `max()` will raise `TypeError`. Not tested. Latent bug.

#### Cross-cutting: 5 calls into `_plugin_loader`

- `load_registry()`
- `project_root` attribute access
- `_registry = None` direct reset in `refresh()` (`:306`) — **breaks encapsulation**

Direct private-attribute write to plugin_loader is a smell. Rust port should expose a proper invalidate API.

### 10. `plugin_loader.py` — `PluginLoader` (+ `PluginInstallation`, `PluginRegistry`)

`plugin_loader.py:11-353`.

#### Registry shape

```python
@dataclass
class PluginInstallation:
    scope: str           # "user" | "project" | "local"
    install_path: str
    version: str
    is_local: bool = False
    project_path: str | None = None

@dataclass
class PluginRegistry:
    installed: dict[str, list[PluginInstallation]]  # multi-installation per plugin_id
    user_enabled: dict[str, bool]
    project_enabled: dict[str, bool]
    local_enabled: dict[str, bool]
```

`plugin_loader.py:11-29`.

#### V2 file location

`<user_cfg>/plugins/installed_plugins.json`. Format:
```json
{
  "version": 2,
  "plugins": {
    "<plugin_id>": [
      {"scope": "user", "version": "1.0.0", "installPath": "...", "isLocal": false, "projectPath": null}
    ]
  }
}
```

`plugin_loader.py:79-101`. Defaults: `scope="user"`, `version="unknown"`, `install_path=""`, `is_local=False`, `project_path=None`. On JSONDecodeError or OSError → `{}`.

#### Enabled-plugins location

`<user_cfg>/settings.json` (USER), `<project_cfg>/settings.json` (PROJECT), `<project_cfg>/settings.local.json` (PROJECT_LOCAL). All read via `_load_json_dict(path, "enabledPlugins")` — `:263-272`.

#### Three-phase plugin enumeration

`get_all_plugins()` (`:108-157`):

**Phase 1: User-scoped**
- For each (plugin_id, installations) in registry.installed:
  - For each installation with `scope == "user"`:
    - Create PluginInfo; if install_path is dir, include

**Phase 2: Project-scoped (driven by `project_enabled` keys)**
- For each plugin_id in `registry.project_enabled` keys:
  - Look up registry.installed[plugin_id]
  - For each installation with `scope == "project"` AND `_matches_current_project(installation.project_path)`:
    - Create PluginInfo (scope_type="project"); if install_path is dir, include

**Phase 3: Local-scoped (driven by `local_enabled` keys)**
- Same as Phase 2 but for `scope == "local"`

**Subtle:** Phase 2/3 are **gated by presence in enabled_plugins dict**, not by enabled-value. A `false` entry still triggers consideration. The `is_enabled` flag on the resulting PluginInfo is then set from the dict value. So a project-scoped plugin only appears in `get_all_plugins()` if it appears in the project's settings.json's `enabledPlugins` map AT ALL.

#### `_matches_current_project`

`:159-166`:
```python
return Path(project_path).resolve() == self.project_root.resolve()
```
- False if either is None
- OSError → False (e.g., broken symlink)
- **Strict resolved-path equality.** Symlinks resolved. Trailing slashes normalized. Case-preserved on Unix; case-folded on Windows.

#### `get_plugin_source_path(plugin_id)`

`:172-212`. Resolves the **source** path for a plugin (not install).

1. Split plugin_id on `@`: parts = `[plugin_name, marketplace_name]` if has `@`, else `[plugin_id]`
2. If marketplace_name:
   - Load `<user_cfg>/plugins/known_marketplaces.json` via `_load_marketplace_info`
   - If marketplace_info has `source.source == "directory"` and `source.path`:
     - `marketplace_root = Path(source.path)`
     - Call `_find_plugin_source_in_marketplace(marketplace_root, plugin_name)`
     - If result, return it
3. Fallback: `registry.installed[plugin_id][0].install_path`
4. Return None

#### `_find_plugin_source_in_marketplace`

`:214-247`:
- Read `<marketplace_root>/.claude-plugin/marketplace.json`
- File missing OR JSON error → return `marketplace_root`
- For each plugin in data["plugins"]:
  - If name matches, get `source_relative`
  - If non-empty: `(marketplace_root / source_relative).resolve()` — if exists as dir, return; else fall through
- Loop ends → return `marketplace_root`

**Note:** the resolved-source is verified-to-exist via `.is_dir()` check before returning. The `marketplace_root` fallback is NOT verified-to-exist.

#### `_create_plugin_info`

`:274-327`. Constructs `PluginInfo` from installation:
- If install_path missing → None
- short_name = plugin_id.split("@")[0] if has @, else plugin_id
- install_path = Path(installation.install_path)
- version = installation.version
- **Latest-version fallback:** if install_path is not a dir BUT install_path.parent IS a dir → call `_find_latest_version_dir(parent_dir)`, use its name as version. This handles cache directories like `<plugin>/1.0.0` where the version was registered before the directory was renamed.

**Edge case:** `_find_latest_version_dir` can raise TypeError if parent contains a mix of semver-tuple and string-tuple names (same risk as `MarketplaceLoader._find_latest_version_dir`). Not tested.

- is_enabled: from registry.user/project/local_enabled[plugin_id], default True
- scope: mapped via `{"user": USER, "project": PROJECT, "local": PROJECT_LOCAL}`
- project_path: `Path(installation.project_path)` if string, else None

#### Test coverage

`test_plugin_source_path.py` covers ONLY `get_plugin_source_path`:
- directory source resolves
- non-directory source returns install_path
- missing marketplace.json returns marketplace_root
- plugin not in marketplace.json returns root
- plugin without `@` returns install_path
- unknown plugin returns None
- malformed marketplace.json returns root

**ZERO tests for:**
- `load_registry()` — V2 JSON parsing
- `get_all_plugins()` — three-phase enumeration
- `get_enabled_plugins()`
- `_matches_current_project()`
- `_create_plugin_info()` and the latest-version-dir fallback
- Cache invalidation via `refresh()`

The integration test `test_plugins.py:33-60` exercises a single disabled-plugin path through discovery → plugin_loader, but it's discovery-side.

## Cross-cutting findings

### Error handling pattern

| Pattern | Count | Citations |
|---|---|---|
| `try: ... except (json.JSONDecodeError, OSError): pass/return defaults` | 9+ | `discovery.py:617`, `gitignore_filter.py:89`, `settings.py:52`, `marketplace_loader.py:51, 102`, `plugin_loader.py:100, 245, 261, 271` |
| `try: subprocess.run(...): except OSError: return error` | 1 | `opener.py:27` |
| `try: ...: except PermissionError as e: return False, msg: except OSError as e: ...` | 7 | All public write/delete methods in `writer.py` |
| Silent `pass` swallow | 4+ | `settings.py:69`, `marketplace_loader.py:225, 274` |

**Pattern: external-input errors are silenced and return safe defaults.** Reasonable for a TUI; would be a smell in a server. Rust port should pair this with structured logging.

### Caching strategies

| Service | Cache key | Invalidation |
|---|---|---|
| `ConfigDiscoveryService` | `_cache: list[Customization] \| None` | `refresh()` → None + plugin_loader.refresh() |
| `PluginLoader` | `_registry: PluginRegistry \| None` | `refresh()` → None |
| `MarketplaceLoader` | 11 cached fields including `_marketplaces_cache` | `refresh()` → all None (preserves display_scope) |

All caches are **per-instance, not process-global.** Rust port can use `OnceCell` per instance, but should provide explicit `refresh()` semantics.

### Concurrency safety

**NONE in this layer.** No locks, no atomic writes, no transactions. Acceptable because:
- Single-user, single-process TUI
- Claude CLI mutations happen out-of-process (via `subprocess.run` from mixin layer)
- User-initiated refresh re-reads from disk

**Risk for Monocle:** if monocle ever adds file watchers or multi-process operation, this layer's mutation surface needs explicit locking or atomic writes.

### Test coverage gaps summary

| File | Coverage % (eyeballed) | Critical gaps |
|---|---|---|
| `config_path_resolver.py` | ~100% | none — all 5 branches explicitly tested |
| `filesystem_scanner.py` | ~80% | parser_factory TypeError fallback not directly tested |
| `gitignore_filter.py` | ~90% | symlink behavior not tested |
| `settings.py` | ~50% | `marketplace_auto_collapse`, `ensure_suggested_marketplaces`, deep equality migration not tested |
| `writer.py` | ~70% | `toggle_plugin_enabled` not tested; crash-during-write not tested; concurrent-write not tested |
| `plugin_loader.py` | ~30% | `load_registry`, `get_all_plugins` 3-phase, `_create_plugin_info` latest-version fallback not tested |
| `marketplace_loader.py` | **0%** | **EVERYTHING uncovered.** Integration tests exercise it indirectly via discovery, but no direct unit tests. |
| `filter.py` | **0%** | **EVERYTHING uncovered.** Pure logic, easy to miss-port. |
| `opener.py` | **0%** | platform dispatch, error paths, github URL construction all unverified |
| `discovery.py` | ~70% | `discover_from_directory` 0%, `_discover_marketplace_components` 0%, `_discover_plugin_lsp_servers` 0% |

## Confirmed P0/P1 findings (and one new P1)

### P0 (re-confirmed from prior passes)

1. **Atomic write gap** — confirmed in `settings.py:64-67`, `writer.py:415-418, 515-518`. ALL three mutation surfaces use naked `write_text`. Rust port MUST use `tempfile + atomic rename`.
2. **Project-slug regex** — confirmed at `discovery.py:484`: `re.sub(r"[^a-zA-Z0-9\-]", "-", str(self.project_root))`. Each match replaced individually; no collapsing. Byte-match required.
3. **Sort order** — confirmed at `discovery.py:247-251`: `(CustomizationType variant index, name.lower())`. Variant order = `auto()` order from `models/customization.py:37-46`.
4. **MCP backslash fuzzing** — confirmed at `discovery.py:600-606`. Tested in `test_mcps.py:187-217`.

### P1 (re-confirmed + new)

5. **Move = copy + delete without rollback** — confirmed in `mixins/customization_actions.py:165-212` (not in writer itself; writer is just copy and delete). The writer doesn't expose a `move_customization` — it's done by composition in the mixin. Rust port should add proper transactional move.
6. **Hardcoded `main` branch in GitHub URL** — NEW finding from `opener.py:40`: `f"{url}/tree/main/{sub_path}"`. Breaks for repos on `master` or feature branches. Consider GitHub API to resolve default branch.
7. **Mix of semver and string version dirs raises TypeError** — `marketplace_loader._parse_version` and `plugin_loader._parse_version` both return `tuple[int, ...] | tuple[str]`. If `max()` sees both shapes, raises. Latent.
8. **Backslash-path-key inconsistency in MCP writes** — writer always writes forward-slash form (`writer.py:205, 255`); discovery reads either form. If a user has historical backslash entries and adds new ones, both coexist and both resolve. Documentation issue.
9. **Filter service has no tests** — pure logic, easy to break during refactor.
10. **Opener service has no tests** — platform dispatch is unverified.
11. **MarketplaceLoader has no direct tests** — exercised only via discovery integration.

### P2 (worth noting)

12. **`MarketplaceLoader.refresh` writes to `_plugin_loader._registry`** — encapsulation break (`marketplace_loader.py:306`).
13. **`FilesystemScanner` parser_factory TypeError fallback** — anti-pattern, replace with uniform constructor (`filesystem_scanner.py:65-68`).
14. **No `max_depth` cap on rules walk** — `_discover_rules` could explode on a deep `.claude/rules/...` tree (`discovery.py:537-567`).
15. **`is_enabled=True` default for non-installed marketplace plugins** is correct but easy to mis-port — verify in Rust port.

## Monocle implications

### Required for parity

| Concern | Rust crate/approach |
|---|---|
| Atomic file writes | `tempfile::NamedTempFile::persist` for same-volume; explicit copy+rename for cross-volume |
| Path slug regex | `regex` crate with `[^a-zA-Z0-9\-]` |
| Walking with gitignore + skip-dirs | `ignore` crate (rust-analyzer's), or `walkdir` + manual `gitignore` crate |
| Cross-platform open | `opener` crate (handles Windows/macOS/Linux dispatch) |
| Glob matching | `glob` crate or `globset` |
| Semver | `semver` crate (replaces the 3× hand-rolled `_parse_version`) |
| JSON read with graceful errors | `serde_json` + `.unwrap_or_default()` pattern |
| Path normalization on Windows | always use `replace("\\", "/")` for keys into `.claude.json`; accept both during reads |

### Architectural decisions to mirror

1. **Cache identity vs clone** — Python returns same list object; Rust should `Arc<Vec<Customization>>` to preserve cheap identity check semantics.
2. **`refresh()` propagation** — `ConfigDiscoveryService.refresh()` invalidates plugin_loader cache too. Rust port should chain.
3. **Three-phase plugin enumeration** — user → project → local, with **presence in enabled_plugins** as the gate for phases 2 and 3. Don't try to simplify.
4. **Hooks unwrapped from plugins, wrapped from settings** — schema asymmetry. Pin per Pass B-r1.

### Architectural decisions to reconsider

1. **`SCAN_CONFIGS` strategy + custom branches** — see D2 in Pass 8. Consider unifying all 7 customization types under a single trait in Rust.
2. **Hand-rolled semver** — replace with `semver` crate.
3. **Silent JSON error swallow** — add `tracing` instrumentation.
4. **Hardcoded GitHub `main` branch** — optional default-branch resolution via GitHub API.

## Delta Summary

- New items added: 10 service-file canonical tables, 1 new P1 (`main` branch hardcode in opener), 3 new P2 (TypeError-mix in version sort, no max_depth on rules, encapsulation break in marketplace_loader.refresh), 4 new gap rows (toggle_plugin_enabled untested, filter.py 0% tests, opener 0% tests, marketplace_loader 0% direct tests)
- Existing items refined: P0 atomic-write confirmed at THREE call sites (settings.py:64-67, writer.py:415-418, 515-518), backslash-path-key inconsistency in MCP writes documented
- Remaining gaps after R1: latest-version-dir TypeError under mixed semver/string names is latent (needs threat model decision), filter.py semantics are pinned only in code, marketplace_loader 3-phase scope set-algebra is pinned only in code

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: This round produced **per-file canonical schemas for all 10 services**, finding 1 new P1 (hardcoded GitHub `main` branch) and 3 new P2 issues. Most consequentially, it documented **three coverage gaps with no test files at all** (filter, opener, marketplace_loader) — meaning the Rust port would have no specification regression net for these surfaces. A Rust developer's plan WOULD change materially given these findings: they would (a) write new tests for filter/opener/marketplace_loader BEFORE implementing, (b) confront the atomic-write requirement at three concrete call sites, (c) plan the latest-version-dir mixed-tuple edge case explicitly. The model of the system has changed: the discovery walker is more than just "calls parsers" — it has a 7-step orchestration with specific cache and dedup semantics, and the writer has a non-trivial per-type target-path table.

## Convergence Declaration

**Another round needed.** Substantive gaps remaining:

1. Filter service: behavior is only pinned in code, no tests, easy to mis-port. Need to enumerate the boolean-composition truth table from code in r2.
2. Marketplace loader 3-phase scope set-algebra (`_load_installed_plugins:167-248`) — the most complex pure-logic method in the codebase and totally untested. R2 should walk through it with example inputs.
3. Plugin loader's `_create_plugin_info` latest-version fallback (`plugin_loader.py:294-296`) — interaction with `_find_latest_version_dir` TypeError edge case.
4. Discovery's `_discover_marketplace_components` (`discovery.py:253-302`) — the entire marketplace-extras override path is 0% tested and has 6 internal branches.
5. Cross-cutting: confirm whether ANY OS-level Rust crate (e.g., `ignore`) gives parity for the DEFAULT_SKIP_DIRS + DEFAULT_IGNORE_PATTERNS + .gitignore hybrid in `gitignore_filter.py`.

## State Checkpoint

```yaml
pass: B
subpass: services
round: 1
status: complete
files_covered: 10
tests_read_in_full: 7
new_p1_findings: 1
new_p2_findings: 3
timestamp: 2026-05-11T19:30:00Z
novelty: SUBSTANTIVE
converged: false
```
