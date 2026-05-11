# Pass 1: Project Discovery — nikiforovall/lazyclaude

**Reference root:** `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/`
**HEAD:** `ebc1f8f3b046a04707340f749b4a441e26df7f6d` (main)
**Verified by:** `git -C .reference/nikiforovall-lazyclaude rev-parse HEAD`

## Project Identity

LazyClaude is a Python Textual TUI (terminal UI) that visualizes the totality of a user's Claude Code customizations across all configuration scopes (User / Project / Project-Local / Plugin). Stylistic posture: lazygit-inspired — keyboard-first, multi-panel, modal-minimal. Distribution: PyPI package runnable via `uvx lazyclaude` (zero-install).

Reference: `README.md:5-7`, `CLAUDE.md:5-7`, `docs/constitution.md:1-110`.

## Tech Stack

| Layer | Choice | Version constraint | Citation |
|---|---|---|---|
| Language | Python | `>=3.11` | `pyproject.toml:6` |
| TUI framework | Textual | `>=8.0.0` | `pyproject.toml:25` |
| Rich text | rich | `>=13.0.0` | `pyproject.toml:26` |
| Frontmatter / YAML | PyYAML | `>=6.0` | `pyproject.toml:27` |
| Clipboard | pyperclip | `>=1.11.0` | `pyproject.toml:28` |
| Gitignore matching | pathspec | `>=1.0.0` (uses `gitignore` syntax) | `pyproject.toml:29`, `services/gitignore_filter.py:74` |
| Tests | pytest + pytest-asyncio + pytest-textual-snapshot + pyfakefs | varies | `pyproject.toml:38-43`, `pyproject.toml:108-113` |
| Lint / format | ruff (line-length 88, py311 target) | `>=0.15.0` | `pyproject.toml:42`, `pyproject.toml:61-91` |
| Type checker | mypy (`disallow_untyped_defs=true`) | `>=1.19.0` | `pyproject.toml:43`, `pyproject.toml:98-103` |
| Build backend | hatchling + hatch-vcs (dynamic version) | — | `pyproject.toml:48-56` |
| Package manager | uv | — | `CLAUDE.md:14`, `pyproject.toml:104-114` |
| Entry point | `lazyclaude = "lazyclaude.__main__:main"` | — | `pyproject.toml:46` |

## File Manifest (LOC, by directory)

Source-tree LOC (excluding `uv.lock`, `.git`, generated `_version.py`):

| Area | Files | LOC | Citation |
|---|---|---|---|
| `src/lazyclaude/*.py` (root + app) | 4 (app.py, bindings.py, themes.py, __init__.py, __main__.py) | ~770 | counted from largest list |
| `src/lazyclaude/models/` | 4 (`__init__`, `customization`, `marketplace`, `settings`) | ~250 | inspection |
| `src/lazyclaude/services/` | 11 (excluding parsers) | ~2,070 | discovery=722, writer=518, plugin_loader=353, marketplace_loader=306, gitignore_filter=149, filter=126, filesystem_scanner=116, settings=110, config_path_resolver=72, opener=42, __init__=18 |
| `src/lazyclaude/services/parsers/` | 8 (7 parsers + __init__) | ~870 | memory_file=148, skill=147, lsp_server=139, mcp=127, hook=88, subagent=80, slash_command=89, __init__=80 |
| `src/lazyclaude/widgets/` | 13 (12 widgets + helpers) | ~3,710 | marketplace_modal=788, type_panel=661, combined_panel=580, detail_pane=381, marketplace_source_input=324, app_footer=141, plugin_confirm=150, level_selector=140, marketplace_confirm=118, delete_confirm=115, filter_input=111, status_panel=79, helpers/rendering=99, helpers/__init__=10 |
| `src/lazyclaude/mixins/` | 6 | ~960 | marketplace=430, customization_actions=279, navigation=132, filtering=107, help=73, __init__=15 |
| `src/lazyclaude/keybindings/` | 1 (`__init__.py`, currently empty namespace) | 3 | `keybindings/__init__.py:1-4` |
| `src/lazyclaude/styles/app.tcss` | 1 | 157 | CSS layout |
| **TOTAL src** | **50 .py files** | **9,280 LOC** | `find src -name '*.py' \| wc -l` and aggregate `wc -l` |
| **TOTAL tests** | **28 .py files** | **5,275 LOC** | `find tests -name '*.py' \| wc -l` |

