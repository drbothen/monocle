---
document_type: test-fixture
purpose: "POL-11 regression fixture — ADV-29 PRIMARY scope gap (same-line stale pin in AC prose)"
expected_pol11_result: FAIL
---

# POL-11 Regression Fixture: Same-Line Stale Active Pin in AC Prose

## Regression context

This fixture covers the PRIMARY scope gap surfaced in Adversarial Pass 29:
a same-line stale version-pin literal in story AC prose (e.g. "BC-2.06.004 v1.1.0
behavior was removed.") was not caught by POL-11 CI because collect_files()
hardcoded workspace_root/.factory as the scan root, so .factory-spec/stories/
(used in CI) was never scanned. Root cause: factory_root parameter was computed
in main() but not passed to collect_files().

Fix: collect_files(workspace_root, factory_root) now receives and uses the
resolved factory_root, scanning whatever directory name factory-artifacts was
checked out into.

## Active Section — must FAIL

The following is an active prose reference with a stale BC version and NO historical-
anchor marker. This is the pattern that survived CI at commit e6f57477:

There is NO ClientDisconnect IPC message — this BC-2.06.004 v1.1.0 behavior was removed.

## §Trace (historical — exempt)

At initial authoring time, BC-2.06.004 v1.1.0 was the canonical version.
