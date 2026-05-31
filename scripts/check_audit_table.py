#!/usr/bin/env python3
"""
check_audit_table.py — CI Step 3: Audit-table gap check.

Compares semgrep findings for the monocle-non-exhaustive-struct-audit-completeness rule
against the Cross-Crate Constructor Audit Table in SS-engine-module.md.
Fails CI if any #[non_exhaustive] pub struct is absent from the audit table.

Contract source: SS-conventions-anti-patterns.md §Semgrep Coverage Hardening §Contract edge cases.
"""

import argparse
import json
import re
import sys

# Known fixture struct names (F-R44-adv-1): excluded from production scan before table lookup.
# These are present in semgrep output because semgrep-fixtures/**/*.rs is included in paths.include
# for Step 1 fixture corpus scanning. They are NOT production structs.
FIXTURE_STRUCT_NAMES = {"AuditFixtureMinimal", "AuditFixtureDerived"}

# Line-anchored delimiter regexes (§Contract edge cases clause 4).
# IMPORTANT: Do not quote these delimiter strings verbatim in prose — use names only.
BEGIN_DELIMITER_REGEX = r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$'
END_DELIMITER_REGEX   = r'^<!-- END: Cross-Crate Constructor Audit Table -->$'

# Regex for extracting struct name from the interpolated message string.
# F-S025-ADV16-MED-001 root cause: semgrep OSS does NOT populate metavars for pattern-either
# rules (confirmed semgrep 1.156.0). The struct name is present in the interpolated message
# field ("Found #[non_exhaustive] pub struct `StructName`.") but absent from metavars.$NAME.
# Primary extraction path: metavars.$NAME.abstract_content (future-proof if semgrep adds this).
# Fallback extraction path: regex against message field (required for semgrep OSS today).
_MESSAGE_NAME_RE = re.compile(r'Found #\[non_exhaustive\] pub struct `([^`]+)`')


def parse_semgrep_json(semgrep_json_text: str, rule_id: str) -> set[str]:
    """Parse semgrep JSON output and return matched struct names for the given rule.

    Name extraction strategy (two-path, F-S025-ADV16-MED-001):
    1. Primary: metavars.$NAME.abstract_content — populated by semgrep Pro / future OSS.
    2. Fallback: regex against the interpolated message string — required for semgrep OSS
       (verified semgrep 1.156.0 does not populate metavars for pattern-either rules).

    Safety assertion: if semgrep matched N findings for this rule but zero names were
    extracted, CI is failed immediately.  This prevents a silent false-green if both
    extraction paths fail (e.g., message format changes in a future semgrep version).
    """
    try:
        data = json.loads(semgrep_json_text)
    except json.JSONDecodeError as exc:
        print(f"Error: failed to parse semgrep JSON output: {exc}", file=sys.stderr)
        sys.exit(1)

    struct_names: set[str] = set()
    matched_count = 0
    for result in data.get("results", []):
        if result.get("check_id") != rule_id:
            continue
        matched_count += 1

        # Path 1: metavars (semgrep Pro / future OSS)
        metavars = result.get("extra", {}).get("metavars", {})
        name_meta = metavars.get("$NAME", {})
        struct_name = name_meta.get("abstract_content", "").strip("`").strip()

        # Path 2: message field fallback (semgrep OSS 1.x does not populate metavars
        # for pattern-either rules — confirmed F-S025-ADV16-MED-001 root-cause analysis)
        if not struct_name:
            message = result.get("extra", {}).get("message", "")
            m = _MESSAGE_NAME_RE.search(message)
            if m:
                struct_name = m.group(1).strip()

        if struct_name:
            struct_names.add(struct_name)

    # Safety assertion: matched findings with zero extracted names is a script defect,
    # not a CI pass.  Fail loudly so the gap is immediately visible.
    if matched_count > 0 and not struct_names:
        print(
            f"Error: semgrep matched {matched_count} finding(s) for rule {rule_id!r} but "
            f"struct name extraction returned zero names. Both metavars and message-regex "
            f"extraction paths failed. Inspect semgrep JSON output for schema changes.",
            file=sys.stderr,
        )
        sys.exit(1)

    return struct_names


