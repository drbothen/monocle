# Pass 6: Security & Dependencies — nikiforovall/lazyclaude

## Dependency surface

### Runtime dependencies (`pyproject.toml:24-30`)

| Package | Version pin | Risk profile | Notes |
|---|---|---|---|
| `textual` | `>=8.0.0` | LOW | Active project, MIT, large team |
| `rich` | `>=13.0.0` | LOW | Widely-used, well-audited |
| `pyyaml` | `>=6.0` | MEDIUM (historic CVEs) | Uses `safe_load` everywhere (`parsers/__init__.py:59`) — safe. No `yaml.load` |
| `pyperclip` | `>=1.11.0` | LOW | Wraps platform clipboard utilities (xclip, pbcopy). No code injection vector. Can crash if no clipboard is available. |
| `pathspec` | `>=1.0.0` | LOW | Pure-Python gitignore matcher. No I/O of its own. Commit message at HEAD `03691ef refactor: rename gitwildmatch to gitignore for pathspec 1.x` confirms recent migration. |

### Dev dependencies (`pyproject.toml:38-43`, `pyproject.toml:104-114`)

| Package | Role |
|---|---|
| `pytest>=9.0.0`, `pytest-asyncio>=0.24.0`, `pytest-textual-snapshot>=1.0.0`, `pyfakefs>=6.0.0` | testing |
| `ruff>=0.15.0` | lint + format |
| `mypy>=1.19.0` | type checking |
| `pre-commit>=4.5.0` | git hooks |
| `types-pyperclip`, `types-pyyaml` | stubs |

`uv.lock` is committed (99K bytes). Versions pinned reproducibly.

### Transitive / indirect

Standard library use:
- `subprocess` — heavy use, see security concerns below
- `webbrowser.open` — opens URLs in default browser; not an injection vector but does network DNS via OS
- `json`, `re`, `pathlib`, `os`, `fnmatch`, `shutil`, `platform`, `shlex`, `argparse`, `traceback`

## Security analysis

### S1 — Subprocess `shell=True` with list args (`mixins/marketplace.py:253-261`)

```python
subprocess.run(
    cmd,                       # list[str], e.g. ["claude", "plugin", "install", "<plugin_id>", "--scope", "user"]
    capture_output=True,
    check=True,
    shell=True,                # <-- with a LIST cmd, shell=True silently uses only cmd[0]
    encoding="utf-8",
    errors="replace",
    cwd=cwd,
)
```

**Behavior on POSIX:** with `shell=True` and `cmd` as a list, Python passes `cmd[0]` (`"claude"`) as the shell command and the rest as positional args to the shell ($0, $1, ...) — **not** as arguments to `claude`. So `claude plugin install foo` would on POSIX run effectively `/bin/sh -c "claude" "plugin" "install" "foo"`, executing only `claude`. **This is broken on POSIX** unless the operator's environment happens to mask the bug.

**Behavior on Windows:** `shell=True` invokes `cmd.exe /c <cmd[0]> <cmd[1]> ...` — works as expected.

**Severity:** **P0 portability bug** if anyone tries to use this on Linux/macOS. (Lazyclaude appears Windows-centric in CI assumptions? Not stated.) Even if working today via interpreter idiosyncrasy, this should be `shell=False` with the list.

