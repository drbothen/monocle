# Phase B Deepening: Parsers — Round 1

Goal: harden the per-parser canonical schema beyond Pass 4 BC entries. Produce a per-parser table that a Rust developer can implement against verbatim.

## Citations re-verified

| Parser | Source | Test |
|---|---|---|
| SlashCommandParser | `services/parsers/slash_command.py:18-89` | `tests/integration/discovery/test_slash_commands.py` |
| SubagentParser | `services/parsers/subagent.py:18-79` | `tests/integration/discovery/test_subagents.py` |
| SkillParser | `services/parsers/skill.py:72-147` | `tests/integration/discovery/test_skills.py` |
| MemoryFileParser | `services/parsers/memory_file.py:18-148` | `tests/integration/discovery/test_memory_files.py`, `test_auto_memory.py`, `tests/unit/test_memory_file_ref.py` |
| MCPParser | `services/parsers/mcp.py:16-127` | `tests/integration/discovery/test_mcps.py` (read, see below) |
| HookParser | `services/parsers/hook.py:14-87` | `tests/integration/discovery/test_hooks.py` (shallow, count-only) |
| LSPServerParser | `services/parsers/lsp_server.py:15-139` | **NONE** — confirmed by ls of tests/ |

## Reading from `test_mcps.py` (cross-check)

`tests/integration/discovery/test_mcps.py:187-217` confirms the Windows-backslash path fuzzing is **intentional and tested**:

```python
def test_handles_backslash_path_format(...):
    backslash_path = resolved_path.replace("/", "\\")
    content = { "projects": { backslash_path: { "mcpServers": {...} } } }
    ...
    assert local_mcps[0].name == "backslash-server"
```

So Seed 3 (G3 in Pass 5) is **resolved** — covered by test. The Rust port must replicate this dual-key lookup.

`test_handles_missing_projects_key` and `test_handles_project_not_in_projects_list` confirm graceful handling of missing data.

## Canonical schema tables for Rust port

### Parser: SlashCommand

| Field | Source | Type | Default | Notes |
|---|---|---|---|---|
| `name` | file path relative to `commands/`, parts joined with `:`, stem of last | `String` | `path.stem` if relative_to fails | Nested becomes `dir:sub:file`. Discriminating from subagent which uses `path.stem` only. |
| `description` | `frontmatter.description` OR first non-`#` body line | `Option<String>` (max 100 chars) | None | Falls through to body heuristic if frontmatter omits |
| `content` | `path.read_text(utf-8)` | `String` | — | Full raw content |
| `metadata.allowed_tools` | frontmatter `allowed-tools` | `Vec<String>` | `[]` | CSV-or-list, see parse_tools_list |
| `metadata.argument_hint` | frontmatter `argument-hint` | `Option<String>` | None | — |
| `metadata.model` | frontmatter `model` | `Option<String>` | None | — |
| `metadata.disable_model_invocation` | frontmatter `disable-model-invocation` | `bool` | `false` | Note hyphen→underscore mapping |

### Parser: Subagent

| Field | Source | Type | Default | Notes |
|---|---|---|---|---|
| `name` | `frontmatter.name` OR `path.stem` | `String` | `path.stem` | Different from slash-command — no `:` joining |
| `description` | `frontmatter.description` ONLY | `Option<String>` | None | No body fallback |
| `metadata.tools` | frontmatter `tools` | `Vec<String>` | `[]` | parse_tools_list |
| `metadata.model` | frontmatter `model` | `Option<String>` | None | — |
| `metadata.permission_mode` | frontmatter `permission-mode` | `Option<String>` | None | hyphen-to-underscore |
| `metadata.skills` | frontmatter `skills` | `Vec<String>` | `[]` | **Inline CSV/list parser, not parse_tools_list** |

### Parser: Skill

| Field | Source | Type | Default | Notes |
|---|---|---|---|---|
| `name` | `frontmatter.name` OR `skill_dir.name` | `String` | `skill_dir.name` | The directory name, not the file name |
| `path` | the `SKILL.md` file itself | `PathBuf` | — | **Not the directory** |
| `description` | `frontmatter.description` | `Option<String>` | None | — |
| `metadata.tags` | frontmatter `tags` | `Vec<String>` | `[]` | CSV-or-list inline parser |
| `metadata.has_reference` | `(skill_dir / "reference.md").exists()` | `bool` | `false` | Flag |
| `metadata.has_examples` | `(skill_dir / "examples.md").exists()` | `bool` | `false` | Flag |
| `metadata.has_scripts` | `(skill_dir / "scripts").is_dir()` | `bool` | `false` | Flag |
| `metadata.has_templates` | `(skill_dir / "templates").is_dir()` | `bool` | `false` | Flag |
| `metadata.files` | recursive walk, sorted (`(is_file, name.lower())`) | `Vec<SkillFile>` | `[]` | Skips: `name.startswith(".")`, the excluded set `{SKILL.md}`, gitignore-dir-matched. Eagerly reads file content into `SkillFile.content`. |

