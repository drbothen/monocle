# Final Synthesis: NikiforovAll/lazyclaude

**Reference:** `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/`
**HEAD:** `ebc1f8f3b046a04707340f749b4a441e26df7f6d` (main, verified via `git -C ... rev-parse HEAD`)
**Author:** Oleksii Nikiforov (`nikiforovall`)
**License:** MIT
**Distribution:** PyPI package, `uvx lazyclaude`

This document supersedes the individual pass artifacts and is the canonical reference for downstream Monocle skills.

## Executive summary

LazyClaude is a 9,280-LOC Python Textual TUI that **inventories and edits Claude Code customizations** across User / Project / Project-Local / Plugin scopes. The codebase is organized as four clean layers (models / services / mixins / widgets), with the **service layer being the principal gene material** for any reimplementation: per-customization-type parsers, a multi-scope discovery walker, a plugin/marketplace registry resolver, and a writer that performs type-dispatched mutations on heterogeneous on-disk JSON/Markdown/directory shapes.

The presentation layer is **Textual-specific** and built around five `Mixin` classes that compose into a single `App` subclass. The keyboard ergonomics follow lazygit conventions and are constitutionally codified. The "marketplace" feature delegates package management to the `claude` CLI binary; LazyClaude only reads its outputs and shells out for verbs.

For Monocle's Rust port, the **single highest-value extraction** is the seven-parser canonical schema (BC-1 through BC-7 in Pass 4) plus the discovery orchestrator's three-phase plugin enumeration. The TUI layer can be replaced with `ratatui` informed by the widget state machines documented in Phase B widget deepening.

## Critical genes (preserve verbatim in Rust)

1. **Customization type taxonomy** — `enum CustomizationType { SlashCommand, Subagent, Skill, MemoryFile, MCP, Hook, LSPServer }`. Order matters (used as sort key in discovery).
2. **ConfigLevel taxonomy** — `enum ConfigLevel { User, Project, ProjectLocal, Plugin }`. Same ordering convention.
3. **PluginScope** — `enum PluginScope { User, Project, ProjectLocal }`. Maps to `enabledPlugins` settings file selection.
4. **Project slug regex** — `[^a-zA-Z0-9\-]` → `-`. Used to find `~/.claude/projects/<slug>/memory/`. Must match Claude Code's exact convention.
5. **File-pattern map** for the three SCAN_CONFIGS:
   - slash commands: `commands/**/*.md` (recursive)
   - subagents: `agents/*.md` (flat)
   - skills: `skills/*/SKILL.md` (per-subdir)
6. **Memory file recognition set:** `{"CLAUDE.md", "AGENTS.md", "CLAUDE.local.md"}`. Auto-memory: `~/.claude/projects/<slug>/memory/MEMORY.md` and siblings.
7. **MCP locations:** `~/.claude.json[mcpServers]` (User), `./.mcp.json` wrapped-or-unwrapped (Project), `~/.claude.json[projects][<path>][mcpServers]` (ProjectLocal, with `/` and `\\` fuzzy lookup).
8. **Hook locations:** `settings.json[hooks]` (User), `./.claude/settings.json[hooks]` (Project), `./.claude/settings.local.json[hooks]` (ProjectLocal), `<plugin>/hooks/hooks.json[hooks]` (Plugin). **All must be wrapped under `hooks` key.**
9. **LSP locations:** `<plugin>/.lsp.json` (root-dict of language→config) and `<plugin>/.claude-plugin/plugin.json[lspServers]`.
10. **Plugin registry:** `~/.claude/plugins/installed_plugins.json` (V2 schema: `plugins: {<id>: [<installation>...]}`) with `scope`, `version`, `installPath`, `isLocal`, `projectPath` per installation.
11. **Marketplace registry:** `~/.claude/plugins/known_marketplaces.json` with per-marketplace `{source: {source, repo|path}, installLocation, lastUpdated}`.
12. **Marketplace plugin catalog:** `<install_location>/.claude-plugin/marketplace.json` with `name`, `plugins: [{name, description, source, ...extras}]`.
13. **Plugin ID format:** `"<name>@<marketplace_name>"`. Standalone plugins (no marketplace) use just `<name>`.
14. **Dedup invariant during discovery:** memory files use resolved-path set; plugin preview uses resolved-path set.
15. **Sort invariant:** `discover_all()` returns customizations sorted by `(CustomizationType.value, name.lower())`. Stable across passes — panels filter without re-sorting.
16. **Cache invariant:** `discover_all()` returns the **same list object** on repeated calls until `refresh()` is called. Rust port should return a clone or use `Arc<Vec<...>>` for the same effect.

