# Phase B Deepening: Models — Round 2

Goal: close the metadata-shape contract by enumerating actual consumer reads; verify equality/hashing assumptions; verify `installed_scopes` literal set exhaustively; surface any new findings before declaring convergence.

## Files re-read this round (full)

- `services/parsers/slash_command.py:1-89` — uses `SlashCommandMetadata(...).__dict__`
- `services/parsers/subagent.py:1-79` — uses `SubagentMetadata(...).__dict__`
- `services/parsers/skill.py:1-147` — uses `SkillMetadata(...).__dict__`
- `services/parsers/memory_file.py:1-148` — uses **raw dict literal** with keys `imports`, `tags`, `refs`
- `services/parsers/mcp.py:1-127` — uses `MCPServerMetadata(...).__dict__`
- `services/parsers/hook.py:1-87` — uses `metadata={}` (empty)
- `services/parsers/lsp_server.py:1-139` — uses `metadata=server_config` (**raw input JSON**)
- `services/plugin_loader.py:79-101` (V2 `installed_plugins.json` schema)

## Finding A — Dataclasses ARE the metadata schema (corrects Round 1's lean)

Round 1 said the metadata dataclasses (`SlashCommandMetadata` etc.) were "essentially dead documentation." That was **partially wrong**. The parsers explicitly construct the dataclass, then write `metadata.__dict__` (`slash_command.py:73`, `subagent.py:78`, `skill.py:146`, `mcp.py:107`). So:

| Type | metadata dict shape source |
|---|---|
| SLASH_COMMAND | `SlashCommandMetadata.__dict__` — keys = `allowed_tools, argument_hint, model, disable_model_invocation` |
| SUBAGENT | `SubagentMetadata.__dict__` — keys = `tools, model, permission_mode, skills` |
| SKILL | `SkillMetadata.__dict__` — keys = `tags, has_reference, has_examples, has_scripts, has_templates, files` |
| MCP | `MCPServerMetadata.__dict__` — keys = `transport_type, command, url, args, env` |
| MEMORY_FILE | Raw dict literal (`memory_file.py:61-65`) — keys = `imports, tags, refs` — **NOT a dataclass** |
| HOOK | `{}` (empty) |
| LSP_SERVER | Raw `server_config` dict from JSON (`lsp_server.py:85`) — **open shape** |

**The closed key set per type is verified** (with the caveat that LSP is open by design). This is the contract a Rust `enum Metadata` should encode.

## Finding B — Metadata is actually read at runtime in only 3 places

I traced every `customization.metadata.get(...)` and `customization.metadata[...]` site:

| Site | Key | Customization type | Notes |
|---|---|---|---|
| `services/discovery.py:506` | `"imports"` | MEMORY_FILE | Reads for dedup against synthesized refs |
| `services/discovery.py:520` | `"refs"` | MEMORY_FILE | Reads existing refs for merge |
| `services/discovery.py:521` | `"refs"` (**mutation**) | MEMORY_FILE | **Assigns** `customization.metadata["refs"] = existing_refs + synth_refs` |
| `widgets/type_panel.py:248` | `"files"` | SKILL | Checks if skill has any files (expansion enabled?) |
| `widgets/type_panel.py:547` | `"files"` | SKILL | Reads file list for flat-list rendering |
| `widgets/helpers/rendering.py:43` | `"refs"` | MEMORY_FILE | Reads refs for flat-list rendering |
| `widgets/helpers/rendering.py:91` | `"refs"` | MEMORY_FILE | Checks if memory has any refs (expansion enabled?) |

**That's it.** Across the entire codebase, only THREE distinct keys are read at runtime:
- MEMORY_FILE: `imports`, `refs`
- SKILL: `files`

All other metadata keys (`allowed_tools`, `argument_hint`, `model`, `tools`, `permission_mode`, `skills`, `tags`, `has_reference`/`has_examples`/`has_scripts`/`has_templates`, `transport_type`, `command`, `url`, `args`, `env`) are **WRITE-ONLY** in the codebase. They exist in the dict but are never read.

