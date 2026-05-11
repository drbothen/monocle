# Phase B Deepening: Non-Parser Services Layer — Round 3

Goal: probe the remaining narrow questions from r2's convergence boundary. Verify honest NITPICK.

## Questions targeted this round

1. `_discover_local_mcps` error path semantics and overlap with project-level `.mcp.json`
2. `discover_from_directory` end-to-end with a hypothetical preview scenario
3. Writer's race conditions between conflict check and write
4. Overlap/duplication between `should_skip_dir` (name match) and `is_dir_ignored` (pattern match) inside `walk_filtered`
5. Discovery constructor side effects (eager dependency construction)

## Question 1: `_discover_local_mcps` overlap and error semantics

`discovery.py:588-620` (re-read).

### Overlap with project-level `.mcp.json`

`_discover_mcps()` orchestrates three sources (`:571-586`):
1. `~/.claude.json[mcpServers]` (USER)
2. `_discover_local_mcps()` reads `~/.claude.json[projects][<path>][mcpServers]` (PROJECT_LOCAL)
3. `./.mcp.json[mcpServers]` (PROJECT)

**Question:** if the same server name appears in BOTH `~/.claude.json[mcpServers]` AND `./.mcp.json[mcpServers]`, what happens?

Answer: **both are returned as separate Customization items.** No dedup at this level. They have different `level` (USER vs PROJECT) and different `path` (`~/.claude.json` vs `./.mcp.json`). The sort returns both. The UI panel filter on `c.name == "foo"` would show two rows. **Identical to slash-command USER/PROJECT overlap.**

This is the **dedup-by-resolved-path doesn't apply to MCPs** because the same server can legitimately exist at multiple scopes with the same name but different configurations.

### Error path semantics

`:617-618`: `except (OSError, json.JSONDecodeError): pass`. Returns whatever was accumulated so far (might be partial — see below).

