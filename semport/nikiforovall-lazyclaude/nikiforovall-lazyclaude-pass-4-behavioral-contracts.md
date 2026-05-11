# Pass 4: Behavioral Contracts — nikiforovall/lazyclaude

This pass extracts the canonical schemas and contracts of the parser layer (the principal gene material for Monocle) plus the discovery walker. Contracts are stated as input → output → preconditions → invariants → known edge cases.

## The seven parsers (per-customization-type schema)

### BC-1: SlashCommandParser

- **File pattern:** `commands/**/*.md` (recursive). Scan strategy: `GlobStrategy.RGLOB` (`discovery.py:34-39`).
- **Citation:** `services/parsers/slash_command.py`
- **Input shape:** Markdown file with optional YAML frontmatter:
  ```yaml
  ---
  description: "Optional explicit description"
  allowed-tools: Read, Edit, Bash         # CSV string or YAML list
  argument-hint: "<file> [flags]"
  model: "claude-opus-4"
  disable-model-invocation: false
  ---
  Body content...
  ```
- **Output `Customization`:**
  - `name`: derived from path relative to `commands/`. Nested `commands/git/log.md` → `"git:log"` (colon-separated parts, file extension stripped) — `slash_command.py:76-89`. Flat `cmd.md` → `"cmd"`. Fallback `path.stem` if `relative_to` fails.
  - `type`: `CustomizationType.SLASH_COMMAND`
  - `description`: explicit frontmatter `description` if present, else **first non-`#` line of body, truncated to 100 chars** (`slash_command.py:53-57`). If body empty, `description = None`.
  - `content`: full raw file content
  - `metadata`: `SlashCommandMetadata.__dict__` with fields:
    - `allowed_tools: list[str]` — from `allowed-tools` (hyphen-key), parsed by `parse_tools_list` (`parsers/__init__.py:67-73`): if list → list-of-strs; if CSV string → split-strip; if `None` → `[]`
    - `argument_hint: str | None` — `argument-hint`
    - `model: str | None`
    - `disable_model_invocation: bool` — default `False`
- **Preconditions:** None; missing file path returns error-customization on `OSError`.
- **Invariants:**
  - Name uniqueness within a level not enforced — discovery doesn't dedupe by name.
  - Frontmatter keys use **hyphen-kebab-case** in YAML; parser maps to **snake_case** dataclass fields.
- **Known edge cases handled:**
  - Read failure → error-customization (`slash_command.py:42-49`).
  - No frontmatter → `({}, content)` from `parse_frontmatter` → description heuristic kicks in.
  - YAML parse failure → silently treated as no frontmatter (`parsers/__init__.py:62-63`).
  - Body starts with `#` heading → description is `None` if no fallback line found.
- **Known edge cases NOT handled:**
  - Duplicate filenames at User vs Project levels: produce two Customizations with same name, sorted side-by-side (no merge / conflict UI).
  - `disable-model-invocation` is **always read as bool** — non-bool values pass through `.get(..., False)` returning whatever was there (e.g., string `"yes"`).
  - Description truncation at 100 char doesn't add ellipsis (`slash_command.py:57`).

### BC-2: SubagentParser

- **File pattern:** `agents/*.md` (flat, **non-recursive**). Scan strategy: `GlobStrategy.GLOB` (`discovery.py:40-45`).
- **Citation:** `services/parsers/subagent.py`
- **Input shape:** Markdown file with optional YAML frontmatter:
  ```yaml
  ---
  name: "explicit-name-override"          # optional, falls back to path.stem
  description: "..."
  tools: Read, Edit                       # CSV or list
  model: "..."
  permission-mode: "ask" | "auto" | "deny"
  skills: skillA, skillB                  # CSV or list
  ---
  Body...
  ```