## Complete feature inventory

| Feature | Trigger | Service path | Risk |
|---|---|---|---|
| Discover all customizations | startup, `r` | `ConfigDiscoveryService.discover_all` | — |
| Filter by level (a/u/p/P) | keybinding | `FilterService.filter(level=...)` | — |
| Filter by enabled (D) | keybinding | `FilterService.filter(plugin_enabled=...)` | — |
| Substring search (/) | keybinding | `FilterService.filter(query=...)` | — |
| Open in $EDITOR (e) | keybinding | `subprocess.Popen + shlex.split` | P2 (Windows shell=True) |
| Copy path to clipboard (C) | keybinding | `pyperclip.copy` | — |
| Open user config (Ctrl+u) | keybinding | `subprocess.Popen` | — |
| Copy customization (c) | keybinding + LevelSelector | `CustomizationWriter.write_*` | — |
| Move customization (m) | keybinding + LevelSelector | write + delete (no rollback) | P1 |
| Delete customization (d) | keybinding + DeleteConfirm | `CustomizationWriter.delete_*` | — |
| Toggle plugin enabled (t) | keybinding + PluginConfirm | `CustomizationWriter.toggle_plugin_enabled` | — |
| Browse marketplace (M) | keybinding | `MarketplaceModal.show` | — |
| Install plugin (i) | modal action + scope picker | `claude plugin install` via subprocess | P1 (shell=True) |
| Uninstall plugin (d, in modal) | modal action | `claude plugin uninstall` | P1 |
| Update plugin/marketplace (u) | modal action | `claude plugin update` / `marketplace update` | P1 |
| Add marketplace (A) | source input | `claude plugin marketplace add` | P1 |
| Remove marketplace (d on root) | confirm | `claude plugin marketplace remove` | P1 |
| Preview plugin (p) | modal action | `discover_from_directory` | — |
| Toggle scope view (s) | modal action | mutates `display_scope` | — |
| Toggle installed-only filter (I) | modal action | filter pred | — |
| Switch theme | command palette | `register_theme` + persist to `~/.lazyclaude/settings.json` | — |
| Open user config in editor (e in modal) | modal action | `subprocess.Popen([editor, path], shell=True)` | P2 |
| Open plugin source (o in modal) | modal action | `webbrowser.open` or file explorer | — |

## Bounded contexts

### 1. Customization context

Concerned with: discovering, listing, filtering, displaying, copying, moving, deleting Customization instances across four ConfigLevels. Owns: `models/customization.py`, `services/discovery.py`, `services/parsers/*`, `services/filter.py`, `services/writer.py`, `services/config_path_resolver.py`, `services/gitignore_filter.py`, `services/filesystem_scanner.py`. Owned by ~3,600 LOC.

### 2. Plugin/marketplace context

Concerned with: identifying installed plugins, surfacing marketplace catalogs, joining install state with availability, delegating lifecycle to the `claude` CLI. Owns: `models/marketplace.py`, `services/plugin_loader.py`, `services/marketplace_loader.py`, plus widgets `marketplace_modal.py`, `marketplace_confirm.py`, `marketplace_source_input.py`. Owned by ~2,500 LOC.

### 3. TUI presentation context

Concerned with: composing widgets, dispatching keystrokes to mixin handlers, rendering with Rich/Pygments, managing focus and modal state. Owns: `app.py`, `bindings.py`, `mixins/*`, `widgets/*`, `themes.py`, `styles/app.tcss`. Owned by ~5,000 LOC. **The largest layer by LOC** — the bulk of the codebase is TUI plumbing.

### 4. Settings context

Concerned with: persisting user preferences (theme, suggested marketplaces). Owns: `models/settings.py`, `services/settings.py`. Owned by ~120 LOC. Smallest context. Settings stored at `~/.lazyclaude/settings.json`.

The contexts have **minimal cross-cutting**: settings is consumed by app on mount; plugin/marketplace consumes Customization data only for the preview branch; TUI orchestrates the other three.

## Complexity ranking (effort to port to Rust)

