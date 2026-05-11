# Phase B Deepening: Non-Parser Services Layer — Round 2

Goal: close the 5 substantive gaps declared at end of r1.

1. Filter service: enumerate boolean-composition truth table
2. Marketplace loader 3-phase scope set-algebra: walk through with concrete inputs
3. Plugin loader `_create_plugin_info` latest-version fallback edge cases
4. Discovery `_discover_marketplace_components` 6 branches
5. Rust crate parity check for the gitignore filter hybrid

## Gap 1: FilterService truth table (filter.py)

`filter.py:60-118`. Three filter dimensions composed by AND. Enumerate every combination using a concrete sample list.

### Sample population

| # | name | type | level | plugin? | plugin.is_enabled | plugin.scope |
|---|---|---|---|---|---|---|
| 1 | `foo` | SLASH_COMMAND | USER | None | — | — |
| 2 | `bar` | SLASH_COMMAND | PROJECT | None | — | — |
| 3 | `baz` | SLASH_COMMAND | PROJECT_LOCAL | None | — | — |
| 4 | `quux` | SUBAGENT | PLUGIN | Some | True | PluginScope.USER |
| 5 | `corge` | SUBAGENT | PLUGIN | Some | False | PluginScope.USER |
| 6 | `grault` | SKILL | PLUGIN | Some | True | PluginScope.PROJECT |
| 7 | `garply` | SKILL | PLUGIN | Some | True | PluginScope.PROJECT_LOCAL |

### Filter results per call

| Call | Result |
|---|---|
| `filter(items)` | all 7 (no filter) |
| `filter(items, query="foo")` | #1 (name match) |
| `filter(items, query="bar")` | #2 |
| `filter(items, query="g")` | #6 grault, #7 garply (both contain 'g') |
| `filter(items, query=":")` | #4, #5, #6, #7 (plugin prefix `:` matches all plugin items) |
| `filter(items, level=USER)` | #1 only (USER) |
| `filter(items, level=PROJECT)` | #2, #3, #6, #7 (PROJECT + PROJECT_LOCAL items + project-scoped plugins) |
| `filter(items, level=PROJECT_LOCAL)` | #3 only (NO promotion from PROJECT) |
| `filter(items, level=PLUGIN)` | #4, #5, #6, #7 (exact match only; #2/#3 don't promote up) |
| `filter(items, plugin_enabled=True)` | #1, #2, #3, #4, #6, #7 (non-plugins pass; only #5 disabled fails) |
| `filter(items, plugin_enabled=False)` | #1, #2, #3, #5 (non-plugins ALSO pass; only #4/#6/#7 enabled fail) |
| `filter(items, level=PROJECT, plugin_enabled=True)` | #2, #3, #6, #7 (#6/#7 are plugin-enabled AND project-scoped) |
| `filter(items, level=PLUGIN, plugin_enabled=False)` | #5 only |
| `filter(items, query="g", level=PROJECT)` | empty — #6/#7 are PROJECT-promoted via plugin scope but neither matches query `g` after lowercase... wait, "grault" and "garply" both contain 'g'. Re-verify: query is lowercased, name is lowercased. So #6 and #7 match. Final result: #6, #7. |

### Subtle behaviors (re-pinned)

1. **`plugin_enabled=False` ALSO includes non-plugins.** `filter.py:74-78` early-returns true if `c.plugin_info is None`. So the predicate is "exclude plugins that don't match the desired enabled-state". Non-plugins are never excluded by this filter.

2. **`query=""` is a no-op** (`filter.py:80`: `if query:`). Empty string is falsy in Python; the branch is skipped. Same as `query=None`.

3. **`_matches_query` returns True if query is substring of either prefix OR full_name** (`filter.py:115-117`). The condition `query in prefix or query in full_name` — a 1-char query like `:` matches because `:` is in `prefix=f"<short_name>:"`. A query like `xxx` only matches if it's in the lowercased name OR in the prefix+name combination.

