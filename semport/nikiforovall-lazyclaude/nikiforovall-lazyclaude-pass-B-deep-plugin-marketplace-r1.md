# Phase B Deepening: Plugin Loader & Marketplace — Round 1

Goal: nail down the scope-algebra in `PluginLoader.get_all_plugins` and `MarketplaceLoader._load_installed_plugins` against test evidence.

## What the tests confirm

From `tests/unit/test_plugin_source_path.py:1-301` (read in full):

- **Directory-source resolution works as designed** — `test_directory_source_resolves_to_plugin_path`. Confirms the marketplace.json → source-path translation.
- **Non-directory (e.g., git) source falls back to install_path** — `test_non_directory_source_returns_install_path:67-110`.
- **Missing `marketplace.json` returns marketplace root** — `test_missing_marketplace_json_returns_marketplace_root`.
- **Plugin not found in marketplace.json's `plugins` array returns marketplace root** (not None) — `test_plugin_not_in_marketplace_json_returns_root:155-204`. Important: lack of registration is NOT an error.
- **Plugin ID without `@` returns install_path** — `test_plugin_without_marketplace_returns_install_path:206-240`. Standalone (non-marketplace) plugins supported.
- **Unknown plugin → None** — `test_unknown_plugin_returns_none:242-260`.
- **Malformed marketplace.json → marketplace root (silent recovery)** — `test_malformed_marketplace_json_returns_root:262-301`.

This is well-tested. Now let me examine what is NOT tested.

## PluginLoader `get_all_plugins` three-phase enumeration

Re-reading `plugin_loader.py:108-157`:

### Phase 1 (User-scoped)
```python
for plugin_id, installations in registry.installed.items():
    for installation in installations:
        if installation.scope == "user":
            plugin_info = self._create_plugin_info(plugin_id, installation, "user")
            if plugin_info and plugin_info.install_path.is_dir():
                plugins.append(plugin_info)
```

Iteration order: dict order (insertion order in 3.11+, governed by `installed_plugins.json` ordering). Multiple user-scoped installations of same plugin_id → multiple entries appended. **Plausible duplicate-entry risk** if the registry has unintentional duplicates.

### Phase 2 (Project-scoped)
```python
for plugin_id in registry.project_enabled:           # keys of project settings.json enabledPlugins
    installations = registry.installed.get(plugin_id, [])
    for installation in installations:
        if installation.scope == "project" and self._matches_current_project(installation.project_path):
            ...
```

Triggered by **keys** in project's enabledPlugins (regardless of value true/false). So a project's settings.json with `{"enabledPlugins": {"foo": false}}` will still enumerate "foo" — and `_create_plugin_info` will set `is_enabled=False`. **Disabled-by-project plugins are visible but flagged.**

`_matches_current_project` (`plugin_loader.py:159-166`): `Path(installation.project_path).resolve() == self.project_root.resolve()`. If the project root contains symlinks, `resolve()` normalizes both sides — consistent. If `project_path` is missing or empty → False (skipped).

### Phase 3 (Local-scoped)
Same as Phase 2 but driven by `settings.local.json.enabledPlugins` and `scope == "local"`.

### Edge cases NOT explicitly tested

- **Same plugin_id with both `scope="user"` and `scope="project"` installations** in `installed_plugins.json` → both appear in `get_all_plugins` output. Each goes through `_create_plugin_info` independently. Result: two `PluginInfo` objects with the same `plugin_id`, different `install_path`, `version`, `scope`. Consumers must dedupe if undesired.
- **A plugin_id listed in `enabledPlugins` but with NO matching installation** → silently skipped (`installations = registry.installed.get(plugin_id, [])` returns `[]`, inner loop no-ops). Operator gets no warning. **Easy debugging trap.**

## MarketplaceLoader scope set algebra (Seed 6)

Re-reading `marketplace_loader.py:167-238`:

```python
enabled_in_user = {pid for pid, enabled in registry.user_enabled.items() if enabled}
enabled_in_project = {pid for pid, enabled in registry.project_enabled.items() if enabled}
enabled_in_local = {pid for pid, enabled in registry.local_enabled.items() if enabled}

self._enabled_plugin_ids = (
    (self._installed_plugin_ids - {
        pid for pid, enabled in {
            **registry.user_enabled,
            **registry.project_enabled,
            **registry.local_enabled,
        }.items() if not enabled
    })
    | enabled_in_user
    | enabled_in_project
    | enabled_in_local
)
```