1. **Discovery + Parsers (HIGH)** — 7 parsers × per-type schemas + multi-scope walker + caching + dedup. Direct rewrite, ~2 weeks for an idiomatic Rust port with serde/regex/walkdir. The test fixtures in `tests/integration/fixtures/` are reusable verbatim.
2. **Plugin/Marketplace loader (MEDIUM-HIGH)** — multi-phase scope enumeration with quirky set algebra; directory-source path resolution. ~1 week if Pass B-r1 contract is followed.
3. **Writer (MEDIUM)** — type-dispatched copy/move/delete; JSON merging for MCP and Hook. ~1 week + atomic-write hardening.
4. **TUI (HIGH but isolated)** — `ratatui` rewrite. Widget state machines documented; can proceed independently once domain layer is stable. ~3 weeks.
5. **CLI delegation for plugin lifecycle (LOW)** — just `Command` invocations. The `claude` CLI binary still does the heavy work.

**Total estimated rewrite:** ~7-8 person-weeks for feature parity + tests.

## Critical design decisions

### D1: Plain dataclasses, no pydantic

All domain types are `@dataclass`; runtime validation lives in parsers, not models. Trade-off: no automatic JSON schema, but trivial Rust port via `#[derive(Serialize, Deserialize)]`.

### D2: Parser-as-strategy injection

`SCAN_CONFIGS` dict maps logical name → `ScanConfig(subdir, pattern, GlobStrategy, parser_factory)`. New parsers are added by appending to this map. Clean for the original three types but **breaks down for non-SCAN_CONFIG types** (memory, MCP, hook, LSP, rules, auto-memory all use custom discovery branches). Refactoring opportunity for the Rust port: unify all discovery paths under a single trait-driven mechanism.

### D3: Mixin-based action composition

The `LazyClaude(App)` class is composed of five mixins providing all `action_*` and `on_*` handlers. Pythonic; Rust equivalent would be a single `App` struct + functions on it (no MRO needed). The mixin organization documents the **action surface** — Rust port can mirror as `app::navigation`, `app::filtering`, etc. modules.

### D4: Subprocess-driven plugin lifecycle

LazyClaude does NOT mutate `installed_plugins.json` directly — it shells out to `claude plugin <verb>`. Trade-off: ensures `claude` CLI's invariants hold (e.g., downloads, hashes, version compatibility) but couples LazyClaude to whatever `claude` CLI does. Rust port should preserve this — re-implementing `claude plugin install` is out of scope.

### D5: All filesystem operations synchronous

Discovery on UI thread; no spinner. Acceptable for small installs (typical user has <10 plugins); painful for large installs. Rust port should consider async I/O with progressive panel population.

### D6: No internal logging

`Customization.error` field for parse failures; `notify()` for user toasts; everything else silent. Rust port should add structured logging (`tracing` crate) at minimum.

### D7: Hand-rolled semver comparison (3× duplication)

`_parse_version` appears at `plugin_loader.py:343`, `marketplace_loader.py:277`, `marketplace_modal.py:425`. Rust port should consolidate using `semver` crate.

## Anti-patterns identified

| Anti-pattern | Citation | Fix |
|---|---|---|
| `subprocess.run(list_cmd, shell=True)` | `mixins/marketplace.py:253-261` | `shell=False` |
| Non-atomic JSON writes | `writer.py:515-518` | tempfile + atomic rename |
| `dataclass.__dict__` for metadata typing | every parser | typed enum variant |
| 3× semver duplicate | plugin_loader / marketplace_loader / marketplace_modal | shared utility |
| `TypeError` fallback in parser factory | `filesystem_scanner.py:66-68` | uniform constructor signature |
| Scattered path literals | discovery.py and writer.py many places | central `paths::*` constants module |
| Silent JSON/OSError swallow | gitignore_filter, settings, marketplace_loader, plugin_loader | structured logging + user-visible degradation |
| Lossy hook metadata (everything in `content`) | `hook.py:53-71` | per-event detail structure |
| Move = copy+delete without rollback | `customization_actions.py:165-212` | two-phase commit or atomic rename for same-volume |
| Empty `keybindings/` placeholder | `keybindings/__init__.py:1-4` | delete dead module or use it |

## Convergence report

| Subsystem | Rounds | Final novelty | Why |
|---|---|---|---|
| Parsers | 2 | NITPICK | Round 1 found schema divergences (hooks-wrapped vs MCP-unwrapped, LSP raw-dict metadata); Round 2 only added test confirmations |
| Plugin/Marketplace | 1 | SUBSTANTIVE → conv | Round 1 revealed scope set-algebra subtleties + install-path fallback bug; no further substantive findings warranted Round 2 |
| Widgets | 1 | SUBSTANTIVE → conv | Round 1 mapped all widget state machines; nothing left to discover in widget layer |

