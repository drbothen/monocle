# Phase B Deepening: Models — Round 3

Goal: final pass — verify mutation surface, fixture invariants, LSP plugin.json metadata shape, Rich-markup escape safety. Decide convergence.

## Files re-read this round

- `tests/conftest.py:1-196` (full)
- `services/discovery.py:380-720` (mutation sites scan)
- `services/plugin_loader.py:280-330` (short_name derivation)
- `services/parsers/lsp_server.py` (already read; cross-check `parse_plugin_json` metadata)
- Full codebase scan for `escape(`, `markup_escape` (none found)
- Full codebase scan for `customization.<attr> =` mutation patterns

## Finding I — `Customization` mutation surface is LARGER than Round 1/Round 2 stated

I scanned all `customization.<field> =` assignments. Eleven mutation sites across `services/`:

| Site | Mutation | Purpose |
|---|---|---|
| `discovery.py:389` | `customization.plugin_info = plugin_info` | Plugin enrichment (commands) |
| `discovery.py:410` | `customization.plugin_info = plugin_info` | Plugin enrichment (agents) |
| `discovery.py:459` | `customization.name = str(rel_path)` | Rewrite name for nested CLAUDE.md memory files |
| `discovery.py:521` | `customization.metadata["refs"] = ...` | Auto-memory ref merge (Round 2 finding) |
| `discovery.py:526` | `customization.name = md_file.name` | Fallback name for individual md memory files |
| `discovery.py:550` | `customization.name = str(rule_file.relative_to(...))` | User rules name rewrite |
| `discovery.py:566` | `customization.name = str(rule_file.relative_to(...))` | Project rules name rewrite |
| `discovery.py:679` | `customization.plugin_info = plugin_info` | Plugin enrichment (skills) |
| `discovery.py:696` | `customization.plugin_info = plugin_info` | Plugin enrichment (memory/mcp/hook/lsp) |
| `discovery.py:711` | `customization.plugin_info = plugin_info` | Plugin enrichment (marketplace-component branch) |
| `discovery.py:719` | `customization.plugin_info = plugin_info` | Plugin enrichment (LSP plugin.json branch) |
| `filesystem_scanner.py:75` | `customization.plugin_info = plugin_info` | Generic scanner plugin enrichment |

**This is a major correction to Rounds 1 and 2.** Both rounds implied that `Customization` is constructed by parsers and "done" — discovery actually performs significant post-parse enrichment:

1. **`plugin_info` is ALWAYS assigned post-construction** when the source is a plugin. Parsers don't know about plugins; they construct without `plugin_info` and discovery attaches it. The Round 1 invariant `level == PLUGIN <=> plugin_info.is_some()` is enforced procedurally, not at the type level.
2. **`name` is rewritten** for memory files / rules where the parser's default (`path.name`) is insufficient — discovery substitutes a more meaningful name (relative path).

**Rust port implications (REVISED):**
- `Customization` cannot be immutable post-construction. Either expose mutable accessors or restructure discovery to build a `CustomizationBuilder` and call `.build()` after all enrichment.
- A cleaner Rust design: have parsers return a `ParsedCustomization` (immutable) and discovery wrap it in a `Customization { parsed, plugin_info: Option<PluginInfo>, name_override: Option<String> }`. Encodes the layering at the type level.
- Or simpler: keep `Customization` mutable with `&mut self` setters, document the construction-then-enrichment pipeline.

This is a structural finding that changes the Rust port. **SUBSTANTIVE.**

## Finding J — `short_name` extraction logic (`plugin_loader.py:290`)

```python
short_name = plugin_id.split("@")[0] if "@" in plugin_id else plugin_id
```

Confirms Round 1: `plugin_id` is `"name@marketplace"`; `short_name` is the prefix. If no `@`, short_name = full plugin_id (defensive fallback for malformed registry entries).

Rust:
```rust
fn derive_short_name(plugin_id: &str) -> String {
    plugin_id.split_once('@').map(|(name, _)| name).unwrap_or(plugin_id).to_string()
}
```

## Finding K — LSP `parse_plugin_json` metadata shape matches `.lsp.json` parse

Both paths invoke `parse_server_config(language_name, server_config, ...)` (`lsp_server.py:57, 117`), which sets `metadata=server_config` (the raw JSON dict). **Same metadata shape from both code paths.** No divergence.

Confirms Round 2 Finding A: LSP metadata is always the raw input JSON, regardless of source file.

## Finding L — Rich markup escape: `display_name` is UNSAFE for names containing `[` or `]`

`customization.py:153, 162`:
```python
base = f"[dim]{self.plugin_info.short_name}:[/]{self.name}"
return f"{self.name} {level_indicator[self.level]}"
```

Names with `[` or `]` characters would render as Rich markup, not literal. **No escaping is performed anywhere in the codebase** — I scanned for `escape(`, `markup_escape`, `Text.from`. None found.

In practice: customization names are derived from filenames (which are filesystem-valid) or YAML frontmatter `name:` fields (free strings). A user-malicious filename like `[bad]name.md` would render as a markup tag.

**Severity:** P2 — latent display corruption issue, not a security issue. Plugin display labels also include `:[/]` after the short_name; that's intentional Rich markup. The Rust port:
- If using Rich-like markup (e.g., `crossterm` or `ratatui` with similar syntax), apply escape logic at name boundaries
- If using literal terminal output, ignore (Rust port likely just renders text)

## Finding M — `conftest.py` reveals fixture conventions, not new model invariants

Reading the full `conftest.py:1-196`: it constructs fixtures around `Path("/fake/home")` and `Path("/fake/project")` via pyfakefs. The fixtures load fixture files from `tests/integration/fixtures/` into the fake filesystem. **No fixture constructs `Customization` directly** — they go through the discovery service.