4. **Level filter asymmetry confirmed.** Only PROJECT level is "broader" — promotes PROJECT_LOCAL items AND project-scoped plugins. USER does NOT include user-scoped plugins (those are level=PLUGIN). PLUGIN does NOT include any non-plugin items. PROJECT_LOCAL is exact-match-only.

5. **`by_type` is independent** — not composed with filter; the caller applies them separately. Two-stage filtering in TUI: filter() → list, then by_type() per panel.

### Rust port truth table

A direct port via Rust closures over `Iterator::filter`:

```rust
items.iter()
    .filter(|c| match level {
        None => true,
        Some(ConfigLevel::Project) => 
            c.level == Project 
            || c.level == ProjectLocal 
            || c.plugin_info.as_ref()
                .map_or(false, |p| matches!(p.scope, PluginScope::Project | PluginScope::ProjectLocal)),
        Some(l) => c.level == l,
    })
    .filter(|c| match plugin_enabled {
        None => true,
        Some(want) => c.plugin_info.as_ref()
            .map_or(true, |p| p.is_enabled == want),
    })
    .filter(|c| match &query {
        s if s.is_empty() => true,
        s => {
            let q = s.to_lowercase();
            c.name.to_lowercase().contains(&q)
            || c.plugin_info.as_ref().map_or(false, |p| {
                let prefix = format!("{}:", p.short_name).to_lowercase();
                let full = format!("{}{}", prefix, c.name.to_lowercase());
                prefix.contains(&q) || full.contains(&q)
            })
        }
    })
```

**Pins:** identical AND composition, identical empty-query short-circuit, identical PROJECT promotion logic, identical `c.plugin_info is None → pass` for plugin_enabled filter.

## Gap 2: MarketplaceLoader `_load_installed_plugins` set-algebra walkthrough

`marketplace_loader.py:167-238`. The most complex pure-logic method. Use the actual fixture inputs.

### Concrete inputs (from `tests/integration/fixtures/`)

`installed_plugins.json`:
```json
{
  "version": 2,
  "plugins": {
    "example-plugin@test": [
      {"scope": "user", "installPath": ".../example-plugin/1.0.0", "version": "1.0.0", "isLocal": false}
    ],
    "disabled-plugin@test": [
      {"scope": "user", "installPath": ".../disabled-plugin/1.0.0", "version": "1.0.0", "isLocal": false}
    ]
  }
}
```

`user-settings.json` `enabledPlugins`:
```json
{
  "example-plugin@test": true,
  "disabled-plugin@test": false
}
```

`project-settings.json` `enabledPlugins`: not set (no `enabledPlugins` key) → empty dict.

### Step-by-step state evolution

Initial registry (from `plugin_loader.load_registry()`):
```python
registry.installed = {
  "example-plugin@test": [Installation(scope="user", ...)],
  "disabled-plugin@test": [Installation(scope="user", ...)]
}
registry.user_enabled = {"example-plugin@test": True, "disabled-plugin@test": False}
registry.project_enabled = {}
registry.local_enabled = {}
```

**Step 1** (`:171`): `self._installed_plugin_ids = {"example-plugin@test", "disabled-plugin@test"}`

**Step 2** (`:172-174`): `enabled_in_user = {"example-plugin@test"}` (only True values)

**Step 3** (`:175-180`): `enabled_in_project = set()`, `enabled_in_local = set()`

**Step 4** — The big set-algebra (`:181-197`):
```python
merged = {**user_enabled, **project_enabled, **local_enabled}
# = {"example-plugin@test": True, "disabled-plugin@test": False}
disabled_set = {pid for pid, e in merged.items() if not e}
# = {"disabled-plugin@test"}
self._enabled_plugin_ids = (
    installed_plugin_ids - disabled_set  # = {"example-plugin@test"}
) | enabled_in_user | enabled_in_project | enabled_in_local
# = {"example-plugin@test"} | {"example-plugin@test"} | {} | {}
# = {"example-plugin@test"}
```

**Step 5** (`:198-204`): All scope maps initialized empty.