- **Output `Customization`:**
  - `name`: frontmatter `name` if set, else `path.stem` (`subagent.py:53`).
  - `type`: `CustomizationType.SUBAGENT`
  - `description`: only from frontmatter — no body fallback (unlike SlashCommand).
  - `content`: raw file content
  - `metadata`: `SubagentMetadata.__dict__` with `tools`, `model`, `permission_mode`, `skills` list.
- **Preconditions:** none.
- **Invariants:**
  - Name precedence: frontmatter > filename.
- **Edge cases handled:**
  - Read failure → error-customization (`subagent.py:42-49`).
  - `skills` CSV split logic is **inlined and duplicated** rather than reusing `parse_tools_list` (`subagent.py:56-62`). Could be reused.
- **NOT handled:**
  - Filename and frontmatter name disagree silently — operator may be confused which is canonical.

### BC-3: SkillParser

- **File pattern:** `skills/*/SKILL.md` — one skill = one directory containing `SKILL.md`. Scan strategy: `GlobStrategy.SUBDIR` (`discovery.py:46-51`).
- **Citation:** `services/parsers/skill.py`
- **Input shape:** Directory:
  ```
  skills/my-skill/
    SKILL.md                # frontmatter + body
    reference.md            # optional, flag turns on has_reference
    examples.md             # optional, flag turns on has_examples
    scripts/                # optional dir
    templates/              # optional dir
    <any other files/dirs>
  ```
  Frontmatter:
  ```yaml
  ---
  name: "explicit-name-override"
  description: "..."
  tags: tagA, tagB                       # CSV or list
  ---
  ```
- **Output `Customization`:**
  - `name`: frontmatter `name` if set, else `skill_dir.name` (`skill.py:115`).
  - `path`: **the `SKILL.md` file**, not the directory (`skill.py:142`).
  - `type`: `CustomizationType.SKILL`
  - `metadata`: `SkillMetadata.__dict__`:
    - `tags: list[str]`
    - `has_reference: bool` — `(skill_dir / "reference.md").exists()` (`skill.py:132`)
    - `has_examples: bool` — `(skill_dir / "examples.md").exists()`
    - `has_scripts: bool` — `(skill_dir / "scripts").is_dir()`
    - `has_templates: bool` — `(skill_dir / "templates").is_dir()`
    - `files: list[SkillFile]` — recursive tree of all files/subdirs in the skill, excluding `SKILL.md` and hidden (dot-prefixed) entries (`skill.py:19-69`)
- **Preconditions:** the skill dir must contain a readable `SKILL.md`.
- **Invariants:**
  - Hidden files/dirs (`name.startswith(".")`) are always excluded.
  - `_read_skill_files` honors the gitignore filter for directories (skips `node_modules`, `.git`, etc.). **But not for files** — a file matching a gitignore pattern in `scripts/` will still be included.
  - File contents are eagerly read into `SkillFile.content` (memory cost for large skills).
- **Edge cases handled:**
  - Skill dir iteration `OSError` → empty file list (`skill.py:32-34`).
  - Per-file `OSError | UnicodeDecodeError` on read → `SkillFile.content = None` (`skill.py:57-60`).
- **NOT handled:**
  - Symlink cycles within skill dirs.
  - Skill dir with no `SKILL.md` → not discovered at all (filtered out by `_get_files` SUBDIR strategy `filesystem_scanner.py:96-115`).

### BC-4: MemoryFileParser

