# Pass 3: Conventions — nikiforovall/lazyclaude

## Naming

- **Modules:** snake_case, single-purpose, one class typically per file (`type_panel.py` → `TypePanel`). Strict.
- **Classes:** PascalCase. Suffix conventions: `*Mixin` for mixin classes (`NavigationMixin`), `*Parser`, `*Loader`, `*Service`, `*Modal`, `*Panel`, `*Confirm`.
- **Interfaces / ABCs:** `I` prefix (`ICustomizationParser` `services/parsers/__init__.py:12`, `IConfigDiscoveryService` `discovery.py:55`, `IFilterService` `filter.py:13`). Consistent across `services/`. **Not Pythonic** by PEP-8 standards, but uniformly applied.
- **Private members:** single underscore (`_render_item`, `_panels`). No name-mangling.
- **Reactive attributes:** plain names (`customization`, `view_mode`, `selected_index`); accessed as instance attributes — Textual reactive injection.
- **Action methods:** `action_<verb>_<noun>` (`action_filter_user`, `action_toggle_marketplace`). Textual auto-discovers these via `BINDINGS`.
- **Message handlers:** `on_<source>_<event>` (`on_type_panel_selection_changed`, `on_marketplace_modal_plugin_toggled`). Textual auto-routes by MRO.
- **Enums:** `auto()` values, UPPER_SNAKE members (`USER`, `PROJECT_LOCAL`, `SLASH_COMMAND`, `MEMORY_FILE`).
- **Constants:** module-level UPPER_SNAKE (`DEFAULT_MAX_WALK_DEPTH=5`, `MAX_IMPORT_DEPTH=5`, `MEMORY_FILE_NAMES`, `DEFAULT_SKIP_DIRS`).
- **Test fixtures:** snake_case in `conftest.py`, share `fake_*` prefix for filesystem fakes.

## Module organization

Each subpackage has an `__init__.py` that re-exports its public surface with explicit `__all__`. Examples:
- `models/__init__.py:19-31` — exports `ConfigLevel`, `Customization`, etc.
- `services/__init__.py:11-18` — exports the orchestrating services.
- `widgets/__init__.py:11-19` — exports widgets used by `app.py`.
- `mixins/__init__.py:9-15` — exports all mixins.
- `keybindings/__init__.py:3` — empty `__all__: list[str] = []` (placeholder package).

Exception: `services/parsers/__init__.py` exports only the shared utilities (`ICustomizationParser`, `parse_frontmatter`, `parse_tools_list`) — individual parsers are imported via fully-qualified paths (e.g. `from lazyclaude.services.parsers.skill import SkillParser`).

## Type hints

- **Required.** mypy enforces `disallow_untyped_defs=true` (`pyproject.toml:103`).
- **Modern PEP-604 syntax** throughout: `Path | None`, `dict[str, Any]`, `list[Customization]`. Target version `py311` (`pyproject.toml:63`).
- `TYPE_CHECKING` guard for circular avoidance: `mixins/navigation.py:7-10`, `widgets/type_panel.py:16-17`, `services/filesystem_scanner.py:11-12`. Forward-string annotations: `"TypePanel"`, `"GitignoreFilter | None"`.
- Cross-mixin attribute typing via stubs: each mixin declares the attributes it touches (e.g. `mixins/filtering.py:19-26`) so mypy is happy. Method calls onto other mixins use `# type: ignore[attr-defined]`.

## Docstrings

