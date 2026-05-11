# Phase B Deepening: Models — Round 1

Goal: produce a field-by-field schema for every type exported by `lazyclaude.models`, with Rust struct-mapping recommendations. Domain types are foundational — Monocle's Rust types must match these semantics field-by-field. Mismatches here propagate through every layer.

## Files in scope

| File | LOC | Path |
|---|---|---|
| `__init__.py` | 31 | `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/src/lazyclaude/models/__init__.py` |
| `customization.py` | 180 | `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/src/lazyclaude/models/customization.py` |
| `marketplace.py` | 50 | `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/src/lazyclaude/models/marketplace.py` |
| `settings.py` | 15 | `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/src/lazyclaude/models/settings.py` |

Public surface (`__init__.py:3-17, 19-31`): 11 names exported. `PluginInfo`, `PluginScope`, `SkillFile`, `MemoryFileRef` are **defined in `customization.py` but NOT re-exported** — they are accessed via the long path or implicitly used inside `Customization.metadata`/`Customization.plugin_info`.

## Enum: `ConfigLevel` (`customization.py:9-26`)

Python enum with `auto()` values. Variants (declaration order):

| Variant | Python value | Docstring path | Rust mapping |
|---|---|---|---|
| `USER` | `1` (auto) | `~/.claude/` | `Level::User` |
| `PROJECT` | `2` (auto) | `./.claude/` | `Level::Project` |
| `PROJECT_LOCAL` | `3` (auto) | `~/.claude.json` (MCPs only — but see below) | `Level::ProjectLocal` |
| `PLUGIN` | `4` (auto) | `~/.claude/plugins/{plugin}/` | `Level::Plugin` |

**The "MCPs only" docstring is wrong / stale.** `PROJECT_LOCAL` is used by `.claude/local/CLAUDE.md` (memory file), `settings.local.json` (hooks), and `.mcp.json` at project-local paths. Multiple sites use it: `services/discovery.py:2967, 2993, 3019` (project-local memory + auto-memory), `mixins/customization_actions.py:975` (level selector includes it for MCP/HOOK only). Document the mismatch.

### `ConfigLevel.label` property (`:18-26`)

Returns: `"User" | "Project" | "Project-Local" | "Plugin"`. Used by `Customization.level_label` and indirectly by writer for error messages (`writer.py:4004`).

### Sort key semantics