**Step 6** (`:205-206`): `resolved_root = self._plugin_loader.project_root.resolve()` if project_root, else None.

**Step 7** — The for-loop (`:208-238`):

Iteration 1: `pid="example-plugin@test"`, installations has 1 entry:
- `:209-211`: `self._install_paths[pid] = Path(".../1.0.0"); self._installed_versions[pid] = "1.0.0"`
- `:213`: `scopes = []`
- Inner loop, single inst:
  - `:215`: `scope = "user"`
  - `:216-218`: matches "user" branch: `self._user_installed_ids.add(pid)`, `scopes = ["user"]`
  - `:230-232`: init `self._scope_install_paths[pid] = {}`, `_scope_versions[pid] = {}`
  - `:233-235`: install_path non-empty, so set `_scope_install_paths[pid]["user"] = Path(...)`, `_scope_versions[pid]["user"] = "1.0.0"`
- `:237-238`: `scopes = ["user"]` is truthy, so `_installed_scopes[pid] = ["user"]` (dedupped)

Iteration 2: `pid="disabled-plugin@test"`, installations has 1 entry: same as above but for disabled.
- `_install_paths["disabled-plugin@test"] = Path(...)`, same install_path/version pattern
- `_user_installed_ids` now also contains `"disabled-plugin@test"`
- `_scope_install_paths["disabled-plugin@test"]["user"] = Path(...)`
- `_installed_scopes["disabled-plugin@test"] = ["user"]`

### Final state

```python
_installed_plugin_ids = {"example-plugin@test", "disabled-plugin@test"}
_enabled_plugin_ids = {"example-plugin@test"}
_install_paths = {
  "example-plugin@test": Path(".../example-plugin/1.0.0"),
  "disabled-plugin@test": Path(".../disabled-plugin/1.0.0"),
}
_installed_versions = {"example-plugin@test": "1.0.0", "disabled-plugin@test": "1.0.0"}
_installed_scopes = {"example-plugin@test": ["user"], "disabled-plugin@test": ["user"]}
_user_installed_ids = {"example-plugin@test", "disabled-plugin@test"}
_project_installed_ids = set()
_scope_install_paths = {
  "example-plugin@test": {"user": Path(...)},
  "disabled-plugin@test": {"user": Path(...)},
}
_scope_versions = {
  "example-plugin@test": {"user": "1.0.0"},
  "disabled-plugin@test": {"user": "1.0.0"},
}
display_scope = "user"  # never reset
```

### Pathological case: project-scoped plugin with mismatched project_path

Inputs (hypothetical):
```python
registry.installed = {
  "myplug@test": [
    Installation(scope="project", install_path="/proj-a/myplug", project_path="/other-project")
  ]
}
self._plugin_loader.project_root = Path("/proj-a")  # current project
```

Walk-through `:208-238`:
- `:209-211`: `_install_paths["myplug@test"] = Path("/proj-a/myplug")`, `_installed_versions["myplug@test"] = "unknown"` (default from V2 parse)
- `:215`: `scope = "project"`
- `:219`: matches "project" or "local" branch
- `:220`: `resolved_root = Path("/proj-a").resolve()` (truthy), `inst.project_path = "/other-project"` (truthy)
- `:221-224`: `Path("/other-project").resolve() == Path("/proj-a").resolve()` → False
- → does NOT add to `_project_installed_ids`, scopes stays `[]`
- `:230-232`: init scope dicts
- `:233-235`: still adds to `_scope_install_paths["myplug@test"]["project"]`, `_scope_versions["myplug@test"]["project"]`

**Outcome:** plugin appears in `_install_paths` (non-scoped fallback) AND in `_scope_install_paths["myplug@test"]["project"]`, but NOT in `_user_installed_ids`, NOT in `_project_installed_ids`, NOT in `_installed_scopes`. **It's "invisible" to scope-aware display logic but visible to fallback `_install_paths` lookup.**

This is intentional but subtle. The Rust port must preserve it: `_install_paths` is a non-scoped fallback, `_scope_install_paths` is the scope-aware view.

