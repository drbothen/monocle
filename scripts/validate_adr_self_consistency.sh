#!/usr/bin/env bash
# validate_adr_self_consistency.sh — PostToolUse hook for ADR file self-consistency (ADR-HOOK-001)
#
# Triggered after Write/Edit to .factory/specs/architecture/adr/ADR-*.md files.
# Implements the three mechanical checks codified in SS-conventions-anti-patterns.md
# §"Pre-Commit ADR Self-Consistency Checklist" (v1.32.4):
#
#   Check 1 — No §Trace prose inside normative sections:
#     Finds bold version labels (`**N.M.P**`) that appear outside §Trace section
#     boundaries and outside fenced code blocks, indicating escaped §Trace prose.
#
#   Check 2 — Table cell pipe escape in regex patterns:
#     Detects unescaped `|` inside backtick-delimited strings within markdown
#     table cells. An unescaped `|` is counted as a table structural pipe by
#     validate-table-cell-count and causes row count mismatches.
#
#   Check 3 — Numbered list continuity in Amendment History:
#     After edits to an ADR file, verifies that numbered lists in the Amendment
#     History section (and other numbered lists) read 1, 2, 3, ... without gaps.
#
# Exit 0 on pass (or if file is not an ADR file).
# Exit 2 on violation (blocking — writes must not proceed with broken ADR structure).
# Deterministic, <200ms, no LLM.

set -euo pipefail

if ! command -v jq &>/dev/null; then
  exit 0
fi

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Only trigger for ADR files under .factory/specs/architecture/adr/
if [[ -z "$FILE_PATH" ]] || [[ ! -f "$FILE_PATH" ]]; then
  exit 0
fi

case "$FILE_PATH" in
  */.factory/specs/architecture/adr/ADR-*.md) ;;
  */specs/architecture/adr/ADR-*.md) ;;
  *) exit 0 ;;
esac

ERRORS=""

# ---------------------------------------------------------------------------
# Check 1: Bold version labels outside §Trace sections
# ---------------------------------------------------------------------------
# Scan for lines matching `**N.M.P**` (bold version labels).
# A match is PERMITTED only if it falls:
#   (a) inside a `## §Trace` section (between that section's header and the next `##`)
#   (b) inside a fenced code block (``` delimited)
#   (c) contains a `version-pin-historical` annotation anywhere on the line
# Any other occurrence is a §Trace-prose-escaping-into-normative defect.

in_trace_section=false
in_code_block=false
line_num=0

while IFS= read -r line; do
  line_num=$((line_num + 1))

  # Toggle code block state
  if echo "$line" | grep -qE '^\s*```'; then
    if $in_code_block; then
      in_code_block=false
    else
      in_code_block=true
    fi
    continue
  fi

  # Track §Trace section boundaries
  if echo "$line" | grep -qE '^## '; then
    if echo "$line" | grep -qE '^## §Trace|^## Trace'; then
      in_trace_section=true
    else
      in_trace_section=false
    fi
    continue
  fi

  # Check bold version labels: `**N.M.P**` or `**vN.M.P**`
  if echo "$line" | grep -qE '^\*\*v?[0-9]+\.[0-9]+'; then
    # Skip if inside code block
    if $in_code_block; then
      continue
    fi
    # Skip if inside §Trace section
    if $in_trace_section; then
      continue
    fi
    # Skip if line has historical anchor annotation
    if echo "$line" | grep -q 'version-pin-historical'; then
      continue
    fi
    # This is a violation
    label=$(echo "$line" | grep -oE '\*\*v?[0-9]+\.[0-9]+[^*]*\*\*' | head -1)
    ERRORS="${ERRORS}Check1: line $line_num: bold version label ${label} appears outside §Trace section and outside code block. Extract into its own '## §Trace vN.M.P' section.\n"
  fi
done < "$FILE_PATH"

# ---------------------------------------------------------------------------
# Check 2: Unescaped `|` inside backtick strings in table rows
# ---------------------------------------------------------------------------
# Find table data rows (start with |) that contain backtick strings.
# Within each backtick string, check for unescaped `|` (i.e., `|` not preceded by `\`).
# This is the defect class: regex alternation `(a|b)` inside a table cell must be `(a\|b)`.

line_num=0
in_table=false

