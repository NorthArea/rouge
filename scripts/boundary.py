#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Repository boundary, one logic for both doors: scan agent-facing files (--scan) or vet one
tool call as a PreToolUse hook (--hook)."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import sys
from pathlib import Path


ESCAPE = re.compile(
    r"(?<![\w-])(?:/tmp(?:/|\b)|/private/(?:tmp|var)\b|/var/folders\b|/Users/|/home/[A-Za-z0-9_.-]|~/|\$(?:HOME|\{HOME\}))"
)
URL = re.compile(r"\b[a-z][a-z0-9+.-]*://\S+", re.IGNORECASE)
DESTRUCTIVE = (
    (re.compile(r"\bsudo\b"), "sudo is forbidden by the repository contract"),
    (re.compile(r"\bgit\s+reset\s+(?:\S+\s+)*--hard\b"), "git reset --hard needs an explicit owner decision"),
    (re.compile(r"\bgit\s+clean\b"), "git clean needs an explicit owner decision"),
    (re.compile(r"\bgit\s+push\s+(?:\S+\s+)*(?:--force\b|-f\b)"), "force-push needs an explicit owner decision"),
    # `git tag` both writes and reads: bare `git tag`, `-l`, `-n`, `--points-at` and friends only list
    # what already exists. Deny the forms that bring a tag into being or take one away — the writing
    # flags, and a first argument that is a tag name rather than an option.
    (
        re.compile(r"\bgit\s+tag\s+(?:-[asudfm]\b|--(?:annotate|sign|local-user|delete|force|message|file)\b|[^-\s|&;][^\s|&;]*)"),
        "creating tags needs an explicit owner decision",
    ),
)
PATH_FIELDS = ("file_path", "notebook_path", "path")

# Standard devices are shell plumbing, not a way out of the repository.
DEVICES = {"/dev/null", "/dev/stdout", "/dev/stderr", "/dev/stdin", "/dev/tty", "/dev/zero"}

# This file describes host paths in order to reject them.
EXEMPT = {"boundary.py"}


def project_root() -> Path:
    root = os.environ.get("CLAUDE_PROJECT_DIR")
    return Path(root).resolve() if root else Path(__file__).resolve().parents[1]


# --- scan mode: reject host paths in files that agents execute or follow as instructions ---


def targets(root: Path) -> list[Path]:
    # .claude/settings*.json are deliberately absent: a permission policy names host paths in order
    # to deny or grant them, which is the opposite of an escape.
    names = [root / "CLAUDE.md", root / "README.md", root / "Makefile"]
    patterns = (
        "workflow/**/*.md",
        "prompts/**/*.md",
        "scripts/**/*",
        ".claude/**/*.md",
    )
    paths = names[:]
    for pattern in patterns:
        paths.extend(root.glob(pattern))
    return [path for path in dict.fromkeys(paths) if path.is_file()]


def lines_to_scan(path: Path) -> list[tuple[int, str]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError):
        return []

    if path.suffix != ".md":
        return list(enumerate(lines, start=1))

    inside_fence = False
    result: list[tuple[int, str]] = []
    for number, line in enumerate(lines, start=1):
        if line.lstrip().startswith("```"):
            inside_fence = not inside_fence
        elif inside_fence:
            result.append((number, line))
    return result


def audit(root: Path) -> int:
    violations: list[tuple[Path, int, str]] = []
    for path in targets(root):
        if path.name in EXEMPT:
            continue
        for number, line in lines_to_scan(path):
            match = ESCAPE.search(URL.sub(" ", line))
            if match:
                violations.append((path.relative_to(root), number, line.strip()))

    if violations:
        print("boundary-check: FAILED", file=sys.stderr)
        for path, number, line in violations:
            print(f"  {path}:{number}: {line}", file=sys.stderr)
        return 1

    print(f"boundary-check: OK ({len(targets(root))} agent-facing files scanned)")
    return 0


# --- hook mode: keep tool calls inside the repository and off destructive git commands ---


def extra_roots(root: Path) -> tuple[Path, ...]:
    """Directories the owner granted via permissions.additionalDirectories.

    settings.local.json is read too: personal grants (reference repositories on this machine) belong
    there, so the tracked settings stay portable.
    """
    roots: list[Path] = []
    for name in (".claude/settings.json", ".claude/settings.local.json"):
        try:
            settings = json.loads((root / name).read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        for item in settings.get("permissions", {}).get("additionalDirectories", []):
            if isinstance(item, str):
                roots.append(Path(item).expanduser().resolve())
    return tuple(roots)


def inside(roots: tuple[Path, ...], candidate: str) -> bool:
    path = Path(candidate).expanduser()
    if not path.is_absolute():
        path = roots[0] / path
    try:
        resolved = path.resolve()
    except OSError:
        return False
    return any(resolved == root or root in resolved.parents for root in roots)


def check_paths(roots: tuple[Path, ...], tool_input: dict) -> str | None:
    for field in PATH_FIELDS:
        value = tool_input.get(field)
        if isinstance(value, str) and value and not inside(roots, value):
            return f"{field} {value!r} is outside the repository root {roots[0]}"
    return None


def looks_like_path(token: str) -> bool:
    """An absolute token counts as a path only if it names something real on the host.

    Host paths under home or temporary directories are caught by ESCAPE regardless. This keeps
    regex patterns, flags and sed expressions that merely start with a slash from reading as escapes.
    """
    if not token.startswith(("/", "~/")) or token in DEVICES:
        return False
    path = Path(token).expanduser()
    parent = path.parent
    return path.exists() or (parent.exists() and parent != Path(path.anchor))


def check_command(roots: tuple[Path, ...], command: str) -> str | None:
    for pattern, reason in DESTRUCTIVE:
        if pattern.search(command):
            return reason

    # Every granted root lives under a host path, so remove them before looking for escapes. The
    # placeholder has to be a word, not a space: a repository-local `tmp/` named absolutely would
    # otherwise be left as a bare ` /tmp` and read as an escape. ESCAPE refuses to match after a
    # word character, so the remainder of a granted path stays anchored to the root it came from.
    stripped = URL.sub(" ", command)
    for root in roots:
        stripped = stripped.replace(str(root), "REPOROOT")
    if ESCAPE.search(stripped):
        return "the command references a path outside the repository; use repository-local tmp/ instead"

    try:
        tokens = shlex.split(command)
    except ValueError:
        return None
    for token in tokens:
        if looks_like_path(token) and not inside(roots, token):
            return f"{token!r} is outside the repository root {roots[0]}"
    return None


def deny(reason: str) -> int:
    payload = {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": (
                f"Repository boundary: {reason}. See the Repository Boundary section of CLAUDE.md. "
                "Report this as a stop condition instead of working around it."
            ),
        }
    }
    print(json.dumps(payload))
    return 0


def hook() -> int:
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    root = project_root()
    roots = (root, *extra_roots(root))
    tool_input = event.get("tool_input") or {}
    if not isinstance(tool_input, dict):
        return 0

    reason = check_paths(roots, tool_input)
    if reason is None and event.get("tool_name") == "Bash":
        command = tool_input.get("command")
        if isinstance(command, str):
            reason = check_command(roots, command)

    return deny(reason) if reason else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--scan", action="store_true", help="scan agent-facing files for host paths")
    mode.add_argument("--hook", action="store_true", help="vet one tool call from stdin (PreToolUse)")
    parser.add_argument("--root", type=Path, default=None)
    args = parser.parse_args()

    if args.hook:
        return hook()
    return audit((args.root or project_root()).resolve())


if __name__ == "__main__":
    raise SystemExit(main())