`SkillFile` schema (`models/customization.py:69-77`):

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Just the basename |
| `path` | `PathBuf` | Absolute |
| `content` | `Option<String>` | None on read error; None for directories |
| `is_directory` | `bool` | — |
| `children` | `Vec<SkillFile>` | Nested |

### Parser: MemoryFile

| Field | Source | Type | Default | Notes |
|---|---|---|---|---|
| `name` | `path.name` (e.g., "CLAUDE.md") | `String` | — | Overridden by discovery for nested CLAUDE.md to relative path, and for rules to rules-relative path |
| `description` | first body line (non-`#`, non-`@`) trimmed to 100 chars | `String` | `"Memory file"` | Always non-None |
| `metadata.imports` | regex `@([\w./~-]+\.md)` against body | `Vec<String>` | `[]` | Frontmatter ignored |
| `metadata.tags` | frontmatter `tags` | `Vec` | `[]` | — |
| `metadata.refs` | recursive resolved tree | `Vec<MemoryFileRef>` | `[]` | See below |

`MemoryFileRef` schema (`models/customization.py:80-88`):

| Field | Type |
|---|---|
| `name` | `String` — original ref text |
| `path` | `Option<PathBuf>` — resolved abs path or None if unresolved/depth-cap |
| `content` | `Option<String>` |
| `exists` | `bool` |
| `children` | `Vec<MemoryFileRef>` |

Resolution algorithm (`memory_file.py:94-148`):
1. If depth >= 5 → ref with path=None, exists=false.
2. Resolve ref:
   - `~/...` → `Path.home() / ref[2:]`
   - `/...` → absolute (POSIX)
   - `<letter>:...` → absolute (Windows)
   - else → `base_dir / ref`
3. Call `Path.resolve()` (canonicalize); on OSError → ref with path=None.
4. If in `visited` set → return ref with path=resolved, exists=true, no children (cycle break).
5. Add to `visited`.
6. If !exists or !is_file → ref with exists=false.
7. Read content; on OSError → ref with content=None.
8. Find nested @imports and recurse.

### Parser: MCP

Two distinct file formats:

**Format A — `.claude.json` (User and global)**

```json
{
  "mcpServers": {
    "<name>": {"type": "stdio", "command": "...", "args": [...], "env": {...}},
    ...
  },
  "projects": {
    "<project_path>": {
      "mcpServers": { "<name>": {...} }
    },
    ...
  }
}
```

The wrapped form is mandatory for `.claude.json`: `parse()` at `mcp.py:53-54` enforces `data.get("mcpServers", {})` only (returns empty if not wrapped).

**Format B — `.mcp.json` and plugin `.mcp.json`**

Either:
```json
{ "mcpServers": { "<name>": {...} } }
```
OR (unwrapped):
```json
{ "<name>": {"type": "stdio", "command": "..."} }
```

`mcp.py:56` `mcp_servers = data.get("mcpServers", data)` — if no `mcpServers` key, the root dict IS treated as the servers map. **Edge case:** if the unwrapped root contains keys named `"projects"`, `"transport"`, etc. they'd be parsed as server entries. Lenient by design.

**Server entry schema:**

| Field | Source | Type | Default | Notes |
|---|---|---|---|---|
| `name` | the dict key | `String` | — | — |
| `metadata.transport_type` | server `type` | `String` | `"stdio"` | Free string, not validated against enum |
| `metadata.command` | server `command` | `Option<String>` | None | For stdio |
| `metadata.url` | server `url` | `Option<String>` | None | For http/sse |
| `metadata.args` | server `args` | `Vec<String>` | `[]` | Coerced to [] if non-list |
| `metadata.env` | server `env` | `Map<String, String>` | `{}` | Coerced to {} if non-dict |
| `description` | synth: `"{TRANSPORT} server: {url}"` OR `"{TRANSPORT} command: {cmd}"` OR `"{TRANSPORT} server"` | `String` | — | — |
| `content` | `json.dumps(server_config, indent=2)` | `String` | — | Per-server slice |

### Parser: Hook

Input formats:
- `settings.json` / `settings.local.json` `hooks` key (`{event_name: [matchers...]}`)
- `<plugin>/hooks/hooks.json` — same shape but without `settings.json` wrapper (the parser reads `data.get("hooks", {})` from the root regardless)

Wait — let me re-read:

`hook.py:53` `hooks_data = data.get("hooks", {})`. So **`hooks.json` must also have a `hooks` top-level key**, not be a raw `{event: matchers}` dict. Interesting — that's a divergence from MCPParser's lenient unwrapped-root tolerance. **Documenting this.**

