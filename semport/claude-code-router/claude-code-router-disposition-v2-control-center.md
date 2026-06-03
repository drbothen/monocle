---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: claude-code-router
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: claude-code-router (CCR) (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — Tune plane includes model routing
configuration. CCR is the external model router monocle integrates with for per-session
and per-profile model selection. The "Interactive Tune" requirement makes CCR config
writable from the TUI, not just readable.

## Original Disposition Summary

The v1.1.2 vision §Explicit Non-Goals stated:
- "Does NOT route LLM API requests — claude-code-router integration is detect-on-PATH +
  config-write; monocle does not proxy or modify LLM traffic."

The Five Planes §Harness specified:
- "CCR integration: detect on PATH, write per-session JSON, set `ANTHROPIC_BASE_URL` —
  integrate-external, not build-in."

monocle's integration with CCR: detect CCR on PATH via `which ccr || which claude-code-router`,
write the per-session routing config JSON, and set `ANTHROPIC_BASE_URL=http://127.0.0.1:3456`
in the session's env. This was already in Phase 1 scope (S-031 profile picker, BC-2.07.004/
BC-2.07.005).

The pivot's "Interactive Tune" adds the requirement that CCR configuration is editable FROM
the TUI, not just applied at session launch.

## Disposition by Capability Area

### 1. Detect-on-PATH Integration (originally ADOPT, partially built)

**New verdict: ADOPT (confirmed, already in S-031).** The profile picker (Ctrl-P) already
integrates CCR path detection. The session launch path (new in control-center) uses the same
detected CCR path:
- At session launch: if profile has CCR enabled, set `ANTHROPIC_BASE_URL=http://127.0.0.1:3456`
  in the `CommandBuilder.envs()` alongside the hook config injection.
- No change to the integration model. CCR remains external; monocle sets the env var.

### 2. Per-Session JSON Config Write (originally ADOPT)

**New verdict: ADOPT + EXTEND (Tune plane activation).**

Original integration: write the per-session `~/.config/claude-code-router/config.json` (or the
CCR-specified config path) with the desired routing rules.

Control-center extension: the Tune plane allows the user to edit the CCR routing config
interactively from monocle. This means monocle needs:
- Read the current CCR config (from the detected config path).
- Display the routing rules in a structured view (default model, background, think, longContext,
  webSearch models).
- Allow the user to select/change a model for each routing slot.
- Write the updated config atomically (tempfile::persist).
- Apply immediately by hot-signaling CCR if it supports it, or by noting that the change
  takes effect on next CCR restart/session launch.

This is v1B Tune wave scope (same wave as NikiforovAll writer genes). The launch-wave (v1A)
only needs the detect + env-inject pattern.

### 3. CCR Routing Decision Tree (from pass-C-synthesis §Routing Core)

**New verdict: MODEL (for Tune plane display).** The 7-step routing decision tree
(subagent escape hatch, explicit provider,model, long-context, Haiku trap, web-search,
thinking, default) is the reference for displaying routing rules in the Tune plane:

```
[Default] → deepseek-v3 (or configured default)
[Background / Haiku] → claude-haiku-4 / local model
[Think / Plan Mode] → claude-opus-4-5 (thinking: true)
[Long Context (>60k tokens)] → gemini-2.5-pro
[Web Search] → claude-opus-4 with web tools
[Subagent] → per-tag model
```

monocle's Tune plane shows these slots. The user selects a model from a picker for each slot.
monocle writes the updated CCR config JSON. monocle does NOT need to understand the full
routing logic — it writes the config, CCR executes the routing.

### 4. CCR Preset System (from pass-C-synthesis §Preset)

**New verdict: LEAVE BEHIND for monocle.** CCR's preset system (ZIP bundles, marketplace,
install/uninstall via CLI) is CCR-internal. monocle does not manage CCR presets. The user
manages their CCR presets via the CCR CLI or CCR UI; monocle only reads the active config.

### 5. CCR Admin UI (React 19 + Vite) (from pass-C-synthesis)

**New verdict: LEAVE BEHIND.** CCR's web admin console is CCR's UI, not monocle's. monocle
provides the TUI-native Tune plane for model routing configuration. The two can coexist:
a user with both monocle and the CCR web UI can use either.

### 6. Token Budget Memory / Long-Context Threshold (from routing §3)

**New verdict: MODEL (for display, not control).** CCR tracks per-session token budgets and
applies long-context routing automatically. monocle's sessions panel can surface the current
token count (from hook events — already built) and annotate when a session is likely in
"long-context mode" (tokens > 60k threshold). This is a display enhancement, not a routing
action. monocle does not control CCR's routing decisions.

### 7. Does NOT Proxy LLM Traffic (confirmed Non-Goal)

**CONFIRMED Non-Goal.** The control-center pivot does not change this. monocle sets env vars
to point Claude Code at CCR; it does not sit in the LLM request path. The API gateway remains
entirely within CCR. monocle's only CCR interaction is config read/write and PATH detection.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| Detect-on-PATH + env inject | ADOPT (built S-031) | ADOPT (confirmed; used in spawn path) | Confirmed |
| Per-session config write | ADOPT | ADOPT + EXTEND (interactive edit in Tune plane v1B) | Extended |
| Routing decision tree | MODEL (for monocle awareness) | MODEL (for Tune plane display) | Confirmed |
| Preset system | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Admin UI (React) | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Token budget display | N/A | MODEL (display enhancement; sessions panel annotation) | NEW |
| Does NOT proxy LLM traffic | Non-Goal | Non-Goal confirmed | Confirmed |

## Net Assessment

CCR's disposition is largely unchanged by the pivot. The single meaningful extension:
the Tune plane's interactive CCR config editing (v1B wave) makes the detect+config-write
pattern bidirectional — monocle can now also READ and WRITE CCR config interactively.

The CCR routing tree (default/background/think/longContext/webSearch/subagent model slots)
is the display model for the Tune plane's routing configuration section.

All other CCR genes remain as originally dispositioned.