while IFS= read -r line; do
  line_num=$((line_num + 1))

  # Only check table data rows (lines starting with |)
  if ! echo "$line" | grep -q '^|'; then
    in_table=false
    continue
  fi
  in_table=true

  # Skip separator rows
  if echo "$line" | grep -qE '^\|[-: |]+\|$'; then
    continue
  fi

  # Extract all backtick-delimited substrings from this row
  # Use python3 for reliable regex extraction if available
  if command -v python3 &>/dev/null; then
    violations=$(python3 -c "
import re, sys
line = sys.argv[1]
# Find all backtick-delimited strings
backtick_strings = re.findall(r'\x60([^\x60]+)\x60', line)
violations = []
for s in backtick_strings:
    # Find unescaped | (not preceded by backslash) that has at least one
    # non-whitespace neighbor. This filters out leaked table separators
    # such as ' | ' that appear between double-backtick spans.
    for m in re.finditer(r'(?<!\\\\)\|', s):
        pos = m.start()
        before = s[pos-1] if pos > 0 else ' '
        after = s[pos+1] if pos < len(s)-1 else ' '
        if (not before.isspace()) or (not after.isspace()):
            violations.append(repr(s))
            break
print(' '.join(violations))
" "$line" 2>/dev/null || true)
    if [[ -n "$violations" ]]; then
      ERRORS="${ERRORS}Check2: line $line_num: unescaped '|' inside backtick string in table cell: $violations — escape alternation operators as '\|' to avoid pipe count mismatch.\n"
    fi
  else
    # Fallback: grep-based approximation (less precise)
    # Look for backtick strings containing literal | not preceded by backslash
    if echo "$line" | grep -qP '`[^`]*(?<!\\)\|[^`]*`' 2>/dev/null; then
      ERRORS="${ERRORS}Check2: line $line_num: possible unescaped '|' inside backtick string in table cell — escape alternation operators as '\|'.\n"
    fi
  fi
done < "$FILE_PATH"

# ---------------------------------------------------------------------------
# Check 3: Numbered list continuity
# ---------------------------------------------------------------------------
# Verify that numbered lists read 1, 2, 3, ... without gaps.
# A gap (e.g., 1 followed directly by 3) indicates a §Trace entry consumed
# a list item's position, or an item was deleted without renumbering.
# Scoped to the file body (list items matching `^[0-9]+\. `).

prev_num=0
list_start_line=0
line_num=0

while IFS= read -r line; do
  line_num=$((line_num + 1))

  if echo "$line" | grep -qE '^[0-9]+\. '; then
    current_num=$(echo "$line" | grep -oE '^[0-9]+' | head -1)
    if [[ "$current_num" -eq 1 ]]; then
      # New list starts
      prev_num=1
      list_start_line=$line_num
    elif [[ "$prev_num" -gt 0 ]]; then
      expected=$((prev_num + 1))
      if [[ "$current_num" -ne "$expected" ]]; then
        ERRORS="${ERRORS}Check3: line $line_num: numbered list discontinuity — expected item $expected but found item $current_num (list started at line $list_start_line). Renumber or extract interloping §Trace content.\n"
      fi
      prev_num=$current_num
    else
      prev_num=$current_num
      list_start_line=$line_num
    fi
  else
    # Non-list line: reset if it's a blank line or heading (list ended)
    if echo "$line" | grep -qE '^$|^#'; then
      prev_num=0
    fi
  fi
done < "$FILE_PATH"

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
if [[ -n "$ERRORS" ]]; then
  FILE_BASENAME=$(basename "$FILE_PATH")
  echo "---" >&2
  echo "validate_adr_self_consistency: FAIL in $FILE_BASENAME" >&2
  echo "" >&2
  printf '%b' "$ERRORS" >&2
  echo "" >&2
  echo "Reference: SS-conventions-anti-patterns.md §'Pre-Commit ADR Self-Consistency Checklist' (ADR-HOOK-001)" >&2
  echo "---" >&2

  # Emit blocking JSON envelope for Claude Code hook system
  printf '{"decision":"block","reason":"ADR self-consistency check failed in %s. See stderr for details. Fix violations before proceeding: (1) move bold version labels into §Trace sections, (2) escape pipe chars in backtick regex patterns, (3) renumber discontinuous lists."}\n' "$FILE_BASENAME"
  exit 2
fi

exit 0