**Implications for Monocle:**
1. Round 1's M-P0-1 recommendation (tagged-enum `Metadata`) is **still right** but the immediate behavioral need is much smaller — only memory `refs`/`imports` and skill `files` need typed access. The other fields are inert.
2. A pragmatic Rust port could expose `Customization::skill_files() -> &[SkillFile]`, `Customization::memory_refs() -> &[MemoryFileRef]`, `Customization::memory_imports() -> &[String]` and treat the rest as `serde_json::Value` for the moment. Lossless but easier.
3. The unused metadata keys are likely **planned future surface** (editing slash command tools, etc.) — Monocle's Rust port should still define the typed shapes upfront so adding the editor is a non-event.

## Finding C — Metadata mutation site (`discovery.py:521`)

```python
customization.metadata["refs"] = existing_refs + synth_refs
```

The auto-memory discovery merges synthesized refs (one per `.md` file in the auto-memory dir) with existing refs from the entrypoint's @imports. This **mutates `metadata` in-place after construction**.

Rust implications:
- `Customization` cannot have `metadata: Arc<Metadata>` or any immutable share — needs `&mut`
- Or, restructure: build all refs first, then construct `Customization` once with the full list (cleaner). Round 1's tagged-enum recommendation makes this clean — `Metadata::MemoryFile { refs, ... }` constructed once.

## Finding D — `Customization` is never hashed or set-key'd

I scanned for `set(Customization`, `Customization)` in dict-key context, and `__hash__` definitions. **None found.** Python's `@dataclass` defaults make `Customization` non-hashable (`__hash__ = None` because `eq=True, frozen=False`). The runtime never tries.

The closest things are `seen_paths: set[Path]` (`discovery.py:258`) and `customization_index: dict[tuple, Customization]` style patterns — neither exists.

Rust port: do NOT implement `Hash` on `Customization`. The natural identity is the `path: PathBuf` (for non-MCP/HOOK/LSP) or `(path, name)` (since MCP/HOOK/LSP can have multiple customizations per file). The discovery code's `seen_paths` set keys on `Path` — Rust port matches with `HashSet<PathBuf>`.

## Finding E — `installed_scopes` literal set is exhaustively `{"user", "project", "local"}`

Verified by enumerating EVERY string literal that flows into `installed_scopes`:

1. `services/plugin_loader.py:91` — `scope=inst.get("scope", "user")` — JSON `scope` field with `"user"` default
2. `services/plugin_loader.py:124` — `if installation.scope == "user"`
3. `services/plugin_loader.py:135` — `if installation.scope == "project"`
4. `services/plugin_loader.py:148` — `if installation.scope == "local"`
5. `services/marketplace_loader.py:216-219` — the only **append site** restricts to:
   ```python
   if scope == "user":
       scopes.append("user")
   elif scope in ("project", "local"):
       scopes.append(scope)
   ```
6. `widgets/marketplace_modal.py:464` — `scope_labels = {"user": "U", "project": "P", "local": "L"}` — display dict

**Conclusion: closed set `{"user", "project", "local"}`.** Anything else from `installed_plugins.json` is silently dropped at `marketplace_loader.py:216-220`. The `PluginScope` enum's three variants exhaustively cover this set.

**P0 confirmed:** Rust serde:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PluginScope {
    User,
    Project,
    #[serde(rename = "local")]
    ProjectLocal,
}
```

## Finding F — V2 `installed_plugins.json` JSON schema (camelCase keys)

Reading `plugin_loader.py:88-98`, the JSON shape of each installation entry:

| JSON key | Python field (`PluginInstallation`) | Default |
|---|---|---|
| `"scope"` | `scope: str` | `"user"` |
| `"installPath"` | `install_path: str` | `""` |
| `"version"` | `version: str` | `"unknown"` |
| `"isLocal"` | `is_local: bool` | `False` |
| `"projectPath"` | `project_path: str \| None` | `None` |

**Note: `PluginInstallation` is a SEPARATE dataclass in `services/plugin_loader.py`, NOT in models.** It's the JSON-shaped sibling of `PluginInfo`. The conversion happens at `plugin_loader.py:323`.

**Why this matters for Monocle:** the camelCase boundary is on the JSON wire format only. Rust port:
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginInstallation {
    scope: PluginScope,
    install_path: String,         // -> installPath
    version: String,
    is_local: bool,                // -> isLocal
    project_path: Option<String>,  // -> projectPath
}
```