LOC ratio src:test ≈ 1.76:1 — strong test investment.

## Entry points

1. **CLI:** `src/lazyclaude/__main__.py:10-47` — argparse with `-V/--version`, `-d/--directory` (project), `-u/--user-config` (override). Default project = `Path.cwd() / ".claude"`; default user = `~/.claude` (from `discovery.py:141`).
2. **Module:** `python -m lazyclaude` (same `main()`).
3. **App factory:** `create_app(user_config_path, project_config_path)` in `app.py:669-687`.

## Module dependency graph (top-level)

```
__main__ -> app.create_app -> LazyClaude(App)
                                |
                                |- mixins/ (Navigation, Filter, Marketplace, CustomizationActions, Help)
                                |- services/discovery -> services/{filesystem_scanner, gitignore_filter, plugin_loader, parsers/*}
                                |- services/filter
                                |- services/config_path_resolver
                                |- services/marketplace_loader -> services/plugin_loader
                                |- services/settings (app-config persistence)
                                |- services/writer (CRUD inverse of parsers, hot-path for c/m/d actions)
                                |- services/opener (cross-platform open + browser)
                                |- widgets/ (TypePanel, CombinedPanel, MainPane, StatusPanel, FilterInput,
                                |            LevelSelector, DeleteConfirm, PluginConfirm,
                                |            MarketplaceModal, MarketplaceConfirm, MarketplaceSourceInput,
                                |            AppFooter, helpers/rendering)
                                |- themes (LAZYGIT_THEME, DEFAULT_THEME="gruvbox")
                                |- bindings (APP_BINDINGS Textual Binding list)
                                |- models/ (customization, marketplace, settings)
```

The `keybindings/` package exists but is empty (`keybindings/__init__.py:1-4` — `__all__: list[str] = []`). All actual bindings live in `bindings.py`. Dead-code candidate or scaffolding for future per-context keymaps.

## Layer architecture (clean three-layer)

| Layer | Folder | Responsibility |
|---|---|---|
| Presentation (Textual widgets) | `widgets/` | Pure visual + event-driven; emit Messages, no I/O |
| Application (mixins on `LazyClaude`) | `mixins/` | Action handlers, orchestrate services, manage focus/state |
| Domain models | `models/` | Plain `@dataclass` types; no Textual import |
| Services (I/O) | `services/` | Filesystem, JSON/YAML parsing, subprocess, clipboard |
| Parsers (sub-services) | `services/parsers/` | Per-type customization parsing (canonical schema gene) |

`models/` is Textual-free. `services/` is Textual-free except no imports observed. `widgets/` is the only Textual layer. Mixins straddle: they import widgets + services and define `action_*` handlers.

## Tests breakdown

| Sub-tree | Files | Citation |
|---|---|---|
| `tests/conftest.py` | 1 (pyfakefs fixtures, `/fake/home`, `/fake/project`) | `conftest.py:11-196` |
| `tests/integration/discovery/` | 8 test files | per-type discovery tests (test_skills, test_subagents, test_slash_commands, test_mcps, test_hooks, test_memory_files, test_plugins, test_gitignore, test_auto_memory, test_behavior) |
| `tests/integration/writer/` | 2 | `test_mcp_writer`, `test_delete_writer` |
| `tests/integration/fixtures/` | reference fixtures dir (skills, commands, agents, mcp, settings, plugins) | non-code, real-on-disk fixtures mounted into pyfakefs via `fs.add_real_directory` |
| `tests/unit/` | 9 unit test files | `test_app_customization_actions`, `test_combined_panel`, `test_config_path_resolver`, `test_customization_writer`, `test_filesystem_scanner`, `test_gitignore_filter`, `test_level_selector`, `test_memory_file_ref`, `test_plugin_source_path`, `test_rules_discovery`, `test_settings_service` |

