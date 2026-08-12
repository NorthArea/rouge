#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Workflow recovery context, one logic for every door: consistency check (--check), the Stop gate
(--stop-hook) and the after-commit gate (--post-commit-hook)."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import re
import sys
from pathlib import Path


TASK_ID = re.compile(r"^T-[A-Z]+-\d+$")
REQUIRED = (
    "CLAUDE.md",
    "Makefile",
    ".claude/settings.json",
    "workflow/progress.md",
    "workflow/backlog.md",
    "workflow/roadmap.md",
    "workflow/tasks/TEMPLATE.md.example",
)
POINTER_FIELDS = (
    "PROJECT_STATUS",
    "CURRENT_TASK",
    "NEXT_BACKLOG_ID",
    "LAST_COMPLETED_TASK",
    "LAST_GREEN_COMMAND",
    "LAST_CHECKPOINT",
    "NEXT_ACTION",
)


def project_root() -> Path:
    root = os.environ.get("CLAUDE_PROJECT_DIR")
    return Path(root).resolve() if root else Path(__file__).resolve().parents[1]


def field(text: str, name: str) -> str | None:
    match = re.search(rf"^{re.escape(name)}:\s*(.+)$", text, re.MULTILINE)
    return match.group(1).strip() if match else None


def queue_ids(text: str) -> set[str]:
    return set(re.findall(r"`(T-[A-Z]+-\d+)`", text))


def registry_rows(text: str) -> list[tuple[str, str]]:
    rows = []
    for line in text.splitlines():
        if not line.startswith("|") or line.startswith("| Task") or line.startswith("|---"):
            continue
        columns = [column.strip() for column in line.strip("|").split("|")]
        if len(columns) >= 2 and TASK_ID.match(columns[0]):
            rows.append((columns[0], columns[1]))
    return rows


def hook_errors(root: Path) -> list[str]:
    """Every make target the Claude Code settings reference must exist in the Makefile."""
    try:
        settings = json.loads((root / ".claude/settings.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f".claude/settings.json is unreadable: {error}"]

    try:
        makefile = (root / "Makefile").read_text(encoding="utf-8")
    except OSError as error:
        return [f"Makefile is unreadable: {error}"]
    defined = set(re.findall(r"^([a-zA-Z0-9_-]+):", makefile, re.MULTILINE))

    commands = [
        hook.get("command", "")
        for matchers in (settings.get("hooks") or {}).values()
        for matcher in matchers
        for hook in matcher.get("hooks", [])
    ]
    status = (settings.get("statusLine") or {}).get("command", "")
    if status:
        commands.append(status)

    errors: list[str] = []
    for command in commands:
        for name in re.findall(r"\b(hook-[a-z-]+|statusline)\b", command):
            if name not in defined:
                errors.append(f"settings reference a missing make target: {name}")
    return errors


def check(root: Path) -> int:
    missing = [path for path in REQUIRED if not (root / path).is_file()]
    if missing:
        print("context-check: missing required files: " + ", ".join(missing), file=sys.stderr)
        return 1

    progress = (root / "workflow/progress.md").read_text(encoding="utf-8")
    backlog = (root / "workflow/backlog.md").read_text(encoding="utf-8")
    errors: list[str] = []

    values = {name: field(progress, name) for name in POINTER_FIELDS}
    errors.extend(name for name, value in values.items() if value is None)
    if errors:
        print("context-check: missing recovery fields: " + ", ".join(errors), file=sys.stderr)
        return 1

    current = values["CURRENT_TASK"]
    next_task = values["NEXT_BACKLOG_ID"]
    if current != "none" and not TASK_ID.match(current or ""):
        errors.append("CURRENT_TASK must be none or a task ID")
    if next_task and next_task.startswith("<"):
        next_task = None
    if next_task == "none" and queue_ids(backlog):
        errors.append("NEXT_BACKLOG_ID cannot be none when the backlog is not empty")
    elif next_task and next_task != "none" and next_task not in queue_ids(backlog):
        errors.append(f"NEXT_BACKLOG_ID {next_task} is absent from the backlog queue")
    if current and current != "none":
        if not any(task == current and status == "IN_PROGRESS" for task, status in registry_rows(progress)):
            errors.append(f"CURRENT_TASK {current} has no IN_PROGRESS registry row")
        if not list((root / "workflow/tasks").glob(f"{current}-*.md")):
            errors.append(f"CURRENT_TASK {current} has no task card")

    invalid_statuses = [
        f"{task}={status}"
        for task, status in registry_rows(progress)
        if status not in {"IN_PROGRESS", "DONE", "BLOCKED"}
    ]
    errors.extend(f"invalid registry status {item}" for item in invalid_statuses)
    errors.extend(hook_errors(root))

    if errors:
        print("context-check: FAILED", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1

    print("context-check: OK")
    return 0


def quiet_check(root: Path) -> tuple[int, str]:
    output = io.StringIO()
    with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
        code = check(root)
    return code, output.getvalue().strip()


def stop_hook() -> int:
    """Refuse to end a session whose recovery context is inconsistent."""
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        event = {}

    if event.get("stop_hook_active"):
        return 0

    code, report = quiet_check(project_root())
    if code == 0:
        return 0

    print(
        "The workflow recovery context is inconsistent, so this session cannot be handed over yet:\n"
        f"{report}\n"
        "Fix workflow/progress.md, workflow/backlog.md and workflow/tasks/ per CLAUDE.md, or record the "
        "blocker as BLOCKED with its four facts. Do not weaken the check to make it pass.",
        file=sys.stderr,
    )
    return 2


def post_commit_hook() -> int:
    """After a git commit, make sure the commit did not leave the registry inconsistent."""
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    command = (event.get("tool_input") or {}).get("command", "")
    if not isinstance(command, str) or "git commit" not in command:
        return 0

    code, report = quiet_check(project_root())
    if code == 0:
        return 0

    print(f"context-check failed after this commit:\n{report}", file=sys.stderr)
    return 2


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="validate the workflow recovery context")
    mode.add_argument("--stop-hook", action="store_true", help="Stop hook: block ending on inconsistency")
    mode.add_argument("--post-commit-hook", action="store_true", help="PostToolUse hook: check after git commit")
    parser.add_argument("--root", type=Path, default=None)
    args = parser.parse_args()

    if args.stop_hook:
        return stop_hook()
    if args.post_commit_hook:
        return post_commit_hook()
    return check((args.root or project_root()).resolve())


if __name__ == "__main__":
    raise SystemExit(main())