### Pathological case: scope claims "user" with project_path

Iteration with `scope="user"` and `project_path="/some/path"`:
- `:216-218`: adds to `_user_installed_ids`, scopes = `["user"]`
- The project_path is **ignored** (the conditional only checks project/local). This means malformed registry data with a project_path on a user-scoped plugin doesn't fail; it's just ignored.

### Pathological case: OSError on resolve

`:225-226`: silent `pass`. Effect: scope is NOT added to scopes list, NOT added to `_project_installed_ids`. Plugin disappears from scope-aware view. **Common cause:** broken symlink in project_path. Latent foot-gun.

### Pathological case: same plugin_id at multiple scopes

```python
registry.installed = {
  "myplug@test": [
    Installation(scope="user", install_path="/cache/myplug", version="1.0.0"),
    Installation(scope="project", install_path="/proj-a/.claude/plugins/myplug", version="1.1.0", project_path="/proj-a")
  ]
}
```

Walk-through (assuming `project_root = /proj-a`):
- `:209-211`: `_install_paths["myplug@test"] = Path("/cache/myplug")` — **first installation wins for non-scoped fallback**
- `_installed_versions["myplug@test"] = "1.0.0"` — first one's version
- Inner loop iteration 1 (user):
  - `_user_installed_ids.add(pid)`, scopes = `["user"]`
  - `_scope_install_paths["myplug@test"]["user"] = Path("/cache/myplug")`, `_scope_versions["myplug@test"]["user"] = "1.0.0"`
- Inner loop iteration 2 (project):
  - project_path resolves to project_root → `_project_installed_ids.add(pid)`, scopes = `["user", "project"]`
  - `_scope_install_paths["myplug@test"]["project"] = Path("/proj-a/.claude/plugins/myplug")`, `_scope_versions["myplug@test"]["project"] = "1.1.0"`
- `_installed_scopes["myplug@test"] = list(dict.fromkeys(["user", "project"])) = ["user", "project"]` (preserves order, dedups)

**Outcome:** scope-aware lookup correctly picks per-scope path/version. Non-scoped fallback uses the user-scope (because it was first). If display_scope toggles to "project", `_parse_plugin:136-141` uses `scope_paths.get("project")` returning the project install path. **Verified by reading `_parse_plugin:122-146`.**

### Display-scope toggle behavior

`_parse_plugin:123-128`:
```python
if self.display_scope == "project":
    scope_ids = self._project_installed_ids or set()
else:
    scope_ids = self._user_installed_ids or set()
is_installed = full_id in scope_ids
```

So toggling `display_scope` from "user" → "project" changes what's reported as "installed" for THIS view. A user-only plugin would show `is_installed=False` under display_scope="project". This is the **UI toggle** for "Show user / project" in the marketplace modal.

Combined with `:134-141`:
```python
if self.display_scope == "project":
    install_path = scope_paths.get("project") or scope_paths.get("local")
    installed_version = scope_vers.get("project") or scope_vers.get("local")
else:
    install_path = scope_paths.get("user")
    installed_version = scope_vers.get("user")
if not install_path:
    install_path = (self._install_paths or {}).get(full_id)
if not installed_version:
    installed_version = (self._installed_versions or {}).get(full_id)
```

**Three-level fallback for install_path:**
1. Scope-specific (project OR local, with project preferred)
2. Non-scoped first-installation fallback `_install_paths[full_id]`
3. None

**The display_scope="project" sees: project install if any, else local install if any, else first-installation fallback, else None.** This is the Rust port's required semantics.

## Gap 3: Plugin loader `_create_plugin_info` edge cases

`plugin_loader.py:274-327`.

### The latest-version fallback

`:294-296`:
```python
if not install_path.is_dir() and install_path.parent.is_dir():
    install_path = self._find_latest_version_dir(install_path.parent)
    version = install_path.name
```

**Triggers when:** registered install_path is stale (e.g., `<cache>/plugin/1.0.0`) but the actual directory has moved (e.g., to `1.1.0`). The fallback computes "latest version directory in the parent" and uses its **name** as the new version string.

