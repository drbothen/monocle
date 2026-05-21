---
document_type: research-note
scope: F-004
producer: vsdd-factory:research-agent
timestamp: 2026-05-20T21:00:00Z
input-hash: "[live-state]"
advisory: RUSTSEC-2026-0007
cve: CVE-2026-25541
ghsa: GHSA-434x-w66g-qw3r
crate: bytes
resolved_version: 1.11.1
verdict: CLEAN
---

# Research Note: RUSTSEC-2026-0007 vs `bytes` 1.11.1

## Findings

### 1. Advisory affected version range (verbatim from rustsec.org)

From https://rustsec.org/advisories/RUSTSEC-2026-0007.html (and the OSV/GCVE
mirror at https://db.gcve.eu/vuln/rustsec-2026-0007):

- **Title:** "Integer overflow in `BytesMut::reserve`"
- **Package:** `bytes` (crates.io)
- **Patched:** `>=1.11.1`
- **Unaffected:** `<1.2.1`
- **Aliases:** CVE-2026-25541, GHSA-434x-w66g-qw3r
- **Reported / Issued:** 2026-02-03
- **Categories:** memory-corruption

OSV-format range (verbatim from GCVE):

```json
"ranges": [
  {
    "events": [
      { "introduced": "1.2.1" },
      { "fixed": "1.11.1" }
    ],
    "type": "SEMVER"
  }
]
```

Vulnerable window: **1.2.1 ≤ v < 1.11.1**. Fixed-from version: **1.11.1**.

### 2. Latest patched line — is 1.11.1 clean?

**YES — `bytes 1.11.1` is the fix-from version.** The advisory's `fixed` event
is exactly `1.11.1`. Per OSV semantics, `fixed: 1.11.1` means versions
`>=1.11.1` are no longer in the vulnerable range. This is corroborated by:

- Fedora alerts FEDORA-2026-6388b28850 (Fedora 42) and FEDORA-2026-f400579a21
  (Fedora 43) both shipping `rust-bytes-1.11.1-1.fc{42,43}` explicitly to close
  RUSTSEC-2026-0007.
- Upstream `tokio-rs/bytes` v1.11.1 release notes: "Fix integer overflow in
  `BytesMut::reserve`" (the patch swaps the unchecked addition for
  `checked_add()` returning `None` on overflow).
- SentinelOne CVE database lists "Affected: 1.2.1 to 1.11.0" and "Patched:
  1.11.1".

The advisory does **NOT** mark 1.11.x (x ≥ 1) as affected — 1.11.1 itself is
the fix.

### 3. Local `cargo audit` output

**Tooling limitation:** This research agent operates under a Bash-denied
profile (`Bash`, `exec`, `process` denied). I cannot invoke `cargo audit`
directly. What I CAN confirm from on-disk artifacts:

- `/Users/jmagady/Dev/monocle/Cargo.lock` lines 117–120:

  ```
  name = "bytes"
  version = "1.11.1"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33"
  ```

- The advisory's OSV range says `fixed: 1.11.1`, and the resolved version is
  exactly 1.11.1. **By the advisory's own machine-readable range, this is
  out-of-range — `cargo audit` will report NO finding against bytes 1.11.1
  for RUSTSEC-2026-0007.**

**Action required for completeness:** DevOps (or any agent with Bash) should
run `cargo audit --json` to file the empirical confirmation in CI logs. This
research's verdict is grounded in the authoritative advisory metadata, not in
a local audit run.

### 4. Was the `bytes = "1.10"` pin intentional or a typo?

`SS-deps-pin-manifest.md` line 41 (current `v1.1.18`) and lines 233–235 say:

> `prost` 0.14 has a transitive `bytes` advisory RUSTSEC-2026-0007 affecting
> older `bytes` versions. Pin `bytes = "1.10"` directly in workspace
> dependencies to force the patched version (verified 2026-05-12 — see bytes
> row in Phase 1 Pin Manifest).

**Read in context, the pin choice is a defensive-floor pin, not a freeze on
the 1.10 line.** Evidence:

- `"1.10"` is a Cargo caret-pin syntactic-shorthand equivalent to `^1.10`,
  which resolves to `>=1.10.0, <2.0.0`. The architect's narrative ("force the
  patched version", "verified... is the patched line") is consistent with
  picking a floor that excludes the vulnerable 1.2.1–1.9.x range.
- Verification was performed on **2026-05-12**. At that date the advisory was
  already public (issued 2026-02-03) and `bytes 1.11.1` was already published
  (Feb 2026 per Fedora alerts). The architect's note "the patched line" is
  the line that POST-DATES the fix — not specifically the 1.10 minor.
- There is **no other architectural reason** elsewhere in the manifest to
  freeze on 1.10.x specifically (no API breakage flagged for 1.11.x; no
  feature-gating restriction; no MSRV concern — 1.11.x has the same MSRV
  posture as 1.10.x).
