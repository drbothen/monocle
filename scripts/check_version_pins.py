#!/usr/bin/env python3
"""
check_version_pins.py — POL-11: Version-pin literal freshness enforcement.

Implements monocle-version-pin-freshness CI gate per ADR-0007
§Implementation Plan and SS-conventions-anti-patterns.md §Citation Discipline.

POLICY SUMMARY (ADR-0007 v1.0.5):
  Every active version-pin literal in the repository must match the canonical
  current version in .factory/specs/version-pin-registry.yaml (or the cited
  document's frontmatter, as fallback). Active means NOT classified as a
  historical anchor per ADR-0007 §Historical Anchor Classification.

HISTORICAL ANCHOR EXEMPTIONS (a citation is exempt if any ONE holds):
  1. It appears inside a "## §Trace" section (header tracking).
  2. The line contains "# version-pin-historical" (Rust/TOML/YAML) or
     "<!-- version-pin-historical -->" (Markdown).
  3. The line contains a time qualifier:
       "at time of", "at S-NNN authoring time", "at T-NNN dispatch time",
       "at spec authoring time", "at time of ratification",
       "at initial authoring", "at S-025 authoring", "authoring time",
       "authoring baseline", "at scan time", "as of vN.M at", "dispatch time"
  4. Multi-line split: when an artifact ID ends line N and the version vX.Y starts
     line N+1, the pair is detected as a cross-line pin. Historical-anchor exemption
     is checked against BOTH lines: if EITHER carries an exemption marker (annotation
     or time qualifier), the pin is historical.

PATTERN B — YAML FRONTMATTER FORM:
  In addition to the inline prose form (Pattern A), POL-11 detects the YAML
  structured inputs[] form used in story and index-doc frontmatter:
    {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}
  Classification per ADR-0007 §Story inputs[] Historical Provenance:
  - File path matches stories/S-[0-9]+-*.md (individual story files):
      → classify HISTORICAL — logged explicitly ("[HISTORICAL] inputs[] provenance,
        exempt") but NOT checked for staleness. Story inputs[] records are authored-
        against provenance frozen at story authoring time.
  - File basename is a living index document (STORY-INDEX.md, BC-INDEX.md,
    ARCH-INDEX.md, VP-INDEX.md, prd.md, L2-INDEX.md):
      → classify ACTIVE — checked against registry. Index documents are
        continuously-maintained and their inputs[] is a live declaration.
  - Any other file with YAML-form pin:
      → classify ACTIVE (conservative default per ADR-0007 §POL-11 implementation).
  The gate never silently ignores the YAML form — handling is always explicit
  (HISTORICAL or ACTIVE, never unhandled).

SCANNED FILE TYPES: .md, .rs, .toml, .yml, .yaml

EXEMPT PATHS (not scanned):
  <factory-root>/cycles/        — closed adversarial cycle records (path relative to factory_root)
  <factory-root>/plans/         — frozen adversary/audit records (historical, not normative)
  <factory-root>/planning/      — validation reports (historical, not normative)
  <factory-root>/code-delivery/ — at-merge PR description records (historical, not normative)
  <factory-root>/STATE.md       — living dashboard/log (excluded as specific file, not a dir)
  target/                       — cargo build output
  .git/                         — git internals
  node_modules/                 — not applicable
  scripts/tests/                — test fixture directory (fixtures are intentionally wrong)

NORMATIVE PATHS (scanned):
  <factory-root>/stories/       — active story files with version-pin literals
  <factory-root>/specs/         — all spec subdirectories (SS-*, ADR-*, BC-INDEX, etc.)
  <factory-root>/prd.md, product-brief.md, dtu-assessment.md — top-level normative docs
  workspace crates/, .github/, scripts/ (excl. scripts/tests/), root config files

USAGE:
  # Run from workspace root:
  python3 scripts/check_version_pins.py

  # With custom registry path (testing):
  python3 scripts/check_version_pins.py --registry /path/to/version-pin-registry.yaml

  # With specific factory root (CI: checks out factory-artifacts separately):
  python3 scripts/check_version_pins.py --factory-root .factory-spec

  # Dry-run (report but do not fail):
  python3 scripts/check_version_pins.py --dry-run

REGISTRY LOCATION:
  The registry is read from <factory-root>/specs/version-pin-registry.yaml relative
  to the workspace root. In CI the factory-artifacts branch is checked out into
  .factory-spec/ for spec access; pass --factory-root .factory-spec in that case.
  The scan also covers the full factory-root tree (including stories/, specs/) because
  story files carry active version-pin literals that must stay fresh.

EXIT CODES:
  0 — all active version-pin literals match canonical versions (or no literals found)
  1 — one or more stale active version-pin literals detected
  2 — configuration error (registry not found, malformed YAML, etc.)
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────────────────
# Version-pin literal detection pattern.
#
# Matches: SS-something.md v1.2.3  |  BC-2.06.005 v1.0.6  |  ADR-0007 v1.0.2
#          ADR-0007-some-name.md v1.0.0
#
# Pattern groups:
#   Group 1: artifact ID (SS-..., BC-N.NN.NNN, ADR-NNNN or ADR-NNNN-name.md)
#   Group 2: version literal (N.N or N.N.N)
# ───────────────────────────────────────────────────────────────────────────────
_VERSION_PIN_RE = re.compile(
    r"""
    (                                          # Group 1: artifact ID
        SS-[a-z][a-z0-9-]*(?:\.md)?            #   SS-something or SS-something.md
      | BC-[0-9]+\.[0-9]+\.[0-9]+              #   BC-N.NN.NNN (numeric only, no .md)
      | ADR-[0-9]+(?:-[a-z0-9-]+)*(?:\.md)?   #   ADR-0007 or ADR-0007-name or .md variant
    )
    [ \t]+                                     # required whitespace separator
    v([0-9]+\.[0-9]+(?:\.[0-9]+)?)             # Group 2: version literal vN.N or vN.N.N
    """,
    re.VERBOSE,
)

# ───────────────────────────────────────────────────────────────────────────────
# Multi-line split detection: artifact ID ends one line, version starts the next.
#
# Pattern PG-SPLIT: the artifact ID is at end-of-line (possibly followed by
# punctuation/whitespace), and the version vX.Y[.Z] opens the next line (possibly
# preceded by whitespace or punctuation). Example from S-025 AC-010:
#   line N:   "... (removed in BC-2.06.004"
#   line N+1: "v1.1.0 <!-- version-pin-historical: ... -->)"
#
# We detect this by matching an artifact-ID tail pattern on line N and a version
# lead pattern on line N+1. The match is verified end-of-line (artifact ID must
# appear as the last token, optionally followed by non-word chars).
# ───────────────────────────────────────────────────────────────────────────────

# Matches an artifact ID that appears near the end of a line (possibly followed
# by non-word chars like punctuation before EOL).
_SPLIT_LINE_ARTIFACT_RE = re.compile(
    r"""
    (                                          # Group 1: artifact ID
        SS-[a-z][a-z0-9-]*(?:\.md)?            #   SS-something or SS-something.md
      | BC-[0-9]+\.[0-9]+\.[0-9]+              #   BC-N.NN.NNN
      | ADR-[0-9]+(?:-[a-z0-9-]+)*(?:\.md)?   #   ADR-0007 or ADR-0007-name
    )
    [^\S\r\n]*                                  # optional trailing whitespace (not newline)
    [^a-zA-Z0-9\r\n]*                           # optional non-alphanumeric suffix (e.g. "(")
    $                                           # must be at end of line
    """,
    re.VERBOSE | re.MULTILINE,
)

# Matches a version literal at (or near) the start of a line (possibly after
# whitespace or leading punctuation like "(" or ")").
_SPLIT_LINE_VERSION_RE = re.compile(
    r"""
    ^                                           # start of line
    [^\S\r\n]*                                  # optional leading whitespace
    [^a-zA-Z0-9\r\n]*                           # optional leading punctuation (e.g. ")")
    v([0-9]+\.[0-9]+(?:\.[0-9]+)?)             # Group 1: version literal
    \b                                          # word boundary
    """,
    re.VERBOSE | re.MULTILINE,
)

# ───────────────────────────────────────────────────────────────────────────────
# Pattern B — YAML frontmatter inputs[] form.
#
# Matches: {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}
#          {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md, version: "1.2.1"}
#
# Also matches multi-line YAML block-sequence entries (each item is one line):
#   - {path: .factory/specs/architecture/SS-tui.md, version: "1.8.2"}
#
# Pattern groups:
#   Group 1: full path value (e.g., ".factory/specs/architecture/SS-tui.md")
#   Group 2: version string (e.g., "1.8.2")
#
# Note: the path field value may have no leading quote; version is always quoted
# in the actual story/index frontmatter. We allow either quoted or unquoted version
# to be robust, but the canonical form is double-quoted.
# ───────────────────────────────────────────────────────────────────────────────
_YAML_INPUT_PIN_RE = re.compile(
    r"""
    \{                                   # opening brace of inline mapping
    \s*path:\s*                          # "path:" key (with optional whitespace)
    ([^\s,}]+)                           # Group 1: path value (no spaces, commas, or braces)
    \s*,\s*                              # comma separator
    version:\s*                          # "version:" key
    ["']?                                # optional opening quote
    ([0-9]+\.[0-9]+(?:\.[0-9]+)?)        # Group 2: version literal (N.N or N.N.N)
    ["']?                                # optional closing quote
    \s*\}                                # closing brace
    """,
    re.VERBOSE,
)

# ───────────────────────────────────────────────────────────────────────────────
# Living index document basenames (ADR-0007 §Story inputs[] Historical Provenance).
#
# Files whose basename matches one of these are classified as ACTIVE when they
# contain YAML-form inputs[] pins. These are continuously-maintained index
# documents, not frozen story records.
# ───────────────────────────────────────────────────────────────────────────────
_LIVING_INDEX_BASENAMES: frozenset[str] = frozenset({
    "STORY-INDEX.md",
    "BC-INDEX.md",
    "ARCH-INDEX.md",
    "VP-INDEX.md",
    "prd.md",
    "L2-INDEX.md",
})

# ───────────────────────────────────────────────────────────────────────────────
# Individual story file pattern (ADR-0007 §Story inputs[] Historical Provenance).
#
# Story files are in stories/ and have basename starting with "S-":
#   - Standard stories: S-025-tui-skeleton-sessions.md (S-<digits>-<slug>)
#   - DTU stories: S-DTU-001-claude-code-hook-clone.md (S-DTU-<NNN>-<slug>)
#   - Prep stories: S-PHASE-3-PREP-spec-kit-mcp-integration.md (S-PHASE-...-<slug>)
#
# All share the pattern: in a stories/ directory with basename starting "S-".
# The living index STORY-INDEX.md is already caught by _LIVING_INDEX_BASENAMES
# before reaching this check (it does NOT start with "S-" followed by a hyphen
# and alphanumeric after "S-", but is named "STORY-INDEX" not "S-...").
#
# Actually STORY-INDEX.md DOES start with "S" but not with "S-" at basename level:
# "STORY-INDEX" != "S-*". The classification uses _LIVING_INDEX_BASENAMES FIRST,
# so STORY-INDEX.md is caught there before the story-file check applies.
# ───────────────────────────────────────────────────────────────────────────────
_STORY_FILE_RE = re.compile(r'(?:^|/)stories/S-[^/]+\.md$')

# Artifact ID normalisation: strip .md suffix and trailing whitespace.
_MD_SUFFIX_RE = re.compile(r'\.md$', re.IGNORECASE)

# ───────────────────────────────────────────────────────────────────────────────
# Historical-anchor time qualifiers (ADR-0007 §Historical Anchor Classification).
#
# A citation is a historical anchor (exempt from CI check) if the line contains
# any of these patterns (case-insensitive substring match).
# ───────────────────────────────────────────────────────────────────────────────
_TIME_QUALIFIERS: list[str] = [
    "at time of",
    "authoring time",
    "authoring baseline",
    "at spec authoring",
    "at initial authoring",
    "at time of ratification",
    "at s-",           # catches "at S-025 authoring time" etc. (lowercase after lower())
    "at t-",           # catches "at T-NNN dispatch time"
    "dispatch time",
    "at scan time",
    "tdd red-gate authoring baseline",
    "tdd red.gate",    # variants
    "as of v",         # "as of v1.0 at" pattern from PG-5 Form 2
    "f-d-01",          # TDD fix anchors that are definitionally historical
    "f-d-02",
    "f-d-03",
    "current canonical is",  # explicit "stale by design" notes in TDD tests
]

# ───────────────────────────────────────────────────────────────────────────────
# Explicit historical-anchor annotation markers.
#
# ADR-0007 §Historical Anchor Classification defines two annotation forms:
#   Rust/TOML/YAML: # version-pin-historical
#   Markdown:       <!-- version-pin-historical -->
#
# In practice, the Markdown form is often written with trailing content:
#   <!-- version-pin-historical: some reason here -->
# We match on the OPENING prefix "<!-- version-pin-historical" rather than the
# exact closed-comment string "<!-- version-pin-historical -->" so that both
# the bare form and the annotated-reason form are recognized.
# ───────────────────────────────────────────────────────────────────────────────
_HISTORICAL_MARKER_RS = "# version-pin-historical"
_HISTORICAL_MARKER_MD_PREFIX = "<!-- version-pin-historical"   # matches bare + annotated forms

# ───────────────────────────────────────────────────────────────────────────────
# §Trace section header pattern.
# A line matching this starts a §Trace section; any version-pin below it (until
# next same-level or higher ## heading) is a historical anchor.
# ───────────────────────────────────────────────────────────────────────────────
_TRACE_HEADER_RE = re.compile(r'^#{1,6}\s+§Trace', re.IGNORECASE)
_ANY_HEADER_RE = re.compile(r'^(#{1,6})\s+')

# ───────────────────────────────────────────────────────────────────────────────
# File extension filter — only scan these suffixes.
# ───────────────────────────────────────────────────────────────────────────────
_SCAN_EXTENSIONS = {".md", ".rs", ".toml", ".yml", ".yaml"}

# ───────────────────────────────────────────────────────────────────────────────
# Path exclusions — directories that are never scanned.
# These are checked as path components, not just prefixes.
# ───────────────────────────────────────────────────────────────────────────────
_EXCLUDED_DIRS = {
    "target",
    ".git",
    "node_modules",
}

# Static paths excluded by prefix (relative to workspace root, forward-slash notation).
# The factory-root/cycles/ exclusion is dynamic (depends on --factory-root arg) and is
# computed in collect_files() so it tracks whatever factory root name is in use.
_EXCLUDED_PATH_PREFIXES_STATIC: list[str] = [
    "scripts/tests/",   # test fixtures are intentionally stale — excluded from main scan
]


@dataclass
class Finding:
    """A stale active version-pin literal detected by POL-11."""

    file_path: str
    line_number: int
    artifact_id: str          # normalised artifact ID (no .md suffix)
    cited_version: str        # version literal found in the file
    canonical_version: str    # version from registry (or "NOT IN REGISTRY" if absent)
    line_text: str            # raw line content (stripped)


@dataclass
class ScanStats:
    """Counters collected during a scan run."""

    files_scanned: int = 0
    files_skipped: int = 0
    lines_checked: int = 0
    pins_found: int = 0
    pins_historical: int = 0
    pins_active: int = 0
    # Pattern B counters (YAML inputs[] form)
    yaml_pins_found: int = 0
    yaml_pins_historical: int = 0   # individual story files (exempt by classification)
    yaml_pins_active: int = 0        # living index docs + other files (checked)
    findings: list[Finding] = field(default_factory=list)
    unknown_artifacts: list[tuple[str, str, str]] = field(default_factory=list)  # (file, artifact_id, version)


def load_registry(registry_path: Path) -> dict[str, str]:
    """
    Load version-pin-registry.yaml and return a dict mapping artifact_id -> current_version.

    Uses stdlib only — no PyYAML dependency. The registry format is simple enough
    for a line-oriented parser: entries are separated by blank lines, each entry has
    a top-level key (the artifact ID) followed by indented fields including
    `current_version: "X.Y.Z"`.

    Returns dict[str, str] — artifact_id (normalised, no .md) -> canonical version string.

    Exits with code 2 on any parse error.
    """
    if not registry_path.is_file():
        print(
            f"Error: version-pin registry not found at {registry_path}.\n"
            f"The registry must be seeded at .factory/specs/version-pin-registry.yaml\n"
            f"(Task #9 m.2 obligation per ADR-0007 §Implementation Plan).",
            file=sys.stderr,
        )
        sys.exit(2)

    try:
        text = registry_path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"Error: failed to read registry {registry_path}: {exc}", file=sys.stderr)
        sys.exit(2)

    registry: dict[str, str] = {}
    current_id: Optional[str] = None

    for lineno, raw_line in enumerate(text.splitlines(), 1):
        # Skip blank lines and comment lines
        stripped = raw_line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        # Top-level key: no leading whitespace, ends with ":"
        if not raw_line[0].isspace() and raw_line.rstrip().endswith(":"):
            # Extract the key name (before the colon)
            key = raw_line.rstrip().rstrip(":").strip()
            current_id = _normalise_artifact_id(key)
            continue

        # Indented "current_version:" field
        if current_id is not None and "current_version:" in stripped:
            # Extract value: current_version: "1.2.3" or current_version: 1.2.3
            after_colon = stripped.split("current_version:", 1)[1].strip().strip('"').strip("'")
            if after_colon:
                registry[current_id] = after_colon

    return registry


def _normalise_artifact_id(raw_id: str) -> str:
    """
    Normalise an artifact ID for registry lookup.

    Strips trailing .md suffix (case-insensitive).
    Strips leading/trailing whitespace.
    Example: "SS-tui.md" -> "SS-tui", "BC-2.06.005" stays "BC-2.06.005"
    """
    return _MD_SUFFIX_RE.sub("", raw_id.strip())


def _is_excluded_path(rel_path: str, extra_prefixes: list[str] | None = None) -> bool:
    """
    Return True if this relative path (from workspace root, forward-slash separated)
    should be excluded from scanning.

    extra_prefixes: dynamic exclusion prefixes (e.g. factory-root/cycles/ where the
      factory-root name varies by invocation). Combined with _EXCLUDED_PATH_PREFIXES_STATIC.
    """
    # Check excluded directory components
    parts = rel_path.replace("\\", "/").split("/")
    for part in parts[:-1]:  # directories, not the filename itself
        if part in _EXCLUDED_DIRS:
            return True

    # Check excluded path prefixes (static + dynamic)
    norm = rel_path.replace("\\", "/")
    all_prefixes = _EXCLUDED_PATH_PREFIXES_STATIC + (extra_prefixes or [])
    for prefix in all_prefixes:
        if norm.startswith(prefix):
            return True

    return False


def _is_historical_anchor(line: str, in_trace_section: bool) -> bool:
    """
    Return True if the given line's version-pin citation is a historical anchor.

    Applies ADR-0007 §Historical Anchor Classification:
    1. Currently in a §Trace section.
    2. Line contains explicit historical-anchor marker.
    3. Line contains a time qualifier (case-insensitive).
    """
    if in_trace_section:
        return True

    # Check explicit markers.
    # RS form: exact substring match (# version-pin-historical).
    # MD form: prefix match to catch both <!-- version-pin-historical --> and
    #   <!-- version-pin-historical: reason --> (annotated form per ADR-0007 usage).
    if _HISTORICAL_MARKER_RS in line or _HISTORICAL_MARKER_MD_PREFIX in line:
        return True

    # Check time qualifiers (case-insensitive)
    lower_line = line.lower()
    for qualifier in _TIME_QUALIFIERS:
        if qualifier in lower_line:
            return True

    return False


def _classify_yaml_pin(file_path: Path) -> str:
    """
    Classify a YAML inputs[] pin per ADR-0007 §Story inputs[] Historical Provenance.

    Returns one of:
      "HISTORICAL" — individual story file (stories/S-NNN-*.md); exempt from staleness check.
      "ACTIVE"     — living index document or other file; must be checked against registry.

    The classification is based on the file path, not the pin content:
    - Individual story files are HISTORICAL by construction (authored-against provenance).
    - Living index documents (STORY-INDEX, BC-INDEX, ARCH-INDEX, VP-INDEX, prd, L2-INDEX) are ACTIVE.
    - All other files with YAML-form pins are ACTIVE (conservative default per ADR-0007).

    Per ADR-0007: the gate must never silently ignore the YAML form. This function
    ensures every YAML pin is classified explicitly before any decision is made.
    """
    # Normalize path to forward-slash form for reliable matching
    path_str = str(file_path).replace("\\", "/")

    # Check: individual story file (stories/S-NNN-*.md) → HISTORICAL
    if _STORY_FILE_RE.search(path_str):
        return "HISTORICAL"

    # Check: living index document by basename → ACTIVE
    basename = file_path.name
    if basename in _LIVING_INDEX_BASENAMES:
        return "ACTIVE"

    # Conservative default: any other file with YAML-form pin → ACTIVE
    return "ACTIVE"


def _process_yaml_pin(
    file_path: Path,
    artifact_path: str,
    cited_version: str,
    lineno: int,
    line_text: str,
    registry: dict[str, str],
    stats: ScanStats,
    *,
    verbose: bool = False,
) -> None:
    """
    Process a single Pattern B (YAML inputs[] form) version-pin match.

    Classification per ADR-0007 §Story inputs[] Historical Provenance:
    - HISTORICAL: logged explicitly, staleness check skipped.
    - ACTIVE: artifact ID derived from path basename, verified against registry.

    Modifies stats in-place (yaml_pins_* counters + findings/unknown_artifacts).
    """
    stats.yaml_pins_found += 1
    stats.pins_found += 1

    classification = _classify_yaml_pin(file_path)

    if classification == "HISTORICAL":
        # Explicit log: never silently skip — classification must be observable.
        if verbose:
            # Extract basename for a readable log
            pin_basename = artifact_path.rstrip("/").split("/")[-1]
            print(
                f"  [HISTORICAL] {file_path}: line {lineno}: "
                f"inputs[] provenance, exempt — {pin_basename} v{cited_version} "
                f"(individual story file, authored-against record per ADR-0007)"
            )
        stats.yaml_pins_historical += 1
        stats.pins_historical += 1
        return

    # ACTIVE: derive artifact ID from path basename, check against registry.
    stats.yaml_pins_active += 1
    stats.pins_active += 1

    # The path field is the full relative path; the artifact ID is the basename
    # without .md extension, normalised to the registry format.
    pin_basename = artifact_path.rstrip("/").split("/")[-1]
    artifact_id = _normalise_artifact_id(pin_basename)

    if artifact_id not in registry:
        stats.unknown_artifacts.append(
            (str(file_path), artifact_id, cited_version)
        )
        return

    canonical_version = registry[artifact_id]
    if cited_version != canonical_version:
        stats.findings.append(
            Finding(
                file_path=str(file_path),
                line_number=lineno,
                artifact_id=artifact_id,
                cited_version=cited_version,
                canonical_version=canonical_version,
                line_text=line_text.strip(),
            )
        )


def _update_trace_state(line: str, in_trace: bool, trace_level: int) -> tuple[bool, int]:
    """
    Given the current line and §Trace tracking state, return updated (in_trace, trace_level).

    §Trace section begins at a markdown header matching ##+ §Trace.
    It ends when a same-level-or-higher header appears (or EOF).
    """
    header_m = _ANY_HEADER_RE.match(line)
    if header_m:
        hashes = header_m.group(1)
        level = len(hashes)
        if _TRACE_HEADER_RE.match(line):
            return True, level
        elif in_trace and level <= trace_level:
            # Another header at same or higher level ends the §Trace section
            return False, 0
    return in_trace, trace_level


def _process_pin_match(
    file_path: Path,
    raw_artifact_id: str,
    cited_version: str,
    lineno: int,
    line_text: str,
    in_trace: bool,
    registry: dict[str, str],
    stats: ScanStats,
    *,
    context_line: str | None = None,
) -> None:
    """
    Process a single version-pin match (same-line or cross-line).

    For cross-line pins, context_line is the adjacent line that carries the
    version or marker. Historical-anchor check covers both the primary line and
    context_line: if EITHER is exempt, the pin is historical.

    Modifies stats in-place.
    """
    stats.pins_found += 1

    # Historical-anchor check: primary line, then context line if provided.
    if _is_historical_anchor(line_text, in_trace):
        stats.pins_historical += 1
        return
    if context_line is not None and _is_historical_anchor(context_line, False):
        stats.pins_historical += 1
        return

    # Active pointer: verify against registry.
    stats.pins_active += 1
    artifact_id = _normalise_artifact_id(raw_artifact_id)

    if artifact_id not in registry:
        # Unknown artifact: record for reporting (not a CI failure,
        # but a maintainability warning — new artifacts should be registered).
        stats.unknown_artifacts.append(
            (str(file_path), artifact_id, cited_version)
        )
        return

    canonical_version = registry[artifact_id]
    if cited_version != canonical_version:
        stats.findings.append(
            Finding(
                file_path=str(file_path),
                line_number=lineno,
                artifact_id=artifact_id,
                cited_version=cited_version,
                canonical_version=canonical_version,
                line_text=line_text.strip(),
            )
        )


def scan_file(
    file_path: Path,
    registry: dict[str, str],
    stats: ScanStats,
    *,
    verbose: bool = False,
) -> None:
    """
    Scan a single file for active version-pin literals and record findings.

    Detects THREE forms:
    - Pattern A (same-line): "BC-2.06.004 v1.2.1" on one line.
    - Pattern A (cross-line split, PG-SPLIT): artifact ID at end of line N, version vX.Y
      at start of line N+1. Historical-anchor exemption applies if EITHER line carries
      an exemption marker.
    - Pattern B (YAML inputs[] form): {path: .factory/specs/.../SS-tui.md, version: "1.8.2"}
      in file frontmatter. Classified per ADR-0007 §Story inputs[] Historical Provenance:
      individual story files → HISTORICAL (logged, not checked); living index docs and
      other files → ACTIVE (checked against registry). Never silently ignored.

    Modifies stats in-place.
    """
    try:
        text = file_path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        # Non-fatal: log and continue
        print(f"Warning: skipping {file_path}: {exc}", file=sys.stderr)
        stats.files_skipped += 1
        return

    stats.files_scanned += 1
    lines = text.splitlines()
    n_lines = len(lines)

    in_trace = False
    trace_level = 0

    # Track artifact IDs found at end of each line for cross-line detection.
    # Maps line index → list of artifact IDs found at end of that line.
    split_artifact_by_line: dict[int, list[str]] = {}

    for idx, raw_line in enumerate(lines):
        lineno = idx + 1
        stats.lines_checked += 1

        # Update §Trace tracking state
        in_trace, trace_level = _update_trace_state(raw_line, in_trace, trace_level)

        # ── Same-line detection ──────────────────────────────────────────────
        if " v" in raw_line:
            for m in _VERSION_PIN_RE.finditer(raw_line):
                raw_artifact_id = m.group(1)
                cited_version = m.group(2)
                _process_pin_match(
                    file_path, raw_artifact_id, cited_version, lineno, raw_line,
                    in_trace, registry, stats,
                )

        # ── Cross-line detection: collect artifact IDs at end of this line ──
        # Only check if line does NOT already contain a full same-line version
        # literal for this artifact (avoid double-counting).
        for am in _SPLIT_LINE_ARTIFACT_RE.finditer(raw_line):
            artifact_at_eol = am.group(1)
            # Only record if this artifact wasn't already matched same-line on this line.
            already_matched = any(
                m.group(1) == artifact_at_eol
                for m in _VERSION_PIN_RE.finditer(raw_line)
            )
            if not already_matched:
                split_artifact_by_line.setdefault(idx, []).append(artifact_at_eol)

        # ── Cross-line detection: check if this line opens with a version ───
        # that pairs with an artifact ID at the end of the previous line.
        if idx > 0 and (idx - 1) in split_artifact_by_line:
            vm = _SPLIT_LINE_VERSION_RE.match(raw_line)
            if vm:
                cited_version = vm.group(1)
                prev_line = lines[idx - 1]
                prev_lineno = idx  # 1-indexed: idx is lineno-1 for previous
                for raw_artifact_id in split_artifact_by_line[idx - 1]:
                    # Determine trace state for the previous line by replaying
                    # up to that point would be expensive; use in_trace of
                    # current line as conservative approximation (they are adjacent).
                    _process_pin_match(
                        file_path, raw_artifact_id, cited_version,
                        prev_lineno,          # report on the line with artifact ID
                        prev_line,            # primary line text
                        in_trace,             # use current trace state (adjacent lines)
                        registry, stats,
                        context_line=raw_line,   # version line carries the marker
                    )

        # ── Pattern B: YAML inputs[] form ───────────────────────────────────
        # Detect {path: <artifact>, version: "<semver>"} in any line.
        # These appear in YAML frontmatter of story and index-doc files.
        # The regex is specific enough to avoid false positives in prose.
        #
        # Classification (per ADR-0007 §Story inputs[] Historical Provenance):
        #   - Individual story files (stories/S-NNN-*.md) → HISTORICAL (logged, exempt)
        #   - Living index docs + other files → ACTIVE (checked against registry)
        # Never silently ignored — explicit classification is always applied.
        if "{path:" in raw_line and "version:" in raw_line:
            for ym in _YAML_INPUT_PIN_RE.finditer(raw_line):
                artifact_path = ym.group(1)
                cited_version = ym.group(2)
                _process_yaml_pin(
                    file_path, artifact_path, cited_version, lineno, raw_line,
                    registry, stats, verbose=verbose,
                )


def collect_files(workspace_root: Path, factory_root: Path) -> list[Path]:
    """
    Collect all scannable files from the workspace root, respecting exclusions.

    Scans:
    - workspace_root/crates/
    - workspace_root/.github/
    - workspace_root/scripts/          (excluding scripts/tests/)
    - workspace_root/*.toml, *.yml, *.yaml, *.md at root level
    - factory_root/stories/            (normative: active story files)
    - factory_root/specs/              (normative: SS-*, ADR-*, BC-INDEX, version-pin-registry)
    - factory_root/*.md at top level   (normative: prd.md, product-brief.md, dtu-assessment.md)

    Explicitly excluded from factory_root (historical/living documents, NOT normative):
    - factory_root/cycles/        — closed adversarial cycle records (pre-existing)
    - factory_root/plans/         — frozen adversary/audit records (POL-11 scope, ADR-0007)
    - factory_root/planning/      — validation reports
    - factory_root/code-delivery/ — at-merge PR description records
    - factory_root/STATE.md       — living dashboard/log (excluded as specific file)

    The factory_root parameter is the resolved Path to the factory-artifacts directory.
    Locally this is workspace_root/.factory; in CI it is workspace_root/.factory-spec
    (because factory-artifacts is checked out there via a second actions/checkout step).
    Passing factory_root explicitly — rather than hardcoding ".factory" — ensures that
    story files (.factory-spec/stories/*.md) are scanned in CI, which was the PRIMARY
    scope gap that allowed stale version-pin literals to survive CI (POL-11 scope fix,
    ADV-29 root cause).

    Returns a sorted list of Path objects.
    """
    collected: list[Path] = []

    # Dynamic factory-root exclusion prefixes: relative to workspace root.
    # Uses factory_root.name so the same logic works for both:
    #   local: .factory/plans/, .factory/STATE.md, etc.
    #   CI:    .factory-spec/plans/, .factory-spec/STATE.md, etc.
    factory_root_name = factory_root.name  # e.g. ".factory" or ".factory-spec"
    extra_prefixes = [
        factory_root_name + "/cycles/",        # closed adversarial cycle records (pre-existing)
        factory_root_name + "/plans/",         # frozen adversary/audit records
        factory_root_name + "/planning/",      # validation reports
        factory_root_name + "/code-delivery/", # at-merge PR description records
        factory_root_name + "/STATE.md",       # living dashboard/log (specific file)
    ]

    def _walk(base: Path, rel_prefix: str = "") -> None:
        """Recursively walk base, skipping excluded dirs."""
        try:
            entries = sorted(base.iterdir())
        except PermissionError:
            return

        for entry in entries:
            rel = f"{rel_prefix}{entry.name}"

            if entry.is_dir():
                # Check if this directory is excluded
                rel_dir = rel + "/"
                if _is_excluded_path(rel_dir, extra_prefixes) or entry.name in _EXCLUDED_DIRS:
                    continue
                _walk(entry, rel + "/")
            elif entry.is_file():
                if entry.suffix.lower() in _SCAN_EXTENSIONS:
                    if not _is_excluded_path(rel, extra_prefixes):
                        collected.append(entry)

    # Scan targeted directories rather than entire workspace root (avoid scanning
    # fixtures, examples, etc. that aren't part of the policy surface).
    #
    # IMPORTANT: factory_root is NOT hardcoded to workspace_root/".factory" here.
    # In CI, factory-artifacts is checked out at workspace_root/".factory-spec"
    # (via a second actions/checkout step). Using the resolved factory_root ensures
    # story files are scanned regardless of the checkout path.
    scan_roots = [
        workspace_root / "crates",
        workspace_root / ".github",
        workspace_root / "scripts",
        factory_root,                   # resolved factory path (local: .factory, CI: .factory-spec)
    ]

    for scan_root in scan_roots:
        if scan_root.is_dir():
            _walk(scan_root, rel_prefix=scan_root.name + "/")

    # Root-level files (Cargo.toml, deny.toml, clippy.toml, etc.)
    for entry in sorted(workspace_root.iterdir()):
        if entry.is_file() and entry.suffix.lower() in _SCAN_EXTENSIONS:
            collected.append(entry)

    return sorted(set(collected))


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "POL-11: Version-pin literal freshness enforcement (monocle-version-pin-freshness).\n"
            "Verifies every active version-pin literal matches canonical version in registry.\n"
            "Implements ADR-0007 §Implementation Plan."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--registry",
        metavar="PATH",
        help=(
            "Path to version-pin-registry.yaml. "
            "Default: <workspace-root>/.factory/specs/version-pin-registry.yaml"
        ),
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
        help="Print detailed progress including skipped files and historical anchors.",
    )
    args = parser.parse_args()

    workspace_root = Path(args.workspace_root).resolve()
    factory_root = workspace_root / args.factory_root

    # Determine registry path
    if args.registry:
        registry_path = Path(args.registry).resolve()
    else:
        registry_path = factory_root / "specs" / "version-pin-registry.yaml"

    # Load registry
    registry = load_registry(registry_path)
    if args.verbose:
        print(f"Loaded registry: {len(registry)} entries from {registry_path}")

    # Collect files to scan.
    # factory_root is passed explicitly so collect_files() scans the correct
    # factory-artifacts directory (local: .factory, CI: .factory-spec).
    files = collect_files(workspace_root, factory_root)
    if args.verbose:
        print(f"Scanning {len(files)} files for version-pin literals (factory: {factory_root})...")

    # Scan
    stats = ScanStats()
    for file_path in files:
        scan_file(file_path, registry, stats, verbose=args.verbose)

    # Report unknown artifacts (maintainability warning, not CI failure)
    if stats.unknown_artifacts:
        # Deduplicate
        seen: set[tuple[str, str]] = set()
        unique_unknowns: list[tuple[str, str, str]] = []
        for (fp, aid, ver) in stats.unknown_artifacts:
            key = (aid, ver)
            if key not in seen:
                seen.add(key)
                unique_unknowns.append((fp, aid, ver))

        print(
            f"\nWarning: {len(unique_unknowns)} active version-pin literal(s) cite artifacts "
            f"NOT present in version-pin-registry.yaml:",
            file=sys.stderr,
        )
        for fp, aid, ver in sorted(unique_unknowns, key=lambda x: x[1]):
            print(f"  {aid} v{ver}  (first seen in {fp})", file=sys.stderr)
        print(
            "  Add these artifacts to version-pin-registry.yaml to enable freshness checks.\n"
            "  (Per ADR-0007 §Version-Pin Registry: state-manager obligation on each bump.)",
            file=sys.stderr,
        )

    # Report findings
    n_findings = len(stats.findings)
    n_stale = n_findings

    if n_stale > 0:
        print(
            f"\nPOL-11 FAIL: {n_stale} stale active version-pin literal(s) detected.\n"
            f"(ADR-0007 §CI enforcement gate — monocle-version-pin-freshness)\n",
            file=sys.stderr,
        )
        for f in sorted(stats.findings, key=lambda x: (x.file_path, x.line_number)):
            print(
                f"  {f.file_path}:{f.line_number}: "
                f"{f.artifact_id} v{f.cited_version} cited but canonical is v{f.canonical_version}\n"
                f"    Line: {f.line_text[:120]}",
                file=sys.stderr,
            )
        print(
            f"\nFix options per ADR-0007 §Decision:\n"
            f"  1. Update the cited version to v{{canonical}} (current).\n"
            f"  2. Convert to version-free form: cite by section anchor only (preferred for new artifacts).\n"
            f"  3. Add time qualifier to make it a historical anchor (if this IS an intentional historical record).\n"
            f"     E.g.: 'implemented against <artifact> v{{cited}} at S-025 authoring time'",
            file=sys.stderr,
        )
        if not args.dry_run:
            sys.exit(1)
        else:
            print("(--dry-run: exiting 0 despite findings)", file=sys.stderr)
    else:
        print(
            f"POL-11 PASS: all active version-pin literals are current "
            f"({stats.pins_active} active, {stats.pins_historical} historical anchors, "
            f"{stats.files_scanned} files scanned)."
        )

    if args.verbose:
        print(
            f"\nScan summary:\n"
            f"  Files scanned:         {stats.files_scanned}\n"
            f"  Files skipped:         {stats.files_skipped}\n"
            f"  Lines checked:         {stats.lines_checked}\n"
            f"  Pins found total:      {stats.pins_found}\n"
            f"  Pins historical:       {stats.pins_historical}\n"
            f"  Pins active:           {stats.pins_active}\n"
            f"  Pattern B (YAML) total:      {stats.yaml_pins_found}\n"
            f"  Pattern B historical (story): {stats.yaml_pins_historical}\n"
            f"  Pattern B active (index/other): {stats.yaml_pins_active}\n"
            f"  Unknown artifacts:     {len(set((a, v) for _, a, v in stats.unknown_artifacts))}\n"
            f"  Findings (stale):      {n_findings}"
        )


if __name__ == "__main__":
    main()