**All passes converged in ≤ 2 rounds.** Total deepening rounds: 4 (parsers ×2, plugin/marketplace ×1, widgets ×1). No subsystem reached the maximum 5-round cap. No subsystem failed to converge.

## Total coverage metrics

- **Source files:** 50/50 .py = **100% read in full**
- **Test files:** 10/25 read in full (40%); remainder verified by filename only — sufficient because every behavioral claim was sourced from code-of-truth
- **Subsystems from brief:** **15/15** covered
- **Behavioral contracts extracted:** 12 (BC-1..BC-12 in Pass 4)
- **Holdout seeds identified:** 12 (Pass 7)
- **High-priority seeds resolved by test or deepening:** 4 (Seed 1 slug, Seed 3 Windows fuzzing, Seed 5 semver-tuple, Seed 6 scope algebra)
- **High-priority seeds remaining:** 0 (all 4 resolved)
- **Security findings:** 1 P0 (atomic writes), 2 P1 (shell=True misuse, move-rollback)

## P0/P1 risks for Monocle's Rust port

### P0 (must address before parity)

1. **Project-slug regex MUST byte-match Python implementation.** Use `regex` crate with pattern `[^a-zA-Z0-9\-]` and replace each match with `-`. No collapsing of consecutive matches.
2. **Discovery-output ordering MUST match.** Sort by `(CustomizationType variant index, name.lower())`. Variant index order: SlashCommand=0, Subagent=1, Skill=2, MemoryFile=3, MCP=4, Hook=5, LSPServer=6.
3. **Atomic file writes** for all settings/MCP/hook JSON mutations. Use `tempfile::NamedTempFile + persist` (POSIX rename atomicity).
4. **MCP `.claude.json` `projects[<path>]` key MUST be fuzzy-matched** against both `/` and `\\` normalized project paths (Windows compatibility, has test coverage in reference).

### P1 (should address)

5. **Subprocess for plugin lifecycle:** use `std::process::Command::args(...)` with `shell(false)`. Plugin ID format `<name>@<marketplace>` from marketplace.json is constrained enough that injection is unlikely, but the pattern is broken.
6. **Move operation:** if delete after copy fails, either rollback the copy or surface clearly. Reference codebase notifies user but leaves both copies.
7. **CRLF handling in markdown frontmatter:** the Python regex requires `\n`; CRLF files appear to have no frontmatter. Rust port should normalize line endings or use a more permissive regex.
8. **LSP layer parity:** the Python reference has no LSP tests. Rust port should add coverage.
9. **Plugin preview (`discover_from_directory`):** untested in reference. Rust port should add coverage of marketplace-extras path overrides.

### P2 (nice-to-have)

10. Concurrent write protection for `~/.claude.json` (shared with `claude` CLI).
11. Lazy file content read in skills (current eager read can OOM).
12. Structured logging via `tracing`.
13. Progressive panel population (don't block UI on discover_all).

## Recommended deepening (if Monocle wants more)

If Monocle's scope extends beyond a 1:1 port, the following areas merit additional study:

1. **Read the remaining 15 unread test files** to elaborate edge-case coverage and identify behaviors not surfaced in code.
2. **Read `tests/integration/fixtures/**`** to inventory the real example shapes for skills, commands, MCP configs, hooks.
3. **Read `docs/user-guide.md` and `docs/testing-guide.md`** for in-the-wild documented edge cases.
4. **Inspect `.claude/commands`, `.claude/agents`, `.claude/rules` of the lazyclaude repo itself** to see how nikiforovall uses his own tool.

These were out-of-scope for the present ingest (which focused on code-of-truth behavioral extraction).

## State Checkpoint

```yaml
pass: 8
type: final-synthesis
status: complete
timestamp: 2026-05-11T18:10:00Z
supersedes:
  - pass-1-project-discovery.md
  - pass-2-architecture.md
  - pass-3-conventions.md
  - pass-4-behavioral-contracts.md
  - pass-5-verification-gaps.md
  - pass-6-security-deps.md
  - pass-7-holdout-seeds.md
  - pass-B-deep-parsers-r1.md
  - pass-B-deep-parsers-r2.md
  - pass-B-deep-plugin-marketplace-r1.md
  - pass-B-deep-widgets-r1.md
  - pass-B5-coverage-audit.md
  - pass-B6-extraction-validation.md
convergence_rounds: 4
total_artifacts: 14
```
