#!/usr/bin/env python3
"""
extract_audit_table.py — Extract the delimited audit-table block from SS-engine-module.md.

Used by:
  - CI job 'audit-table-drift' to produce a canonical copy for diffing against scripts/audit-table.md
  - Maintainers regenerating scripts/audit-table.md after SS-engine-module.md changes

Usage:
  python3 scripts/extract_audit_table.py \\
    --source .factory-spec/specs/architecture/SS-engine-module.md \\
    --output /tmp/audit-table-canonical.md

Output format: the delimited block extracted from --source, including the BEGIN/END delimiter
comment lines. The output is designed to diff cleanly against the delimited section of
scripts/audit-table.md (everything from <!-- BEGIN ... --> through <!-- END ... -->).

Exit codes:
  0 — block extracted successfully
  1 — delimiter not found, ambiguous, or I/O error
"""

import argparse
import re
import sys

# Line-anchored delimiter regexes — must match check_audit_table.py exactly.
BEGIN_DELIMITER_REGEX = re.compile(r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$')
END_DELIMITER_REGEX   = re.compile(r'^<!-- END: Cross-Crate Constructor Audit Table -->$')


def extract_block(source_path: str) -> list[str]:
    """
    Read source_path and return all lines from BEGIN delimiter through END delimiter (inclusive).
    Raises SystemExit(1) on any error condition.
    """
    try:
        with open(source_path, "r", encoding="utf-8") as fh:
            lines = fh.readlines()
    except FileNotFoundError:
        print(f"Error: source file not found: {source_path}", file=sys.stderr)
        sys.exit(1)

    # Strip trailing newlines for regex matching; keep original line for output.
    stripped = [line.rstrip("\n") for line in lines]

    begin_indices = [i for i, l in enumerate(stripped) if BEGIN_DELIMITER_REGEX.fullmatch(l)]
    end_indices   = [i for i, l in enumerate(stripped) if END_DELIMITER_REGEX.fullmatch(l)]

    if len(begin_indices) > 1:
        print(f"Error: multiple BEGIN delimiters found in {source_path}", file=sys.stderr)
        sys.exit(1)
    if len(end_indices) > 1:
        print(f"Error: multiple END delimiters found in {source_path}", file=sys.stderr)
        sys.exit(1)
    if not begin_indices:
        print(f"Error: BEGIN delimiter not found in {source_path}", file=sys.stderr)
        sys.exit(1)
    if not end_indices:
        print(f"Error: END delimiter not found in {source_path}", file=sys.stderr)
        sys.exit(1)

    begin_idx = begin_indices[0]
    end_idx   = end_indices[0]

    if end_idx <= begin_idx:
        print(
            f"Error: END delimiter appears before or at BEGIN delimiter in {source_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    # Return lines from BEGIN through END inclusive, preserving original line endings.
    # Use original lines (not stripped) to preserve trailing whitespace/newlines faithfully.
    block = lines[begin_idx : end_idx + 1]

    # Normalise: ensure each line ends with exactly one newline (for clean diffs).
    block = [l.rstrip("\n") + "\n" for l in block]
    return block


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Extract the Cross-Crate Constructor Audit Table delimited block from "
            "SS-engine-module.md for use in drift detection CI."
        )
    )
    parser.add_argument(
        "--source",
        required=True,
        metavar="SS_ENGINE_MODULE_MD",
        help="Path to SS-engine-module.md (typically .factory-spec/specs/architecture/SS-engine-module.md in CI)",
    )
    parser.add_argument(
        "--output",
        required=True,
        metavar="OUTPUT_MD",
        help="Path to write the extracted block (e.g. /tmp/audit-table-canonical.md)",
    )
    args = parser.parse_args()

    block = extract_block(args.source)

    try:
        with open(args.output, "w", encoding="utf-8") as fh:
            fh.writelines(block)
    except OSError as exc:
        print(f"Error: could not write output file {args.output}: {exc}", file=sys.stderr)
        sys.exit(1)

    print(f"Extracted {len(block)} lines to {args.output}")


if __name__ == "__main__":
    main()
