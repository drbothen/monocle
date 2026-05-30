#!/usr/bin/env python3
"""
run_pol_tests.py — Test runner for POL-11 + POL-12 fixture tests.

Verifies that check_version_pins.py (POL-11) and check_structural_claims.py (POL-12)
produce the expected outcomes on fixture files.

Each fixture file has a YAML frontmatter field:
  expected_pol11_result: PASS | FAIL   (for POL-11 fixtures)
  expected_pol12_result: PASS | FAIL   (for POL-12 fixtures)

This script runs each check script against the fixture and asserts the result
matches the expected outcome.

USAGE (from workspace root):
  python3 scripts/tests/run_pol_tests.py

EXIT CODES:
  0 — all fixture tests pass (scripts produce expected outcomes)
  1 — one or more fixture tests produced unexpected outcomes
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

# Locate scripts relative to this file
TESTS_DIR = Path(__file__).parent
SCRIPTS_DIR = TESTS_DIR.parent
WORKSPACE_ROOT = SCRIPTS_DIR.parent

# Registry search order:
#   1. <workspace-root>/.factory/specs/version-pin-registry.yaml (direct, or CI checkout)
#   2. Walk up from WORKSPACE_ROOT looking for a .factory/ directory (worktree support:
#      the main repo .factory/ may be several levels up from .worktrees/<story>/)
#   3. <workspace-root>/.factory-spec/specs/version-pin-registry.yaml (CI alternate checkout)


def _resolve_registry() -> Path:
    """
    Find version-pin-registry.yaml by walking up the directory tree.

    Worktrees are at <repo-root>/.worktrees/<story>/; the .factory/ worktree
    is mounted at <repo-root>/.factory/. Walking up from the worktree finds it.
    """
    # Start from WORKSPACE_ROOT and walk upward (up to 5 levels)
    search_root = WORKSPACE_ROOT
    for _ in range(6):
        candidate = search_root / ".factory" / "specs" / "version-pin-registry.yaml"
        if candidate.is_file():
            return candidate
        ci_candidate = search_root / ".factory-spec" / "specs" / "version-pin-registry.yaml"
        if ci_candidate.is_file():
            return ci_candidate
        parent = search_root.parent
        if parent == search_root:
            break
        search_root = parent

    print(
        f"Error: version-pin registry not found.\n"
        f"Searched for .factory/specs/version-pin-registry.yaml walking up from {WORKSPACE_ROOT}.\n"
        f"Ensure the registry is seeded at the repo root's .factory/specs/ directory.",
        file=sys.stderr,
    )
    sys.exit(1)


def _read_expected(fixture_path: Path, key: str) -> str | None:
    """Read the expected result from a fixture file's frontmatter (PASS or FAIL)."""
    try:
        text = fixture_path.read_text(encoding="utf-8")
    except OSError:
        return None

    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith(f"{key}:"):
            value = stripped.split(":", 1)[1].strip().strip('"').strip("'")
            return value.upper()
    return None


def _read_frontmatter_str(fixture_path: Path, key: str) -> str | None:
    """Read a string field from a fixture file's frontmatter."""
    try:
        text = fixture_path.read_text(encoding="utf-8")
    except OSError:
        return None

    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith(f"{key}:"):
            value = stripped.split(":", 1)[1].strip().strip('"').strip("'")
            return value if value else None
    return None