Let me trace concrete scenarios:

| Plugin state | user_enabled["X"] | project_enabled["X"] | local_enabled["X"] | Installed? | Effective `is_enabled` |
|---|---|---|---|---|---|
| Default install | (absent) | (absent) | (absent) | yes | **YES** — installed but no disable flag, so in `installed - disabled_subset`, not in any positive set. Set membership: in `installed - {}`. Verdict: enabled. |
| User explicitly enabled | True | (absent) | (absent) | yes | YES (in enabled_in_user) |
| User explicitly disabled | False | (absent) | (absent) | yes | **NO** — included in disabled_subset, removed from installed; not in any positive set. Verdict: disabled. |
| User disabled + Project enabled | False | True | (absent) | yes | **YES** — in `installed - disabled` removes X (because dict-merge: project_enabled["X"]=True overrides... wait) |

Wait, the dict merge:
```python
{**user_enabled, **project_enabled, **local_enabled}
```
Later dicts win. So if `user_enabled["X"] = False` and `project_enabled["X"] = True`, the merged dict has `"X": True`. The `if not enabled` filter excludes X from disabled_subset. So X stays in `installed_plugin_ids - disabled_subset`. Then `enabled_in_project` also includes X. Net: X is in the result set, **enabled**.

| Plugin state | user_enabled["X"] | project_enabled["X"] | local_enabled["X"] | Verdict |
|---|---|---|---|---|
| User True, Project False | True | False | — | merged["X"]=False (project wins). disabled_subset includes X. installed - disabled removes X. But enabled_in_user includes X → final union includes X. **Enabled.** |
| User False, Project True | False | True | — | merged["X"]=True. disabled_subset excludes X. installed - disabled keeps X. enabled_in_project also includes X. **Enabled.** |
| User True, Project True | True | True | — | merged["X"]=True. Kept everywhere. **Enabled.** |
| User False, Project False | False | False | — | merged["X"]=False. disabled_subset includes X. installed - disabled removes X. No positive sets. **Disabled.** |

So the semantics work out to: **"enabled in ANY scope OR installed and not explicitly disabled in ALL of {user, project, local}"** — but the dict-merge means "explicitly disabled" actually means "the LAST scope to mention it has it disabled". This is **a quirky interaction**: if user disables X and a subsequent project re-enables it, `_enabled_plugin_ids` says enabled. If user enables X and project disables, `enabled_in_user` still wins via the union. So the union arm dominates.

**Net empirical rule:** `_enabled_plugin_ids` ≈ "explicitly enabled anywhere ∪ (installed minus the merged-shadowed-disabled-set)". Last-wins shadowing means the disabled-subtraction is fragile against same-key disagreements; but it doesn't matter because the positive unions cover the case.

**Verdict on Seed 6:** the algebra is correct in net effect (`OR`-of-explicit-enable + installed-and-not-explicitly-disabled), but **the path is more complex than it needs to be**. A Rust port should simplify to:

```rust
enabled = installed
    && (
        user_enabled.get(pid).copied().unwrap_or(true)
        && project_enabled.get(pid).copied().unwrap_or(true)
        && local_enabled.get(pid).copied().unwrap_or(true)
        || user_enabled.get(pid) == Some(&true)
        || project_enabled.get(pid) == Some(&true)
        || local_enabled.get(pid) == Some(&true)
    )
```

…or just: **enabled iff explicitly enabled in any scope, OR not explicitly disabled in any scope.**

## Marketplace tracking dual-axis (installed scope vs display scope)

`MarketplaceLoader.display_scope` (`marketplace_loader.py:35`, mutated by `MarketplaceModal.action_toggle_scope_view` `widgets/marketplace_modal.py:687-697`) determines which installation set drives the "is_installed" view in the UI:

- `display_scope == "user"` → `is_installed` reflects user-scope installation
- `display_scope == "project"` → `is_installed` reflects project-or-local-scope installation matching current project_root