### Edge case: parent doesn't exist

`install_path.parent.is_dir()` → False → fallback skipped. `install_path` remains the registered (non-existent) path. The downstream `get_all_plugins:128, 142, 154` check `plugin_info.install_path.is_dir()` and filter out non-dir entries. **Plugin silently disappears from the result.**

### Edge case: parent exists but is empty

Inside `_find_latest_version_dir` (`:329-341`):
```python
try:
    subdirs = [d for d in parent_dir.iterdir() if d.is_dir()]
    if subdirs:
        return max(subdirs, key=lambda d: self._parse_version(d.name))
except OSError:
    pass
return parent_dir
```

- Empty parent_dir: `subdirs = []`, no `max` call, returns `parent_dir` (which is_dir → True).
- Then back in `_create_plugin_info`: `install_path = parent_dir`, `version = parent_dir.name`. Plugin is created with parent dir as install path. **The plugin's downstream scans (`commands/*.md` etc.) would search inside parent_dir** — possibly finding nothing or wrong things.

### Edge case: mixed semver/string subdirs raises TypeError

```python
parent_dir = Path("/cache/myplug")
# Contains: ["1.0.0", "1.1.0", "latest"]
subdirs = [Path("1.0.0"), Path("1.1.0"), Path("latest")]
versions = [(1,0,0), (1,1,0), ("latest",)]
max(subdirs, key=...)  # raises TypeError on Python 3 when comparing (1,1,0) and ("latest",)
```

Caught by the outer `except OSError`? **NO** — TypeError is NOT caught. The exception propagates. **Where is `_find_latest_version_dir` called?**

- `plugin_loader.py:295` — inside `_create_plugin_info` — NOT in a try/except. TypeError propagates up.
- `plugin_loader.py:338` — same shape, only `OSError` caught.
- `marketplace_loader.py:272` — only `OSError` caught.

**Critical latent bug.** A user with a cache dir containing both numeric and non-numeric version subdirs would crash:
1. `load_marketplaces()` calls `_load_installed_plugins()` calls `plugin_loader.load_registry()` (safe — no `_find_latest_version_dir` call inside load_registry itself)
2. `get_all_plugins()` calls `_create_plugin_info` for each installation
3. If install_path is stale and parent has mixed versions, TypeError raises
4. Propagates up through `discover_all() → _discover_plugins() → _plugin_loader.get_all_plugins()`
5. TUI crashes during refresh

Severity: P1, latent. Trigger: rare but reproducible (e.g., manual cache surgery, or `claude` CLI version drift).

**Rust port should catch the equivalent error**: use a custom comparator that always returns `Ordering::Equal` between mixed-shape tuples, or pre-classify and prefer numeric tuples.

### Other edge cases

- `installation.install_path == ""` (empty string): `:287-288` returns None early. PluginInfo never created. **Safe.**
- `plugin_id without "@"`: `:290` → `short_name = plugin_id`. Treated as standalone plugin. **Safe.**
- `installation.project_path` is None vs empty string: `:314-316` `Path(installation.project_path)` if truthy, else None. Empty string is falsy. **Safe.**

## Gap 4: `_discover_marketplace_components` 6 branches

`discovery.py:253-302` (re-read).

### Branch matrix

The function reads 6 keys from `marketplace_plugin.extra_metadata` (which is everything in `marketplace.json[plugins][*]` minus `name/description/source`):