This is a new finding — not surfaced in Round 1 because I only read `models/` then. Adding now.

## Finding G — `MemoryFileRef`'s `tags` field on memory metadata comes from frontmatter

`memory_file.py:63` `"tags": frontmatter.get("tags", [])`.

**Caveat:** frontmatter `tags` is **not parsed via `parse_tools_list`**. The raw frontmatter value passes through. If user writes `tags: foo,bar` (string), it stays as the literal string `"foo,bar"` — NOT a list. If they write `tags: [foo, bar]` (YAML list), it's a list of strings.

**Inconsistency with skill `tags`** (`skill.py:118-124`) which DOES the CSV-or-list normalization. Memory's `tags` is raw. Rust port:
- Match Python: keep memory `tags` as `serde_json::Value` (lossy)
- Or normalize: use the same CSV-or-list parser for memory `tags` (divergence from reference)

Recommendation: match reference behavior; document the inconsistency.

## Finding H — `MemoryFileRef.children` recursion edge case: cycle-break returns NO children

`memory_file.py:114-115`:
```python
if resolved in visited:
    return MemoryFileRef(name=ref_name, path=resolved, exists=True)
```

On cycle hit, the ref is returned with `path=resolved, exists=True, content=None, children=[]`. **The ref looks "loaded" (exists=True) but has no content.** The widgets layer's render path needs to handle this — but `widgets/helpers/rendering.py:43+` doesn't have any cycle-detection logic visible; it just walks children, so an empty children list naturally terminates. Safe.

**Rust port note:** the cycle break creates a `MemoryFileRef` that looks like a "leaf" loaded ref. This is semantically distinct from a depth-cap hit (`path=None, exists=False`). Two different "non-recursing" states, both legitimate. Document both.

## Round 1 corrections / refinements