- One-line summary, sometimes followed by `Args:` / `Returns:` sections (Google-style).
- All public methods/functions have docstrings.
- `CLAUDE.md:39` mandate: "Comments explain WHY not WHAT". `customization_actions.py:170` is a good example: `# rollback NOT implemented` (well, that's our annotation — the actual code is sparser, the rule is observed by absence of trivial what-comments).

## Error handling

Two distinct error postures coexist:

### 1. Parsers — soft errors via `Customization.error`

Pattern (e.g. `slash_command.py:40-49`):
```python
try:
    content = path.read_text(encoding="utf-8")
except OSError as e:
    return Customization(
        name=self._derive_name(path),
        type=CustomizationType.SLASH_COMMAND,
        level=level,
        path=path,
        error=f"Failed to read file: {e}",
    )
```

Every parser does this on file-read; multi-output parsers (`MCPParser.parse:38-49`) wrap the entire JSON parse in a single try and return a list of one error-customization.

YAML errors are even more permissive: `parsers/__init__.py:57-63` swallows `YAMLError` and returns `({}, content)` — the file appears as a customization without frontmatter rather than as an error. **Inconsistent**.

### 2. Services — silent swallow

Pattern (e.g. `gitignore_filter.py:81-90`, `settings.py:52-53`, `settings.py:68-69`, `plugin_loader.py:100-101`, `marketplace_loader.py:50-51`):
```python
try:
    ...
except (json.JSONDecodeError, OSError):
    return ...  # empty default
```

No logging. No surfacing to the user. The file is treated as if it doesn't exist. This is the most idiosyncratic — and most fragile — pattern in the codebase.

### 3. UI surface — toast notifications

Centralized helpers `app.py:660-666`:
- `_show_status_success(msg)` → `notify(msg, severity="information", timeout=3.0)`
- `_show_status_error(msg)` → `notify(msg, severity="error", timeout=3.0)`

But there's no consistent gateway — many places call `self.notify(...)` directly with various severities and timeouts (`mixins/marketplace.py:200, 269, 271, 311, etc.`). Inconsistency.

## Imports

- Standard order enforced by ruff isort: stdlib → third-party → first-party (`lazyclaude.*`).
- `pyproject.toml:84-85` declares `known-first-party = ["lazyclaude"]`.
- Star imports forbidden by code review style, not seen anywhere.
- Re-exports via `__init__.py` allow consumers to write `from lazyclaude.services import ConfigDiscoveryService` rather than full path. Tests use this short form.

## Dataclasses everywhere

All domain types are `@dataclass` (`models/customization.py:49, 59, 69, 80, 91, 103, 114, 128`; `models/marketplace.py:8, 17, 27, 44`; `models/settings.py:9`). No pydantic, no attrs. Default factories used for mutable defaults (`field(default_factory=list)`).

The `__dict__` of metadata dataclasses is **dumped into `Customization.metadata: dict[str, Any]`** at parse time:
```python
metadata = SlashCommandMetadata(...)
return Customization(..., metadata=metadata.__dict__)
```
(`slash_command.py:59-73`, similar for subagent/skill/mcp).

This loses static typing on `metadata` — consumers must `metadata.get(...)`. Trade-off chosen for serialization simplicity (the metadata is later JSON-encoded for display). **Anti-pattern hint** — a typed enum-variant model would be safer.

Exception: `parsers/lsp_server.py:85` stores the **raw server_config dict** as metadata (not the dataclass dict). Inconsistency.

## Reactive / message patterns (Textual idioms)

- **`reactive[T]`** declared as class attributes on widgets (`type_panel.py:87-93`, `detail_pane.py:71-86`). Mutations auto-trigger `watch_<name>` callbacks.
- **`always_update=True`** is used for collection reactives to defeat identity-based change detection (`customizations: reactive[list[Customization]] = reactive(list, always_update=True)` — `type_panel.py:90`).
- **`set` reactives** require copy-then-reassign for changes to register (`type_panel.py:601-603`):
  ```python
  new_expanded = self.expanded_skills.copy()
  new_expanded.add(skill.name)
  self.expanded_skills = new_expanded
  ```
- **Messages** subclass `textual.message.Message`, defined as inner classes on the widget that emits them (`TypePanel.SelectionChanged`, `MarketplaceModal.PluginInstallWithScope`). Handlers in `App` are auto-named `on_<Widget>_<MessageClass>` (snake-cased) and signature-matched.

## CSS conventions

`styles/app.tcss` is a small file that mostly tunes the **outer container layout**; per-widget styling lives in each widget's `DEFAULT_CSS` class attribute (e.g. `type_panel.py:48-85`, `marketplace_modal.py:84-127`). Three sources of style:

1. Global `app.tcss` — sets Screen grid, borders, dock for modals
2. Per-widget `DEFAULT_CSS` — focus/hover/selected states, scrollbar gutters
3. Theme variables (`$primary`, `$accent`, `$error`, etc.) consumed by both — set in `themes.py` or Textual built-in themes

Modal positioning convention: every modal `dock: bottom; layer: overlay; display: none;` with a `.visible` class toggle (`level_selector.py:24-35`, `filter_input.py:19-30`, `delete_confirm.py:21-31`). Show/hide is via `add_class("visible")` / `remove_class("visible")`. The `MarketplaceModal` deviates — `dock: top; height: 100%` full-screen overlay.

## Keybinding conventions (constitution-driven)

From `docs/constitution.md:121-137`:

| Key | Action | Universal scope |
|---|---|---|
| `j/k` or arrows | up/down | every list |
| `h/l` | collapse / expand | tree-like context |
| `g/G` | top / bottom | list |
| `Enter` | drill / confirm | context |
| `Esc` | back / cancel | context |
| `/` | search | global |
| `?` | help | global |
| `q` | quit | global |

Every `BINDINGS` list across panels honors this (compare `type_panel.py:31-46`, `combined_panel.py:43-60`, `marketplace_modal.py:58-82`).

**Numeric keys 1-7** focus type panels. **a/u/p/P** filter by level. **c/m/d** copy/move/delete (uppercase variants don't exist for those — single-step). **t** toggles plugin enabled. **C** copies path to clipboard. **e** opens in $EDITOR. **M** opens marketplace. **D** toggles disabled-plugin filter.

Important: **`d` is overloaded** — it means delete in main app context, but uninstall/remove inside the marketplace modal. Context determined by which widget owns focus.

## Subprocess invocation conventions

Two subprocess invocation styles coexist:

1. **`subprocess.Popen([editor, str(path)], shell=(sys.platform == "win32"))`** for editor opens (`app.py:574-576`, `app.py:585-587`, `mixins/marketplace.py:292-293`). Note `shell=True` on Windows even when args are list — invitation to argument-splitting bugs.

2. **`subprocess.run(cmd, shell=True, ...)`** for `claude plugin ...` invocations (`mixins/marketplace.py:253-261`). **`shell=True` with a list `cmd`** is technically a Python misuse — on POSIX shells, only `cmd[0]` is used. Plugin commands happen to work because the IDs are well-formed identifiers, but this is a latent **command-injection-adjacent** risk if `plugin_id` is ever user-controlled. P1 security smell.

## Test conventions (from `.claude/rules/testing.md`)

- File prefix `test_`, class prefix `Test`, method prefix `test_<scenario>_<expected_outcome>`.
- Unit vs integration split: `tests/unit/`, `tests/integration/discovery/`, `tests/integration/writer/`.
- All filesystem ops via **pyfakefs** with `/fake/home` and `/fake/project` fakes.
- Real fixtures mounted via `fs.add_real_directory(FIXTURES_DIR / "...", target_path=...)`. Fixtures kept as actual files at `tests/integration/fixtures/`.
- pytest-asyncio mode `auto` (no per-test markers needed).
- Arrange / Act / Assert blocks explicit in docstrings.
- Type hints required on tests (mypy `disallow_untyped_defs=true` applies).

## Things this codebase does NOT do (consistency observations)

- **No async I/O for filesystem work** (despite Textual being async-friendly).
- **No `pathlib.Path.expanduser()`** anywhere — instead `Path.home() / ".claude"` (`discovery.py:141`, `app.py:591`). Consistent.
- **No environment variable abstractions** — `os.environ.get("EDITOR", "vi")` and `os.environ.get("HOME"/"USERPROFILE")` used directly.
- **No abstraction over `pyperclip`** — used directly in app (`app.py:622`).
- **No abstraction over `subprocess`** — used directly with different conventions.
- **No internal events bus** — relies entirely on Textual `Message` mechanism.
- **No DI container** — manual constructor injection. Services receive their dependencies explicitly (`PluginLoader(user_config_path, project_config_path, project_root)`).
- **No abstract path constants module** — paths like `"settings.json"`, `".claude.json"`, `".mcp.json"`, `"installed_plugins.json"`, `"known_marketplaces.json"`, `"marketplace.json"`, `".claude-plugin/marketplace.json"`, `".claude-plugin/plugin.json"`, `"hooks/hooks.json"`, `".lsp.json"` are scattered as string literals. **Monocle should consolidate these into a single `paths` constants module.** P0 for portability.
- **No version comparison library** — semver parsing is hand-rolled at `plugin_loader.py:343-353` and duplicated at `marketplace_loader.py:277-283` and `marketplace_modal.py:425-437`. Three copies of the same logic. **Duplication anti-pattern.**

## Pattern catalogue

### Strategy / Visitor — Parser dispatch

`services/parsers/__init__.py:12-41` defines `ICustomizationParser`. Each per-type parser implements `parse(path, level)` and `can_parse(path)`. Discovery dispatches based on `SCAN_CONFIGS` (a dict mapping logical name → `ScanConfig(subdir, pattern, strategy, parser_factory)`) — `discovery.py:33-52`. Clean strategy pattern.

### Factory injection — `parser_factory` in ScanConfig

Note `filesystem_scanner.py:66-68`:
```python
try:
    parser = config.parser_factory(target_dir, gitignore_filter=self._filter)
except TypeError:
    parser = config.parser_factory(target_dir)
```
The `TypeError` fallback is a duck-typed feature check: parsers that accept `gitignore_filter` (`SkillParser`) get it; those that don't (`SlashCommandParser`, `SubagentParser`) silently fail and retry without it. **Brittle.** Better: a uniform constructor signature. P1 refactor.

### Mixin composition

The whole `LazyClaude` action surface is composed of five mixins (`app.py:53-60`). MRO is left-to-right then up: `NavigationMixin → FilterMixin → MarketplaceMixin → CustomizationActionsMixin → HelpMixin → App`. Documented in `mixins/CLAUDE.md`.

### Reactive watch / message emit

The reactive→watch→post_message chain is the canonical UI update pipeline. Each panel emits `SelectionChanged` from `_emit_selection_message`, called by `watch_selected_index`, `watch_customizations`, `on_focus`. App receives, updates `MainPane.customization` (also reactive) which triggers `watch_customization` which calls `_refresh_display`.

### Confirm-then-act

Destructive ops always go through a confirm widget that emits `*Confirmed` or `*Cancelled`. The pattern is uniform: Delete, PluginToggle, MarketplaceRemove all use the same y/n/Esc skeleton. New destructive ops should follow.

### Auto-collapse heuristic

`marketplace_modal.py:373-378`: marketplaces with >20 plugins or 0 installed are collapsed by default. **Magic number 20** is hardcoded.

## Consistency assessment

| Convention | Application | Notes |
|---|---|---|
| File-per-class | Strong | Some `mixins/*.py` have helper functions alongside class |
| Type hints | Strong (mypy-enforced) | — |
| Docstrings on public | Strong | — |
| Error surfaces | **Mixed** | Parsers use `error` field; services silently swallow |
| Path literals | **Weak** | Scattered string literals for config paths |
| Subprocess invocation | **Weak** | Three styles in three places, `shell=True` misuse |
| Version parsing | **Weak (3× duplication)** | should be a single utility |
| Logging | **Absent** | — |
| Frontmatter parsing | Strong | `parse_frontmatter` reused by 4 parsers |
| Reactive widget update pattern | Strong | Every panel follows the same shape |
| Modal `add_class('visible')` pattern | Strong | — |
| Test naming | Strong | rule-enforced |

## State Checkpoint

```yaml
pass: 3
status: complete
timestamp: 2026-05-11T17:10:00Z
next_pass: 4
```