Note one detail: `plugins_config` fixture at lines 154-176 creates a V2 plugin install at `cache/test/example-plugin/1.0.0/`. The V2 directory layout is **`{cache_dir}/{plugin_id_without_marketplace}/{version}/`** — confirming Round 2's V2 schema understanding.

**No new model invariants surfaced from conftest.** This was a planned gap-close, now closed.

## Finding N — Round 2's "tags is never read" claim — re-verifying

Round 2 said `tags` is never read. Let me verify once more across widgets / mixins / detail rendering:

I already scanned for `metadata.get("tags"` and `metadata["tags"]` — zero hits. Confirmed.

But wait — the detail pane's frontmatter rendering may read tags directly from the YAML frontmatter, bypassing `metadata`. Let me check `widgets/detail_pane.py`:

Spot-check: `widgets/detail_pane.py` invokes `_render_markdown_with_frontmatter` for `.md` files. Frontmatter is re-parsed from content — NOT read from `customization.metadata`. So:
- `tags` exists in `metadata` (write-only)
- `tags` is re-parsed from frontmatter for display in the detail view

This is a **minor architectural smell** (parse-twice) but matches what the parsers-r1 round captured. **No new model finding** — refinement of Round 2.

## Summary of all corrections across rounds

| Round | Claim | Corrected To |
|---|---|---|
| R1 | Slug regex lives in models | Lives in `services/discovery.py:484` — verified |
| R1 | Dataclasses are "dead documentation" | Used at construction via `__dict__`; only runtime-dead |
| R1 | `Customization.metadata` shape closed per type | True except LSP (raw JSON) and the dataclasses aren't full schema for memory (uses dict literal) |
| R1/R2 | `Customization` is "constructed once" | **WRONG** — discovery mutates `name`, `plugin_info`, and `metadata["refs"]` after parsing (11 mutation sites) |
| R2 | Metadata mutation is "only at discovery.py:521" | Plus `customization.name = ...` at 4 sites, `customization.plugin_info = ...` at 8 sites |
| R2 | "tags is never read" | Confirmed (Round 3) |

The R1/R2 → R3 correction on the mutation surface is the **single biggest structural finding** across the three rounds. It changes the Rust port's `Customization` design (must be mutable, OR introduce a builder pattern).

## Updated Rust port recommendation

Two options for handling the mutation surface:

**Option A (literal port):** keep `Customization` mutable:
```rust
pub struct Customization {
    pub name: String,
    pub r#type: CustomizationType,
    pub level: Level,
    pub path: PathBuf,
    pub description: Option<String>,
    pub content: Option<String>,
    pub metadata: Metadata,
    pub error: Option<String>,
    pub plugin_info: Option<PluginInfo>,
}

impl Customization {
    pub fn override_name(&mut self, name: String) { self.name = name; }
    pub fn attach_plugin_info(&mut self, info: PluginInfo) { self.plugin_info = Some(info); }
    pub fn extend_memory_refs(&mut self, more: Vec<MemoryFileRef>) -> Result<()> { ... }
}
```

**Option B (typed pipeline):** separate parse-time and discovery-time:
```rust
pub struct ParsedCustomization { /* name, type, level, path, description, content, metadata, error */ }

pub struct Customization {
    parsed: ParsedCustomization,
    pub name_override: Option<String>,    // discovery override
    pub plugin_info: Option<PluginInfo>,  // discovery attachment
    extra_refs: Vec<MemoryFileRef>,        // discovery merge
}
```

**Recommendation:** Option A. The mutation pattern is shallow, well-localized to discovery, and matches the reference's mental model. Option B is more "rust-y" but adds friction without changing semantics.

## Delta Summary

- **New items added:** 11-site `Customization` mutation surface (the biggest correction); `short_name` extraction logic; Rich-markup escape gap; conftest review; LSP `parse_plugin_json` metadata shape parity; corrections summary across all 3 rounds; revised Rust port recommendation with two structural options.
- **Existing items refined:** `Customization` immutability assumption refuted; `tags` write-only confirmed (with subtle re-parse at display time).
- **Remaining gaps:** None substantive for the models layer. The remaining things (test coverage for `display_name` / `type_label`, Rich escape) are nitpicks Monocle's port should resolve at its own discretion.

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: This round's headline finding — the **11-site mutation surface for `Customization`** — is the single most important structural finding across all three rounds. It directly changes the recommended Rust port design (either mutable struct or builder pattern). Without this finding, a port targeting "immutable parsed types" would mismatch the reference's mental model on 11 sites.

The Round 1 understanding (parse → finished `Customization`) was incomplete. The Round 2 understanding (one mutation site) was incomplete. Round 3 gives the full picture.

Removing this round's findings would let a Rust developer build a `Customization` type that's structurally incompatible with the parser → discovery → widget pipeline. That's a model-changing finding, not a refinement.

## Convergence Declaration

**One more round needed** — Round 4 should be a short convergence verification:
1. Spot-check whether any tests assert on the **post-mutation** name / plugin_info (i.e., do tests pin the override semantics, or just the parser's initial output?). If tests pin override semantics, that's a real invariant; otherwise it's incidental.
2. Verify `extend_memory_refs` semantics: when the same ref name appears in both existing and synthesized, does dedup happen? (Already partially covered by parsers-r2; cross-link.)
3. Audit `writer.py` for any reads/mutations of `Customization.metadata` we haven't seen.

If those checks yield only refinements (expected), declare NITPICK in Round 4.

## State Checkpoint

```yaml
pass: B
subpass: models
round: 3
status: complete
timestamp: 2026-05-11T18:35:00Z
novelty: SUBSTANTIVE
```
