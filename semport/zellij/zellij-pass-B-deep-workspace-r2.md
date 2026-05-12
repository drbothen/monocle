# Phase B Deep — Workspace Architecture (Round 2)

## Refinements From Round 1

| Refinement | File:line | Was missed in r1? |
|---|---|---|
| `xtask` has 10 sub-modules: `build`, `ci`, `clippy`, `dist`, `flags`, `format`, `metadata`, `pipelines`, `test`, plus `main.rs` | `xtask/src/*.rs` | r1 had module names; this round confirms LOC distribution (`pipelines.rs` 462 LOC is the largest, holding `make`/`install`/`run`/`publish`/`dist`) |
| `xtask test --no-web` uses `metadata::get_no_web_features(sh, crate_name)` to compute the right `--features <X>` invocation per-crate | `xtask/src/test.rs:62-84` | r1 noted the `--no-web` job exists; this is the mechanism (per-crate feature subtraction) |
| Plugin assets are added via `add_plugin!(assets, "<name>.wasm")` macro, with conditional `include_bytes!` from either `assets/plugins/` (release / no plugins_from_target) OR `target/wasm32-wasip1/debug/` (debug + plugins_from_target) | `consts.rs:130-180` | r1 mentioned the embedding but not this dual-source macro |
| 12 plugins embedded into `ASSET_MAP` (not 13 — `fixture-plugin-for-tests` is excluded from the runtime asset map) | `consts.rs:166-180` | r1 said 13 plugins build; only 12 are baked into the binary |

## Confirmed

The Round 1 workspace map is structurally complete. No new crates discovered. No new boundary-type relocations found. The Round 1 dependency graph holds.

## Round 2 Status

This round's findings are refinements (macro-level details and per-crate feature subtraction mechanics) — not new subsystems. Pass converges here.

```yaml
pass: B
category: workspace
round: 2
status: complete
timestamp: 2026-05-11T21:15:00Z
classification: nitpick
```