def read_audit_table_struct_names(spec_file_path: str) -> set[str]:
    """
    Parse the Cross-Crate Constructor Audit Table from the spec file and return
    all struct names from the first column of every data row.

    Applies all §Contract edge cases:
    - Validates spec file exists
    - Detects duplicate/malformed delimiters
    - Skips separator rows (cells with only -/:/ /|)
    - Skips header rows (first cell == "Struct" or starts with "**")
    - Fails on empty table
    """
    # Contract edge case 2: missing spec file
    try:
        with open(spec_file_path, "r", encoding="utf-8") as fh:
            lines = fh.readlines()
    except FileNotFoundError:
        print(f"Error: spec file not found: {spec_file_path}", file=sys.stderr)
        sys.exit(1)

    # Strip trailing newlines for consistent matching
    stripped_lines = [line.rstrip("\n") for line in lines]

    # Contract edge case 4: duplicate delimiter detection (runs before table content is read)
    begin_indices = [
        i for i, line in enumerate(stripped_lines)
        if re.fullmatch(BEGIN_DELIMITER_REGEX, line)
    ]
    end_indices = [
        i for i, line in enumerate(stripped_lines)
        if re.fullmatch(END_DELIMITER_REGEX, line)
    ]

    if len(begin_indices) > 1:
        print(
            f"Error: multiple BEGIN delimiters found in {spec_file_path}; spec file is ambiguous",
            file=sys.stderr,
        )
        sys.exit(1)
    if len(end_indices) > 1:
        print(
            f"Error: multiple END delimiters found in {spec_file_path}; spec file is ambiguous",
            file=sys.stderr,
        )
        sys.exit(1)

    # Contract edge case 3: malformed delimiter pairs
    if begin_indices and not end_indices:
        print(
            f"Error: found BEGIN delimiter with no matching END delimiter in {spec_file_path}",
            file=sys.stderr,
        )
        sys.exit(1)
    if end_indices and not begin_indices:
        print(
            f"Error: found END delimiter with no preceding BEGIN delimiter in {spec_file_path}",
            file=sys.stderr,
        )
        sys.exit(1)
    if not begin_indices and not end_indices:
        print(
            f"Error: Cross-Crate Constructor Audit Table delimiters not found in {spec_file_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    begin_idx = begin_indices[0]
    end_idx = end_indices[0]

    if end_idx <= begin_idx:
        print(
            f"Error: found END delimiter with no preceding BEGIN delimiter in {spec_file_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    # Extract lines between delimiters (exclusive)
    table_lines = stripped_lines[begin_idx + 1 : end_idx]

    # Contract edge case 1: skip separator rows and header rows; extract struct names from data rows
    # Separator row pattern: row whose cells contain only hyphens, colons, spaces, and pipes
    separator_regex = re.compile(r'^\|[-: |]+\|$')

    data_row_struct_names: set[str] = set()
    data_row_count = 0

    for line in table_lines:
        if not line.strip():
            continue
        if separator_regex.match(line):
            continue
        if not line.startswith("|"):
            continue
        # Extract first cell: text between first | and second |
        parts = line.split("|")
        if len(parts) < 3:
            continue
        first_cell = parts[1].strip().strip("`").strip("*").strip()
        # Skip header rows: first cell == "Struct" or starts with "**"
        if first_cell == "Struct" or first_cell.startswith("**"):
            continue
        if not first_cell:
            continue
        data_row_count += 1
        data_row_struct_names.add(first_cell)

    # Contract edge case 5: empty table
    if data_row_count == 0:
        print(
            f"Error: Cross-Crate Constructor Audit Table in {spec_file_path} has no data rows; this is a spec gap",
            file=sys.stderr,
        )
        sys.exit(1)

    return data_row_struct_names


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Audit-table gap check: verify all #[non_exhaustive] pub structs appear in the audit table."
    )
    parser.add_argument(
        "--semgrep-json",
        required=True,
        help="Path to semgrep JSON output file (or use process substitution: <(semgrep --json ...))",
    )
    parser.add_argument(
        "--spec-file",
        required=True,
        help="Path to SS-engine-module.md containing the Cross-Crate Constructor Audit Table",
    )
    parser.add_argument(
        "--rule-id",
        default="monocle-non-exhaustive-struct-audit-completeness",
        help="Semgrep rule ID to filter from JSON output (default: monocle-non-exhaustive-struct-audit-completeness)",
    )
    args = parser.parse_args()

    # Read semgrep JSON
    try:
        with open(args.semgrep_json, "r", encoding="utf-8") as fh:
            semgrep_json_text = fh.read()
    except FileNotFoundError:
        print(f"Error: semgrep JSON file not found: {args.semgrep_json}", file=sys.stderr)
        sys.exit(1)

    # Parse struct names from semgrep output
    semgrep_struct_names = parse_semgrep_json(semgrep_json_text, args.rule_id)

    # Remove known fixture struct names (F-R44-adv-1)
    production_struct_names = semgrep_struct_names - FIXTURE_STRUCT_NAMES
    removed = semgrep_struct_names & FIXTURE_STRUCT_NAMES
    if removed:
        print(f"Fixture exclusion: removed {sorted(removed)} from semgrep output (fixture structs, not production).")

    if not production_struct_names:
        print(f"Audit table: complete (0 production structs declared; no #[non_exhaustive] pub structs found by semgrep).")
        sys.exit(0)

    # Read struct names from audit table
    table_struct_names = read_audit_table_struct_names(args.spec_file)

    # Compute gaps: structs in semgrep output but NOT in audit table
    gaps = production_struct_names - table_struct_names
    n = len(production_struct_names)

    if gaps:
        gap_list = ", ".join(f"`{name}`" for name in sorted(gaps))
        print(
            f"Audit table gap: following structs carry #[non_exhaustive] but are absent from the "
            f"Cross-Crate Constructor Audit Table: {gap_list}. "
            f"Update SS-engine-module.md §Cross-Crate Constructor Audit and add a constructor if any "
            f"cross-crate construction site exists.",
            file=sys.stderr,
        )
        sys.exit(1)

    print(f"Audit table: complete ({n} structs declared, {n} structs found by semgrep).")
    sys.exit(0)


if __name__ == "__main__":
    main()