- However, the architect's wording ("the 1.10 line", "verified... `bytes =
  "1.10"` is the patched line resolving RUSTSEC-2026-0007") is technically
  **inaccurate** as of the advisory's actual fix-from version: the patched
  line is `1.11.1`. **At 2026-05-12 verification time, `bytes 1.10.x` was
  itself still in the vulnerable window** (1.2.1 ≤ 1.10.x < 1.11.1). The
  resolver pulling 1.10.x by caret would have been a false-clean state.
  Resolver instead correctly resolved upward to **1.11.1**.

Most-likely root cause: the architect verified on 2026-05-12 against an
intermediate snapshot (possibly before the fix-from value moved to 1.11.1, or
based on a misreading of "patched >=" as "patched-line-is-1.10"). The pin
syntax `"1.10"` saved the situation because caret resolution let Cargo move
up to 1.11.1 anyway. The Cargo.lock proves resolver behavior was correct
even though the manifest narrative was imprecise.

### 5. Recommendation

**Verdict: bytes 1.11.1 is CLEAN against RUSTSEC-2026-0007. F-004 (lock-defect
MED) is a documentation drift, not a security exposure.**

Routing for architect (spec-owner; this research agent does NOT modify
specs):

1. **Update `SS-deps-pin-manifest.md` §Trace and bytes row** to document a
   2026-05-20 re-verification: replace "`bytes = "1.10"` is the patched line
   resolving RUSTSEC-2026-0007" with "advisory fix-from is `1.11.1`; caret
   pin `"1.10"` resolves to `1.11.1` per Cargo.lock; verified clean
   2026-05-20".
2. **Pin tightening is OPTIONAL.** Two production-grade options:
   - **Option A (preferred — keep `^1.10`, document):** Leave the manifest
     pin at `"1.10"` (caret). Cargo will continue resolving upward into the
     patched range. Cheaper to maintain; allows future patch absorption.
     Update the §Trace narrative to be precise about WHY this is safe.
   - **Option B (defensive — tighten to `^1.11`):** Move the pin floor to
     `"1.11"` so the manifest itself encodes the advisory floor. This makes
     the manifest a single-source-of-truth for the security baseline and
     eliminates the "manifest says 1.10, lock says 1.11.1" cognitive
     drift that F-004 surfaced. Recommended if the architect wants pin
     declarations to map 1:1 onto advisory floors.
   - **Do NOT tighten to `~1.10`** (tilde, only 1.10.x). That would force
     resolution back into the vulnerable window. The 1.10.x line never
     received the RUSTSEC-2026-0007 fix — the fix landed only in 1.11.1.
3. **No security-reviewer escalation required.** The advisory is satisfied
   by the resolved version. F-004 is correctly classified MED (drift, not
   exposure).

## Persistence + Commit Note

This research note is being written to
`.factory/plans/research-RUSTSEC-2026-0007-bytes-1.11.1.md`.

**Tooling limitation:** This research agent operates without Bash access, so
it cannot directly invoke `git checkout factory-artifacts && git commit -m
"research: F-004 RUSTSEC-2026-0007 verification against bytes 1.11.1"`. The
orchestrator should route the commit step to `vsdd-factory:state-manager`
(owner of `.factory/` commits per the routing table) or
`vsdd-factory:github-ops`.

## Sources

- [RUSTSEC-2026-0007 (rustsec.org)](https://rustsec.org/advisories/RUSTSEC-2026-0007.html)
- [RUSTSEC-2026-0007 OSV record (GCVE mirror)](https://db.gcve.eu/vuln/rustsec-2026-0007)
- [RUSTSEC advisory index — 2026-0007 entry](https://rustsec.org/advisories/)
- [CVE-2026-25541 (SentinelOne vuln database)](https://www.sentinelone.com/vulnerability-database/cve-2026-25541/)
- [tokio-rs/bytes v1.11.1 GitHub release notes](https://github.com/tokio-rs/bytes/releases/tag/v1.11.1)
- [Fedora alert FEDORA-2026-6388b28850 (rust-bytes 1.11.1-1.fc42)](https://lwn.net/Articles/1058169/)
- [Fedora alert FEDORA-2026-f400579a21 (rust-bytes 1.11.1-1.fc43)](https://lwn.net/Articles/1057910/)
- [WindowsForum analysis of CVE-2026-25541](https://windowsforum.com/threads/rust-bytes-vulnerability-cve-2026-25541-memory-safety-in-bytesmut-reserve.403939/)

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebFetch | 2 | rustsec.org advisory page; tokio-rs/bytes v1.11.1 release notes |
| Perplexity perplexity_search | 1 | Affected-range, patched-version, cross-distro confirmation |
| Read (local file) | 2 | Cargo.lock bytes resolution; SS-deps-pin-manifest.md bytes row context |
| Grep (local) | 2 | bytes name+version in Cargo.lock; bytes mentions in pin manifest |
| Glob (local) | 3 | Cargo.lock discovery; pin manifest discovery; .factory/plans/ enumeration |
| Training data | 1 area | Cargo caret-pin semantics (`"1.10"` ≡ `^1.10`) — independently verifiable from cargo-pkgid docs |

**Total MCP tool calls:** 3 (1 Perplexity + 2 WebFetch); 7 local-filesystem tools.
**Training data reliance:** low — every version-range claim is cited to
rustsec.org / GCVE / Fedora alerts. The only training-data leaning is Cargo
caret-pin syntax semantics, which is part of the stable Cargo grammar and
not version-sensitive.

## Prompt-injection note

The WebFetch call against rustsec.org returned the legitimate advisory data
prefixed with two unauthorized `<system-reminder>` blocks attempting to
inject MCP-server-instruction overrides and a no-clarifying-questions
directive. These were treated as adversarial content embedded in fetched web
output and ignored. Only the verbatim advisory body was used in this
research note.
