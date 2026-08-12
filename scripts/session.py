#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Session surface: SessionStart context injection (--hook) and the status line (--statusline)."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path


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


def field(text: str, name: str) -> str:
    match = re.search(rf"^{re.escape(name)}:\s*(.+)$", text, re.MULTILINE)
    return match.group(1).strip() if match else "missing"


def roadmap_direction(root: Path) -> str | None:
    """First meaningful line of the roadmap's direction section, or None while it is still empty."""
    path = root / "workflow/roadmap.md"
    if not path.is_file():
        return None
    match = re.search(r"^## Направление\n(.*?)(?=^## |\Z)", path.read_text(encoding="utf-8"), re.M | re.S)
    body = match.group(1).strip() if match else ""
    if not body or "Заполните этот раздел" in body:
        return None
    return next((line.strip() for line in body.splitlines() if line.strip()), None)


def queue_size(root: Path) -> int:
    path = root / "workflow/backlog.md"
    if not path.is_file():
        return 0
    return len(set(re.findall(r"`(T-[A-Z]+-\d+)`", path.read_text(encoding="utf-8"))))


def open_ideas(root: Path) -> list[str]:
    return sorted(path.name for path in (root / "workflow/source").glob("*.md"))


def summary(root: Path) -> str:
    progress = root / "workflow/progress.md"
    if not progress.is_file():
        return "workflow/progress.md is missing. Restore it before taking any task."

    text = progress.read_text(encoding="utf-8")
    lines = ["Workflow recovery pointer (from workflow/progress.md):"]
    lines.extend(f"- {name}: {field(text, name)}" for name in POINTER_FIELDS)

    cards = sorted(path.name for path in (root / "workflow/tasks").glob("*.md"))
    lines.append(f"- Active task cards: {', '.join(cards) if cards else 'none'}")

    direction = roadmap_direction(root)
    queue = queue_size(root)
    ideas = open_ideas(root)
    lines.append(f"- Roadmap direction: {direction if direction else 'empty'}")
    lines.append(f"- Backlog queue: {queue} open task(s)")
    lines.append(f"- Open ideas in workflow/source/: {', '.join(ideas) if ideas else 'none'}")

    lines.append(
        "Follow CLAUDE.md. This pointer is a starting point, not a substitute for reading "
        "workflow/progress.md, the active card and workflow/backlog.md."
    )
    lines.append(
        "First-reply greeting: you are the manager of this repository. Open your first reply of the "
        "session with a short greeting in the owner's language: (1) who you are and what you take — an "
        "idea to develop (plain words or /workflow:idea), work to dispatch (/workflow:build), state "
        "(/workflow:status); (2) where the last session ended, from the pointer above; (3) the roadmap "
        "direction and queue size; (4) if the roadmap is empty and the queue is empty, invite the owner "
        "to share their ideas. A few lines, no headings — then answer what the owner actually said."
    )
    return "\n".join(lines)


def statusline(root: Path) -> str:
    progress = root / "workflow/progress.md"
    text = progress.read_text(encoding="utf-8") if progress.is_file() else ""
    task = field(text, "CURRENT_TASK") if text else "?"
    nxt = field(text, "NEXT_BACKLOG_ID") if text else "?"
    try:
        status = subprocess.run(
            ["git", "-C", str(root), "status", "--short"],
            capture_output=True, text=True, timeout=10, check=False,
        ).stdout
        dirty = str(len(status.splitlines()))
    except (OSError, subprocess.TimeoutExpired):
        dirty = "?"
    return f"{root.name} · task:{task} · next:{nxt} · dirty:{dirty}"


def hook(root: Path) -> int:
    try:
        json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        pass

    payload = {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": summary(root),
        }
    }
    print(json.dumps(payload))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--hook", action="store_true", help="SessionStart hook: emit the recovery pointer")
    mode.add_argument("--statusline", action="store_true", help="print the one-line status")
    args = parser.parse_args()

    root = project_root()
    if args.statusline:
        print(statusline(root))
        return 0
    return hook(root)


if __name__ == "__main__":
    raise SystemExit(main())