- **File patterns (5 distinct sources):** explicit at `discovery._discover_memory_files()` and `_discover_auto_memory()`:
  1. `~/.claude/CLAUDE.md` (USER)
  2. `~/.claude/AGENTS.md` (USER)
  3. `~/.claude/CLAUDE.local.md` (USER, the local-user override)
  4. `./.claude/CLAUDE.md`, `./.claude/AGENTS.md`, `./CLAUDE.md`, `./AGENTS.md` (PROJECT) — first match wins; subsequent are dedupe-skipped (`discovery.py:437-448`)
  5. Recursive walk of `project_root` for nested `CLAUDE.md` within `DEFAULT_MAX_WALK_DEPTH=5` (`discovery.py:450-462`)
  6. `./CLAUDE.local.md`, `./.claude/CLAUDE.local.md` (PROJECT_LOCAL)
  7. `~/.claude/projects/<project-slug>/memory/MEMORY.md` and `~/.claude/projects/<project-slug>/memory/*.md` (PROJECT_LOCAL, "auto memory") — `discovery.py:486-529`
  8. Rules: `~/.claude/rules/**/*.md` and `./.claude/rules/**/*.md` — parsed by `MemoryFileParser` but with `Customization.name = relative path to rules dir` (`discovery.py:531-569`)
- **Citation:** `services/parsers/memory_file.py`
- **Input shape:** Markdown with optional frontmatter and `@path/to/another.md` style imports in the body.
- **Output `Customization`:**
  - `name`: `path.name` (e.g. `"CLAUDE.md"`); overridden in rules-discovery to be the rules-relative path.
  - `type`: `CustomizationType.MEMORY_FILE`
  - `description`: first non-`#`, non-`@` line truncated to 100 chars; fallback to `"Memory file"` (`memory_file.py:50-58`).
  - `content`: raw text
  - `metadata`:
    - `imports: list[str]` — captured by regex `r"@([\w./~-]+\.md)"` against body (`memory_file.py:47`). **Frontmatter `@`-refs are not captured** because the parser passes `body` to the regex, not `content`.
    - `tags: list` — from frontmatter
    - `refs: list[MemoryFileRef]` — resolved tree of imports (see below)