| Field | Source | Type |
|---|---|---|
| `name` | `"hooks"` if source file is `hooks.json`, else the source filename | `String` |
| `description` | comma-joined event names from `hooks_data.keys()` | `String` |
| `content` | `json.dumps(hooks_data, indent=2)` | `String` |
| `metadata` | `{}` empty | `Map` |

**Critical: hook metadata is LOSSY** — per-event hook entries are only in `content`. Monocle's port would need a richer schema for hook editing.

### Parser: LSP Server

Two input formats:
- `.lsp.json` — root dict mapping `language_name → server_config`
- `plugin.json` `lspServers` key — same shape

Per-language entry:

| Field | Source | Type | Notes |
|---|---|---|---|
| `name` | language name (JSON key, e.g., `"python"`) | `String` | — |
| `description` | synth: `"{TRANSPORT} command: {cmd}"` OR `"{TRANSPORT} server"` | `String` | — |
| `content` | pretty JSON | `String` | — |
| `metadata` | **raw server_config dict** | `Map<String, Any>` | **NOT dataclass-`__dict__`, just the parsed JSON** |

## Edge cases revisited (with citations)

### EC-1: Frontmatter regex doesn't handle CRLF — confirmed unresolved

`parsers/__init__.py:55`: regex pattern uses `\n`. `read_text(encoding="utf-8")` does not auto-translate `\r\n`. Files authored on Windows without explicit `newline=` argument retain CRLF.

**Verification:** No newline normalization anywhere in the codebase. CRLF risk is real.

### EC-2: parse_tools_list asymmetric empty-string handling — confirmed

`parsers/__init__.py:67-73`:
- list path: `[str(t).strip() for t in tools_value]` — preserves empty strings
- string path: `[t.strip() for t in str(tools_value).split(",") if t.strip()]` — filters empties

**Net:** YAML `tools: ["", "Read"]` → `["", "Read"]`. YAML `tools: ", Read"` → `["Read"]`. Inconsistent.

### EC-3: Skill files don't filter ignored files (only dirs)

`skill.py:36-67`:
- For directories, calls `gitignore_filter.should_skip_dir(entry.name)` and `is_dir_ignored(entry)`.
- For files, NO gitignore check — every non-hidden file is included.

**Verification:** Yes, confirmed. A skill with a `node_modules` dir would be pruned, but a stray `.coverage` file at the top level would be slurped.

### EC-4: Hooks file must be wrapped — confirmed

`hook.py:53`: `data.get("hooks", {})`. Unwrapped hook JSONs are silently treated as having no hooks. Different from MCP's lenient unwrap.

### EC-5: LSP plugin.json key is `lspServers`

`lsp_server.py:109`: `data.get("lspServers", {})`. NOT `lsp_servers`. Camel case.

### EC-6: Plugin commands recursive, subagents flat

`discovery.py:33-45`:
- `SCAN_CONFIGS["slash_commands"]` uses `GlobStrategy.RGLOB` — recursive `commands/**/*.md`.
- `SCAN_CONFIGS["subagents"]` uses `GlobStrategy.GLOB` — flat `agents/*.md`.
- `SCAN_CONFIGS["skills"]` uses `GlobStrategy.SUBDIR` — `skills/*/SKILL.md`.

These strategies are then applied uniformly to ALL scope roots (user, project, plugin). Plugin scan reuses the same SCAN_CONFIGS via `_discover_plugins` → `_scanner.scan_directory(plugin_path, config, ConfigLevel.PLUGIN, plugin_info)`.

## Delta Summary

- New items added: 6 schema tables, 6 edge cases with code citations, 1 confirmed-by-test resolution (Seed 3 → resolved)
- Existing items refined: parser dispatch quirks (TypeError fallback), CRLF risk, hooks-must-be-wrapped vs MCP-unwrap-tolerant difference, LSP metadata-shape divergence
- Remaining gaps: per-parser unit-test coverage of inverse cases (test for description fallback heuristic edge cases), CRLF coverage absent

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: Discovered the **hooks-wrapped-only / mcp-unwrap-tolerant** divergence which was not in Pass 4. Discovered the **LSP `metadata = raw dict` divergence** from the dataclass-`__dict__` pattern — meaningful for any consumer of metadata. Confirmed Seed 3 (windows path fuzzing) is covered by an explicit test, narrowing the gap list. These findings change the schema map.

## Convergence Declaration

Another round needed — Round 2 should target the **plugin loader scope algebra** (Seed 6) and the **marketplace install/uninstall workflow** which is currently the largest untested surface (the entire `widgets/marketplace_modal.py` has no test file).

## State Checkpoint

```yaml
pass: B
subpass: parsers
round: 1
status: complete
timestamp: 2026-05-11T17:35:00Z
novelty: SUBSTANTIVE
```