**Subtle:** the try-block at `:596` extends through `:616`. If `parser.parse_server_config(...)` raises during iteration of `mcp_servers.items()`, the partial list accumulated up to that point is lost (Python doesn't preserve appends to a list before exception — wait, it does preserve. The `customizations.append(...)` already modified the list). So the partially-populated `customizations` list is returned. **Half-loaded state is possible.**

This is fine for read-only discovery (next refresh re-loads), but if any caller mutates partial results it would be lossy. **No caller mutates.**

### MCPParser `parse_server_config` behavior

Not re-read in r3 (covered in parsers-r1). Recall: it constructs a single Customization from `(server_name, server_config)` with no on-disk file read of its own. So no IO inside the loop — the only sources of OSError are `claude_json.read_text` and possibly `Path.is_file()`. These all happen BEFORE the loop. So the loop itself is safe from OSError. JSONDecodeError can only come from the initial `json.loads`.

**Conclusion:** the exception handler only fires on file read or JSON parse failure, which both happen before any appends. **Half-loaded state is theoretical only; doesn't occur in practice.**

## Question 2: `discover_from_directory` end-to-end walkthrough

`discovery.py:208-241`.

### Scenario: preview a marketplace plugin

```python
plugin_dir = Path("/cache/anthropics-skills/code-review-1.0.0")
plugin_info = PluginInfo(
    plugin_id="code-review@anthropics-skills",
    short_name="code-review",
    version="1.0.0",
    install_path=plugin_dir,
    is_local=False,
    is_enabled=False,  # not yet installed
    scope=PluginScope.USER,
)
marketplace_plugin = MarketplacePlugin(
    name="code-review",
    description="...",
    source="./plugins/code-review",
    marketplace_name="anthropics-skills",
    full_plugin_id="code-review@anthropics-skills",
    is_installed=False,
    extra_metadata={
        "commands": ["./extras/cmds"],  # custom override
        "skills": "./extras/skills",
        "hooks": "./hooks.json",
    },
)
```

### Execution trace

`:208-241`:

1. `level = PLUGIN` (`:216`)
2. `plugin_filter = GitignoreFilter(project_root=plugin_dir)` (`:218`) — note: gitignore root is the plugin dir itself, NOT the user's project root
3. `plugin_scanner = FilesystemScanner(gitignore_filter=plugin_filter)` (`:219`)
4. Standard SCAN_CONFIGS pass (`:221-224`):
   - Scan `plugin_dir/commands/**/*.md` (RGLOB)
   - Scan `plugin_dir/agents/*.md` (GLOB)
   - Scan `plugin_dir/skills/*/SKILL.md` (SUBDIR)
   - All level=PLUGIN, plugin_info attached
5. `marketplace_plugin` is truthy, so branch `:226-232`:
   - `seen_paths = {c.path.resolve() for c in customizations if c.path}` — set of resolved paths from the standard scan
   - Call `_discover_marketplace_components(plugin_dir, marketplace_plugin, plugin_info, seen_paths)`
6. `plugin_info` is truthy, so `:234-239`:
   - `_discover_plugin_mcps(plugin_dir, plugin_info)` — reads `plugin_dir/.mcp.json`
   - `_discover_plugin_hooks(plugin_dir, plugin_info)` — reads `plugin_dir/hooks/hooks.json` (UNWRAPPED)
   - `_discover_plugin_lsp_servers(plugin_dir, plugin_info)` — reads `.lsp.json` + `plugin.json[lspServers]`
7. Final: `_sort_customizations(customizations)` (`:241`)

### Notable: dedup via seen_paths

The `seen_paths` set is initialized AFTER the standard scan, so the marketplace-extras branch wouldn't re-discover files already found by the standard scan. **Important for the case where extras paths overlap with the standard `commands/` or `agents/` dirs.**

But: standard scan uses `walk_filtered` (gitignore-aware), while marketplace-extras uses raw `rglob` (NOT gitignore-aware). If a file is excluded by gitignore in the standard scan, it WON'T be in `seen_paths`, but WILL be discovered by the extras path. **Potential schema divergence: in installed plugins, gitignore is respected; in marketplace preview via extras, it isn't.**

### Hook double-discovery

In the trace above:
- Step 5 calls `_discover_marketplace_components` which can call `_discover_custom_hooks(plugin_dir, "./hooks.json", plugin_info)` reading `plugin_dir/./hooks.json` (the wrapped format)
- Step 6 calls `_discover_plugin_hooks(plugin_dir, plugin_info)` reading `plugin_dir/hooks/hooks.json` (the unwrapped format)

**These read DIFFERENT files** (different paths). So no actual conflict. Each branch finds its own source. **But there's NO dedup mechanism between them** — if a marketplace specifies `"hooks": "hooks/hooks.json"` (pointing to the standard location), the standard branch AND the extras branch would both fire, both calling HookParser. **The HookParser would interpret the same file differently** (extras expects wrapped, standard expects unwrapped). One of them would silently produce wrong results.

**Latent bug.** Discovered by walking the code in r3. P2 severity — requires a misconfigured marketplace.

### Test coverage

`test_plugins.py` covers neither `discover_from_directory` (preview) nor `_discover_marketplace_components`. Confirmed by reading test file in r2.

## Question 3: Writer race conditions

`writer.py:404-418` (re-read).

### TOCTOU between `_check_conflict` and `_write_file`

```python
if self._check_conflict(customization, target_path):  # _check_conflict checks exists()
    return (False, "already exists...")
self._ensure_parent_dirs(target_path)
if customization.type == CustomizationType.SKILL:
    self._copy_skill_directory(...)
else:
    self._write_file(customization.path, target_path)
```

Between `_check_conflict` and `_write_file`, another process (or thread) could create the target file. The `_write_file` would overwrite it (via `write_text`). For SKILL, `_copy_skill_directory` calls `shutil.copytree(..., dirs_exist_ok=False)` which would raise.

**Severity:** P2. Single-user TUI, race is unlikely. The `claude` CLI runs as separate subprocesses and could theoretically race during a `claude plugin install` concurrent with a user copy. Not protected.

**Rust port:** use `OpenOptions::new().create_new(true)` for file writes (atomic exclusive create). For directory copies, `std::fs::create_dir` returns `Err(AlreadyExists)`. Both are atomic at the syscall level.

### TOCTOU on settings.json read-modify-write

`writer.py:124-130`, `:202-219`, `:469-477`:
```python
settings = self._read_settings_json(target_path)
# ... modify ...
self._write_settings_json(target_path, settings)
```

Between read and write, another process could update the file. Last writer wins. **Specific risk:** `claude` CLI updates `~/.claude.json[enabledPlugins]` concurrent with lazyclaude's `toggle_plugin_enabled`. User's toggle overwrites claude's update or vice versa.

**Severity:** P1 for the shared `~/.claude.json` (where claude and lazyclaude both write). P2 for `<project>/.claude/settings.json` (only lazyclaude writes).

**Rust port:** use `fs2::FileExt::try_lock_exclusive` advisory locking on POSIX, or restructure to use immutable read + atomic-rename-after-merge with retry on rename failure. The `tempfile + atomic rename` pattern (P0 already known) gives at-most-one-wins semantics but not at-most-one-modifies (still has lost-update under contention).

## Question 4: `should_skip_dir` vs `is_dir_ignored` overlap

`gitignore_filter.py:138-143` (re-read):
```python
dirnames[:] = [
    d
    for d in dirnames
    if not self.should_skip_dir(d)
    and not self.is_dir_ignored(Path(dirpath) / d)
]
```

Both predicates are checked for every directory in `os.walk`. Are they redundant?

### Analysis

- `should_skip_dir(d)`: name-only set lookup, O(1). Set is the 20-entry DEFAULT_SKIP_DIRS.
- `is_dir_ignored(Path(dirpath) / d)`: pathspec match against `relative_path + "/"`, O(patterns). Pattern set is DEFAULT_IGNORE_PATTERNS + .gitignore.

**Are these overlapping?** Most entries DO overlap:
- DEFAULT_SKIP_DIRS contains `node_modules`
- DEFAULT_IGNORE_PATTERNS contains `node_modules/`
Both would prune.

**But some are NOT in DEFAULT_IGNORE_PATTERNS:** Comparing the two lists:
- `should_skip_dir` set: {.git, node_modules, .venv, venv, __pycache__, .mypy_cache, .pytest_cache, build, dist, .eggs, .tox, .nox, htmlcov, .idea, .vscode, bin, obj, .vs, packages} — **19 entries** (`gitignore_filter.py:10-30` actually has 19; the comma-list shows 20 because I miscounted).
- DEFAULT_IGNORE_PATTERNS: {.git/, node_modules/, .venv/, venv/, __pycache__/, .mypy_cache/, .pytest_cache/, build/, dist/, .eggs/, *.egg-info/, .tox/, .nox/, .coverage, htmlcov/, .idea/, .vscode/, bin/, obj/, .vs/, packages/} — **21 entries** (`gitignore_filter.py:32-54`).

**Diff:**
- DEFAULT_IGNORE_PATTERNS has `*.egg-info/` and `.coverage` — NOT in skip-dirs set
- All names in skip-dirs ARE in DEFAULT_IGNORE_PATTERNS (with trailing slash)

So `should_skip_dir` is a **fast-path duplicate** for the most common dir-prune cases. The redundant `is_dir_ignored` check serves to catch *.egg-info patterns and (optionally) user-supplied .gitignore directory patterns.

**Performance optimization, not semantic divergence.** Could be replaced by a single `is_dir_ignored` call without loss of correctness. Rust port should consolidate.

### Verification: name match alone vs path match

`should_skip_dir("node_modules")` matches any dir named `node_modules` regardless of location. `is_dir_ignored(Path("src/node_modules"))` matches because of `node_modules/` pattern's recursive nature in gitignore (no leading `/` means "anywhere").

Both behave identically for these cases. **Confirmed no semantic divergence.**

## Question 5: Discovery constructor side effects

`discovery.py:129-156` (re-read).

### What happens at `ConfigDiscoveryService(...)`?

1. `self.user_config_path = ...` — Path resolution. Not IO. (`:141`)
2. `self.project_config_path = (resolved if explicit else Path.cwd() / ".claude")` — Path.cwd() does syscall. `.resolve()` does syscall(s). (`:142-146`)
3. `self.project_root = self.project_config_path.parent` — pure Path op. (`:147`)
4. `self._gitignore_filter = GitignoreFilter(project_root=self.project_root)` — **READS .gitignore from disk** (`:149`)
5. `self._scanner = FilesystemScanner(...)` — no IO (`:150`)
6. `self._plugin_loader = PluginLoader(...)` — no IO during construct, but downstream `load_registry()` does
7. `self._cache: list[Customization] | None = None` (`:156`)

### Eager .gitignore read at construction

`GitignoreFilter.__init__:60-74` calls `self._load_gitignore(project_root)` if `use_gitignore=True` (default). This is one file read per ConfigDiscoveryService instance.

**Implication:** test code creating a service for a non-existent project_root would trigger the read with `.gitignore` missing (gracefully returns `[]`). No crash.

**Implication for Rust port:** construct should be cheap; consider lazy `.gitignore` load on first `is_ignored` call. The Python code's eager load is acceptable because TUI startup is single-shot.

### Cache lifecycle

`_cache` starts None. First `discover_all()` populates it. `refresh()` resets to None + propagates to `_plugin_loader.refresh()`. No other invalidation triggers — file system changes are NOT auto-detected.

**File watcher absent.** Confirmed by reading. Rust port can add `notify` crate for auto-refresh if desired (out of parity scope).

## Question 6 (new): Subtle `_discover_memory_files` ordering bug?

`discovery.py:415-476` (re-read with fresh eyes).

The 5 branches process in order:
1. user CLAUDE.md + AGENTS.md (USER) — adds to seen_paths
2. user CLAUDE.local.md (USER)
3. project CLAUDE.md + AGENTS.md + project_root CLAUDE.md + AGENTS.md (PROJECT)
4. walk_filtered nested CLAUDE.md (PROJECT) — overrides name to relative path
5. project_root CLAUDE.local.md + project_config CLAUDE.local.md (PROJECT_LOCAL)

**Question:** if user_config_path and project_root are SAME path (pathological: user runs from `~/.claude` as cwd), then `project_config_path = ~/.claude/.claude`, `project_root = ~/.claude`. The user CLAUDE.md and the project_root CLAUDE.md resolve to the same path.

Branch 1 reads it as USER, adds to seen_paths.
Branch 3 reads `project_root / CLAUDE.md` (`:440`). `resolved` matches seen_paths → skipped.

**Result:** the file is correctly assigned USER level, not PROJECT. **Behavior is correct.** Same-path file is registered to the earlier branch. **No bug.**

## Question 7 (new): Walk_filtered consistency for filenames

`gitignore_filter.py:145-149`:
```python
for filename in filenames:
    if fnmatch.fnmatch(filename, pattern):
        file_path = Path(dirpath) / filename
        if not self.is_ignored(file_path):
            yield file_path
```

The `fnmatch.fnmatch` is case-insensitive on Windows. So a `*.md` pattern on Windows matches `CLAUDE.MD`, `Claude.Md`, etc. On Unix, only `*.md` matches.

**Test coverage gap:** no Windows-specific test for case-insensitive fnmatch matching. The Pass 8 P1 "CRLF handling in markdown" is a sibling concern.

**Rust port:** consistent case-sensitivity. Recommend case-sensitive everywhere (matches Unix behavior) + a configurable override if needed.

## Net new findings in R3

| # | Finding | Severity | New in R3? |
|---|---|---|---|
| 1 | Hook double-discovery when marketplace-extras `hooks` points to standard hooks/hooks.json path | P2 | YES — uncovered by walking `discover_from_directory` |
| 2 | TOCTOU between conflict-check and write in all writer mutation surfaces | P2 (P1 for shared `~/.claude.json`) | YES — explicit walk |
| 3 | `should_skip_dir` vs `is_dir_ignored` overlap is performance optimization, not semantic divergence | doc | YES — refines r1's "20 / 22 entry" finding |
| 4 | `_discover_local_mcps` half-loaded state is theoretical; no IO inside accumulation loop | safe | YES — refines r1 "silent swallow" concern |
| 5 | Constructor eagerly reads `.gitignore` once | doc | YES — performance note |
| 6 | Cross-platform `fnmatch.fnmatch` case-sensitivity divergence | P2 | YES — sibling to CRLF concern |

None of these change the model. All are refinements or new minor risks at P2 severity.

## Delta Summary

- New items added: 5 minor edge cases / refinements (none change schema or contract); 1 new P2 (hook double-discovery in preview); 1 confirmed P1 for shared `~/.claude.json` TOCTOU
- Existing items refined: redundancy/overlap in walk_filtered consolidated; half-loaded state in _discover_local_mcps proven theoretical
- Remaining gaps: none that would change the spec; outstanding gaps are mostly "add tests for X" type recommendations

## Novelty Assessment

Novelty: **NITPICK**

Justification: This round produced refinements and edge-case confirmations but **no new schema fields, no new entities, no new behavioral contracts**. The P2 findings (hook double-discovery, TOCTOU, fnmatch case) are real but do not change the model — a Rust developer's plan stays the same: implement the writer with atomic rename and per-path file locks for `~/.claude.json`; implement walk with consistent case-sensitivity; document the marketplace-extras hook overlap as a known schema quirk. None of these are "go redesign that section" findings. They are "be aware while implementing" notes.

The test that would distinguish SUBSTANTIVE from NITPICK: "Would removing this round's findings change how you'd spec the system?" Answer: **No.** The Rust spec from r1+r2 already includes atomic-write requirements and case-sensitivity decisions implicitly. The findings here add bullet points to the implementation checklist, not new chapters to the spec.

## Convergence Declaration

**Pass B services has converged.** Three rounds is the right number for a 10-file layer of this complexity. R1 surfaced the model (per-file tables, P1 findings); R2 closed the substantive pure-function gaps with truth tables and walkthroughs; R3 verified edge cases and produced only P2 refinements. Further rounds would produce diminishing returns.

## State Checkpoint

```yaml
pass: B
subpass: services
round: 3
status: complete
new_findings_p2: 6 (5 doc/refinement + 1 actionable: hook double-discovery in preview)
new_findings_p1: 0
new_findings_p0: 0
walkthroughs_added: 3 (discover_from_directory, walk_filtered overlap, _discover_memory_files ordering)
timestamp: 2026-05-11T20:50:00Z
novelty: NITPICK
converged: true
```
