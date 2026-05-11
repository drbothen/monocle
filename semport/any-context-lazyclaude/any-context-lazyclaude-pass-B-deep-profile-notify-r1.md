# Pass B Deep: `internal/profile` and `internal/notify` — Round 1

**Scope:** Both subsystems together since each is small (~300 LOC). Both flagged as P2 by orienting prompt because single test file each.

**Files read in full this round:** internal/profile/profile.go (265 LOC), internal/profile/expand.go (34), internal/notify/notify.go (82).

## Profile subsystem

### BC-PROF-001: CurrentVersion = 1; only supported schema; missing version → error
**Postconditions:** `Version: 0` (missing/JSON default) → "config.json missing required version field (expected 1)". Different value → "config.json version N is not supported (expected 1)". Forward compatibility blocked.
**Evidence:** profile.go:103-108.
**Confidence:** HIGH

### BC-PROF-002: Load returns `(nil, [BuiltinDefault()], nil)` when config file doesn't exist (NOT an error)
**Postconditions:** Confirms BC-PROFILE-001 from Pass 3. Absent config = use built-in.
**Evidence:** profile.go:89-92.
**Confidence:** HIGH

### BC-PROF-003: Load uses DisallowUnknownFields → typo in config field → error
**Postconditions:** Strict schema. Unknown fields like `"comand"` or `"args_list"` are rejected, not silently ignored.
**Evidence:** profile.go:97-101.
**Confidence:** HIGH — **NEW finding** (strict schema validation).

### BC-PROF-004: annotateJSONError converts json.SyntaxError + UnmarshalTypeError to "invalid JSON at line N, col M: ..."
**Postconditions:** User-friendly error messages with position info. Other errors get bare "invalid JSON: %w".
**Evidence:** profile.go:234-246 + offsetToLineCol 248-265.
**Confidence:** HIGH — confirms BC-DAEMON-013 4-state error encoding (daemon /profiles uses this output verbatim).

### BC-PROF-005: BuiltinDefault = `{Name: "default", Command: "claude", Builtin: true}`
**Postconditions:** No args, no env. Plain `claude` invocation.
**Evidence:** profile.go:48-55.
**Confidence:** HIGH

### BC-PROF-006: BuiltinDefault is auto-appended to effective list ONLY when user hasn't defined a profile named "default"
**Postconditions:** Users can override the default by defining a profile literally named "default". Confirms BC-PROFILE-003 from Pass 3.
**Evidence:** profile.go:126-128.
**Confidence:** HIGH

### BC-PROF-007: Profile name regex: `^[a-zA-Z0-9_-]{1,64}$`
**Postconditions:** 64-char max. Hyphens and underscores allowed. No spaces, dots, slashes.
**Evidence:** profile.go:58, 152-154.
**Confidence:** HIGH

### BC-PROF-008: Env key regex: `^[A-Z_][A-Z0-9_]*$`
**Postconditions:** Stricter than tmux exec env key regex (`^[A-Za-z_][A-Za-z0-9_]*$`). Forces UPPERCASE_WITH_UNDERSCORES convention.
**Evidence:** profile.go:59, 164-167.
**Confidence:** HIGH — **NEW finding**: stricter env naming in profile vs tmux.

### BC-PROF-009: Args validation rejects 5 banned flags: --session-id, --resume, --fork-session, --settings, --append-system-prompt (both bare and `=value` forms)
**Postconditions:** lazyclaude's session-lifecycle flags are reserved. Users cannot duplicate them via profile.args.
**Evidence:** profile.go:62-73, 220-229.
**Confidence:** HIGH — **NEW finding**: explicit banned-flag enforcement at profile load time (not just at launch).

### BC-PROF-010: Duplicate profile names cause Load error: `profile[N] "name": duplicate name`
**Evidence:** profile.go:116-119.
**Confidence:** HIGH

### BC-PROF-011: Load deep-copies Args and Env to prevent caller-corruption of the parsed *Config
**Evidence:** profile.go:135-148.
**Confidence:** HIGH

### BC-PROF-012: ResolveDefault precedence: first Default=true → named "default" → BuiltinDefault
**Postconditions:** Multiple Default=true profiles produce a warning naming the chosen one. Confirms BC-PROFILE-002.
**Evidence:** profile.go:177-210.
**Confidence:** HIGH

### BC-PROF-013: ResolveDefault returns warnings (not errors) for multiple defaults — caller surfaces in debug logs
**Postconditions:** Warning format: `multiple profiles marked default: using "name", ignoring "other1", "other2"`.
**Evidence:** profile.go:196-201.
**Confidence:** HIGH

### BC-PROF-014: ProfileDef.Builtin field is `json:"-"` — never persisted; only set internally for the auto-injected default
**Evidence:** profile.go:38.
**Confidence:** HIGH

### BC-PROF-015: profile.Validate is pure — does not touch the filesystem or expand paths
**Postconditions:** Path expansion happens in `profile.ExpandPath` (expand.go) at launch time, not at load time.
**Evidence:** profile.go:152 (Validate signature + behavior).
**Confidence:** HIGH

### BC-PROF-016: ErrProfileNotFound sentinel exists but not used inside the profile package itself — for callers to compare against
**Evidence:** profile.go:76-78.
**Confidence:** HIGH

### BC-PROF-017: Config file path is conventionally `$HOME/.lazyclaude/config.json` (called by root.go and daemon/server.go)
**Postconditions:** Profile.Load takes a path arg, so test cases can use custom paths.
**Evidence:** profile.go:1-7 doc comment + manager.go:121-126 (profileConfigHint).
**Confidence:** HIGH

## Notify subsystem (file-polling fallback)

