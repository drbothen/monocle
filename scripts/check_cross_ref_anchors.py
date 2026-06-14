#!/usr/bin/env python3
"""
check_cross_ref_anchors.py — POL-13: Cross-reference anchor integrity enforcement.

Scans Markdown files under .factory/specs/ for cross-references of the form
  path/to/file.md#anchor-slug
and verifies that every cited anchor actually exists in the target file.

POLICY SUMMARY:
  Every active cross-reference anchor cited in .factory/specs/ prose must resolve
  to a valid heading slug or explicit HTML anchor in the target file. Dead anchors
  indicate heading renames that were not propagated to citing documents, silently
  breaking the spec's internal traceability.

ANCHOR RESOLUTION (valid anchor set for a target file):
  a) GitHub-computed heading slugs for every ATX heading (# ... ######):
       - Strip leading '#'s and surrounding whitespace.
       - Strip Markdown formatting markers (* and backticks) from visible text.
         NOTE: underscores are KEPT (GitHub keeps them in identifier text).
       - Lowercase.
       - Remove every character that is not alphanumeric, space, hyphen, or underscore.
         (This DROPS: parentheses, colon, em-dash —, forward/back slash, dot, etc.)
       - Replace spaces with hyphens.
       - Do NOT collapse consecutive hyphens (GitHub keeps them).
         E.g. 'foo() — bar' → 'foo--bar' (parens dropped, — produces two spaces → --)
       - For duplicate slugs: append -1, -2, … in document order.
  b) Explicit HTML anchors in the target: <a id="X">, <a name="X">.
  c) {#X} attribute syntax (kramdown/Pandoc style).

EXEMPTIONS (a cross-reference line is exempt from checking if):
  1. It appears inside a fenced code block (``` ... ```).
  2. The line contains "version-pin-historical" (explicit historical annotation).
  3. The line contains "§Trace" (§Trace section marker).
  4. The line contains "<!-- cross-ref-historical -->" annotation.

SCANNED PATHS:
  .factory/specs/                    — all Markdown under the specs tree
  Includes: behavioral-contracts/**/*.md, architecture/**/*.md, domain-spec/**/*.md,
            prd.md, product-brief.md, dtu-assessment.md, prd-expansion-scope.md,
            research/**/*.md, holdout-scenarios/**/*.md, verification-properties/**/*.md

EXEMPT PATHS (not scanned as source of cross-references):
  .factory/cycles/                   — closed adversarial cycle records (historical)
  .factory/plans/                    — frozen adversary/audit records (historical)
  .factory/planning/                 — validation reports (historical)
  .factory/code-delivery/            — at-merge PR description records (historical)
  .factory/STATE.md                  — living dashboard/log (excluded as specific file)

USAGE:
  # Run from workspace root:
  python3 scripts/check_cross_ref_anchors.py

  # With custom factory root (CI):
  python3 scripts/check_cross_ref_anchors.py --factory-root .factory-spec

  # Dry-run (report but do not fail):
  python3 scripts/check_cross_ref_anchors.py --dry-run

  # Verbose (print all resolved anchors):
  python3 scripts/check_cross_ref_anchors.py --verbose

SLUG ALGORITHM VALIDATION (ground-truth GitHub cases):
  '## spawn_recipe() — new trait method'         → 'spawn_recipe--new-trait-method'
  '### EngineError (new in v1A)'                 → 'engineerror-new-in-v1a'
  '### 2. daemon_start_sequence() — session …'   → '2-daemon_start_sequence--session-re-discovery-step'

EXIT CODES:
  0 — all cross-reference anchors resolve (or no cross-references found)
  1 — one or more unresolved anchors detected
  2 — configuration error (factory root not found, etc.)
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────────────────
# Cross-reference detection pattern.
#
# Matches Markdown link text containing "path/to/file.md#anchor" in any of:
#   - Inline links: [text](path/to/file.md#anchor)
#   - Bare cross-references: `path/to/file.md#anchor`
#   - Plain text: path/to/file.md#anchor (no brackets required)
#
# We want to capture the file path and anchor from any occurrence of:
#   <path>.md#<slug>
# where <path> is a relative path (alphanumeric, dots, hyphens, underscores, slashes)
# and <slug> is a heading slug (alphanumeric, hyphens, underscores).
#
# Groups: (1) file path including .md, (2) anchor slug
# ───────────────────────────────────────────────────────────────────────────────
_CROSS_REF_RE = re.compile(
    r'([A-Za-z0-9_./][A-Za-z0-9_./-]*\.md)#([A-Za-z0-9_-]+)',
)

# ───────────────────────────────────────────────────────────────────────────────
# ATX heading detection: lines starting with 1-6 '#' followed by space.
# ───────────────────────────────────────────────────────────────────────────────
_ATX_HEADING_RE = re.compile(r'^(#{1,6})\s+(.*)')

# ───────────────────────────────────────────────────────────────────────────────
# Explicit HTML anchor patterns.
# ───────────────────────────────────────────────────────────────────────────────
_HTML_ANCHOR_RE = re.compile(r'<a\s+(?:id|name)=["\']([^"\']+)["\']')
_ATTR_ANCHOR_RE = re.compile(r'\{#([A-Za-z0-9_-]+)\}')

# ───────────────────────────────────────────────────────────────────────────────
# Exemption markers on source lines (skip cross-reference check).
# ───────────────────────────────────────────────────────────────────────────────
_EXEMPT_MARKERS = (
    "version-pin-historical",
    "§Trace",
    "<!-- cross-ref-historical -->",
)

# ───────────────────────────────────────────────────────────────────────────────
# Fenced code block fence detector.
# ───────────────────────────────────────────────────────────────────────────────
_FENCE_RE = re.compile(r'^\s*(`{3,}|~{3,})')

# ───────────────────────────────────────────────────────────────────────────────
# Exempt path prefixes and files (relative to factory root).
# ───────────────────────────────────────────────────────────────────────────────
_EXEMPT_FACTORY_PREFIXES = (
    "cycles/",
    "plans/",
    "planning/",
    "code-delivery/",
)
_EXEMPT_FACTORY_FILES = frozenset({
    "STATE.md",
})


@dataclass
class CrossRefFinding:
    """An unresolved cross-reference anchor."""

    source_file: str       # relative path to source file
    line_number: int
    ref_text: str          # the full reference as it appears: "path/to/file.md#anchor"
    target_file: str       # resolved absolute path (or "<not found>" if absent)
    anchor: str            # the anchor slug that failed to resolve
    heading_context: str   # the nearest heading text in target (empty if file not found)
    error_class: str       # "dead_anchor" | "file_not_found"


@dataclass
class ScanStats:
    """Counters collected during a scan run."""

    files_scanned: int = 0
    refs_found: int = 0
    refs_exempt: int = 0
    refs_active: int = 0
    refs_resolved: int = 0
    target_files_checked: int = 0
    findings: list[CrossRefFinding] = field(default_factory=list)


# ───────────────────────────────────────────────────────────────────────────────
# GitHub slug algorithm.
#
# Ground-truth validation (must pass before trusting output):
#   '## spawn_recipe() — new trait method'
#       → 'spawn_recipe--new-trait-method'
#   '### EngineError (new in v1A)'
#       → 'engineerror-new-in-v1a'
#   '### 2. daemon_start_sequence() — session re-discovery step'
#       → '2-daemon_start_sequence--session-re-discovery-step'
#
# Algorithm (faithful to GitHub):
#   1. Strip leading '#'s and surrounding whitespace.
#   2. Strip Markdown formatting markers (* and backtick) from visible text.
#      Note: underscores are KEPT (they appear in identifier text like spawn_recipe).
#   3. Lowercase.
#   4. Remove every character that is not alphanumeric, space, hyphen, or underscore.
#      This DROPS: ()  :  .  /  \  em-dash (—)  and other punctuation.
#      Note: em-dash (—) is a Unicode char (U+2014) that is NOT alphanumeric/space/
#      hyphen/underscore, so it is dropped. The spaces surrounding "—" in headings
#      produce a double-hyphen after space→hyphen replacement.
#   5. Replace spaces with hyphens.
#   6. Do NOT collapse consecutive hyphens.
# ───────────────────────────────────────────────────────────────────────────────

def github_slug(heading_line: str) -> str:
    """
    Compute the GitHub heading slug for an ATX heading line.

    heading_line: the raw heading line (e.g., '## spawn_recipe() — new trait method')
    Returns the slug (e.g., 'spawn_recipe--new-trait-method').
    """
    # Strip leading '#'s and surrounding whitespace
    text = _ATX_HEADING_RE.sub(lambda m: m.group(2), heading_line).strip()
    # Strip Markdown formatting markers (* and backticks) but NOT underscores
    text = re.sub(r'[*`]', '', text)
    # Lowercase
    text = text.lower()
    # Remove every character that is not alphanumeric, space, hyphen, or underscore
    text = re.sub(r'[^a-z0-9 _-]', '', text)
    # Replace spaces with hyphens (do NOT collapse consecutive hyphens)
    text = text.replace(' ', '-')
    return text


def _validate_slug_algorithm() -> None:
    """
    Validate the GitHub slug algorithm against 3 ground-truth cases.
    Exits with code 2 if any case fails — algorithm must be correct before scanning.
    """
    cases = [
        (
            '## spawn_recipe() — new trait method',
            'spawn_recipe--new-trait-method',
        ),
        (
            '### EngineError (new in v1A)',
            'engineerror-new-in-v1a',
        ),
        (
            '### 2. daemon_start_sequence() — session re-discovery step',
            '2-daemon_start_sequence--session-re-discovery-step',
        ),
    ]
    failed = False
    for heading, expected in cases:
        got = github_slug(heading)
        if got != expected:
            print(
                f"Error: slug algorithm failed validation.\n"
                f"  Input:    {heading!r}\n"
                f"  Expected: {expected!r}\n"
                f"  Got:      {got!r}",
                file=sys.stderr,
            )
            failed = True
    if failed:
        print(
            "Slug algorithm validation FAILED. Fix the algorithm before scanning.",
            file=sys.stderr,
        )
        sys.exit(2)


# ───────────────────────────────────────────────────────────────────────────────
# Anchor set computation for a target file.
# ───────────────────────────────────────────────────────────────────────────────

def compute_anchors(file_path: Path) -> frozenset[str]:
    """
    Compute the complete set of valid anchors for a Markdown file.

    Returns a frozenset of anchor strings (all lowercase for headings, exact-case
    for explicit HTML anchors — though in practice GitHub normalises them).

    Anchor sources:
      a) GitHub-computed heading slugs (with duplicate-suffix deduplication).
      b) <a id="X"> and <a name="X"> explicit anchors.
      c) {#X} attribute syntax (kramdown/Pandoc style).
    """
    try:
        text = file_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return frozenset()

    anchors: set[str] = set()
    lines = text.splitlines()

    in_code_block = False
    slug_counts: dict[str, int] = {}  # for duplicate-suffix deduplication

    for raw_line in lines:
        # Track fenced code blocks (don't parse headings inside them)
        fence_m = _FENCE_RE.match(raw_line)
        if fence_m:
            in_code_block = not in_code_block
            continue

        # Always scan for explicit HTML anchors (even inside code blocks,
        # GitHub resolves them, though in practice they appear in prose)
        for m in _HTML_ANCHOR_RE.finditer(raw_line):
            anchors.add(m.group(1))
        for m in _ATTR_ANCHOR_RE.finditer(raw_line):
            anchors.add(m.group(1))

        if in_code_block:
            continue

        # ATX heading slug computation
        heading_m = _ATX_HEADING_RE.match(raw_line)
        if heading_m:
            slug = github_slug(raw_line)
            # Handle duplicates: first occurrence → slug, subsequent → slug-N
            if slug not in slug_counts:
                slug_counts[slug] = 0
                anchors.add(slug)
            else:
                slug_counts[slug] += 1
                suffixed = f"{slug}-{slug_counts[slug]}"
                anchors.add(suffixed)

    return frozenset(anchors)


# ───────────────────────────────────────────────────────────────────────────────
# Target file resolution.
# ───────────────────────────────────────────────────────────────────────────────

def _resolve_target(ref_path: str, source_file: Path, specs_root: Path) -> Optional[Path]:
    """
    Resolve a cross-reference file path to an absolute Path.

    Resolution strategy (in order):
      1. If the ref_path is relative to the source file's directory, use that.
      2. If the ref_path starts with "architecture/", "specs/", etc., resolve from
         the specs root (.factory/specs/).
      3. If none of the above, try relative to the factory root (.factory/).

    Returns the resolved Path if the file exists, otherwise None.
    """
    # Attempt 1: relative to the source file's directory
    candidate = source_file.parent / ref_path
    if candidate.is_file():
        return candidate.resolve()

    # Attempt 2: relative to the specs root
    candidate = specs_root / ref_path
    if candidate.is_file():
        return candidate.resolve()

    # Attempt 3: relative to the factory root (one level up from specs)
    factory_root = specs_root.parent
    candidate = factory_root / ref_path
    if candidate.is_file():
        return candidate.resolve()

    return None


# ───────────────────────────────────────────────────────────────────────────────
# Source file scanning.
# ───────────────────────────────────────────────────────────────────────────────

def scan_file(
    file_path: Path,
    specs_root: Path,
    anchor_cache: dict[Path, frozenset[str]],
    stats: ScanStats,
    *,
    verbose: bool = False,
) -> None:
    """
    Scan a single Markdown file for cross-reference anchors and verify them.

    Modifies stats in-place.
    anchor_cache: dict mapping resolved target Path → frozenset of valid anchors.
      Populated lazily so each target file is read at most once.
    """
    try:
        text = file_path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"Warning: skipping {file_path}: {exc}", file=sys.stderr)
        return

    stats.files_scanned += 1
    lines = text.splitlines()
    in_code_block = False

    for lineno, raw_line in enumerate(lines, 1):
        # Track fenced code blocks
        fence_m = _FENCE_RE.match(raw_line)
        if fence_m:
            in_code_block = not in_code_block
            continue

        # Skip lines inside code blocks
        if in_code_block:
            continue

        # Check exemption markers
        is_exempt = any(marker in raw_line for marker in _EXEMPT_MARKERS)

        # Scan for cross-references on this line
        for m in _CROSS_REF_RE.finditer(raw_line):
            ref_file_part = m.group(1)
            anchor = m.group(2)
            ref_text = m.group(0)
            stats.refs_found += 1

            if is_exempt:
                stats.refs_exempt += 1
                if verbose:
                    print(
                        f"  [EXEMPT] {file_path}:{lineno}: {ref_text} (exemption marker)"
                    )
                continue

            stats.refs_active += 1

            # Resolve target file
            target_path = _resolve_target(ref_file_part, file_path, specs_root)

            if target_path is None:
                # File not found
                stats.findings.append(
                    CrossRefFinding(
                        source_file=str(file_path),
                        line_number=lineno,
                        ref_text=ref_text,
                        target_file="<not found>",
                        anchor=anchor,
                        heading_context="",
                        error_class="file_not_found",
                    )
                )
                if verbose:
                    print(
                        f"  [FILE-NOT-FOUND] {file_path}:{lineno}: {ref_text}"
                    )
                continue

            # Load anchors for target file (cached)
            if target_path not in anchor_cache:
                if verbose:
                    print(f"  [LOAD] {target_path}")
                anchor_cache[target_path] = compute_anchors(target_path)
                stats.target_files_checked += 1

            valid_anchors = anchor_cache[target_path]

            if anchor in valid_anchors:
                stats.refs_resolved += 1
                if verbose:
                    print(
                        f"  [OK] {file_path}:{lineno}: {ref_text}"
                    )
            else:
                # Dead anchor — find nearest matching heading text for context
                heading_context = _find_nearest_heading(target_path, anchor)
                stats.findings.append(
                    CrossRefFinding(
                        source_file=str(file_path),
                        line_number=lineno,
                        ref_text=ref_text,
                        target_file=str(target_path),
                        anchor=anchor,
                        heading_context=heading_context,
                        error_class="dead_anchor",
                    )
                )
                if verbose:
                    print(
                        f"  [DEAD] {file_path}:{lineno}: {ref_text}"
                        + (f" (nearest: {heading_context!r})" if heading_context else "")
                    )


def _find_nearest_heading(target_path: Path, anchor: str) -> str:
    """
    Find the heading text in target_path whose slug most closely matches `anchor`.

    Returns the raw heading text (with leading '#' stripped) of the closest slug match,
    or the empty string if no close match is found.

    This is used for diagnostics only — to help the spec author know which heading
    needs a stable anchor added, or what the correct current anchor slug is.
    """
    try:
        text = target_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""

    in_code_block = False
    all_headings: list[tuple[str, str]] = []  # [(slug, heading_text)]

    for raw_line in text.splitlines():
        fence_m = _FENCE_RE.match(raw_line)
        if fence_m:
            in_code_block = not in_code_block
            continue
        if in_code_block:
            continue
        heading_m = _ATX_HEADING_RE.match(raw_line)
        if heading_m:
            heading_text = heading_m.group(2).strip()
            slug = github_slug(raw_line)
            all_headings.append((slug, heading_text))

    if not all_headings:
        return ""

    # Find closest slug using prefix/suffix matching for diagnostic purposes.
    # First try: exact prefix match (the anchor is a prefix of an actual slug or vice versa).
    anchor_lower = anchor.lower()
    for slug, heading_text in all_headings:
        if slug.startswith(anchor_lower) or anchor_lower.startswith(slug):
            return heading_text

    # Second try: any word in the anchor appears in the slug.
    anchor_words = set(anchor_lower.replace("-", "_").replace("_", " ").split())
    best_score = 0
    best_heading = ""
    for slug, heading_text in all_headings:
        slug_words = set(slug.replace("-", "_").replace("_", " ").split())
        score = len(anchor_words & slug_words)
        if score > best_score:
            best_score = score
            best_heading = heading_text

    return best_heading


# ───────────────────────────────────────────────────────────────────────────────
# File collection.
# ───────────────────────────────────────────────────────────────────────────────

def collect_spec_files(factory_root: Path) -> list[Path]:
    """
    Collect all Markdown files under factory_root/specs/ to scan as cross-reference sources.

    Excludes:
      - .factory/cycles/, plans/, planning/, code-delivery/ (historical/frozen records)
      - .factory/STATE.md (living dashboard)
      - semport/ (reference ingestion — historical)
    Returns a sorted list of Path objects.
    """
    specs_root = factory_root / "specs"
    if not specs_root.is_dir():
        return []

    # Also scan top-level factory docs that may cite cross-refs (prd.md, etc.)
    files: list[Path] = []

    # Walk specs tree
    for entry in sorted(specs_root.rglob("*.md")):
        if entry.is_file():
            files.append(entry)

    # Also check factory root top-level .md files (STATE.md excluded by exact name)
    for entry in sorted(factory_root.iterdir()):
        if entry.is_file() and entry.suffix.lower() == ".md":
            if entry.name not in _EXEMPT_FACTORY_FILES:
                files.append(entry)

    # Stories also cite cross-refs
    stories_root = factory_root / "stories"
    if stories_root.is_dir():
        for entry in sorted(stories_root.rglob("*.md")):
            if entry.is_file():
                files.append(entry)

    return sorted(set(files))


def _is_factory_exempt(file_path: Path, factory_root: Path) -> bool:
    """Return True if this file is in an exempt factory subdirectory."""
    try:
        rel = file_path.relative_to(factory_root)
        rel_str = str(rel).replace("\\", "/")
        for prefix in _EXEMPT_FACTORY_PREFIXES:
            if rel_str.startswith(prefix):
                return True
        if rel_str in _EXEMPT_FACTORY_FILES:
            return True
    except ValueError:
        pass
    return False


# ───────────────────────────────────────────────────────────────────────────────
# Main entry point.
# ───────────────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "POL-13: Cross-reference anchor integrity enforcement "
            "(monocle-cross-ref-anchor-check).\n"
            "Verifies every path/to/file.md#anchor cross-reference in .factory/specs/\n"
            "resolves to a valid heading slug or explicit anchor in the target file.\n"
            "Dead anchors indicate heading renames not propagated to citing documents."
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
        help="Print detailed progress including resolved and exempt references.",
    )
    parser.add_argument(
        "--validate-slug-algorithm",
        action="store_true",
        help="Run slug algorithm validation only and exit. Useful for regression testing.",
    )
    args = parser.parse_args()

    # Always validate the slug algorithm at startup.
    _validate_slug_algorithm()

    if args.validate_slug_algorithm:
        print("POL-13: Slug algorithm validation PASS (3/3 ground-truth cases correct).")
        sys.exit(0)

    workspace_root = Path(args.workspace_root).resolve()
    factory_root = workspace_root / args.factory_root

    if not factory_root.is_dir():
        print(
            f"Error: factory root not found at {factory_root}.\n"
            f"In CI, pass --factory-root .factory-spec after checking out factory-artifacts.",
            file=sys.stderr,
        )
        sys.exit(2)

    specs_root = factory_root / "specs"
    if not specs_root.is_dir():
        print(
            f"Error: specs directory not found at {specs_root}.",
            file=sys.stderr,
        )
        sys.exit(2)

    # Collect source files
    all_files = collect_spec_files(factory_root)
    source_files = [
        f for f in all_files
        if not _is_factory_exempt(f, factory_root)
    ]

    if args.verbose:
        print(
            f"POL-13: scanning {len(source_files)} Markdown file(s) for "
            f"cross-reference anchors (factory: {factory_root})..."
        )

    # Anchor cache: target Path → frozenset of valid anchors
    anchor_cache: dict[Path, frozenset[str]] = {}
    stats = ScanStats()

    for file_path in sorted(source_files):
        scan_file(file_path, specs_root, anchor_cache, stats, verbose=args.verbose)

    # ── Report ────────────────────────────────────────────────────────────────
    n_findings = len(stats.findings)

    # Separate by error class
    file_not_found = [f for f in stats.findings if f.error_class == "file_not_found"]
    dead_anchors = [f for f in stats.findings if f.error_class == "dead_anchor"]

    if n_findings > 0:
        print(
            f"\nPOL-13 FAIL: {n_findings} unresolved cross-reference anchor(s) detected "
            f"({len(file_not_found)} file-not-found, {len(dead_anchors)} dead-anchor).\n"
            f"(monocle-cross-ref-anchor-check)\n",
            file=sys.stderr,
        )

        if file_not_found:
            print(f"── FILE NOT FOUND ({len(file_not_found)}) ──", file=sys.stderr)
            for f in sorted(file_not_found, key=lambda x: (x.source_file, x.line_number)):
                print(
                    f"  {f.source_file}:{f.line_number}  ->  {f.ref_text}   "
                    f"(target file not found)",
                    file=sys.stderr,
                )
            print("", file=sys.stderr)

        if dead_anchors:
            print(f"── DEAD ANCHORS ({len(dead_anchors)}) ──", file=sys.stderr)
            for f in sorted(dead_anchors, key=lambda x: (x.source_file, x.line_number)):
                context_note = ""
                if f.heading_context:
                    context_note = f"   (nearest heading: {f.heading_context!r})"
                print(
                    f"  {f.source_file}:{f.line_number}  ->  {f.ref_text}{context_note}",
                    file=sys.stderr,
                )

        print(
            f"\nFix options:\n"
            f"  1. Add a stable explicit anchor to the target heading:\n"
            f'     <a id="cited-anchor"></a>\n'
            f"     ## Actual Heading Text\n"
            f"  2. Update the citation to match the current heading slug.\n"
            f"  3. Convert to version-free citation without anchor "
            f"(if section reference is not load-bearing).",
            file=sys.stderr,
        )

        if not args.dry_run:
            sys.exit(1)
        else:
            print("(--dry-run: exiting 0 despite findings)", file=sys.stderr)
    else:
        print(
            f"POL-13 PASS: all cross-reference anchors resolve "
            f"({stats.refs_active} active, {stats.refs_exempt} exempt, "
            f"{stats.files_scanned} files scanned, "
            f"{stats.target_files_checked} target files checked)."
        )

    if args.verbose:
        print(
            f"\nScan summary:\n"
            f"  Files scanned:         {stats.files_scanned}\n"
            f"  Target files checked:  {stats.target_files_checked}\n"
            f"  Cross-refs found:      {stats.refs_found}\n"
            f"  Cross-refs exempt:     {stats.refs_exempt}\n"
            f"  Cross-refs active:     {stats.refs_active}\n"
            f"  Cross-refs resolved:   {stats.refs_resolved}\n"
            f"  Findings (unresolved): {n_findings}\n"
            f"    File-not-found:      {len(file_not_found)}\n"
            f"    Dead anchors:        {len(dead_anchors)}"
        )


if __name__ == "__main__":
    main()