- **Reference resolution algorithm** (`memory_file.py:77-148`):
  - Constants: `MAX_IMPORT_DEPTH = 5`.
  - Path resolution rules (`_resolve_path:140`):
    - `~/...` → `Path.home() / ...`
    - `/...` or `C:` style → `Path(ref)` absolute
    - Otherwise → `base_dir / ref` (relative to including file's directory)
  - Cycle detection: `visited: set[Path]` of resolved paths; if seen → return ref with `exists=True` but no recursion.
  - Depth limit: at depth >= 5 → return ref with `path=None, exists=False`.
  - Per-file read failure: ref with `exists=True, content=None`, no children.
  - Recursive: matches nested `@import` regex against resolved file content.
- **Auto-memory synthesis** (`discovery.py:496-522`): For `~/.claude/projects/<slug>/memory/MEMORY.md`, the parser is run normally, and **any sibling `*.md` topic file not already imported is synthesized as a `MemoryFileRef` with `exists=True`** and appended to `refs`. Mechanism: project slug = `re.sub(r"[^a-zA-Z0-9\-]", "-", str(project_root))` — matches Claude Code's own slugging convention (`discovery.py:478-484`).
- **Edge cases NOT handled:**
  - Imports in frontmatter are silently ignored.
  - Description heuristic skips lines starting with `#` and `@` but not blockquotes (`>`) or HTML.

### BC-5: MCPParser

- **File patterns:**
  - `~/.claude.json` (USER) — reads from wrapped `{"mcpServers": {...}}`
  - `./.mcp.json` (PROJECT) — reads from wrapped or unwrapped (root dict is treated as `mcpServers` dict if no `mcpServers` key)
  - `~/.claude.json` `projects[<project_path>].mcpServers` (PROJECT_LOCAL) — handled by `_discover_local_mcps` (`discovery.py:588-620`) which calls `MCPParser.parse_server_config` directly per server
  - Plugin: `<plugin>/.mcp.json` (PLUGIN)
- **Citation:** `services/parsers/mcp.py`
- **Output:** **List[Customization]** — one Customization per server entry. **This violates the `ICustomizationParser.parse` interface** (which returns a single `Customization`) — silently overridden with `# type: ignore[override]`. There's a separate `parse_single` that exists to satisfy the interface (`mcp.py:110-127`) but is not used by discovery.
- **Per-server `Customization`:**
  - `name`: server key from JSON
  - `type`: `CustomizationType.MCP`
  - `path`: the source JSON file (the same path is shared across all server-customizations from that file)
  - `description`: synthesized: `"{STDIO|HTTP|SSE} {server: <url> | command: <cmd> | server}"` (`mcp.py:85-90`)
  - `content`: `json.dumps(server_config, indent=2)` — **per-server slice**, not the whole file
  - `metadata`: `MCPServerMetadata.__dict__`:
    - `transport_type: str` — defaults to `"stdio"` (`mcp.py:79`)
    - `command: str | None`, `url: str | None`, `args: list`, `env: dict`
- **Cross-scope path-key fuzzing** (`discovery.py:600-607`): for PROJECT_LOCAL lookups, the discovery service tries both `/`-normalized and `\\`-normalized project paths against `projects` keys. Windows-path tolerance.
- **Edge cases handled:**
  - File parse failure → single error-customization (`mcp.py:40-49`).
  - Server-config not a dict → skipped (`mcp.py:63-64`).
  - `args` or `env` of wrong type → coerced to `[]` / `{}` (`mcp.py:96-97`).
- **NOT handled:**
  - Server with both `command` and `url` set — both stored; description uses `command` form for stdio (`mcp.py:85-90`).
  - **Default `transport_type` is `"stdio"` if missing** — but if the field is present with `"http"` and `url` is missing, the description falls through to `"HTTP server"` without warning.

### BC-6: HookParser

- **File patterns:**
  - `~/.claude/settings.json` (USER)
  - `./.claude/settings.json` (PROJECT)
  - `./.claude/settings.local.json` (PROJECT_LOCAL)
  - `<plugin>/hooks/hooks.json` (PLUGIN)
- **Citation:** `services/parsers/hook.py`
- **Output:** **List[Customization]** with 0 or 1 element — `[]` if no `hooks` key or empty; one customization otherwise. **Again signature-override.**
- **Single output Customization:**
  - `name`: `"hooks"` if file is `hooks.json`; else the source filename (e.g., `"settings.json"`)
  - `type`: `CustomizationType.HOOK`
  - `description`: comma-joined hook event names (e.g., `"PreToolUse, PostToolUse"`)
  - `content`: pretty-printed JSON of the `hooks` sub-dict
  - `metadata`: empty `{}` — **no structured metadata captured**; the granular hook entries are only in `content`.
- **Edge cases handled:**
  - Parse failure → single error-customization.
- **NOT handled:**
  - Per-event detail extraction (the consumer must re-parse `content` JSON to display individual hooks). Loses fidelity vs MCP parser which gives per-server detail.

### BC-7: LSPServerParser

- **File patterns:**
  - `<plugin>/.lsp.json` (PLUGIN) — language → server config dict
  - `<plugin>/.claude-plugin/plugin.json` `lspServers` key (PLUGIN) — alternative source
- **Citation:** `services/parsers/lsp_server.py`
- **Output:** **List[Customization]** per language entry.
- **Per-language Customization:**
  - `name`: language name (JSON key, e.g., `"python"`)
  - `type`: `CustomizationType.LSP_SERVER`
  - `description`: `"{TRANSPORT} command: <cmd>"` or `"{TRANSPORT} server"` (`lsp_server.py:73-76`)
  - `content`: pretty-printed JSON of the server-config dict
  - `metadata`: **the raw server_config dict itself** (`lsp_server.py:85`) — inconsistent with the dataclass-`__dict__` pattern used elsewhere.
- **Two entry methods:** `parse()` (for `.lsp.json`) and `parse_plugin_json()` (for `plugin.json` — reads `lspServers` key).
- **Edge cases handled:** Parse failure / non-dict data → empty list or error-customization.
- **NOT handled:** Conflict between `.lsp.json` and `plugin.json` LSP servers (both are merged via `_discover_plugin_lsp_servers` `discovery.py:701-722`).

## BC-8: Frontmatter parsing (shared)

`services/parsers/__init__.py:44-64`:

- **Regex:** `r"^---\s*\n(.*?)\n---\s*\n(.*)$"` with `re.DOTALL`.
- **Returns:** `(dict, body)`. On YAML parse failure, returns `({}, original_content)` — **lossy**.
- **`yaml.safe_load` only** — no executable YAML constructs accepted.
- **Edge case:** files starting with `---` but no closing `---` → no match → no frontmatter recognized.
- **Edge case:** literal `---` lines inside a fenced code block before any real frontmatter terminator → won't be misparsed because the regex anchors to the start of the document.

## BC-9: Discovery walker contract

`ConfigDiscoveryService.discover_all` (`discovery.py:158-186`):

### Order of operations

1. Scan `SCAN_CONFIGS` (commands/agents/skills) at USER then PROJECT.
2. `_discover_memory_files` (user + project + nested project + project-local files).
3. `_discover_auto_memory` (synthesized CLAUDE-Code-conventional projects dir).
4. `_discover_rules` (user + project rules dirs).
5. `_discover_mcps` (user `~/.claude.json` + project-local from same file's `projects` key + project `./.mcp.json`).
6. `_discover_hooks` (user settings + project settings + project-local settings).
7. `_discover_plugins` (every plugin's commands/agents/skills + `.mcp.json` + `hooks/hooks.json` + `.lsp.json` + `plugin.json` LSP servers).
8. Sort by `(CustomizationType.value, name.lower())`.
9. Cache in `self._cache`. Subsequent calls return cached unless `refresh()` clears.

### Plugin scoping (three phases, `plugin_loader.py:108-157`)

1. **User-scoped:** every installation with `scope == "user"`.
2. **Project-scoped:** for each `plugin_id` in this project's `settings.json.enabledPlugins`, find installations with `scope=="project"` AND matching `projectPath`.
3. **Local-scoped:** same as project but driven by `settings.local.json.enabledPlugins`, scope `"local"`.

`_matches_current_project` (`plugin_loader.py:159-166`) compares `Path(installation.project_path).resolve() == self.project_root.resolve()`.

### Deduplication

- **Memory files use `set[Path] resolve()`** — `discovery.py:419` etc. — to dedupe across overlapping discovery sources.
- **Plugin preview** uses `seen_paths = {c.path.resolve() for c in ...}` to avoid re-discovering files already found by the SCAN_CONFIGS pass when marketplace-extras specify custom paths (`discovery.py:227-231`).
- **No deduplication across levels** — the same logical name appearing at User and Project is two separate Customizations.

### Conflict resolution

**There is no merge.** If a slash command `lint` exists at both USER and PROJECT, both appear in the listing; the level indicator `[U]` / `[P]` differentiates. The filter `a/u/p/P` narrows the view. **This is intentional and the user disambiguates visually.**

### Marketplace-driven custom paths

When previewing a plugin from a marketplace, the plugin's `marketplace.json` entry may specify custom relative paths (`commands`, `agents`, `skills`, `mcpServers`, `hooks`) that override the default layout (`discovery.py:253-302`). These augment the standard SCAN_CONFIGS pass; deduplication via `seen_paths` prevents double-counting.

## BC-10: Filter contract (`FilterService.filter`)

`services/filter.py:60-84`:

- Input: `(customizations, query, level, plugin_enabled)`. All filters are AND-composed.
- Order of application: level → plugin_enabled → query. Maintains original list order.
- **Level matching** (`_matches_level:86-107`):
  - Direct equality.
  - **`PROJECT` filter also matches `PROJECT_LOCAL` items.**
  - **`PROJECT` filter also matches plugins whose `plugin_info.scope` is `PROJECT` or `PROJECT_LOCAL`.**
- **Query matching** (`_matches_query:109-118`):
  - Substring on `name.lower()`.
  - For plugin customizations, also matches `<short_name>:<name>` prefix form.
- **Plugin enabled** filter compares `plugin_info.is_enabled == plugin_enabled`. **Non-plugin customizations always pass the enabled filter** (`filter.py:77-78`).

## BC-11: Writer contract (`CustomizationWriter`)

`services/writer.py`:

### Copy

- **Slash command**: nested name `foo:bar:baz` is reconstructed as nested directory `commands/foo/bar/baz.md` (`writer.py:383-390`).
- **Subagent**: flat — `agents/<name>.md` (`writer.py:392-393`).
- **Skill**: `shutil.copytree(source_dir, target_dir, dirs_exist_ok=False)` — copies the **entire directory** (`writer.py:420-432`).
- **Memory file**: target = `base / <filename>` (`writer.py:398-399`).
- **MCP**: JSON merge into target file's `mcpServers` dict; for `PROJECT_LOCAL` it nests under `projects[<project_path>].mcpServers` (`writer.py:178-229`). Pre-check for collision (cannot copy if same-named server exists).
- **Hook**: JSON merge — for each event type, **appends** source matchers to existing matchers (`writer.py:343-359`). Always merge-by-append; duplication possible.

### Delete

- **MD-type** (commands/subagents/memory): `Path.unlink()`.
- **Skill**: `shutil.rmtree` of `customization.path.parent`.
- **MCP**: remove server from JSON; if empty `mcpServers`, remove that key; if PROJECT and document becomes empty `{}`, **unlink the file** (`writer.py:278-279`). PROJECT_LOCAL keeps the `~/.claude.json` file but cleans the `projects[...]` subtree.
- **Hook**: remove `hooks` key from JSON; if document empty after, **unlink the file** (`writer.py:166-169`).

### Toggle plugin enabled

- Modifies `enabledPlugins` dict in the right settings file (`writer.py:442-484`).
- Maps `PluginScope` → `settings.json` (USER), `./.claude/settings.json` (PROJECT), `./.claude/settings.local.json` (PROJECT_LOCAL).

### Atomicity

- No `.tmp + rename` atomic writes anywhere — `_write_settings_json` (`writer.py:515-518`) does direct `write_text`. A SIGKILL mid-write corrupts the JSON. **P0 for Rust port** (use `tempfile + persist`).

## BC-12: ConfigPathResolver contract

`services/config_path_resolver.py`:

- Non-plugin → returns the path unchanged.
- Plugin with no `plugin_info` → returns the path unchanged.
- Plugin with `plugin_info` but no resolvable source → returns the path unchanged.
- Plugin with directory-source marketplace → translates `<install_path>/file.md` → `<source_root>/file.md` by:
  1. Asking `PluginLoader.get_plugin_source_path(plugin_id)` for the source root.
  2. Computing `file.relative_to(install_path)` then joining with `source_root`.
  3. If `relative_to` fails, return `file_path` unchanged.

`PluginLoader.get_plugin_source_path` (`plugin_loader.py:172-212`):
- Parses `plugin_id` as `name@marketplace_name`.
- Loads `~/.claude/plugins/known_marketplaces.json[marketplace_name]`.
- If marketplace source type == `"directory"`, reads `<marketplace_root>/.claude-plugin/marketplace.json` to find plugin's `source` relative path.
- Returns `<marketplace_root>/<source>` (resolved); falls back to install_path.

This is the canonical algorithm Monocle's TUI must implement for "open this plugin's file in editor" to land on the editable source, not the cached install.

## State Checkpoint

```yaml
pass: 4
status: complete
timestamp: 2026-05-11T17:15:00Z
next_pass: 5
contracts_extracted: 12
high_confidence_count: 12  # all sourced from code, with test corroboration in tests/integration/discovery/test_*
```