def _resolve_fixture_target(
    fixture_path: Path,
    factory_dir: Path,
    workspace_root: Path | None = None,
) -> Path:
    """
    Determine where to place a fixture file during testing.

    Reads the optional 'target_subpath' frontmatter field:
      - 'plans/'                 → .factory/plans/<filename>
      - 'planning/'              → .factory/planning/<filename>
      - 'code-delivery/'         → .factory/code-delivery/<filename>
      - 'specs/'                 → .factory/specs/<filename>
      - 'STATE.md'               → .factory/STATE.md  (replaces the file directly)
      - 'tech-debt-register.md'  → .factory/tech-debt-register.md  (specific-file exemption)
      - 'root/'                  → <workspace-root>/<filename>  (workspace-root placement,
                                   for files like CLAUDE.md excluded via _EXCLUDED_ROOT_FILES)
      - (none/absent)            → .factory/stories/<filename>  (default)

    Reads the optional 'target_filename' frontmatter field to override the
    destination filename. This allows Pattern B fixtures to be placed with a
    specific name (e.g., 'S-999-patternb-story.md' for story classification,
    or 'STORY-INDEX.md' for living-index-doc classification) without renaming
    the fixture file itself.

    Returns the full target Path (file path, not directory). Creates parent dirs.
    """
    target_subpath = _read_frontmatter_str(fixture_path, "target_subpath")
    # target_filename overrides the destination filename (for Pattern B classification tests)
    target_filename = _read_frontmatter_str(fixture_path, "target_filename") or fixture_path.name

    if target_subpath is None or target_subpath == "stories/":
        # Default: place in stories/ (pre-existing behaviour)
        target_dir = factory_dir / "stories"
        target_dir.mkdir(parents=True, exist_ok=True)
        return target_dir / target_filename

    if target_subpath == "STATE.md":
        # Place as the STATE.md file itself
        factory_dir.mkdir(parents=True, exist_ok=True)
        return factory_dir / "STATE.md"

    if target_subpath == "tech-debt-register.md":
        # Place as factory-root/tech-debt-register.md (specific-file exemption at factory root)
        factory_dir.mkdir(parents=True, exist_ok=True)
        return factory_dir / target_filename

    if target_subpath == "root/":
        # Place at the workspace root (for files like CLAUDE.md excluded via _EXCLUDED_ROOT_FILES)
        if workspace_root is None:
            raise ValueError("target_subpath 'root/' requires workspace_root to be provided")
        workspace_root.mkdir(parents=True, exist_ok=True)
        return workspace_root / target_filename

    # All other subpaths: strip trailing slash, create dir, place fixture inside
    subdir_name = target_subpath.rstrip("/")
    target_dir = factory_dir / subdir_name
    target_dir.mkdir(parents=True, exist_ok=True)
    return target_dir / target_filename


def run_pol11_fixture(fixture_path: Path, registry: Path) -> tuple[bool, str]:
    """
    Run POL-11 against a single fixture file.

    Creates a minimal test directory containing only the fixture file so the
    scan is isolated. Returns (passed, detail_message).

    Supports 'target_subpath' frontmatter field to place fixtures in exempt
    directories (plans/, planning/, code-delivery/, STATE.md, tech-debt-register.md,
    root/) for regression testing of path-level and root-file exclusions.
    Default is .factory/stories/ (normative).
    """
    expected_str = _read_expected(fixture_path, "expected_pol11_result")
    if expected_str is None:
        return True, f"  SKIP {fixture_path.name} (no expected_pol11_result field)"

    expected_fail = (expected_str == "FAIL")
    target_subpath = _read_frontmatter_str(fixture_path, "target_subpath") or "stories/"

    # Run check_version_pins.py with the fixture file as the only scan target.
    # We pass --workspace-root to a temp directory containing only the fixture,
    # so the scan is isolated.
    import tempfile, shutil
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        factory_dir = tmp_path / ".factory"
        factory_dir.mkdir(parents=True)

        dest_path = _resolve_fixture_target(fixture_path, factory_dir, workspace_root=tmp_path)
        shutil.copy(fixture_path, dest_path)

        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPTS_DIR / "check_version_pins.py"),
                "--workspace-root", str(tmp_path),
                "--registry", str(registry),
            ],
            capture_output=True,
            text=True,
        )

    actual_fail = (result.returncode != 0)

    # Build human-readable placement label
    if target_subpath == "root/":
        target_filename = _read_frontmatter_str(fixture_path, "target_filename") or fixture_path.name
        placement = f"placed at workspace root as {target_filename}"
    else:
        placement = f"placed in .factory/{target_subpath}"

    if actual_fail == expected_fail:
        status = "FAIL" if expected_fail else "PASS"
        return True, f"  OK  {fixture_path.name} (expected {status}, got {status}) [{placement}]"
    else:
        expected_label = "FAIL" if expected_fail else "PASS"
        actual_label = "FAIL" if actual_fail else "PASS"
        detail = result.stderr.strip()[:200] if result.stderr else result.stdout.strip()[:200]
        return False, (
            f"  ERR {fixture_path.name}: expected {expected_label}, got {actual_label} [{placement}]\n"
            f"      stdout: {result.stdout.strip()[:100]}\n"
            f"      stderr: {detail}"
        )