Testing posture: heavy use of **pyfakefs** to simulate `~/.claude`, `~/.claude/plugins/`, project `.claude/`, `.mcp.json` etc. — meaning Monocle's Rust reimplementation can study these fixtures for canonical filesystem shapes. Real fixtures live in `tests/integration/fixtures/` and would be reusable verbatim.

## Build & distribution

- `pyproject.toml:48-56` — Hatchling + `hatch-vcs` writes version to `src/lazyclaude/_version.py` (gitignored, generated at build).
- `pyproject.toml:46` — Console script `lazyclaude` → `lazyclaude.__main__:main`.
- `pyproject.toml:13` — Status `Development Status :: 3 - Alpha`.
- `.github/workflows/` directory exists (4 files) — CI presumably runs ruff + mypy + pytest per `CLAUDE.md:32`.
- `.pre-commit-config.yaml` exists at repo root (326 bytes) — enforces hooks before commit per `CLAUDE.md:30-32`.
- `docs/` ships `constitution.md`, `user-guide.md`, `testing-guide.md`, `index.html` (GitHub Pages landing).
- `assets/demo.gif` (~4.2MB) is the largest single file in the source tree and primary marketing asset.

## High-priority files (gene density)

Ranked by translation difficulty + behavioral payload (NOT by LOC):

1. **`services/discovery.py`** (722 LOC) — the canonical "what to look for, where, and how to merge" discovery orchestrator. Monocle's Rust explorer plane must replicate this near-exactly.
2. **`services/parsers/*.py`** (7 files, 818 LOC) — per-customization-type schemas; each is a small one-shot parser with explicit edge-case handling.
3. **`services/plugin_loader.py`** (353 LOC) — multi-scope plugin registry (user/project/local) and source-path resolution including directory-source marketplaces.
4. **`services/writer.py`** (518 LOC) — type-dispatched copy/move/delete with shape preservation (skill = directory; slash command = nested-path-flattened name; MCP = JSON merge; hook = JSON merge).
5. **`services/gitignore_filter.py`** (149 LOC) — pruning walker using `pathspec` + a hardcoded `DEFAULT_SKIP_DIRS` allowlist.
6. **`services/filesystem_scanner.py`** (116 LOC) — `GlobStrategy` enum (RGLOB/GLOB/SUBDIR) + dataclass `ScanConfig` driving a generic scan loop.
7. **`models/customization.py`** (180 LOC) — `ConfigLevel`, `CustomizationType`, `Customization`, plus per-type metadata `@dataclass`es.
8. **`models/marketplace.py`** (49 LOC) — Marketplace + plugin schemas.
9. **`widgets/marketplace_modal.py`** (788 LOC) — the most complex widget, encoding the marketplace UX gene.
10. **`widgets/type_panel.py`** (661 LOC) — single-type list with skill-tree and memory-ref expansion, the prototypical list panel.

## Hot paths (runtime cost expected)

| Path | Why |
|---|---|
| `discovery.discover_all` (cached) | walks user + project + plugin trees, parses every markdown/JSON, builds memory ref graphs |
| `MemoryFileParser._resolve_references` | recursive, depth-bounded (5), cycle-guarded — touched by every CLAUDE.md/AGENTS.md |
| `GitignoreFilter.walk_filtered` | wraps `os.walk` with directory pruning + per-file `fnmatch` + per-file pathspec match |
| `MarketplaceLoader._load_installed_plugins` | quadratic-ish scanning of installations × scopes |

## State Checkpoint

```yaml
pass: 1
status: complete
timestamp: 2026-05-11T17:00:00Z
files_scanned: 50_src + 28_test + 6_docs/config = 84
next_pass: 2
```
