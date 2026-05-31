#!/usr/bin/env python3
"""
check_structural_claims.py — POL-12 Phase 1: Structural-claim freshness enforcement.

Implements monocle-structural-claim-check CI gate per ADR-0008
§Implementation Plan (Phase 1 scope) and SS-conventions-anti-patterns.md
§Structural-Claim Discipline.

POLICY SUMMARY (ADR-0008 v1.0.4):
  Every active structural claim in story Tasks checklists, Downstream Consumer
  Contract code blocks, and BC postcondition prose must match the canonical source
  of truth. Phase 1 scope (ADR-0008 §CI enforcement gate):
  - Type-identifier claims: Vec<X>, VecDeque<X>, Option<X> for App struct fields
  - Canonical source: SS-tui.md §App struct (lines 833-864 per ADR-0008 §Canonical
    Source Registry, corrected by architect Fix F-S025-ADV28-MED-002)

PHASE 2 scope (deferred to Phase 5 per ADR-0008 §Implementation Plan):
  - Markdown table shape extraction from crates/**/*.rs doc-comment tables

HISTORICAL ANCHOR EXEMPTIONS (a claim is exempt if any ONE holds):
  1. It appears inside a "## §Trace" section.
  2. The line is annotated with "<!-- structural-claim-historical -->" (same or
     adjacent line).
  3. The line contains a time qualifier establishing past state:
     "at S-NNN authoring time", "at T-NNN dispatch time", "as of v", etc.

SCANNED PATHS (Phase 1 — ADR-0008 §CI enforcement gate):
  .factory/stories/*.md                          — Tasks checklists + Consumer Contract blocks
  .factory/specs/behavioral-contracts/**/*.md    — postcondition prose

EXEMPT PATHS (ADR-0008 §CI enforcement gate; Phase 1 scope is stories/ and
  behavioral-contracts/ only — plans/planning/code-delivery/STATE.md are implicitly
  outside that scope and require no additional exemptions per ADR-0008 §Scope note):
  .factory/cycles/             — closed adversarial cycle records
  .factory/plans/              — frozen adversary/audit records (historical, not normative)
  .factory/planning/           — validation reports (historical, not normative)
  .factory/code-delivery/      — at-merge PR description records (historical, not normative)
  .factory/STATE.md            — living dashboard/log (excluded as specific file)
  scripts/tests/               — test fixture directory

CANONICAL SOURCE REGISTRY (ADR-0008 §Canonical Source Registry):
  App struct field types → SS-tui.md §App struct (lines 833-864)
  Canonical fields:
    mode: AppMode
    overlay_stack: VecDeque<PromptModal>
    sessions: Vec<EnrichedSession>
    events: VecDeque<HookEventRow>
    drop_counter: u64
    dispatcher: Dispatcher
    matcher: nucleo::Matcher
    ipc_tx: tokio::sync::mpsc::Sender<ClientToServer>
    ipc_rx: tokio::sync::mpsc::Receiver<ServerToClient>

  This registry is itself subject to POL-12 per ADR-0008 §Canonical Source Registry
  self-application policy: if SS-tui.md §App struct changes, this script MUST be
  updated to match. State-manager obligation on each SS-tui bump that affects App struct.

USAGE:
  # Run from workspace root (reads SS-tui.md from .factory/):
  python3 scripts/check_structural_claims.py

  # With custom factory root (CI):
  python3 scripts/check_structural_claims.py --factory-root .factory-spec

  # Dry-run (report but do not fail):
  python3 scripts/check_structural_claims.py --dry-run

EXIT CODES:
  0 — all active structural claims match canonical types (or no active claims found)
  1 — one or more stale structural claims detected
  2 — configuration error (canonical source not found, etc.)
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────────────────
# Canonical App struct fields (ADR-0008 §Canonical Source Registry, Row 1).
#
# Source: SS-tui.md §App struct (lines 833-864 per ADR-0008 §Canonical Source Registry,
# corrected from off-by-2 lines 831-864 by F-S025-ADV28-MED-002 / architect 12170b4).
#
# Key: field name (lowercase, exact)
# Value: canonical type string (exactly as in SS-tui.md §App struct)
#
# MAINTAINER OBLIGATION (ADR-0008 §Implementation Plan row 4):
#   When a new canonical type is added to App struct in a future story,
#   update this dict in the same PR that updates SS-tui.md §App struct.
#   This is a per-story-cycle obligation for the implementing architect.
# ───────────────────────────────────────────────────────────────────────────────
_CANONICAL_APP_FIELDS: dict[str, str] = {
    "mode":         "AppMode",
    "overlay_stack": "VecDeque<PromptModal>",
    "sessions":     "Vec<EnrichedSession>",
    "events":       "VecDeque<HookEventRow>",
    "drop_counter": "u64",
    "dispatcher":   "Dispatcher",
    "matcher":      "nucleo::Matcher",
    "ipc_tx":       "tokio::sync::mpsc::Sender<ClientToServer>",
    "ipc_rx":       "tokio::sync::mpsc::Receiver<ServerToClient>",
}

# ───────────────────────────────────────────────────────────────────────────────
# Patterns for extracting type-identifier claims from story Markdown.
#
# Phase 1 scope covers:
#   a) Rust code blocks (```rust ... ```) — field declarations in struct bodies
#   b) Tasks checklist lines — "sessions: Vec<...>" style references
#
# We extract:
#   - "field_name: TypeExpression" from struct field lines
#   - Inline Vec<X> / VecDeque<X> / Option<X> patterns in checklist lines
#     that mention an App field by name
# ───────────────────────────────────────────────────────────────────────────────

# Matches a struct field declaration (possibly with pub/pub(crate)):
#   pub sessions: Vec<EnrichedSession>,
#   pub(crate) overlay_stack: VecDeque<PromptModal>,
#   mode: AppMode,
_STRUCT_FIELD_RE = re.compile(
    r"""
    (?:pub(?:\([a-z:]+\))?\s+)?         # optional pub/pub(crate)
    ([a-z_][a-z0-9_]*)                  # field name (group 1)
    \s*:\s*
    (                                    # type expression (group 2) — greedy until comma/newline/}
        [A-Za-z_<>:,\[\] \t0-9]+
    )
    \s*[,}]?
    """,
    re.VERBOSE,
)

# Matches "fieldname: TypeExpr" in non-code prose (Tasks checklist, inline descriptions).
# Less strict: allows the form `sessions: Vec<...>` anywhere on a line.
_INLINE_FIELD_TYPE_RE = re.compile(
    r'([a-z_][a-z0-9_]*)\s*:\s*([A-Za-z_<>:,\[\]0-9]+)',
)

# Matches "App.<field>: <Type>" directly — the qualified form that unambiguously
# asserts a claim about the App struct field type.
# Example: "App.sessions: Vec<EnrichedSession>" or "`App.overlay_stack: VecDeque<PromptModal>`"
_APP_QUALIFIED_FIELD_RE = re.compile(
    r'[Aa]pp\.([a-z_][a-z0-9_]*)\s*:\s*([A-Za-z_<>:,0-9]+)',
)

# Container type pattern — matches Vec<X>, VecDeque<X>, Option<X> with type arg
_CONTAINER_TYPE_RE = re.compile(
    r'(Vec|VecDeque|Option)<([A-Za-z_<>:,0-9 ]+)>',
)

# ───────────────────────────────────────────────────────────────────────────────
# §Trace section tracking (same logic as check_version_pins.py)
# ───────────────────────────────────────────────────────────────────────────────
_TRACE_HEADER_RE = re.compile(r'^#{1,6}\s+§Trace', re.IGNORECASE)
_ANY_HEADER_RE = re.compile(r'^(#{1,6})\s+')

# Structural-claim historical marker
_STRUCTURAL_HISTORICAL_MARKER = "<!-- structural-claim-historical -->"

# Time qualifiers for historical-anchor classification
_TIME_QUALIFIERS: list[str] = [
    "at time of",
    "authoring time",
    "at spec authoring",
    "at initial authoring",
    "at s-",       # "at S-025 authoring time" etc.
    "at t-",       # "at T-NNN dispatch time"
    "dispatch time",
    "at scan time",
    "as of v",
    "illustrative subset",  # explicit "not a structural claim" marker
    "s-025-introduced fields",  # explicit "partial/subset" marker
]


@dataclass
class StructuralFinding:
    """A stale structural claim detected by POL-12."""

    file_path: str
    line_number: int
    field_name: str
    cited_type: str        # type cited in the artifact
    canonical_type: str    # type from canonical source
    context: str           # "code_block" or "tasks_inline"
    line_text: str         # raw line content (stripped)


@dataclass
class ScanStats:
    """Counters collected during a scan run."""

    files_scanned: int = 0
    claims_found: int = 0
    claims_historical: int = 0
    claims_active: int = 0
    findings: list[StructuralFinding] = field(default_factory=list)


def _update_trace_state(line: str, in_trace: bool, trace_level: int) -> tuple[bool, int]:
    """Update §Trace section tracking state."""
    header_m = _ANY_HEADER_RE.match(line)
    if header_m:
        hashes = header_m.group(1)
        level = len(hashes)
        if _TRACE_HEADER_RE.match(line):
            return True, level
        elif in_trace and level <= trace_level:
            return False, 0
    return in_trace, trace_level


def _is_historical_anchor(
    line: str,
    prev_line: str,
    in_trace: bool,
    pre_block_context: list[str] | None = None,
) -> bool:
    """
    Return True if this structural claim line is a historical anchor.

    ADR-0008 §Historical Anchor Classification:
    1. Inside a §Trace section.
    2. Line, adjacent preceding line, OR any line in pre_block_context contains
       <!-- structural-claim-historical -->.
    3. Line, adjacent preceding line, OR any line in pre_block_context contains
       a time qualifier (for code blocks: the comment on the line before the
       code block's field declaration, or the annotation before the block).
    """
    if in_trace:
        return True

    context_lines = [line, prev_line] + (pre_block_context or [])

    for ctx in context_lines:
        if _STRUCTURAL_HISTORICAL_MARKER in ctx:
            return True

    # Check time qualifiers on line itself, prev_line, and pre_block_context
    for ctx in context_lines:
        lower_ctx = ctx.lower()
        for qualifier in _TIME_QUALIFIERS:
            if qualifier in lower_ctx:
                return True

    return False


def _extract_type_clean(raw_type: str) -> str:
    """Strip whitespace and trailing punctuation from a type expression."""
    return raw_type.strip().rstrip(",;").strip()


def scan_file(
    file_path: Path,
    stats: ScanStats,
) -> None:
    """
    Scan a single story markdown file for structural claims.

    Phase 1 scope:
    - Rust code blocks: scan for field declarations matching App struct fields
    - Tasks checklist lines: scan for inline "field: Type" patterns
    """
    try:
        text = file_path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"Warning: skipping {file_path}: {exc}", file=sys.stderr)
        return

    stats.files_scanned += 1
    lines = text.splitlines()

    in_trace = False
    trace_level = 0
    in_code_block = False
    code_block_lang = ""
    prev_line = ""
    _CONTEXT_WINDOW = 5
    # Lines immediately before the current code block's opening fence (pre-block context).
    # Checked for block-level historical-anchor annotations.
    pre_block_context: list[str] = []
    # Lines inside the current code block preceding the current field line
    # (intra-block context). Checked for comment-level historical qualifiers.
    intra_block_context: list[str] = []
    # Track whether we're currently inside a `struct App { ... }` body.
    # Only field declarations inside App struct bodies are checked.
    in_app_struct = False
    app_struct_brace_depth = 0

    for lineno, raw_line in enumerate(lines, 1):
        # Update §Trace tracking (before any other processing)
        in_trace, trace_level = _update_trace_state(raw_line, in_trace, trace_level)

        # Track code block boundaries
        stripped = raw_line.strip()
        if stripped.startswith("```"):
            if not in_code_block:
                in_code_block = True
                code_block_lang = stripped[3:].strip().lower()
                # Snapshot pre-block context at block open. Reset intra-block context.
                intra_block_context = []
                in_app_struct = False
                app_struct_brace_depth = 0
            else:
                in_code_block = False
                code_block_lang = ""
                pre_block_context = []
                intra_block_context = []
                in_app_struct = False
                app_struct_brace_depth = 0
            prev_line = raw_line
            continue

        if in_code_block and code_block_lang in ("rust", ""):
            # Track App struct body boundaries within the code block.
            # We enter App struct scope when we see a line matching:
            #   "struct App {" or "pub struct App {" (with or without opening brace)
            # We exit when the matching closing "}" is found.
            if not in_app_struct:
                if re.search(r'\bstruct\s+App\s*\{', raw_line) or re.search(r'\bstruct\s+App\s*$', raw_line):
                    in_app_struct = True
                    app_struct_brace_depth = raw_line.count("{") - raw_line.count("}")
            else:
                app_struct_brace_depth += raw_line.count("{") - raw_line.count("}")
                if app_struct_brace_depth <= 0:
                    in_app_struct = False
                    app_struct_brace_depth = 0

            # Scan struct field declarations inside Rust code blocks.
            # Pass both pre_block_context (before the fence) and
            # intra_block_context (comment lines within the block)
            # so all historical-anchor forms are detected.
            _check_code_block_line(
                raw_line, lineno, str(file_path), in_trace, prev_line,
                pre_block_context + intra_block_context, stats,
                in_app_struct=in_app_struct,
            )
            # Maintain intra-block context window
            intra_block_context.append(raw_line)
            if len(intra_block_context) > _CONTEXT_WINDOW:
                intra_block_context.pop(0)
        elif not in_code_block:
            # Scan Tasks checklist lines and prose for inline type references
            _check_prose_line(
                raw_line, lineno, str(file_path), in_trace, prev_line, stats
            )
            # Maintain pre_block_context window (only updated outside code blocks)
            pre_block_context.append(raw_line)
            if len(pre_block_context) > _CONTEXT_WINDOW:
                pre_block_context.pop(0)

        prev_line = raw_line


def _check_code_block_line(
    line: str,
    lineno: int,
    file_path: str,
    in_trace: bool,
    prev_line: str,
    pre_block_context: list[str],
    stats: ScanStats,
    in_app_struct: bool,
) -> None:
    """
    Check a line inside a Rust code block for App struct field claims.

    Only checks field declarations when inside an `App` struct body (in_app_struct=True).
    This prevents false positives from other struct definitions (DaemonState, etc.)
    that share field names with App (mode, drop_counter, sessions, etc.).

    pre_block_context: lines appearing before the opening ``` fence, used to
    detect block-level historical-anchor annotations.
    """
    if not in_app_struct:
        return

    m = _STRUCT_FIELD_RE.search(line)
    if not m:
        return

    field_name = m.group(1).strip()
    raw_type = m.group(2)
    cited_type = _extract_type_clean(raw_type)

    # Only check fields that are known App struct fields
    if field_name not in _CANONICAL_APP_FIELDS:
        return

    stats.claims_found += 1

    if _is_historical_anchor(line, prev_line, in_trace, pre_block_context):
        stats.claims_historical += 1
        return

    stats.claims_active += 1
    canonical_type = _CANONICAL_APP_FIELDS[field_name]

    if not _types_match(cited_type, canonical_type):
        stats.findings.append(
            StructuralFinding(
                file_path=file_path,
                line_number=lineno,
                field_name=field_name,
                cited_type=cited_type,
                canonical_type=canonical_type,
                context="code_block",
                line_text=line.strip(),
            )
        )


def _check_prose_line(
    line: str,
    lineno: int,
    file_path: str,
    in_trace: bool,
    prev_line: str,
    stats: ScanStats,
) -> None:
    """
    Check a prose line (Tasks checklist, description, BC postcondition) for
    inline type-identifier claims that are specifically about App struct fields.

    To avoid false positives from unrelated structs (DaemonState, ServerToClient,
    IPC messages, etc.) that share field names, we only check prose lines that
    explicitly mention "App" in an App-struct context, AND we require the cited
    type to be a well-formed Rust container type (Vec<>, VecDeque<>, Option<>, or
    a named type with angle-bracket generics starting with uppercase).

    Two detection paths:

    Path A — App-qualified form (highest confidence):
      "App.field: Vec<...>" or "`App.field: Vec<...>`" (direct struct qualifier).
      Used by _APP_QUALIFIED_FIELD_RE. Only flags the field: Type that directly
      follows "App." — avoids cross-clause contamination where "App.foo" appears
      in one clause and "foo: DaemonType" appears in another clause on the same line.

    Path B — Tasks-checklist / BC bullet form (list context):
      "`field: Vec<...>`" in backtick inline code on a line without an "App." qualifier
      that would make Path A more appropriate. Used for BC postcondition bullet lists
      like "- `sessions: Vec<EnrichedSession>` — the full current session roster".
      Triggered by "`field:" fast-path. Requires the type to be a well-formed Rust
      container type (must start with Vec, VecDeque, Option, or uppercase letter with
      angle-brackets) to filter IPC message value forms like `sessions: [<remaining>]`.
    """
    # Fast-path: must have a container type indicator
    if "<" not in line:
        return

    line_lower = line.lower()

    # ── Path A: App-qualified form ──────────────────────────────────────────
    # Scan for "App.<field>: <Type>" directly — the unambiguous App-struct form.
    # This path is the primary detector for BC postcondition prose where the
    # author writes "App.sessions: Vec<EnrichedSession>" or similar.
    app_qualified_fields_found: set[str] = set()
    for m in _APP_QUALIFIED_FIELD_RE.finditer(line):
        field_name = m.group(1).strip()
        raw_type = m.group(2)
        cited_type = _extract_type_clean(raw_type)

        if field_name not in _CANONICAL_APP_FIELDS:
            continue
        if "<" not in cited_type:
            continue

        app_qualified_fields_found.add(field_name)
        stats.claims_found += 1

        if _is_historical_anchor(line, prev_line, in_trace):
            stats.claims_historical += 1
            continue

        stats.claims_active += 1
        canonical_type = _CANONICAL_APP_FIELDS[field_name]

        if not _types_match(cited_type, canonical_type):
            stats.findings.append(
                StructuralFinding(
                    file_path=file_path,
                    line_number=lineno,
                    field_name=field_name,
                    cited_type=cited_type,
                    canonical_type=canonical_type,
                    context="tasks_inline",
                    line_text=line.strip(),
                )
            )

    # ── Path B: Tasks-checklist / BC bullet form ────────────────────────────
    # For lines with "`field: Type`" (backtick-enclosed field+type on the same
    # token) that are NOT already handled by Path A.
    #
    # Require ALL of:
    #   1. The line triggers a specific App-context fast-path signal
    #      (app struct / struct app context, OR "`field:" backtick form)
    #   2. The cited type is a well-formed Rust container type:
    #      starts with Vec/VecDeque/Option, or starts with uppercase letter
    #      followed by '<' (named generic). This filters IPC value forms like
    #      `sessions: [<remaining>]` and non-type values.
    #   3. The field was NOT already processed by Path A (no double-counting).
    #
    # The "App." fast-path signal from the OLD logic is intentionally NOT used
    # here: if "App." appears on the line, Path A already handles it specifically.
    # Using "App." as a signal for Path B causes cross-clause contamination
    # (where "App.overlay_stack" in one clause triggers Path B to scan all
    # field: Type pairs, including daemon IPC fields on the same line).

    # Path B signals: struct-context words OR backtick-enclosed field: pattern
    has_struct_context = (
        "app struct" in line_lower
        or "struct app" in line_lower
        or "impl app" in line_lower
    )

    # Check for "`field:" pattern where field is a known App field
    has_backtick_field_signal = False
    for field in _CANONICAL_APP_FIELDS:
        if f"`{field}:" in line_lower:
            has_backtick_field_signal = True
            break

    if not has_struct_context and not has_backtick_field_signal:
        return

    # IPC-ownership guard (Path B only): if the line describes the daemon/IPC layer
    # owning the field, it is NOT describing an App struct field even if a field name
    # matches. These markers indicate the `field: Type` belongs to the IPC message
    # or DaemonState, not the TUI App struct.
    #
    # This prevents false positives from lines like:
    #   "the daemon's `overlay_stack: Vec<PermissionPromptPayload>` in IPC"
    # which describe the daemon's IPC registry, not App.overlay_stack (VecDeque<PromptModal>).
    #
    # Path A (App-qualified form: "App.field: Type") is NOT subject to this guard —
    # it already requires the literal "App." qualifier which unambiguously names the
    # TUI struct owner.
    _IPC_OWNERSHIP_MARKERS = (
        " in ipc",                # "overlay_stack: Vec<...> in IPC"
        "(ipc)",                  # field labeled as belonging to IPC
        "in ipc.",                # trailing form
        "daemon's ",              # "the daemon's overlay_stack"
        "daemon owns",            # "daemon owns the pending-prompt registry"
        "daemonstate",            # DaemonState struct reference
        "pending-prompt registry",  # the IPC registry descriptor
        "initialstate.",          # InitialState IPC message field reference
        "initialstate {",         # InitialState struct literal
    )
    if any(marker in line_lower for marker in _IPC_OWNERSHIP_MARKERS):
        return

    # IPC-homonym exclusion (Path B only): fields whose names appear in BOTH
    # the TUI App struct AND IPC message types with DIFFERENT types cannot be
    # reliably classified in the backtick bullet-list form (Path B) without
    # multi-line context. These fields are excluded from Path B detection.
    # Path A (App.field: Type) still detects App-qualified claims for these fields.
    #
    # Known homonyms (App type → IPC type):
    #   overlay_stack: VecDeque<PromptModal> (App) vs Vec<PermissionPromptPayload> (IPC/InitialState)
    #
    # When overlay_stack appears as "`overlay_stack: Vec<PermissionPromptPayload>`" in a
    # BC postcondition bullet listing InitialState fields, Path B cannot determine
    # ownership from the line alone — the IPC context is established by the surrounding
    # paragraph. Excluding overlay_stack from Path B avoids this false positive class.
    # If a story or BC authors an App.overlay_stack type claim, they should use the
    # App-qualified form (Path A): "App.overlay_stack: VecDeque<PromptModal>".
    _PATH_B_EXCLUDED_FIELDS: frozenset[str] = frozenset({
        "overlay_stack",  # IPC homonym: Vec<PermissionPromptPayload> vs VecDeque<PromptModal>
    })

    # _RUST_CONTAINER_RE: matches types that start with Vec/VecDeque/Option
    # or start with an uppercase letter followed by '<' (named generic).
    # This excludes IPC value forms like `[<remaining>]` and bare array literals.
    _RUST_CONTAINER_RE = re.compile(
        r'^(?:Vec|VecDeque|Option)<|^[A-Z][A-Za-z_0-9]*(?:::[A-Za-z_0-9]+)*<'
    )

    for m in _INLINE_FIELD_TYPE_RE.finditer(line):
        field_name = m.group(1).strip()
        raw_type = m.group(2)
        cited_type = _extract_type_clean(raw_type)

        # Only check fields that are known App struct fields
        if field_name not in _CANONICAL_APP_FIELDS:
            continue

        # Skip if already handled by Path A (avoid double-counting)
        if field_name in app_qualified_fields_found:
            continue

        # Skip IPC-homonym fields in Path B (unresolvable without multi-line context).
        # Path A handles the App-qualified form for these fields.
        if field_name in _PATH_B_EXCLUDED_FIELDS:
            continue

        # Require the cited type to be a well-formed Rust container type
        if not _RUST_CONTAINER_RE.match(cited_type):
            continue

        stats.claims_found += 1

        if _is_historical_anchor(line, prev_line, in_trace):
            stats.claims_historical += 1
            continue

        stats.claims_active += 1
        canonical_type = _CANONICAL_APP_FIELDS[field_name]

        if not _types_match(cited_type, canonical_type):
            stats.findings.append(
                StructuralFinding(
                    file_path=file_path,
                    line_number=lineno,
                    field_name=field_name,
                    cited_type=cited_type,
                    canonical_type=canonical_type,
                    context="tasks_inline",
                    line_text=line.strip(),
                )
            )


def _types_match(cited: str, canonical: str) -> bool:
    """
    Determine whether a cited type matches the canonical type.

    Performs normalised comparison: strips whitespace, ignores trailing commas.
    A cited type is a prefix-match if it is a non-generic simplification
    (e.g., "Vec<EnrichedSession>" cited as "Vec" is NOT a match — we require
    the full generic form).

    For partial matches where the cited type is a valid subset prefix of the
    canonical type (e.g., "Sender<ClientToServer>" vs the fully-qualified
    "tokio::sync::mpsc::Sender<ClientToServer>"), we allow the shorter form
    if the type base name and generic argument both match exactly.
    """
    cited_norm = cited.strip().rstrip(",;").strip()
    canon_norm = canonical.strip().rstrip(",;").strip()

    if cited_norm == canon_norm:
        return True

    # Allow path-shortened forms: e.g., "mpsc::Sender<ClientToServer>" matches
    # "tokio::sync::mpsc::Sender<ClientToServer>", and "Sender<ClientToServer>"
    # also matches if the type argument is identical.
    if "::" in canon_norm:
        canon_last = canon_norm.rsplit("::", 1)[-1]
        if cited_norm == canon_last:
            return True

    # If cited ends with canonical (suffix match for path-qualified types)
    if canon_norm.endswith(cited_norm) or cited_norm.endswith(canon_norm):
        return True

    return False


@dataclass
class DeferralEntry:
    """An authorized deferral from structural-claim-deferrals.yaml."""
    file: str            # relative to factory root (e.g., "stories/S-028-...")
    line_pattern: str    # substring pattern to match the deferred line
    field: str           # App struct field name
    cited_type: str      # stale type being deferred
    fix_target: str      # story/wave that will fix this


def load_deferrals(workspace_root: Path) -> list[DeferralEntry]:
    """
    Load authorized structural-claim deferrals from scripts/structural-claim-deferrals.yaml.

    Returns empty list if the file doesn't exist (no deferrals authorized).
    Uses stdlib only — simple line-oriented YAML parser.
    """
    deferrals_path = workspace_root / "scripts" / "structural-claim-deferrals.yaml"
    if not deferrals_path.is_file():
        return []

    try:
        text = deferrals_path.read_text(encoding="utf-8")
    except OSError:
        return []

    deferrals: list[DeferralEntry] = []
    current: dict[str, str] = {}

    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("#") or not stripped:
            # New entry boundary: flush if we have a complete entry
            if current.get("file") and current.get("line_pattern"):
                deferrals.append(
                    DeferralEntry(
                        file=current["file"],
                        line_pattern=current.get("line_pattern", ""),
                        field=current.get("field", ""),
                        cited_type=current.get("cited_type", ""),
                        fix_target=current.get("fix_target", ""),
                    )
                )
                current = {}
            continue

        if stripped.startswith("- file:"):
            # Flush previous entry
            if current.get("file") and current.get("line_pattern"):
                deferrals.append(
                    DeferralEntry(
                        file=current["file"],
                        line_pattern=current.get("line_pattern", ""),
                        field=current.get("field", ""),
                        cited_type=current.get("cited_type", ""),
                        fix_target=current.get("fix_target", ""),
                    )
                )
            current = {}
            current["file"] = stripped.split(":", 1)[1].strip()
        elif stripped.startswith("line_pattern:"):
            current["line_pattern"] = stripped.split(":", 1)[1].strip().strip('"')
        elif stripped.startswith("field:"):
            current["field"] = stripped.split(":", 1)[1].strip()
        elif stripped.startswith("cited_type:"):
            current["cited_type"] = stripped.split(":", 1)[1].strip().strip('"')
        elif stripped.startswith("fix_target:"):
            current["fix_target"] = stripped.split(":", 1)[1].strip().strip('"')

    # Flush final entry
    if current.get("file") and current.get("line_pattern"):
        deferrals.append(
            DeferralEntry(
                file=current["file"],
                line_pattern=current.get("line_pattern", ""),
                field=current.get("field", ""),
                cited_type=current.get("cited_type", ""),
                fix_target=current.get("fix_target", ""),
            )
        )

    return deferrals


def _is_deferred(finding: StructuralFinding, deferrals: list[DeferralEntry], factory_root: Path) -> bool:
    """
    Return True if this finding matches an authorized deferral entry.

    Matches on:
    - file path ending (finding.file_path ends with deferral.file)
    - line text contains deferral.line_pattern
    - field matches
    """
    for d in deferrals:
        # Normalize path comparison: check if finding path ends with the deferral's relative file
        norm_finding = finding.file_path.replace("\\", "/")
        norm_deferral = d.file.replace("\\", "/").strip()
        # Match if the finding path ends with the deferral file path
        if not (norm_finding.endswith(norm_deferral) or norm_finding.endswith("/" + norm_deferral.lstrip("/"))):
            continue
        if d.line_pattern and d.line_pattern not in finding.line_text:
            continue
        if d.field and d.field != finding.field_name:
            continue
        return True
    return False


def collect_story_files(factory_root: Path) -> list[Path]:
    """
    Collect story markdown files from .factory/stories/.

    Excludes .factory/cycles/ (always excluded by parent path).
    Returns sorted list of Path objects.
    """
    stories_dir = factory_root / "stories"
    if not stories_dir.is_dir():
        return []

    files = []
    for entry in sorted(stories_dir.iterdir()):
        if entry.is_file() and entry.suffix.lower() == ".md":
            files.append(entry)

    return files


def collect_bc_files(factory_root: Path) -> list[Path]:
    """
    Collect behavioral-contract markdown files from .factory/specs/behavioral-contracts/.

    Recursively walks all subdirectories (ss-01, ss-02, etc.) and collects *.md files.
    ADR-0008 §CI enforcement gate Phase 1 mandates scanning
    .factory/specs/behavioral-contracts/**/*.md for postcondition prose structural claims.

    Returns sorted list of Path objects.
    """
    bc_dir = factory_root / "specs" / "behavioral-contracts"
    if not bc_dir.is_dir():
        return []

    files = []
    for entry in sorted(bc_dir.rglob("*.md")):
        if entry.is_file():
            files.append(entry)

    return sorted(files)


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "POL-12 Phase 1: Structural-claim freshness enforcement "
            "(monocle-structural-claim-check).\n"
            "Verifies type-identifier claims in story Tasks + Consumer Contract blocks\n"
            "match canonical App struct field types from SS-tui.md §App struct.\n"
            "Implements ADR-0008 §Implementation Plan Phase 1."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--factory-root",
        metavar="DIR",
        default=".factory",
        help=(
            "Directory where factory-artifacts are mounted (default: .factory). "
            "In CI, factory-artifacts are checked out into .factory-spec/ — "
            "pass --factory-root .factory-spec in that case."
        ),
    )
    parser.add_argument(
        "--workspace-root",
        metavar="DIR",
        default=".",
        help="Workspace root directory (default: current working directory).",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Report findings but exit 0 (do not fail CI). For investigation use only.",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print detailed progress.",
    )
    args = parser.parse_args()

    workspace_root = Path(args.workspace_root).resolve()
    factory_root = workspace_root / args.factory_root

    if not factory_root.is_dir():
        print(
            f"Error: factory root not found at {factory_root}.\n"
            f"In CI, pass --factory-root .factory-spec after checking out factory-artifacts.",
            file=sys.stderr,
        )
        sys.exit(2)

    # Collect story files + BC files (Phase 1 scope — ADR-0008 §CI enforcement gate)
    story_files = collect_story_files(factory_root)
    bc_files = collect_bc_files(factory_root)
    all_files = story_files + bc_files
    if args.verbose:
        print(
            f"POL-12 Phase 1: scanning {len(story_files)} story file(s) + "
            f"{len(bc_files)} behavioral-contract file(s) for structural claims..."
        )

    stats = ScanStats()
    for file_path in all_files:
        scan_file(file_path, stats)

    # Apply authorized deferrals
    deferrals = load_deferrals(workspace_root)
    if deferrals and args.verbose:
        print(f"Loaded {len(deferrals)} authorized deferral(s) from scripts/structural-claim-deferrals.yaml")

    active_findings: list[StructuralFinding] = []
    deferred_findings: list[StructuralFinding] = []
    for f in stats.findings:
        if _is_deferred(f, deferrals, factory_root):
            deferred_findings.append(f)
        else:
            active_findings.append(f)

    if deferred_findings and args.verbose:
        print(f"\nAuthorized deferrals (not blocking CI):")
        for f in deferred_findings:
            # Find matching deferral for fix_target info
            for d in deferrals:
                norm_finding = f.file_path.replace("\\", "/")
                if d.file.replace("\\", "/") in norm_finding:
                    print(f"  DEFERRED {f.file_path}:{f.line_number} — fix target: {d.fix_target}")
                    break

    n_findings = len(active_findings)

    # Reassign stats.findings to only active (non-deferred) findings for reporting
    stats.findings = active_findings

    if n_findings > 0:
        print(
            f"\nPOL-12 FAIL: {n_findings} stale structural claim(s) detected.\n"
            f"(ADR-0008 §CI enforcement gate — monocle-structural-claim-check Phase 1)\n",
            file=sys.stderr,
        )
        for f in sorted(stats.findings, key=lambda x: (x.file_path, x.line_number)):
            print(
                f"  {f.file_path}:{f.line_number} [{f.context}]: "
                f"App.{f.field_name} cited as '{f.cited_type}' "
                f"but canonical SS-tui.md §App struct declares '{f.canonical_type}'\n"
                f"    Line: {f.line_text[:120]}",
                file=sys.stderr,
            )
        print(
            f"\nFix options per ADR-0008 §Decision:\n"
            f"  1. Update the cited type to match the canonical source exactly.\n"
            f"  2. Add '<!-- structural-claim-historical -->' annotation if this IS\n"
            f"     an intentional historical record of past state.\n"
            f"  3. Add a time qualifier: 'at S-NNN authoring time', 'as of v...', etc.",
            file=sys.stderr,
        )
        if not args.dry_run:
            sys.exit(1)
        else:
            print("(--dry-run: exiting 0 despite findings)", file=sys.stderr)
    else:
        print(
            f"POL-12 PASS: all active structural claims match canonical types "
            f"({stats.claims_active} active, {stats.claims_historical} historical anchors, "
            f"{stats.files_scanned} file(s) scanned)."
        )

    if args.verbose:
        print(
            f"\nScan summary:\n"
            f"  Files scanned:      {stats.files_scanned}\n"
            f"  Claims found total: {stats.claims_found}\n"
            f"  Claims historical:  {stats.claims_historical}\n"
            f"  Claims active:      {stats.claims_active}\n"
            f"  Findings (stale):   {n_findings}"
        )


if __name__ == "__main__":
    main()
