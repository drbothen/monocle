---
document_type: test-fixture
purpose: "POL-11 Pattern B fixture — non-index spec file (SS-*.md) with stale YAML inputs[] pin is HISTORICAL (not flagged)"
expected_pol11_result: PASS
target_subpath: specs/
target_filename: SS-foo.md
---

# POL-11 Pattern B Regression Fixture: Non-Index Spec File inputs[] YAML Pin — HISTORICAL

This fixture validates the ADR-0007 v1.0.6 default-HISTORICAL rule for Pattern B YAML
inputs[] pins. A non-index spec file (SS-foo.md, BC-*.md, BC-HOOK-*.md, etc.) with a
stale inputs[] YAML pin must be classified HISTORICAL and must NOT trigger a POL-11
failure.

Prior behavior (commit 55eafeb, conservative-default-ACTIVE): this file would have been
classified ACTIVE and the stale pin would have caused a FAIL. That behavior over-flagged
~92 inputs[] pins across SS-*/BC-*/BC-HOOK-*/dependency-graph/prd-expansion-scope.

New behavior (ADR-0007 v1.0.6, closed-active-set): ACTIVE only for *-INDEX.md and prd.md.
All other files (including this SS-foo.md) are HISTORICAL by default.

## Simulated Frontmatter Content

The inputs[] YAML pins below are intentionally stale but must be classified HISTORICAL
because the containing file (SS-foo.md) is not an *-INDEX.md file nor prd.md:

inputs:
  - {path: .factory/specs/architecture/SS-tui.md, version: "1.5.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}

All three YAML pins above reference stale spec versions (each below canonical). However,
because this file is placed as SS-foo.md (a non-index spec file), all three must be
classified HISTORICAL per ADR-0007 v1.0.6 and not flagged as stale.
POL-11 must log "[HISTORICAL] inputs[] provenance, exempt" for each and exit 0.
