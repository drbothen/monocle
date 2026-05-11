# Phase B Deepening: Models — Round 4

Goal: short verification pass to close the three remaining gaps from Round 3 (test pinning of mutations, ref dedup semantics, writer-metadata reads). Decide final convergence.

## Files re-read this round

- `tests/integration/discovery/test_auto_memory.py:118, 172` (mutation-output assertions)
- `services/discovery.py:498-527` (auto-memory ref merge full block)
- `services/writer.py` — scan for `customization.metadata` (zero hits)

## Finding O — Mutation outputs ARE pinned by tests

Two test sites assert on `result[0].metadata["refs"]`:
- `tests/integration/discovery/test_auto_memory.py:118` — `test_topic_files_become_refs` checks that auto-memory dir's `.md` files appear as refs after the merge
- `tests/integration/discovery/test_auto_memory.py:172` — likely the dedup assertion (`test_existing_imports_not_duplicated` per parsers-r2)

**Implication:** the mutation semantics from Round 3 are not incidental — they are spec'd by integration tests. Rust port MUST preserve:
1. The metadata `refs` key is the merged list of (a) parser-resolved refs from `@imports` AND (b) synthesized refs from auto-memory topic files
2. Dedup is by **basename** (see Finding P)

## Finding P — Ref dedup is by basename, not full path

`discovery.py:505-510`:
```python
existing_import_names = {
    r.split("/")[-1] for r in customization.metadata.get("imports", [])
}
synth_refs: list[MemoryFileRef] = []
for tf in topic_files:
    if tf.name not in existing_import_names:
        ...
```

The `existing_import_names` set is built by **splitting on `/` and taking the last segment** — i.e., basename. Note this uses `/` as separator, NOT `os.path.sep`. On Windows, an import like `notes\foo.md` would NOT be split (Windows separator) — entire string would be the "basename". Possible Windows portability bug, but consistent with the rest of the codebase which uses POSIX-style path separators throughout @import resolution.

Rust port:
```rust
let existing_names: HashSet<String> = imports.iter()
    .map(|r| r.rsplit('/').next().unwrap_or(r).to_string())
    .collect();
```

(matches: split on `/`, take last segment; if no `/`, the whole string is the basename.)

**Confirmed P1 by parsers-r2 round 2** (slug discovery's `test_existing_imports_not_duplicated`). Cross-validated here.

## Finding Q — `writer.py` does NOT read `customization.metadata` (zero hits)

I grep'd `writer.py` for `customization.metadata` and `\.metadata\[` — zero hits. The writer accesses:
- `customization.name`
- `customization.path`
- `customization.content`
- `customization.type`
- `customization.level`
- `customization.type_label` (computed property)

**No metadata access.** This means write-back semantics are based on `content` (which is the full file text) + `path` + `name`. The metadata dict is purely for discovery enrichment + widget display. The writer round-trips `content` directly — no re-serialization from typed metadata.

**Rust port implications (final):**
1. The "tagged enum Metadata" recommendation from Rounds 1-2 is **purely for display/discovery purposes**, NOT for serialization.
2. Editing a customization in Monocle would write `customization.content` back to disk — there's no re-rendering from metadata fields. (If Monocle wants editing, it must round-trip through content, not metadata.)
3. **HOOK's "lossy" metadata is fine** — `content` carries the data. The metadata being `{}` is by design.

This is a confidence-boost finding: it confirms the Rust port doesn't need bidirectional serde for the metadata enum. Only deserialize/discovery → struct, never struct → file.

## Final corrections / refinements summary

| Round | Net finding |
|---|---|
| R1 | First complete schema; 3 P0 / 7 P1 surfaced |
| R2 | Closed metadata-key set per type; runtime-read inventory; PluginInstallation JSON schema; tags inconsistency |
| R3 | 11-site `Customization` mutation surface (structural correction) |
| R4 | Mutations are test-pinned; dedup by basename; writer does NOT read metadata |

All findings from R1 through R4 are now closed. The deliverable is the complete Rust struct-mapping table from R1, refined by R2 (metadata enum), R3 (mutation pattern), R4 (write-back via content only).

## Test coverage gaps still present (P2, but cheap to close)

- `Customization.display_name` semantics with markup-bearing names
- `Customization.type_label` for all 7 variants
- `Customization.level_label` for all 4 ConfigLevels
- `MarketplacePlugin.is_enabled = True when not is_installed` invariant
- `MemoryFileRef` cycle-break vs depth-cap distinction
- `SkillFile.content = None` on `UnicodeDecodeError`
- Memory `tags` raw vs skill `tags` normalized inconsistency

These are P2 — Monocle should add these as it ports.

## Delta Summary

- **New items added:** Test pinning confirmation for mutation surface (Finding O); ref dedup by basename details (Finding P); writer-metadata-zero-hits confirmation (Finding Q).
- **Existing items refined:** None — all corrections happened in prior rounds.
- **Remaining gaps:** None substantive for the models layer.

## Novelty Assessment

Novelty: **NITPICK**

Justification: This round confirmed three things that prior rounds implied but hadn't verified:
1. Mutations are tested (so they're contract, not incidental) — refinement of confidence, not new model
2. Dedup is by basename — already established by parsers-r2; this round just cross-cited
3. Writer doesn't read metadata — confirms the obvious; meaningful but doesn't change the schema

Would removing this round's findings change how I'd spec the system? **No.** The schema map from R1+R2+R3 is complete and a Rust developer could port from it without R4. R4 increases confidence in two specific invariants (basename dedup, writer-decoupled metadata) but doesn't introduce new types, fields, or constraints.

By the binary definition in the rubric: **NITPICK**.

## Convergence Declaration

**Models layer has converged.** Four rounds, with substantive findings through Round 3 and a confirmation round for Round 4. Pass complete.

The Rust port should reference:
- **Round 1** for the complete struct-mapping table (canonical schema)
- **Round 2** for the `Metadata` tagged-enum design + `PluginInstallation` serde shape
- **Round 3** for the mutation-surface design decision (Option A vs B)
- **Round 4** for write-back via `content`-only, not metadata

## State Checkpoint

```yaml
pass: B
subpass: models
round: 4
status: complete
timestamp: 2026-05-11T18:45:00Z
novelty: NITPICK
converged: true
```