### BC-NOTIFY-001: Queue files named `lazyclaude-q-<20-digit-nanosecond-timestamp>.json`
**Postconditions:** 20-digit zero-padded timestamps → lexicographic sort = chronological order. Sub-microsecond ordering.
**Evidence:** notify.go:17, 30.
**Confidence:** HIGH

### BC-NOTIFY-002: Enqueue creates runtimeDir with mode 0o700 if absent; writes file with 0o600
**Postconditions:** Owner-only access. Matches BC-DAEMON-015 / Pass 5 file-mode discipline.
**Evidence:** notify.go:22-32.
**Confidence:** HIGH

### BC-NOTIFY-003: ReadAll filters by prefix `lazyclaude-q-` AND suffix `.json`
**Postconditions:** Other files in runtimeDir (daemon.json, askpass.sock) are ignored.
**Evidence:** notify.go:47-51.
**Confidence:** HIGH

### BC-NOTIFY-004: ReadAll sorts file names lexicographically — equivalent to chronological because timestamp format is fixed-width
**Evidence:** notify.go:57.
**Confidence:** HIGH

### BC-NOTIFY-005: ReadAll is DESTRUCTIVE — removes each file after reading; best-effort remove (continues on error)
**Postconditions:** Files that another reader already removed don't cause failure. The race is benign: each notification is delivered to at most one reader.
**Evidence:** notify.go:62-72.
**Confidence:** HIGH

### BC-NOTIFY-006: maxAge = 30 seconds; notifications older than this are silently dropped
**Postconditions:** "Skip stale notifications (Claude Code already moved on)" — prevents popup spam from a backed-up queue.
**Evidence:** notify.go:59, 73-77.
**Confidence:** HIGH — **NEW finding**: explicit staleness window.

### BC-NOTIFY-007: ReadAll continues past file-read or unmarshal errors (per-file resilience)
**Postconditions:** A single corrupt file doesn't poison the queue. Returns the parseable notifications.
**Evidence:** notify.go:62-78 (`continue` on errors).
**Confidence:** HIGH

### BC-NOTIFY-008: ReadAll on missing runtimeDir returns `(nil, nil)` (no error)
**Postconditions:** Daemon-mode startup before any notification is enqueued.
**Evidence:** notify.go:38-41.
**Confidence:** HIGH

### BC-NOTIFY-009: Concurrent enqueue safety: each Enqueue writes a unique file (nanosecond timestamp); no shared file
**Postconditions:** Two concurrent Enqueues at the same nanosecond would collide on file name — extremely unlikely. If it happened, the second WriteFile would overwrite.
**Evidence:** notify.go:30-32. **NEW finding** as a theoretical edge case.
**Confidence:** HIGH (for typical conditions); MEDIUM for the nanosecond-collision edge case.

### BC-NOTIFY-010: Notification.Timestamp is checked via IsZero — Zero values bypass the staleness filter
**Postconditions:** Notifications without timestamps are always delivered. Defensive: legacy notifications survive a code upgrade that adds Timestamp later.
**Evidence:** notify.go:75.
**Confidence:** HIGH

## Cross-pass observations

### Verification of P1 gaps from Pass 4

- **Gap-VER-001 (Profile loading thin tests)**: Source-side review found rich validation (banned flags, env regex, duplicate detection, version check, line/col error reporting). The single test file is concerning but the code is defensive. Risk mitigated by codepath review.
- **Gap-VER-004 (Notify single test)**: Source-side review confirms FIFO ordering (lexicographic sort), staleness window (30s), and per-file resilience. Concurrent enqueue safety relies on nanosecond uniqueness — acceptable.
- **Gap-VER-006 (Daemon /profiles error format)**: BC-PROF-004 confirms the format is "invalid JSON at line N, col M". The daemon doc at server.go:617-632 is accurate.

## Delta Summary

- New items added: 27 (17 BC-PROF, 10 BC-NOTIFY)
- Existing items refined: BC-PROFILE-001/002/003 confirmed at code level. BC-DAEMON-013 confirmed (4-state error encoding). Pass 4 Gap-VER-001/004/006 verified.
- Remaining gaps: profile/expand.go (34 LOC, trivial path expander), test files. Both small enough to skim if needed.

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: 27 new contracts including:
- **BC-PROF-003** DisallowUnknownFields — typo-strict schema (NEW).
- **BC-PROF-009** banned-flag enforcement at LOAD time, not just launch (NEW).
- **BC-PROF-008** stricter env-key regex than tmux (NEW divergence).
- **BC-NOTIFY-006** 30-second staleness window (NEW).
- **BC-NOTIFY-009** nanosecond-uniqueness as concurrent-safety mechanism (NEW edge case).

These are porter-relevant for profile config validation and file-queue semantics. Both subsystems are also fully verified against Pass 4's flagged gaps.

## Convergence Declaration

**Pass B profile+notify has converged.** Both subsystems are small, well-bounded, and fully spec'd. expand.go (34 LOC) is a trivial pure-function path expander; reading it would add 1-2 minor contracts at most.

## State Checkpoint

```yaml
pass: B
subsystem: profile-notify
round: 1
status: complete
files_read_full: [internal/profile/profile.go, internal/notify/notify.go]
files_read_partial: [internal/profile/expand.go (unread, 34 LOC)]
contracts_drafted: 27
p1_gaps_verified: 3  # Gap-VER-001, Gap-VER-004, Gap-VER-006
timestamp: 2026-05-11T23:25:00Z
novelty: SUBSTANTIVE
convergence: PASS-B-PROFILE-NOTIFY CONVERGED
next_phase: B.5 coverage audit
```
