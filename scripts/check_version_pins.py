#!/usr/bin/env python3
"""
check_version_pins.py — POL-11: Version-pin literal freshness enforcement.

Implements monocle-version-pin-freshness CI gate per ADR-0007
§Implementation Plan and SS-conventions-anti-patterns.md §Citation Discipline.

POLICY SUMMARY (ADR-0007 v1.0.2):
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

SCANNED FILE TYPES: .md, .rs, .toml, .yml, .yaml

EXEMPT PATHS (not scanned):
  .factory/cycles/  — closed adversarial cycle records
  target/           — cargo build output
  .git/             — git internals
  node_modules/     — not applicable
  scripts/tests/    — test fixture directory (fixtures are intentionally wrong)

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
  The registry is read from .factory/specs/version-pin-registry.yaml relative
  to the workspace root. In CI the factory-artifacts branch is checked out into
  .factory-spec/ for spec access; pass --factory-root .factory-spec in that case.

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
# ───────────────────────────────────────────────────────────────────────────────
_HISTORICAL_MARKER_RS = "# version-pin-historical"
_HISTORICAL_MARKER_MD = "<!-- version-pin-historical -->"

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

# Paths excluded by prefix (relative to workspace root, using forward-slash notation).
_EXCLUDED_PATH_PREFIXES: list[str] = [
    ".factory/cycles/",
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


def _is_excluded_path(rel_path: str) -> bool:
    """
    Return True if this relative path (from workspace root, forward-slash separated)
    should be excluded from scanning.
    """
    # Check excluded directory components
    parts = rel_path.replace("\\", "/").split("/")
    for part in parts[:-1]:  # directories, not the filename itself
        if part in _EXCLUDED_DIRS:
            return True

    # Check excluded path prefixes
    norm = rel_path.replace("\\", "/")
    for prefix in _EXCLUDED_PATH_PREFIXES:
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

    # Check explicit markers
    if _HISTORICAL_MARKER_RS in line or _HISTORICAL_MARKER_MD in line:
        return True

    # Check time qualifiers (case-insensitive)
    lower_line = line.lower()
    for qualifier in _TIME_QUALIFIERS:
        if qualifier in lower_line:
            return True

    return False


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


def scan_file(
    file_path: Path,
    registry: dict[str, str],
    stats: ScanStats,
) -> None:
    """
    Scan a single file for active version-pin literals and record findings.

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

    in_trace = False
    trace_level = 0

    for lineno, raw_line in enumerate(lines, 1):
        stats.lines_checked += 1

        # Update §Trace tracking state
        in_trace, trace_level = _update_trace_state(raw_line, in_trace, trace_level)

        # Fast-path: no version literal on this line
        if " v" not in raw_line:
            continue

        # Find all version-pin literals on this line
        for m in _VERSION_PIN_RE.finditer(raw_line):
            raw_artifact_id = m.group(1)
            cited_version = m.group(2)
            stats.pins_found += 1

            # Classify: historical anchor or active pointer?
            if _is_historical_anchor(raw_line, in_trace):
                stats.pins_historical += 1
                continue

            # Active pointer: verify against registry
            stats.pins_active += 1
            artifact_id = _normalise_artifact_id(raw_artifact_id)

            if artifact_id not in registry:
                # Unknown artifact: record for reporting (not a CI failure,
                # but a maintainability warning — new artifacts should be registered)
                stats.unknown_artifacts.append(
                    (str(file_path), artifact_id, cited_version)
                )
                continue

            canonical_version = registry[artifact_id]
            if cited_version != canonical_version:
                stats.findings.append(
                    Finding(
                        file_path=str(file_path),
                        line_number=lineno,
                        artifact_id=artifact_id,
                        cited_version=cited_version,
                        canonical_version=canonical_version,
                        line_text=raw_line.strip(),
                    )
                )


def collect_files(workspace_root: Path) -> list[Path]:
    """
    Collect all scannable files from the workspace root, respecting exclusions.

    Scans:
    - workspace_root/crates/
    - workspace_root/.github/
    - workspace_root/scripts/   (excluding scripts/tests/)
    - workspace_root/*.toml, *.yml, *.yaml, *.md at root level
    - workspace_root/.factory/  (full spec tree, excluding .factory/cycles/)

    Returns a sorted list of Path objects.
    """
    collected: list[Path] = []

    def _walk(base: Path, rel_prefix: str = "") -> None:
        """Recursively walk base, skipping excluded dirs."""
        try:
            entries = sorted(base.iterdir())
        except PermissionError:
            return

        for entry in entries:
            rel = f"{rel_prefix}{entry.name}"
            rel_path_str = rel if not rel_prefix else rel

            if entry.is_dir():
                # Check if this directory is excluded
                rel_dir = rel + "/"
                if _is_excluded_path(rel_dir) or entry.name in _EXCLUDED_DIRS:
                    continue
                _walk(entry, rel + "/")
            elif entry.is_file():
                if entry.suffix.lower() in _SCAN_EXTENSIONS:
                    if not _is_excluded_path(rel_path_str):
                        collected.append(entry)

    # Scan targeted directories rather than entire workspace root (avoid scanning
    # fixtures, examples, etc. that aren't part of the policy surface)
    scan_roots = [
        workspace_root / "crates",
        workspace_root / ".github",
        workspace_root / "scripts",
        workspace_root / ".factory",
    ]
    root_level_extensions = _SCAN_EXTENSIONS

    for scan_root in scan_roots:
        if scan_root.is_dir():
            _walk(scan_root, rel_prefix=scan_root.name + "/")

    # Root-level files (Cargo.toml, deny.toml, clippy.toml, etc.)
    for entry in sorted(workspace_root.iterdir()):
        if entry.is_file() and entry.suffix.lower() in root_level_extensions:
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

    # Collect files to scan
    files = collect_files(workspace_root)
    if args.verbose:
        print(f"Scanning {len(files)} files for version-pin literals...")

    # Scan
    stats = ScanStats()
    for file_path in files:
        scan_file(file_path, registry, stats)

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
            f"  Files scanned:     {stats.files_scanned}\n"
            f"  Files skipped:     {stats.files_skipped}\n"
            f"  Lines checked:     {stats.lines_checked}\n"
            f"  Pins found total:  {stats.pins_found}\n"
            f"  Pins historical:   {stats.pins_historical}\n"
            f"  Pins active:       {stats.pins_active}\n"
            f"  Unknown artifacts: {len(set((a, v) for _, a, v in stats.unknown_artifacts))}\n"
            f"  Findings (stale):  {n_findings}"
        )


if __name__ == "__main__":
    main()
