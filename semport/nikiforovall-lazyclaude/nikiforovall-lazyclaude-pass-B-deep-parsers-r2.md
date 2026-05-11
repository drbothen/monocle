# Phase B Deepening: Parsers — Round 2

Goal: verify test coverage and confirm convergence on the parser layer.

## Test files read in full this round

- `tests/integration/discovery/test_mcps.py:1-218` (Round 1)
- `tests/integration/discovery/test_skills.py:1-193` (this round)
- `tests/integration/discovery/test_auto_memory.py:1-176` (this round)
- `tests/integration/discovery/test_behavior.py:1-227` (this round)
- `tests/integration/discovery/test_plugins.py:1-95` (this round)
- `tests/integration/discovery/test_hooks.py:1-39` (Round 1)

## Findings from Round 2 reads

### Skills — well-covered

`test_skills.py` covers:
- happy path discovery (`test_discovers_user_skills`)
- metadata parsing including tags (`test_skill_tags_parsed`)
- file tree population including content (`test_skill_files_have_content`)
- nested directory recursion (`test_skill_nested_directories_discovered`)
- `has_reference`, `has_scripts` flags
- **node_modules exclusion** — `test_skill_files_exclude_node_modules:158-192` confirms the gitignore-dir-filter works on skill subdirs

What's NOT tested: hidden file exclusion (the `.startswith(".")` filter), `OSError` on individual file read (`SkillFile.content = None`), `has_examples` and `has_templates` flags. P2 gaps.

### Auto memory — well-covered

`test_auto_memory.py` covers all the critical behaviors:
- `test_separators_become_hyphens` — confirms Seed 1's slug algorithm
- `test_dotfiles_in_path` — confirms `.config` becomes `--config-`
- `test_no_auto_memory_dir_returns_empty`
- `test_discovers_memory_md_entrypoint`
- `test_topic_files_become_refs` — confirms the synth-ref logic
- `test_topic_file_content_loaded` — confirms refs have content
- `test_no_memory_md_discovers_individual_files` — confirms the "fallback to individual files" branch (`discovery.py:523-527`)
- `test_non_md_files_ignored` — confirms `.txt` files are skipped
- `test_existing_imports_not_duplicated` — confirms the dedup based on `existing_import_names` (`discovery.py:505-507`)

**Seed 1 is fully resolved.** The Rust port must reproduce the slug exactly: `regex `[^a-zA-Z0-9\-]` → `-`. The test pins the convention.

### test_behavior.py — covers cross-cutting

- `test_discover_all_returns_sorted_results` — confirms type-then-name ordering invariant
- `test_discover_by_level_user/project/plugin` — three level views
- `test_empty_directories_returns_empty` — discovery is total
- `test_missing_directories_handled_gracefully` — defensive
- `test_malformed_json_sets_error` — `.mcp.json` with invalid JSON produces a single error-customization
- `test_malformed_yaml_frontmatter_falls_back_gracefully` — confirms YAML-error → empty frontmatter → no `has_error`
- `test_discover_all_caches_results` — `first_call is second_call` (identity check, confirming cache)
- `test_refresh_clears_cache` — `refreshed is not first_call`
- `test_returns_project_path_when_exists` / `test_returns_user_path_when_project_missing` — `get_active_config_path` logic

This is the **integration-level safety net** for parser/discovery contracts.

### Plugins — coverage gaps

`test_plugins.py` covers:
- enabled plugin commands appear
- **disabled plugins ARE included** (`test_disabled_plugins_included:33-60`) — with `plugin_info.is_enabled = False`
- subagents + skills from plugins

What's NOT tested:
- LSP servers from plugins (`_discover_plugin_lsp_servers` `discovery.py:701-722`)
- `plugin.json` `lspServers` parsing (`lsp_server.parse_plugin_json:88-120`)
- Marketplace-extras custom path overrides (`discovery._discover_marketplace_components:253-302`) — the entire branch that's invoked only from `discover_from_directory` during preview
- `discover_from_directory` itself (the preview entry point) — confirmed by grep absence
- Project-scoped plugins (Phase 2 of `get_all_plugins`)
- Local-scoped plugins (Phase 3)

**P1 verification gap.** Monocle's port must implement these but with knowledge that they're untested in the reference.

## YAML lenience — confirmed and pinned

`test_malformed_yaml_frontmatter_falls_back_gracefully` (`test_behavior.py:133-156`):
```python
fs.create_file(... contents="---\n[unclosed bracket\n---\n# Bad")
# ...
bad_cmd = next((c for c in commands if c.name == "bad"), None)
assert bad_cmd is not None
assert not bad_cmd.has_error
assert bad_cmd.metadata.get("allowed_tools") == []
```

So: malformed frontmatter does NOT set `has_error`. The command appears with default metadata. `description` would fall through to the body heuristic. **This is the only test that exercises the YAML fallback path.**

## Caching invariant pinned

`first_call is second_call` confirms the cache returns the **same list object**, not a copy. Any mutation by the caller would corrupt subsequent reads. **Subtle aliasing risk** that Rust's ownership model would prevent.

## Sort invariant pinned

The test verifies that `discover_all()` returns customizations sorted by `(type_order_index, name.lower())`. Type order matches `enum CustomizationType`. **This means panels can display the relevant subset by filtering** without re-sorting. Important for Rust port: maintain the same enum order.

## Remaining genuine gaps

1. **`discover_from_directory` (plugin preview)** — 0 tests. The branch that handles `marketplace_plugin.extra_metadata` paths is unverified.
2. **LSP plumbing** — `LSPServerParser`, `_discover_plugin_lsp_servers`, `parse_plugin_json` — 0 tests.
3. **Plugin scope phases 2 and 3** — project-scope and local-scope plugin discovery — fixtures don't appear to exercise these.
4. **Concurrent file modification during discovery** — no test (would require threading).
5. **Symlink behavior in skill walks** — no test.
6. **CRLF line endings in markdown** — no test, and the regex won't match. This is a P1 portability issue.

## Delta Summary

- New items added: 6 test-confirmed pinned behaviors (sort, cache, YAML lenience, slug, ref dedup, auto-memory fallback), 6 confirmed gaps (LSP, preview, scope phases, concurrency, symlinks, CRLF)
- Existing items refined: Seed 1 (slug) resolved; YAML lenience promoted to pinned behavior
- Remaining gaps: see "remaining genuine gaps" list

## Novelty Assessment

Novelty: **NITPICK**

Justification: This round confirmed existing pass-4/pass-B-r1 findings with test citations and identified specific test coverage gaps, but discovered **no new behavioral contracts or schema fields**. The findings are refinements to confidence levels — moving items from "claimed by code" to "pinned by test" — without changing the model of the system. A Rust developer's plan for the parser layer would be identical with or without Round 2's findings.

## Convergence Declaration

**Parser/discovery subsystem has converged.** Findings are nitpicks: test-coverage observations, not new behavioral surface. Two rounds is sufficient.

## State Checkpoint

```yaml
pass: B
subpass: parsers
round: 2
status: complete
timestamp: 2026-05-11T17:50:00Z
novelty: NITPICK
converged: true
```