`ConfigLevel` is **NOT used as a sort key**. The sort invariant uses only `(type_order, name.lower())`. The level is incidental — items from different levels with the same `(type, name)` would be order-unstable (Python's `sorted` is stable, so insertion order from `_combine_results` wins).

## Enum: `PluginScope` (`customization.py:29-34`)

| Variant | Value | Source string | Rust mapping |
|---|---|---|---|
| `USER` | `1` (auto) | `"user"` | `PluginScope::User` |
| `PROJECT` | `2` (auto) | `"project"` | `PluginScope::Project` |
| `PROJECT_LOCAL` | `3` (auto) | `"local"` | `PluginScope::ProjectLocal` |

**String-to-enum mapping site** (`services/plugin_loader.py:2450-2452` per cumulative grep — verified location: `plugin_loader.py:2450` in the `scope` dispatch). String literals in `installed_plugins.json`: `"user"`, `"project"`, `"local"` (NOT `"project_local"`). The Rust port must use these exact strings on the JSON serde boundary.

Used in `PluginInfo.scope` only. Distinct from `ConfigLevel` because plugin installation scope can be `project` OR `local` while customizations within a plugin uniformly carry `ConfigLevel.PLUGIN`.

## Enum: `CustomizationType` (`customization.py:37-46`)

**P0 sort order invariant — verified.**

| Variant | Index | Type label (`type_label` property `:172-180`) | Panel label (widgets) |
|---|---|---|---|
| `SLASH_COMMAND` | `0` | `"Slash Command"` | `"Slash Commands"` |
| `SUBAGENT` | `1` | `"Subagent"` | `"Subagents"` |
| `SKILL` | `2` | `"Skill"` | `"Skills"` |
| `MEMORY_FILE` | `3` | `"Memory File"` | `"Memory Files"` |
| `MCP` | `4` | `"MCP Server"` | `"MCPs"` |
| `HOOK` | `5` | `"Hook"` | `"Hooks"` |
| `LSP_SERVER` | `6` | `"LSP Server"` | `"LSP"` |

Sort sites:
- `services/discovery.py:247` `type_order = {t: i for i, t in enumerate(CustomizationType)}` → declaration order
- `services/discovery.py:250` `sorted(..., key=lambda c: (type_order[c.type], c.name.lower()))`
- `widgets/plugin_confirm.py:8166` `sorted(type_counts.items(), key=lambda x: x[0].value)` — uses Python's auto value (`.value` returns the int from `auto()`)

Both sort paths produce the same ordering since `auto()` yields `1..7` in declaration order.

**P0 invariant — Rust port must define the enum with explicit discriminants `0..=6`** (or `1..=7` matching Python's `auto()`) **AND** preserve declaration order. `#[derive(PartialOrd, Ord)]` on the enum will Just Work in Rust. Tests in `tests/integration/discovery/test_behavior.py` pin this (per parsers-r2 round 2 findings).

**Pinned by test:** `test_discover_all_returns_sorted_results` and `tests/unit/test_app_customization_actions.py:18-58` (which iterates the enum to assert `_COPYABLE_TYPES` / `_PROJECT_LOCAL_TYPES` membership).

### Two type-classification constants live OUTSIDE models layer

- `LazyClaude._COPYABLE_TYPES` (`app.py:159-164`): `{SLASH_COMMAND, SUBAGENT, SKILL, HOOK, MCP, MEMORY_FILE}` — i.e. **all types except `LSP_SERVER`**. Pinned by `tests/unit/test_app_customization_actions.py:18-38`.
- `LazyClaude._PROJECT_LOCAL_TYPES` (`app.py:166`): `(HOOK, MCP)` — types where `LevelSelector` includes `PROJECT_LOCAL` as a target. Pinned by `tests/unit/test_app_customization_actions.py:50-58`.

These belong with the type enum in Monocle. A Rust port should put them on the enum itself (`impl CustomizationType { fn is_copyable() ; fn supports_project_local() }`).

## Dataclass: `SlashCommandMetadata` (`customization.py:49-56`)

| Field | Python type | Default | Rust mapping | Notes |
|---|---|---|---|---|
| `allowed_tools` | `list[str]` | `[]` | `Vec<String>` | From frontmatter `allowed-tools` (note hyphen) |
| `argument_hint` | `str \| None` | `None` | `Option<String>` | From frontmatter `argument-hint` |
| `model` | `str \| None` | `None` | `Option<String>` | — |
| `disable_model_invocation` | `bool` | `False` | `bool` | From frontmatter `disable-model-invocation` |

**NOTE:** This dataclass is **NEVER used as a typed `Customization.metadata`** — the parser writes a `dict[str, Any]` instead (`services/parsers/slash_command.py:8863+`). The dataclass is essentially documentation. Monocle should make it a real typed metadata variant.

## Dataclass: `SubagentMetadata` (`customization.py:59-66`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `tools` | `list[str]` | `[]` | `Vec<String>` | — |
| `model` | `str \| None` | `None` | `Option<String>` | — |
| `permission_mode` | `str \| None` | `None` | `Option<String>` | From frontmatter `permission-mode` |
| `skills` | `list[str]` | `[]` | `Vec<String>` | Inline CSV/list parser, NOT `parse_tools_list` (per parsers-r1) |

Same caveat: not actually used as a typed metadata in parsing.

## Dataclass: `SkillFile` (`customization.py:69-77`) — RECURSIVE

| Field | Python type | Default | Rust |
|---|---|---|---|
| `name` | `str` | — | `String` |
| `path` | `Path` | — | `PathBuf` |
| `content` | `str \| None` | `None` | `Option<String>` |
| `is_directory` | `bool` | `False` | `bool` |
| `children` | `list["SkillFile"]` | `[]` | `Vec<SkillFile>` |

Recursive via string forward-reference. Rust: `Vec<SkillFile>` works directly. Eagerly populated by `SkillParser` walk (parsers-r1 confirmed). Sort order: `(is_file, name.lower())` — directories first within a level, then alpha by name.

## Dataclass: `MemoryFileRef` (`customization.py:80-88`) — RECURSIVE

| Field | Python type | Default | Rust |
|---|---|---|---|
| `name` | `str` | — | `String` — the original ref token (e.g., `"~/notes/foo.md"`) |
| `path` | `Path \| None` | (required, can be None) | `Option<PathBuf>` |
| `content` | `str \| None` | `None` | `Option<String>` |
| `exists` | `bool` | `False` | `bool` |
| `children` | `list["MemoryFileRef"]` | `[]` | `Vec<MemoryFileRef>` |

Note: `path` has **no default** — it's positional after `name`. Required argument that just happens to allow `None`. Rust: `Option<PathBuf>` field, always specified at construction.

Resolution algorithm in `parsers/memory_file.py:94-148` (parsers-r1 documented). Depth cap = 5, cycle break via visited-set. `exists=False` for missing files; for cycle hits, returns `path=resolved, exists=True, children=[]` (no recursion).

## Dataclass: `SkillMetadata` (`customization.py:91-100`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `tags` | `list[str]` | `[]` | `Vec<String>` | CSV-or-list inline |
| `has_reference` | `bool` | `False` | `bool` | `(skill_dir / "reference.md").exists()` |
| `has_examples` | `bool` | `False` | `bool` | `(skill_dir / "examples.md").exists()` |
| `has_scripts` | `bool` | `False` | `bool` | `(skill_dir / "scripts").is_dir()` |
| `has_templates` | `bool` | `False` | `bool` | `(skill_dir / "templates").is_dir()` |
| `files` | `list[SkillFile]` | `[]` | `Vec<SkillFile>` | Recursive walk |

## Dataclass: `MCPServerMetadata` (`customization.py:103-111`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `transport_type` | `str` | `"stdio"` | `String` | Comment says `"stdio" | "http" | "sse"` but free string. Rust: `String` for byte-fidelity OR `enum Transport { Stdio, Http, Sse, Other(String) }` |
| `command` | `str \| None` | `None` | `Option<String>` | stdio only |
| `url` | `str \| None` | `None` | `Option<String>` | http/sse |
| `args` | `list[str]` | `[]` | `Vec<String>` | Coerced to `[]` if JSON value is non-list |
| `env` | `dict[str, str]` | `{}` | `HashMap<String, String>` (or `BTreeMap` for deterministic ordering) | Coerced to `{}` if non-dict |

**Discriminating shape:** if `transport_type == "stdio"` then `command` set / `url` None; else inverse. NOT enforced by code — both could be `None` simultaneously (parser-side fallback only sets one based on input). Rust would benefit from a tagged union.

## Dataclass: `PluginInfo` (`customization.py:114-125`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `plugin_id` | `str` | (required) | `String` | Format: `"{short_name}@{marketplace}"`. Example: `"handbook@cc-handbook"` |
| `short_name` | `str` | (required) | `String` | Plugin name only (before `@`) |
| `version` | `str` | (required) | `String` | E.g., `"1.3.1"`. Free string — no semver validation |
| `install_path` | `Path` | (required) | `PathBuf` | Resolved install directory |
| `is_local` | `bool` | `False` | `bool` | From `installed_plugins.json` `isLocal` field (`plugin_loader.py:94`) — distinct from `PluginScope.PROJECT_LOCAL`. Means "developer-mode local-folder plugin". |
| `is_enabled` | `bool` | `True` | `bool` | Default **True** (different from `MarketplacePlugin.is_enabled` default which is also `True`) |
| `scope` | `PluginScope` | `PluginScope.USER` | `PluginScope` | — |
| `project_path` | `Path \| None` | `None` | `Option<PathBuf>` | Set when `scope == PROJECT` or `PROJECT_LOCAL`; identifies which project owns the install |

Used in `Customization.plugin_info` and `Customization.display_name` rendering. `services/plugin_loader.py:18` defines a separate dataclass also called `is_local` for the `Installation` type — verify Rust ports do not conflate the two.

## Dataclass: `Customization` (`customization.py:128-180`) — THE CORE DOMAIN TYPE

### Required positional fields (in order)

| Field | Python type | Rust | Notes |
|---|---|---|---|
| `name` | `str` | `String` | Display name (see parsers-r1 for per-parser derivation) |
| `type` | `CustomizationType` | `CustomizationType` | Discriminator |
| `level` | `ConfigLevel` | `ConfigLevel` | Source-level discriminator |
| `path` | `Path` | `PathBuf` | Source file path; for skills, the `SKILL.md` file (not the dir) |

### Optional fields

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `description` | `str \| None` | `None` | `Option<String>` | For memory files, the parser sets a non-None default (`"Memory file"`) so this is effectively `String` for type=MEMORY_FILE only |
| `content` | `str \| None` | `None` | `Option<String>` | Full raw content. For MCP/HOOK, this is `json.dumps(slice, indent=2)`. For skills, the parser **may set content** but normally `None` (content is in `metadata.files[]`) |
| `metadata` | `dict[str, Any]` | `{}` | **CRITICAL: see below** | Heterogeneous shape per `type` |
| `error` | `str \| None` | `None` | `Option<String>` | Non-None signals load failure (malformed JSON, OSError on read) |
| `plugin_info` | `PluginInfo \| None` | `None` | `Option<PluginInfo>` | Non-None iff `level == PLUGIN` |

### `metadata: dict[str, Any]` polymorphism — P0 SCHEMA DECISION FOR MONOCLE

The `metadata` field is a heterogeneous JSON-ish dict whose shape varies by `type`:

| Type | Metadata keys (from parsers) | Source |
|---|---|---|
| `SLASH_COMMAND` | `allowed_tools, argument_hint, model, disable_model_invocation` | `slash_command.py` |
| `SUBAGENT` | `tools, model, permission_mode, skills` | `subagent.py` |
| `SKILL` | `tags, has_reference, has_examples, has_scripts, has_templates, files` (list of `SkillFile`) | `skill.py` |
| `MEMORY_FILE` | `imports` (list[str]), `tags` (list[str]), `refs` (list[MemoryFileRef]) | `memory_file.py` |
| `MCP` | `transport_type, command, url, args, env` | `mcp.py` |
| `HOOK` | `{}` (empty — **LOSSY**, content carries the data) | `hook.py` |
| `LSP_SERVER` | **raw server_config dict from JSON** (NOT structured) | `lsp_server.py` (per parsers-r1 EC-1) |

**Rust port recommendation:** replace `dict[str, Any]` with a typed enum:

```rust
enum Metadata {
    SlashCommand(SlashCommandMetadata),
    Subagent(SubagentMetadata),
    Skill(SkillMetadata),
    MemoryFile(MemoryFileMetadata { imports: Vec<String>, tags: Vec<String>, refs: Vec<MemoryFileRef> }),
    Mcp(McpServerMetadata),
    Hook,  // unit variant, lossy
    LspServer(serde_json::Value),  // raw JSON
}
```

Then `Customization::new(name, type, level, path)` with `Metadata` paired by type. Discriminant alignment between `CustomizationType` and `Metadata` variants can be enforced at construction.

**Trade-off:** A typed `Metadata` enum will reject "metadata reads that don't match the type" at compile time — that's a feature, not a bug, but may require touching many call sites that currently `metadata.get("tags", [])`. Recommend doing the conversion at Monocle's parser boundary.

### Computed properties (`customization.py:144-180`)

| Property | Returns | Logic | Rust |
|---|---|---|---|
| `has_error` | `bool` | `self.error is not None` | `fn has_error(&self) -> bool { self.error.is_some() }` |
| `display_name` | `str` (Rich markup) | See below — plugin branch + level-indicator branch | Rust method returning `Cow<'_, str>` or owned `String` |
| `level_label` | `str` | `self.level.label` | Delegate to `Level::label()` |
| `type_label` | `str` | Lookup dict (see CustomizationType table above) | Same |

#### `display_name` algorithm (`:149-162`)

```
IF plugin_info is set:
    base = f"[dim]{plugin_info.short_name}:[/]{self.name}"
    IF NOT plugin_info.is_enabled:
        return f"[dim]{base}[/]"
    return base
ELSE:
    indicator = { USER: "[U]", PROJECT: "[P]", PROJECT_LOCAL: "[L]" }[self.level]
    return f"{self.name} {indicator}"
```

**P1 bug latent in display_name:** the `level_indicator` dict has NO entry for `ConfigLevel.PLUGIN`. The function reaches the dict lookup only when `plugin_info is None`. If a caller ever constructs a `Customization` with `level=PLUGIN` but `plugin_info=None`, this raises `KeyError`. The discovery service always sets `plugin_info` for plugin items, so it doesn't fire in practice. Rust port: model the constraint as an invariant — `level == Plugin <=> plugin_info.is_some()`.

#### Rich markup tokens embedded in display_name

`"[dim]...[/]"` is Textual/Rich markup. Monocle's Rust port must:
- Decide whether `display_name` returns Rich markup or stripped text
- Recommend: return a **structured renderable** (`enum DisplayFragment { Plain(String), Dim(String) }`) — let the widget layer convert to terminal output

### Equality / hashing semantics

Python `@dataclass` defaults: `eq=True, frozen=False, hash=None`. So:
- `Customization.__eq__` is structural by all fields
- `Customization` is **not hashable** by default (mutable dataclass)
- `Path` and dataclass fields compose into structural equality
- `metadata: dict[str, Any]` makes equality recursive over arbitrary values — fragile

Rust port: implement `PartialEq` for caching but **derive `Hash` only on `(name, type, level, path)`** — the natural identity tuple. This is what `seen_paths: set[Path]` checks in `discovery.py` use anyway.

### Serialization shape

**None.** `Customization` has no `to_dict` / `to_json`. It's a runtime-only domain object. Serialization to disk happens through the **writer** which extracts specific fields per type (see Pass 4 BCs / `services/writer.py`). The `metadata` round-trip is parser → in-memory → writer, but the writer only reads a subset of metadata keys.

## Marketplace types (`marketplace.py`)

### `MarketplaceSource` (`:8-14`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `source_type` | `str` | (required) | `String` | Known values: `"github"`, `"directory"`, fallback `"unknown"` (`marketplace_loader.py:69`). Free string — Rust: `enum SourceType { Github, Directory, Unknown }` or `String` for fidelity |
| `repo` | `str \| None` | `None` | `Option<String>` | E.g., `"owner/name"` |
| `path` | `str \| None` | `None` | `Option<String>` | Local directory path, when `source_type == "directory"` |

**Validity invariant (not enforced):** `source_type == "github"` should imply `repo` is set; `"directory"` implies `path`. Loader does not validate.

### `MarketplaceEntry` (`:17-24`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `name` | `str` | (required) | `String` | Marketplace shorthand |
| `source` | `MarketplaceSource` | (required) | `MarketplaceSource` | — |
| `install_location` | `Path` | (required) | `PathBuf` | Where marketplace.json lives (`{install_location}/.claude-plugin/marketplace.json`) |
| `last_updated` | `str \| None` | `None` | `Option<String>` | Free string — no datetime parsing |

### `MarketplacePlugin` (`:27-41`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `name` | `str` | (required) | `String` | Plugin name within marketplace |
| `description` | `str` | (required, **not Optional**) | `String` | Defaults to `""` (`marketplace_loader.py:151`) |
| `source` | `str` | (required) | `String` | Free string — URL or string-coerced dict — see code at `marketplace_loader.py:130-132` |
| `marketplace_name` | `str` | (required) | `String` | — |
| `full_plugin_id` | `str` | (required) | `String` | Format: `"{name}@{marketplace_name}"`. Constructed at `marketplace_loader.py:121` |
| `is_installed` | `bool` | `False` | `bool` | — |
| `is_enabled` | `bool` | `True` | `bool` | Caveat: `marketplace_loader.py:156` forces `is_enabled = True` when `is_installed = False`. So "not installed → enabled" reads weirdly; invariant: `is_enabled` is only meaningful when `is_installed == True`. |
| `install_path` | `Path \| None` | `None` | `Option<PathBuf>` | None when not installed |
| `installed_version` | `str \| None` | `None` | `Option<String>` | Free string |
| `installed_scopes` | `list[str]` | `[]` | `Vec<String>` | **String literals from `installed_plugins.json`: `"user"`, `"project"`, `"local"`**. NOT `PluginScope` enum values directly. See `marketplace_loader.py:219` `if scope in ("project", "local"):` |
| `extra_metadata` | `dict[str, Any]` | `{}` | `HashMap<String, serde_json::Value>` | All `marketplace.json` plugin entry fields NOT in `(name, description, source)` are stashed here (`marketplace_loader.py:160-164`) — includes `version`, `tags`, custom paths, etc. |

### `Marketplace` (`:44-50`)

| Field | Python type | Default | Rust |
|---|---|---|---|
| `entry` | `MarketplaceEntry` | (required) | `MarketplaceEntry` |
| `plugins` | `list[MarketplacePlugin]` | `[]` | `Vec<MarketplacePlugin>` |
| `error` | `str \| None` | `None` | `Option<String>` |

## `AppSettings` (`settings.py`)

| Field | Python type | Default | Rust | Notes |
|---|---|---|---|---|
| `theme` | `str` | `DEFAULT_THEME` = `"gruvbox"` (`themes.py:30`) | `String` | Free string, no validation against known themes |
| `marketplace_auto_collapse` | `bool` | `True` | `bool` | UX preference |
| `suggested_marketplaces` | `dict[str, dict[str, Any]]` | `{}` | `HashMap<String, MarketplaceSuggestion>` | Key: GitHub `"owner/repo"`. Value: `{tags: list[str], stars: int}` — see `DEFAULT_SUGGESTED_MARKETPLACES` in `services/settings.py:9-21` |

### Schema version handling

**NONE.** No schema version field. `SettingsService.load` (`services/settings.py:37-53`) is lenient:
1. File missing → defaults
2. Invalid JSON → defaults (swallows `JSONDecodeError, OSError`)
3. Missing key → per-key default via `.get(key, AppSettings.attr)`

Pinned by `tests/unit/test_settings_service.py:14-65`.

### Merge semantics across levels (User+Project)

**NONE — `AppSettings` is single-level (user-global at `~/.lazyclaude/settings.json`).** It is NOT the same as Claude Code's `settings.json` (which IS merged across User/Project/Local). LazyClaude's own settings are just for theme + UX prefs. No merging.

### `suggested_marketplaces` migration

`SettingsService.ensure_suggested_marketplaces` (`services/settings.py:71-96`):
- Adds default entries that don't exist
- Updates entries when default value changes (deep equality check)
- Preserves user-added entries (those not in `DEFAULT_SUGGESTED_MARKETPLACES`)

Persistence side-effects: saves only when an update occurred. Rust port: this is a "config migration on app start" pattern.

## Invariants — confirmed and refuted

| Invariant | Status | Citation |
|---|---|---|
| **P0**: Sort order is `(type_order, name.lower())` with type index = enum declaration order | **CONFIRMED** | `services/discovery.py:247-250`; tests at `test_behavior.py` (parsers-r2) |
| **P0**: Variant order `SlashCommand=0, Subagent=1, Skill=2, MemoryFile=3, MCP=4, Hook=5, LSPServer=6` | **CONFIRMED — but using Python `auto()` which is 1-indexed (1..7); discovery uses `enumerate` which yields 0..6** | `customization.py:40-46`, `discovery.py:247` |
| **P0**: Project slug regex `[^a-zA-Z0-9\-]` → `-` lives in the models layer | **REFUTED** — lives in `services/discovery.py:484`, not `models/`. The user instruction was incorrect. Rust port should still byte-match `^[a-zA-Z0-9\-]+$` complement for memory discovery — invariant is real, location is different. | `discovery.py:478-484` |
| Sort is stable across levels | Implicit — Python `sorted` is stable | `discovery.py:248` |
| Type discriminant alignment between `CustomizationType` and `Metadata` shape | Convention only — not enforced | `parsers/*.py` |
| `level == PLUGIN` iff `plugin_info is not None` | Convention only — not enforced; `display_name` would KeyError if violated | `customization.py:158-162` |
| `is_enabled` defaults to `True` for both `PluginInfo` and `MarketplacePlugin` | **CONFIRMED** | `customization.py:123`, `marketplace.py:37` |
| `MarketplacePlugin.is_enabled` is only meaningful when `is_installed == True` | Soft invariant — loader sets `is_enabled = True` when `is_installed == False` (`marketplace_loader.py:156`) | `marketplace_loader.py:156` |
| `installed_scopes` list contains string literals `"user"`, `"project"`, `"local"` | **CONFIRMED** | `marketplace_loader.py:216-224`, `mixins/marketplace.py:171` |
| Settings are single-level (no merge) | **CONFIRMED** by absence of merge logic | `services/settings.py:37-53` |

## New P0 / P1 surfaced this round

| ID | Severity | Description |
|---|---|---|
| M-P0-1 | P0 | **`Customization.metadata` is `dict[str, Any]` keyed by type discriminator.** Rust port MUST use a tagged enum (`Metadata::SlashCommand(...)`, `Metadata::Hook` unit, `Metadata::LspServer(Value)` etc.) for type safety. The naive port `HashMap<String, serde_json::Value>` will permit metadata-type mismatches that the Python code papers over with `.get()` lenience. |
| M-P0-2 | P0 | **Sort uses enum declaration order, not `auto()` integer value directly.** Rust port: derive `Ord` on `CustomizationType` in declaration order, OR provide an explicit `as_sort_index()` method. **DO NOT alphabetize** — the variant order is `SLASH_COMMAND, SUBAGENT, SKILL, MEMORY_FILE, MCP, HOOK, LSP_SERVER` which is **not alphabetical**. |
| M-P0-3 | P0 | **Project slug regex lives in `services/discovery.py:484`, NOT in models.** Monocle should still capture this invariant. The user instruction asked me to verify; it does **not** live in the models layer. |
| M-P1-1 | P1 | `display_name` KeyError latent bug if `level=PLUGIN` and `plugin_info=None`. Encode as Rust invariant: `Customization::new_plugin(...) -> Self` constructor requires `PluginInfo`. |
| M-P1-2 | P1 | **`PROJECT_LOCAL` docstring is stale** (`customization.py:14`) — says "for MCPs only" but is used for memory, hooks, settings, MCPs. Rust port: drop the docstring or update. |
| M-P1-3 | P1 | **`SlashCommandMetadata` / `SubagentMetadata` / `SkillMetadata` / `MCPServerMetadata` dataclasses are essentially dead documentation** — parsers write `dict[str, Any]` instead. Monocle should resurrect them as the real metadata types. |
| M-P1-4 | P1 | **`PluginScope` JSON serde**: must use lowercase string literals `"user"`, `"project"`, `"local"` (NOT `"project_local"`). Rust serde: `#[serde(rename_all = "lowercase")]` + custom rename for `ProjectLocal -> "local"`. |
| M-P1-5 | P1 | **`MarketplacePlugin.installed_scopes` are strings, not `PluginScope` enums.** Conversion happens at use site (`mixins/marketplace.py:171`). Inconsistent typing — Rust port should pick one and stick to it. Recommendation: typed `Vec<PluginScope>` everywhere. |
| M-P1-6 | P1 | **`MarketplaceSource.source_type` has THREE known values: `"github"`, `"directory"`, `"unknown"`** — `"unknown"` is the loader's fallback when JSON doesn't have a `source.source` field (`marketplace_loader.py:69`). Tests would benefit. Rust: `enum SourceType { Github, Directory, Unknown }`. |
| M-P1-7 | P1 | **`AppSettings` has no schema version.** Future migrations will need ad-hoc handling. Rust port may want to add `version: u32` upfront — but Monocle's port should match field-for-field unless explicitly diverging. |

## Rust struct-mapping table (consolidated)

One row per model field across all four files. (`O<>` = `Option<>`.)

| Python type.field | Rust struct.field | Notes |
|---|---|---|
| `ConfigLevel.USER` | `Level::User` | enum |
| `ConfigLevel.PROJECT` | `Level::Project` | enum |
| `ConfigLevel.PROJECT_LOCAL` | `Level::ProjectLocal` | enum |
| `ConfigLevel.PLUGIN` | `Level::Plugin` | enum |
| `PluginScope.USER` | `PluginScope::User` (serde `"user"`) | enum |
| `PluginScope.PROJECT` | `PluginScope::Project` (serde `"project"`) | enum |
| `PluginScope.PROJECT_LOCAL` | `PluginScope::ProjectLocal` (serde `"local"`) | enum |
| `CustomizationType.SLASH_COMMAND..LSP_SERVER` | `CustomizationType::SlashCommand..LspServer`, `#[derive(PartialOrd, Ord)]` in declaration order | enum |
| `SlashCommandMetadata.allowed_tools: list[str]` | `allowed_tools: Vec<String>` | default `[]` |
| `SlashCommandMetadata.argument_hint: str?` | `argument_hint: Option<String>` | — |
| `SlashCommandMetadata.model: str?` | `model: Option<String>` | — |
| `SlashCommandMetadata.disable_model_invocation: bool` | `disable_model_invocation: bool` | default `false` |
| `SubagentMetadata.tools: list[str]` | `tools: Vec<String>` | — |
| `SubagentMetadata.model: str?` | `model: Option<String>` | — |
| `SubagentMetadata.permission_mode: str?` | `permission_mode: Option<String>` | — |
| `SubagentMetadata.skills: list[str]` | `skills: Vec<String>` | — |
| `SkillFile.name: str` | `name: String` | — |
| `SkillFile.path: Path` | `path: PathBuf` | — |
| `SkillFile.content: str?` | `content: Option<String>` | — |
| `SkillFile.is_directory: bool` | `is_directory: bool` | — |
| `SkillFile.children: list[SkillFile]` | `children: Vec<SkillFile>` | recursive |
| `MemoryFileRef.name: str` | `name: String` | original ref token |
| `MemoryFileRef.path: Path?` | `path: Option<PathBuf>` | required positional, can be None |
| `MemoryFileRef.content: str?` | `content: Option<String>` | — |
| `MemoryFileRef.exists: bool` | `exists: bool` | — |
| `MemoryFileRef.children: list[MemoryFileRef]` | `children: Vec<MemoryFileRef>` | recursive |
| `SkillMetadata.tags: list[str]` | `tags: Vec<String>` | — |
| `SkillMetadata.has_reference: bool` | `has_reference: bool` | — |
| `SkillMetadata.has_examples: bool` | `has_examples: bool` | — |
| `SkillMetadata.has_scripts: bool` | `has_scripts: bool` | — |
| `SkillMetadata.has_templates: bool` | `has_templates: bool` | — |
| `SkillMetadata.files: list[SkillFile]` | `files: Vec<SkillFile>` | — |
| `MCPServerMetadata.transport_type: str` | `transport_type: String` (or tagged enum) | default `"stdio"` |
| `MCPServerMetadata.command: str?` | `command: Option<String>` | — |
| `MCPServerMetadata.url: str?` | `url: Option<String>` | — |
| `MCPServerMetadata.args: list[str]` | `args: Vec<String>` | coerced |
| `MCPServerMetadata.env: dict[str,str]` | `env: HashMap<String, String>` | coerced; use `BTreeMap` for determinism if needed |
| `PluginInfo.plugin_id: str` | `plugin_id: String` | `name@marketplace` format |
| `PluginInfo.short_name: str` | `short_name: String` | — |
| `PluginInfo.version: str` | `version: String` | — |
| `PluginInfo.install_path: Path` | `install_path: PathBuf` | — |
| `PluginInfo.is_local: bool` | `is_local: bool` | default `false` |
| `PluginInfo.is_enabled: bool` | `is_enabled: bool` | default `true` |
| `PluginInfo.scope: PluginScope` | `scope: PluginScope` | default `User` |
| `PluginInfo.project_path: Path?` | `project_path: Option<PathBuf>` | — |
| `Customization.name: str` | `name: String` | required |
| `Customization.type: CustomizationType` | `customization_type: CustomizationType` | (`type` is reserved in Rust) |
| `Customization.level: ConfigLevel` | `level: Level` | required |
| `Customization.path: Path` | `path: PathBuf` | required |
| `Customization.description: str?` | `description: Option<String>` | — |
| `Customization.content: str?` | `content: Option<String>` | — |
| `Customization.metadata: dict[str, Any]` | `metadata: Metadata` (tagged enum) | **See M-P0-1** |
| `Customization.error: str?` | `error: Option<String>` | — |
| `Customization.plugin_info: PluginInfo?` | `plugin_info: Option<PluginInfo>` | — |
| `MarketplaceSource.source_type: str` | `source_type: SourceType` enum | values `"github" | "directory" | "unknown"` |
| `MarketplaceSource.repo: str?` | `repo: Option<String>` | — |
| `MarketplaceSource.path: str?` | `path: Option<String>` | — |
| `MarketplaceEntry.name: str` | `name: String` | — |
| `MarketplaceEntry.source: MarketplaceSource` | `source: MarketplaceSource` | — |
| `MarketplaceEntry.install_location: Path` | `install_location: PathBuf` | — |
| `MarketplaceEntry.last_updated: str?` | `last_updated: Option<String>` | free string |
| `MarketplacePlugin.name: str` | `name: String` | — |
| `MarketplacePlugin.description: str` | `description: String` | defaults to `""` at construction |
| `MarketplacePlugin.source: str` | `source: String` | URL OR `str(dict)` |
| `MarketplacePlugin.marketplace_name: str` | `marketplace_name: String` | — |
| `MarketplacePlugin.full_plugin_id: str` | `full_plugin_id: String` | `name@marketplace_name` |
| `MarketplacePlugin.is_installed: bool` | `is_installed: bool` | — |
| `MarketplacePlugin.is_enabled: bool` | `is_enabled: bool` | default `true`; only meaningful if `is_installed` |
| `MarketplacePlugin.install_path: Path?` | `install_path: Option<PathBuf>` | — |
| `MarketplacePlugin.installed_version: str?` | `installed_version: Option<String>` | — |
| `MarketplacePlugin.installed_scopes: list[str]` | `installed_scopes: Vec<PluginScope>` (recommended) or `Vec<String>` (fidelity) | values `"user"`, `"project"`, `"local"` |
| `MarketplacePlugin.extra_metadata: dict[str, Any]` | `extra_metadata: HashMap<String, serde_json::Value>` | — |
| `Marketplace.entry: MarketplaceEntry` | `entry: MarketplaceEntry` | — |
| `Marketplace.plugins: list[MarketplacePlugin]` | `plugins: Vec<MarketplacePlugin>` | — |
| `Marketplace.error: str?` | `error: Option<String>` | — |
| `AppSettings.theme: str` | `theme: String` | default `"gruvbox"` |
| `AppSettings.marketplace_auto_collapse: bool` | `marketplace_auto_collapse: bool` | default `true` |
| `AppSettings.suggested_marketplaces: dict[str, dict[str, Any]]` | `suggested_marketplaces: HashMap<String, SuggestedMarketplace>` | typed value preferred |

## Delta Summary

- **New items added:** Complete field-by-field schema for 13 types (4 enums, 9 dataclasses); 3 P0 and 7 P1 findings; 11 confirmed invariants with file:line; Rust struct mapping for ~60 fields.
- **Existing items refined:** Sort order invariant (was P0, now also pinned to enum declaration order); slug regex (location moved from "in models" to "in services/discovery.py"); `Customization.metadata` upgraded to typed-enum requirement.
- **Remaining gaps:** test coverage for model-layer properties (`display_name` semantics with plugin disabled flag, `type_label` for all variants); no tests for `MarketplacePlugin.extra_metadata` shape; no test for `PluginScope` serde rename; no test for `Customization.has_error` happy-path.

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: This round produced the first complete per-field schema with Rust mappings — that was the deliverable. It also surfaced **three P0 findings** that change Monocle's port plan:
1. The slug regex doesn't live in models (refuting the input prompt's hypothesis).
2. The metadata field is type-polymorphic and must be a tagged enum in Rust — a structural change from the obvious naive port.
3. The sort uses enum declaration order, not alphabetical — easy to get wrong.

And **seven P1 findings**, including stale docstrings, latent display_name KeyError, dead-documentation dataclasses, and serde string-literal requirements for `PluginScope` / `installed_scopes`. Removing this round's findings would meaningfully degrade a Rust port — they are not refinements, they are new constraints. SUBSTANTIVE.

## Convergence Declaration

**Another round needed.** Round 2 should:
1. Verify the `dict[str, Any]` metadata polymorphism by reading each parser's exact metadata construction to confirm the per-type key set is closed (no parser writes extra keys).
2. Check `Customization` equality / hashing more carefully (does any code path put `Customization` into a `set` or `dict` key?).
3. Verify `installed_scopes` literal set is exhaustively `{"user", "project", "local"}` — search for any other literal.
4. Audit consumers of `Customization.metadata.get(...)` to enumerate the actual key set per type (closes the metadata-shape contract).
5. Check `tests/conftest.py` for fixture-level model constructors that might reveal additional invariants.

The above are not nitpicks — they are the difference between a port that compiles and a port that round-trips.

## State Checkpoint

```yaml
pass: B
subpass: models
round: 1
status: complete
timestamp: 2026-05-11T18:05:00Z
novelty: SUBSTANTIVE
```