| Branch | Key | Type accepted | Dispatch |
|---|---|---|---|
| 1 | `commands` | str or list[str] | `_discover_md_files_from_paths(SlashCommandParser, ...)` |
| 2 | `agents` | str or list[str] | `_discover_md_files_from_paths(SubagentParser, ...)` |
| 3 | `skills` | str or list[str] | `_discover_custom_skills(...)` |
| 4 | `mcpServers` | str only (singular path) | `_discover_custom_mcps(...)` |
| 5 | `hooks` | str only (singular path) | `_discover_custom_hooks(...)` |
| 6 | (default) | n/a | (no marketplace-extras processing; fall back to discover_from_directory's standard plugin discovery) |

**Asymmetry:** branches 1-3 accept str or list (via `_normalize_paths`); branches 4-5 accept str only (`isinstance(value, str)` guard). LSP servers have NO marketplace-extras branch — they're discovered exclusively via `.lsp.json` and `plugin.json[lspServers]` standard paths.

### `_discover_md_files_from_paths` (used by branches 1-2)

`discovery.py:313-344`. For each path_str:
- `target = (plugin_dir / path_str).resolve()` — relative paths joined with plugin_dir; absolute paths take over
- If `target.is_file() AND target.suffix == ".md"`:
  - If not in seen_paths: parse, add to customizations, add to seen_paths
- Else if `target.is_dir()`:
  - For each `target.rglob("*.md")`: same dedup
- **NO `walk_filtered` — uses raw `rglob`.** So gitignore is NOT applied in marketplace-extras path. Skip-dirs also NOT applied. **Divergence from standard scan.**
- **NO max_depth — could explode on deep dirs.**

### `_discover_custom_skills` (branch 3)

`discovery.py:346-371`. Different from `_discover_md_files_from_paths`:
- Expects target to be a directory containing `SKILL.md` directly (not nested)
- Does NOT recurse — only looks at `target / "SKILL.md"`
- If file → parse
- If not → silently skipped

**No support for nested skills under a custom path.** A marketplace specifying `"skills": "my-skills"` where `my-skills/` contains multiple skill subdirs each with their own SKILL.md would only find the top-level SKILL.md (if any), not the children. **Constraint to document for Rust port.**

### `_discover_custom_mcps` (branch 4)

`discovery.py:373-392`. Reads a single MCP config file at `(plugin_dir / mcp_path).resolve()`. Uses `MCPParser().parse(..., ConfigLevel.PLUGIN)`. Attaches plugin_info if provided.

**No dedup with seen_paths** — the MCP discovery doesn't share the seen_paths set with other branches. This is a quirk: if a plugin has BOTH a standard `.mcp.json` AND a marketplace-extras `mcpServers` pointing to the same file, both branches would discover it. **In practice this doesn't happen** because marketplace-extras branch is in `discover_from_directory` (preview only) and standard `.mcp.json` discovery is in `_discover_plugins` (installed). They're never called together.

### `_discover_custom_hooks` (branch 5)

`discovery.py:394-413`. Same shape as MCP — reads a single hook file (assumed wrapped in `hooks` key? Need to verify).

Re-reading `HookParser.parse` (per Pass B-r1 parser tables): the parser expects the file's root JSON to have a `hooks` key. So marketplace-extras `hooks` MUST point to a settings.json-shaped file (with top-level `hooks`), NOT to a bare hooks.json (unwrapped).

**vs.** the standard plugin discovery at `_discover_plugin_hooks` (`discovery.py:684-699`) reads `<install_path>/hooks/hooks.json` which IS unwrapped (per Pass B-r1).

**Schema inconsistency** between marketplace-extras `hooks` path (wrapped) and standard plugin `hooks/hooks.json` (unwrapped). This is a known divergence; the Rust port must support both.

### Coverage gap restated

Zero tests for any of the 6 marketplace-extras branches. Zero fixtures with `marketplace.json` containing `commands`/`agents`/`skills`/`mcpServers`/`hooks` overrides. **Spec exists only in code.**

## Gap 5: Rust crate parity for gitignore_filter

`gitignore_filter.py:1-149`.

### Required semantics (from code-of-truth)

1. Hardcoded skip-dirs by name (20 entries)
2. Hardcoded default ignore-patterns (22 patterns)
3. Optional `.gitignore` file loading (comments stripped, blanks skipped)
4. `should_skip_dir(name)` — name-only lookup, fast path
5. `is_ignored(path)` — pathspec match against absolute or project-relative path
6. `is_dir_ignored(dir_path)` — same but appends `/` to match dir patterns like `temp/`
7. `walk_filtered(root, pattern, max_depth)` — `os.walk` with in-place dirnames pruning, fnmatch on filenames, post-yield is_ignored filter

### Rust crate options

| Crate | Fit | Notes |
|---|---|---|
| `ignore` (rust-analyzer's, BurntSushi) | **HIGH** | Provides `WalkBuilder` with `.gitignore` + `.ignore` + custom overrides + standard filters. Default skip-dirs differ from this codebase's set (ripgrep's defaults exclude binary files but DO walk `node_modules`). Need to apply custom skip-dirs as override. |
| `walkdir` + manual `gitignore` (separate crate) | MEDIUM | More control; replicate the exact two-list semantics. More code. |
| `globset` (alone) | LOW | Only pattern matching, no walker. |
| `pathspec` (Python crate equivalent) | NONE | No Rust equivalent; pathspec is Python-only. |

### Recommendation

**Use `ignore` crate with custom `OverrideBuilder`** to inject the lazyclaude DEFAULT_SKIP_DIRS + DEFAULT_IGNORE_PATTERNS as global overrides, then layer `.gitignore` automatically. The `ignore` crate already handles:
- `.gitignore` parsing (including comments, blanks, negation)
- Directory pruning via `WalkBuilder::add_custom_ignore_filename`
- `max_depth` via `WalkBuilder::max_depth`
- Case-sensitivity (configurable; default platform-aware)

**Gap:** the lazyclaude code's `DEFAULT_IGNORE_PATTERNS` includes `*.egg-info/` which is gitignore-syntax (anchored to root or relative). The `ignore` crate uses standard gitignore semantics so this should Just Work.

**Risk:** the **`fnmatch.fnmatch` case-sensitivity quirk** (case-insensitive on Windows, case-sensitive on Unix) is platform-dependent in Python. The `ignore` crate's behavior is **platform-independent** (case-sensitive everywhere by default). If lazyclaude has files like `Commands/foo.md` (capital C), they'd match `*.md` on Windows but not Unix. Monocle port should pick a single behavior and document it.

### What's NOT covered by `ignore` crate

- **Hardcoded skip-dir name match without pattern** — `ignore`'s overrides work on patterns. To replicate `should_skip_dir(name)`, can add `WalkBuilder::filter_entry(|e| !DEFAULT_SKIP_DIRS.contains(e.file_name().to_str().unwrap_or("")))`.
- **The dual nature of "ignore" pattern (defaults + .gitignore)** — `ignore` separates these; can use `OverrideBuilder` for defaults and let WalkBuilder pick up `.gitignore` automatically.

## Cross-cutting: TypeError fix for `_parse_version`

Two locations: `plugin_loader.py:343-353` and `marketplace_loader.py:277-283`. Both return `tuple[int, ...] | tuple[str]`.

### Proposed unified version comparator

For Rust port: use `semver::Version::parse` (returns `Result`); pre-classify into Ok-semver list and Err-string list; sort each list separately; concatenate Ok-list (newest first) before Err-list. Eliminates the TypeError risk.

For a verbatim Python port: wrap `max()` in try/except TypeError, fall back to alphabetic comparison on all subdirs.

## Test coverage gaps re-evaluated

| Surface | Status after R2 |
|---|---|
| `FilterService` | Truth table enumerated (r2); pure functions easy to test; **add tests in monocle port** |
| `MarketplaceLoader._load_installed_plugins` | Set-algebra walked through with fixtures (r2); coverage gap real; **add direct unit tests in monocle port** |
| `PluginLoader._create_plugin_info` | Latest-version fallback documented; **TypeError risk surfaces as new finding** |
| `discovery._discover_marketplace_components` | 6 branches mapped; **0% test coverage confirmed**; **add fixtures with marketplace-extras for monocle** |
| `Opener` | Platform dispatch + URL construction documented (r1); **add platform-mocked tests in monocle port** |
| `Settings` migration / `ensure_suggested_marketplaces` | Confirmed gap; **add tests** |
| `Writer.toggle_plugin_enabled` | Confirmed gap; **add tests** |

## New P1 finding from R2

**P1-R2-1: `_find_latest_version_dir` TypeError on mixed-shape parent.** Two call sites in `_create_plugin_info` (`plugin_loader.py:295`) and `_parse_plugin` via `get_plugin_source_dir` (`marketplace_loader.py:272`). Untested. Triggers TUI crash. Rust port should consolidate via `semver` crate with pre-classification.

## Updated Monocle implication notes

1. **Filter service tests are mandatory** — pure logic, no tests in reference, easy to miss-port. Use the truth table in this round as test cases.
2. **MarketplaceLoader `_load_installed_plugins` must be tested directly** — the set-algebra is the trickiest pure function. Use the concrete walkthrough in this round as a test scenario.
3. **TypeError on mixed-shape version dirs is a real risk** — replace with `semver` crate + pre-classification. Do NOT port the Python `_parse_version` verbatim.
4. **Marketplace-extras paths bypass gitignore** — by design. Rust port should preserve this divergence (raw `rglob`-style traversal, no `.gitignore` consultation).
5. **Schema asymmetry between standard hooks (`hooks/hooks.json` unwrapped) and marketplace-extras hooks (settings.json-shaped, wrapped)** — already in Pass 8 but worth restating: this is the most counter-intuitive part of the on-disk schema.

## Delta Summary

- New items added: full truth table for FilterService (12 query combinations); 6-step walkthrough of MarketplaceLoader set-algebra with reference fixtures; 4 pathological-case edge cases for set-algebra; 1 new P1 (TypeError on mixed-shape version dirs); 6-branch matrix for marketplace-extras discovery; Rust crate parity analysis for gitignore_filter
- Existing items refined: `_discover_md_files_from_paths` confirmed to BYPASS gitignore (vs standard scan which uses `walk_filtered`); `_discover_custom_skills` doesn't recurse; PROJECT level filter is the only "broader" level; opener has no shell injection risk because args is a list
- Remaining gaps: writer crash-safety still untested (out of scope — requires fault injection); filter service still has zero tests in reference (recommended action item, not a model change)

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: R2 produced a **new P1 finding** (TypeError on mixed-shape version dirs — latent crash bug) that's a real risk to monocle's Rust port. It also produced **concrete walkthroughs** that turn pure-code semantics into testable scenarios for the Rust port — these change how a Rust developer would write tests (not just "test it" but "test these specific 12 inputs"). The 6-branch marketplace-extras matrix surfaces a previously under-documented schema asymmetry between wrapped hooks (extras) and unwrapped hooks (standard plugin). A Rust developer's plan WOULD change: they would write a specific unit test for the version-sort TypeError, write a 12-case truth-table test for FilterService, and write 6 separate fixtures for marketplace-extras coverage.

## Convergence Declaration

**Another round MAY be needed.** Honest assessment: the remaining concerns are getting narrower (specific edge cases in pure functions). I'm on the boundary. Let me list what r3 would do:

- Re-verify the `_discover_local_mcps` (`:588-620`) error paths and confirm dedup with project-level `.mcp.json` (a server defined in BOTH user-local and project file)
- Walk through `discover_from_directory` end-to-end with a hypothetical preview scenario
- Map writer's race conditions explicitly (between conflict check and write)
- Check whether `os.walk` skip-dir filtering is shared correctly with the `is_dir_ignored` check (overlap or duplication?)

These are narrowing toward nitpicks — the next round should be either a focused r3 confirming honest NITPICK, OR an explicit NITPICK call now. I'll do one more round to be sure.

## State Checkpoint

```yaml
pass: B
subpass: services
round: 2
status: complete
files_re_examined: 4
new_p1_findings: 1  # TypeError on mixed-shape version dirs
walkthroughs_added: 3  # filter truth table, set-algebra, marketplace-extras 6-branch
timestamp: 2026-05-11T20:15:00Z
novelty: SUBSTANTIVE
converged: false
```