**Security implication:** If `plugin.full_plugin_id` ever contained shell metacharacters (it shouldn't — `<name>@<marketplace>` format from JSON) and the shell-list-arg interaction were exercised, command injection would be feasible. The format is constrained by `marketplace.json` schemas, so attack surface is low in practice. **P1 to fix.**

Same pattern at `mixins/marketplace.py:293`:
```python
subprocess.Popen([editor, str(plugin.install_path)], shell=True)
```
Same risk profile.

### S2 — `os.environ.get("EDITOR", "vi")` + `shlex.split`

`app.py:574-576`:
```python
editor = os.environ.get("EDITOR", "vi")
cmd = shlex.split(editor) + [str(file_path)]
subprocess.Popen(cmd, shell=(sys.platform == "win32"))
```

User-controlled `$EDITOR` is shell-split then passed as argv. **Safe on POSIX** (`shell=False`), risky on Windows (`shell=True`) if `EDITOR` contains shell metacharacters. Low priority because Windows users typically set a path.

### S3 — Subprocess with file-path arg

`services/opener.py:19-25` uses `subprocess.run(["explorer"/"open"/"xdg-open", str(path)], check=False)`. Args are list, no shell. **Safe.**

### S4 — Reading user-controlled JSON / YAML files

All parsers use `yaml.safe_load` (`parsers/__init__.py:59`) and plain `json.loads`. No `pickle`, no `eval`. **No code-execution risk** from parsing customizations.

### S5 — Path traversal via `~` in memory imports

`memory_file.py:142-144`:
```python
if ref.startswith("~/"):
    return Path.home() / ref[2:]
```

A malicious memory file at User level could `@~/.ssh/known_hosts.md` — but the parser only includes Markdown files (regex `r"@([\w./~-]+\.md)"`), and `.ssh/known_hosts` lacks `.md`. Combined with cycle detection and read-only display, this is **low risk**. The widest exposure is leaking content of any `.md` file the user can read — but user is reading their own files. Self-XSS at worst.

### S6 — Clipboard

`app.py:622` `pyperclip.copy(path_str)` writes a resolved file path. Only the user can read their own clipboard. No risk.

### S7 — Webbrowser.open

`opener.py:31-41` constructs URLs via f-string interpolation of `repo` (string from marketplace JSON) and `sub_path` (plugin source path string). `webbrowser.open` on most browsers will URL-encode the input; even so, malformed values from marketplace JSON could open arbitrary URLs. **Trust boundary:** the marketplace JSON authored by the user (or fetched by `claude` CLI from their declared marketplaces). **Low risk** because attacker would need to control a marketplace the user has already added.

### S8 — Reading from `~/.claude.json`

`~/.claude.json` is the Claude CLI's main settings file. Lazyclaude reads it via `discovery._discover_mcps` and `_discover_local_mcps`. **No locking** is used — concurrent writes from the `claude` CLI may produce partial JSON that triggers `JSONDecodeError`, which is silently swallowed (`discovery.py:617-618`). Result: temporary blank-out of MCP listings. Not a vulnerability — a UX bug.

### S9 — Filesystem write atomicity

`writer.py:515-518`:
```python
def _write_settings_json(self, path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
```

No `tempfile.NamedTemporaryFile` + `Path.replace`. SIGKILL or crash mid-write **truncates** the settings file. P0 for Rust port. The same applies to `SettingsService.save` (`settings.py:55-69`).

### S10 — Race between move/copy/delete

`mixins/customization_actions.py:165-212`: copy succeeds → delete source. If delete fails for any reason (permission, file moved), the source remains and target also exists — **partial state** is reported but **not rolled back**. Operator must manually delete. P1.

### S11 — Resource exhaustion via skill discovery

`skill.py:19-69`: recursive scan + eager file read. A symlink loop is avoided by `Path.is_dir()` not following symlinks for `iterdir`, but a deep legitimate directory tree (e.g., a skill that includes `node_modules`) is partially mitigated by `_filter` skipping known dirs. **A non-gitignored bulky dir within a skill can OOM the process.** Real-world: a user dragging a giant doc set into their skill folder.

### S12 — TUI input handling

Textual handles raw terminal input. No injection vectors via keybindings — events are dispatched in-process. Safe.

## Supply chain

- `uv.lock` committed (`pyproject.toml:108-114` declares lock-managed dev deps).
- Pre-commit config (`pyproject.toml:107`) runs ruff, mypy, pytest before commit.
- GitHub Actions workflows exist (4 files in `.github/workflows/` — names not inspected). Release-drafter present (`.github/release-drafter.yml`).
- `hatch-vcs` generates `_version.py` from git tags. Build reproducibility hinges on git history integrity.

## Permissions / scope

LazyClaude **only reads and writes within the user's own home and project directories**. It does not:
- Open network sockets directly (only via `claude plugin install` subprocess and `webbrowser.open`).
- Escalate privileges.
- Listen on ports.
- Create services.

It **delegates installation** (with all its trust implications) to the `claude` CLI which lazyclaude assumes is already trusted.

## Vulnerabilities summary

| ID | Severity | Surface | Fix |
|---|---|---|---|
| S1 | **P1** | `subprocess.run(list, shell=True)` for plugin commands | Use `shell=False`. Untrusted IDs come only from marketplace.json which is user-curated — but the pattern is incorrect. |
| S9 | **P0** | Non-atomic JSON writes to settings/MCP files | Atomic-temp-rename pattern. Critical for `~/.claude.json` (shared with CLI). |
| S10 | **P1** | Move = copy+delete without rollback | Two-phase write or atomic rename for same-volume moves |
| S11 | P2 | Skill-discovery memory blow-up on huge dirs | Lazy file content read; size limit per file |
| S8 | P2 | Concurrent CLI writes to `~/.claude.json` produce inconsistent reads | Use file locks or retry on `JSONDecodeError` |

## State Checkpoint

```yaml
pass: 6
status: complete
timestamp: 2026-05-11T17:25:00Z
next_pass: 7
dependencies_audited: 5_runtime + 8_dev
vulnerabilities_p0: 1
vulnerabilities_p1: 2
```