This is the "Scope view" toggle (`s` key). It allows the user to see what's installed at user level vs at project level. **Plugins installed only at project level WILL NOT show as installed in the user view** — they'd appear as uninstalled even though they exist on disk. This is intentional separation but possibly surprising. P2 UX consideration.

## Plugin install path resolution fallback chain

`MarketplaceLoader._parse_plugin` (`marketplace_loader.py:134-146`):
1. Try `_scope_install_paths[full_id][display_scope]` (or local fallback for project view).
2. Fall back to `_install_paths[full_id]` (the FIRST installation regardless of scope).
3. If still nothing → None.

**Concrete bug:** the first-installation-regardless-of-scope fallback can return a project-scope path when the display scope is user. UI will show user-mode "installed" for something only project-installed. Confirmed by reading flow — no test catches it.

## Marketplace tree label rendering

`marketplace_modal.py:453-488` `_render_plugin_label`:

Status icon decision:
- `is_installed AND is_enabled` → `[green]I[/]`
- `is_installed AND not is_enabled` → `[yellow]D[/]`
- `not is_installed AND has installed_scopes` (i.e., installed in a non-display scope) → `[dim]I[/]` ("ghost installed" — visible in this view only via scope badge)
- else → `[ ]`

Scope badge: a `[UPL]` style abbreviation showing which scopes the plugin is installed in (`U`=user, `P`=project, `L`=local). Hidden when scope == user (because user-installed is the implicit default). **Visual cue that this plugin lives in multiple scopes.**

Version display: `(1.2.3)` for installed; `(1.2.3 → 1.3.0) ↑` if `marketplace_plugin.extra_metadata["version"]` (from marketplace.json) is a higher semver than the installed version.

Semver detection (`marketplace_modal.py:414-423`): requires all `.`-split parts to be digits AND at least 2 parts. So `1.0` is semver; `1` is not (single part); `1.0-rc` is not (non-digit part). **Mixed-format plugins won't show update arrows.**

## Plugin lifecycle command path

`mixins/marketplace.py:248-280`:

```python
@work(thread=True)
def _run_plugin_command(self, cmd: list[str], success_msg: str) -> None:
    try:
        cwd = str(self._discovery_service.project_root)
        subprocess.run(cmd, capture_output=True, check=True, shell=True,
                       encoding="utf-8", errors="replace", cwd=cwd)
        self.call_from_thread(self._on_plugin_command_success, success_msg)
    except subprocess.CalledProcessError as e:
        error_msg = f"Failed: {e.stderr or e}"
        self.call_from_thread(self._on_plugin_command_error, error_msg)
    except FileNotFoundError:
        self.call_from_thread(self._on_plugin_command_error, "Claude CLI not found")
```

Key observations:
- **`cwd=project_root`** — the `claude` CLI must be invoked from within the project so it picks up the right project context for project-scope install.
- **`shell=True` with list `cmd`** — confirmed P1 bug from Pass 6 (S1).
- **No timeout** — plugin install can hang indefinitely. UI shows "Installing X..." forever.
- **No cancellation** — once started, no way to stop.
- **`call_from_thread` marshals back to UI** — correct pattern for Textual workers.

## Untested: the entire plugin command happy/sad path

There's no test that mocks subprocess and validates the success/error toast pathway. The behavior is documented entirely in code.

## Delta Summary

- New items added: Detailed scope-algebra trace with concrete examples, plugin install path fallback bug, scope badge meaning, semver-detection rule
- Existing items refined: Seed 6 net behavior cleaned up; first-installation fallback identified as latent display bug
- Remaining gaps: Plugin command timeout/cancel; no E2E test for plugin lifecycle; `display_scope`-vs-`installed_scopes` mismatch edge case

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: Tracing the dict-merge / set-algebra explicitly revealed the **last-wins shadowing** in disabled-subset construction — which is a quirky and brittle implementation but happens to produce the right net result. Also identified the **install-path fallback bug** when scope-specific paths are missing. Both findings would change how the Rust port is written.

## Convergence Declaration

Another round needed for the **widgets layer** specifically the marketplace modal sub-states. The parsers and plugin paths feel close to converged; widget UX state machines are still under-mapped.

## State Checkpoint

```yaml
pass: B
subpass: plugin-marketplace
round: 1
status: complete
timestamp: 2026-05-11T17:40:00Z
novelty: SUBSTANTIVE
```