| Round 1 claim | Refinement |
|---|---|
| "Metadata dataclasses are essentially dead documentation" | **Partially wrong.** They ARE used at construction time via `__dict__`. They define the dict shape but are never read back as dataclasses. The reduction is in *runtime usage*, not in *schema role*. |
| "MEMORY_FILE metadata uses a metadata dict shape via dataclass" | **Wrong (Round 1 didn't say this explicitly but the schema table implied a dataclass).** Memory uses a raw dict literal — NO dataclass exists for memory metadata. |
| "Only `imports`, `refs`, `files`, `tags` are read" | **Refined: only `imports`, `refs`, `files`** are read (tags is read by widgets only at render time — let me verify). |

Verification on `tags`:
<br>Search for `metadata.get("tags"` — none found across the codebase. **`tags` is never read.** Pure write-only. Same for skill flags `has_reference` etc.

## Test coverage gaps for the model layer (P2)

Round 1 noted these; Round 2 confirms no tests exist for:
- `Customization.display_name` (with/without plugin_info, enabled/disabled, all level indicators)
- `Customization.type_label` for all 7 variants
- `Customization.level_label` for all 4 ConfigLevels
- `Customization.has_error` true and false paths
- Marketplace `MarketplacePlugin.is_enabled = True when not is_installed` invariant
- `MemoryFileRef` cycle-break vs depth-cap distinction
- `SkillFile.content = None on UnicodeDecodeError` (the catch-`UnicodeDecodeError` at `skill.py:59`)

These would be cheap unit tests for the Rust port to ship alongside the models.

## Updated Rust struct-mapping refinements

### `Customization.metadata` — refined recommendation

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Metadata {
    SlashCommand(SlashCommandMetadata),
    Subagent(SubagentMetadata),
    Skill(SkillMetadata),
    MemoryFile {
        imports: Vec<String>,
        tags: serde_json::Value,   // raw frontmatter — see Finding G
        refs: Vec<MemoryFileRef>,
    },
    Mcp(McpServerMetadata),
    Hook,                          // unit variant, lossy
    LspServer(serde_json::Value),  // raw server_config JSON
}

impl Customization {
    // Runtime accessors — only these THREE are actually read in the reference codebase.
    pub fn memory_imports(&self) -> Option<&[String]> { ... }
    pub fn memory_refs(&self) -> Option<&[MemoryFileRef]> { ... }
    pub fn skill_files(&self) -> Option<&[SkillFile]> { ... }
}
```

The `&mut` for the auto-memory merge can be:
```rust
impl Customization {
    pub fn extend_memory_refs(&mut self, more: Vec<MemoryFileRef>) -> Result<(), TypeMismatch> { ... }
}
```

### `PluginInstallation` — net-new struct for Monocle

```rust
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallation {
    #[serde(default = "default_user_scope")]
    pub scope: PluginScope,
    pub install_path: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub is_local: bool,
    pub project_path: Option<String>,
}
```

Lives in the registry layer, not models. But Monocle's models module should still export it for symmetry with how `models/customization.py` declares `PluginInfo` alongside the higher-level types.

## Delta Summary

- **New items added:** Closed metadata key set per type (7 types); runtime metadata-read inventory (only 7 read sites, 3 distinct keys); metadata mutation site at `discovery.py:521`; verification that `Customization` is never hashed; exhaustive verification of `installed_scopes` literal set; `PluginInstallation` JSON schema with camelCase serde mapping; cycle-break vs depth-cap `MemoryFileRef` distinction; inconsistency between memory `tags` (raw) and skill `tags` (normalized).
- **Existing items refined:** Round 1's "dead documentation" claim corrected (dataclasses ARE used at construction); Round 1's metadata-type table corrected (memory uses raw dict, not dataclass).
- **Remaining gaps:** No test coverage for model-layer properties (P2); JSON schema for V1 `installed_plugins.json` (legacy?) not investigated (P3 — Monocle only needs V2 since reference treats V2 as canonical at `plugin_loader.py:79`).

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: This round produced four findings that meaningfully change a Rust port plan:
1. The **runtime read inventory** (only 3 keys actually consumed) significantly reduces immediate typing pressure on `metadata`.
2. The **metadata mutation site** changes the mutability story for `Customization`.
3. The **`PluginInstallation` JSON schema** is net-new content with camelCase serde requirements.
4. The **memory-vs-skill `tags` inconsistency** is a behavioral subtlety that would silently corrupt round-trips if not preserved.

Also corrected two specific claims from Round 1 (dataclasses as documentation; memory metadata shape). Refinements that change the recommended Rust code are not nitpicks — they are deliverable corrections.

## Convergence Declaration

**One more round needed.** Round 3 should be a final pass to:
1. Verify the `Customization`-mutation surface beyond `discovery.py:521` (are there other mutation sites I missed? — `writer.py` writes files, doesn't mutate `Customization` directly).
2. Read `tests/conftest.py` in full to capture any fixture-level invariants the production code doesn't exercise.
3. Confirm there are no other parsers (LSP plugin.json parser uses different code path — confirm metadata shape there matches `.lsp.json`).
4. Spot-check whether `display_name`'s Rich markup is escape-safe (does it handle names containing `[` or `]`?).

These are getting close to nitpicks but Round 3 should let me declare NITPICK honestly rather than prematurely.

## State Checkpoint

```yaml
pass: B
subpass: models
round: 2
status: complete
timestamp: 2026-05-11T18:20:00Z
novelty: SUBSTANTIVE
```