def run_pol12_fixture(fixture_path: Path, factory_root: Path) -> tuple[bool, str]:
    """
    Run POL-12 against a single fixture file.

    Creates a minimal factory directory containing only the fixture file.
    Returns (passed, detail_message).
    """
    expected_str = _read_expected(fixture_path, "expected_pol12_result")
    if expected_str is None:
        return True, f"  SKIP {fixture_path.name} (no expected_pol12_result field)"

    expected_fail = (expected_str == "FAIL")

    import tempfile, shutil
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        target_dir = tmp_path / "factory-test" / "stories"
        target_dir.mkdir(parents=True)
        shutil.copy(fixture_path, target_dir / fixture_path.name)

        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPTS_DIR / "check_structural_claims.py"),
                "--workspace-root", str(tmp_path),
                "--factory-root", "factory-test",
            ],
            capture_output=True,
            text=True,
        )

    actual_fail = (result.returncode != 0)

    if actual_fail == expected_fail:
        status = "FAIL" if expected_fail else "PASS"
        return True, f"  OK  {fixture_path.name} (expected {status}, got {status})"
    else:
        expected_label = "FAIL" if expected_fail else "PASS"
        actual_label = "FAIL" if actual_fail else "PASS"
        detail = result.stderr.strip()[:200] if result.stderr else result.stdout.strip()[:200]
        return False, (
            f"  ERR {fixture_path.name}: expected {expected_label}, got {actual_label}\n"
            f"      stdout: {result.stdout.strip()[:100]}\n"
            f"      stderr: {detail}"
        )


def main() -> None:
    registry = _resolve_registry()

    # Collect fixture files
    pol11_fixtures = sorted(TESTS_DIR.glob("pol11_test_*.md"))
    pol12_fixtures = sorted(TESTS_DIR.glob("pol12_test_*.md"))

    if not pol11_fixtures and not pol12_fixtures:
        print("No fixture files found in scripts/tests/. Nothing to test.", file=sys.stderr)
        sys.exit(1)

    all_passed = True
    results: list[str] = []

    print(f"POL-11 fixture tests (registry: {registry}):")
    for fixture in pol11_fixtures:
        passed, message = run_pol11_fixture(fixture, registry)
        results.append(message)
        print(message)
        if not passed:
            all_passed = False

    print(f"\nPOL-12 fixture tests:")
    for fixture in pol12_fixtures:
        passed, message = run_pol12_fixture(fixture, None)  # type: ignore[arg-type]
        results.append(message)
        print(message)
        if not passed:
            all_passed = False

    total = len(pol11_fixtures) + len(pol12_fixtures)
    failed = sum(1 for r in results if r.startswith("  ERR"))
    skipped = sum(1 for r in results if r.startswith("  SKIP"))
    passed_count = total - failed - skipped

    print(f"\nResults: {passed_count}/{total} passed, {failed} failed, {skipped} skipped")

    if all_passed:
        print("All fixture tests PASS.")
        sys.exit(0)
    else:
        print("FAIL: one or more fixture tests produced unexpected outcomes.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
