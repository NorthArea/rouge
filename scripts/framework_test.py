#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Self-tests for the portable agent framework."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import re
import sys
import unittest
from pathlib import Path

# The framework runs everything through uv; stray bytecode caches are the only build artifact
# it could produce, so forbid them at the importer.
sys.dont_write_bytecode = True

import boundary
import context
import session


ROOT = Path(__file__).resolve().parents[1]

COMMAND_DIR = ROOT / ".claude/commands/workflow"
NAMESPACE = "workflow"

COMMANDS = (
    "status",
    "idea",
    "build",
)

AGENTS = ("developer", "architect")

# The framework is portable: agent-facing files must not name a language, framework, tool or product.
DOMAIN_TERMS = (
    "java", "spring", "maven", "gradle", "hibernate", "jpa", "postgres", "pgvector", "flyway",
    "testcontainers", "surefire", "failsafe", "jacoco", "spotless", "archunit", "junit", "assertj",
    "docker", "compose", "kubernetes", "npm", "node", "django", "rails", "kotlin", "golang",
)

# Built-in Claude Code commands and skills a project command must never shadow.
RESERVED = {
    "status", "resume", "run", "init", "review", "compact", "clear", "config", "context", "cost",
    "doctor", "export", "help", "hooks", "mcp", "memory", "model", "permissions", "todos", "usage",
    "agents", "rewind", "plan", "loop", "schedule", "simplify", "code-review", "security-review",
}


class FrameworkTests(unittest.TestCase):
    def test_required_files_exist(self) -> None:
        required = (
            "CLAUDE.md",
            "Makefile",
            ".claude/README.md",
            ".claude/settings.json",
            "scripts/boundary.py",
            "scripts/context.py",
            "scripts/session.py",
            "workflow/backlog.md",
            "workflow/progress.md",
            "workflow/roadmap.md",
            "workflow/tasks/TEMPLATE.md.example",
        )
        missing = [path for path in required if not (ROOT / path).is_file()]
        self.assertEqual(missing, [])

    def test_boundary_check(self) -> None:
        output = io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            result = boundary.audit(ROOT)
        self.assertEqual(result, 0, output.getvalue())

    def test_context_check(self) -> None:
        output = io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            result = context.check(ROOT)
        self.assertEqual(result, 0, output.getvalue())


class ClaudeIntegrationTests(unittest.TestCase):
    def test_every_workflow_command_exists_with_a_description(self) -> None:
        for name in COMMANDS:
            path = COMMAND_DIR / f"{name}.md"
            with self.subTest(command=name):
                self.assertTrue(path.is_file(), f"missing slash command /{NAMESPACE}:{name}")
                text = path.read_text(encoding="utf-8")
                self.assertTrue(text.startswith("---\n"), f"/{NAMESPACE}:{name} has no frontmatter")
                self.assertIn("description:", text.split("---")[1])

    def test_claude_md_documents_every_command(self) -> None:
        contract = (ROOT / "CLAUDE.md").read_text(encoding="utf-8")
        undocumented = [name for name in COMMANDS if f"`/{NAMESPACE}:{name}`" not in contract]
        self.assertEqual(undocumented, [])

    def test_commands_stay_inside_the_workflow_namespace(self) -> None:
        """A command outside .claude/commands/workflow/ would be addressed bare and could collide."""
        unnamespaced = sorted(path.name for path in (ROOT / ".claude/commands").glob("*.md"))
        self.assertEqual(unnamespaced, [], "commands must live in .claude/commands/workflow/")
        present = sorted(path.stem for path in COMMAND_DIR.glob("*.md"))
        self.assertEqual(present, sorted(COMMANDS))
        for name in present:
            with self.subTest(command=name):
                self.assertNotIn(f"{NAMESPACE}:{name}", RESERVED)

    def test_every_agent_declares_a_name_and_description(self) -> None:
        for name in AGENTS:
            path = ROOT / ".claude/agents" / f"{name}.md"
            with self.subTest(agent=name):
                self.assertTrue(path.is_file(), f"missing subagent {name}")
                frontmatter = path.read_text(encoding="utf-8").split("---")[1]
                self.assertIn(f"name: {name}", frontmatter)
                self.assertIn("description:", frontmatter)

    def test_agent_files_stay_project_neutral(self) -> None:
        """A stack-specific agent stops being portable; that knowledge belongs in the project."""
        for path in sorted((ROOT / ".claude/agents").glob("*.md")):
            words = set(re.findall(r"[a-z]+", path.read_text(encoding="utf-8").lower()))
            leaked = sorted(words.intersection(DOMAIN_TERMS))
            with self.subTest(agent=path.name):
                self.assertEqual(leaked, [], f"{path.name} names a specific stack: {leaked}")

    def test_claude_md_documents_every_agent(self) -> None:
        contract = (ROOT / "CLAUDE.md").read_text(encoding="utf-8")
        undocumented = [name for name in AGENTS if f"`{name}`" not in contract]
        self.assertEqual(undocumented, [])

    def test_settings_reference_only_existing_make_targets(self) -> None:
        """Hooks and the status line are thin make adapters; context.hook_errors verifies the wiring."""
        self.assertEqual(context.hook_errors(ROOT), [])
        settings = json.loads((ROOT / ".claude/settings.json").read_text(encoding="utf-8"))
        self.assertEqual(
            sorted(settings.get("hooks", {})),
            ["PostToolUse", "PreToolUse", "SessionStart", "Stop"],
        )
        for matchers in settings["hooks"].values():
            for matcher in matchers:
                for hook in matcher["hooks"]:
                    with self.subTest(hook=hook["command"]):
                        self.assertIn("make --no-print-directory", hook["command"])


class BoundaryHookTests(unittest.TestCase):
    ROOTS = (ROOT,)

    def test_repository_paths_are_allowed(self) -> None:
        self.assertIsNone(boundary.check_paths(self.ROOTS, {"file_path": str(ROOT / "workflow/progress.md")}))
        self.assertIsNone(boundary.check_paths(self.ROOTS, {"file_path": "scripts/context.py"}))
        self.assertIsNone(boundary.check_command(self.ROOTS, "make check"))

    def test_absolute_paths_inside_the_repository_are_allowed(self) -> None:
        """The repository root itself lives under a host path; that must not read as an escape."""
        self.assertIsNone(boundary.check_command(self.ROOTS, f"cd {ROOT} && make check"))
        self.assertIsNone(boundary.check_command(self.ROOTS, f"grep -rn pattern {ROOT}/scripts"))

    def test_owner_granted_directories_are_allowed(self) -> None:
        """permissions.additionalDirectories widens the boundary declaratively, per owner decision."""
        neighbour = ROOT.parent
        roots = (ROOT, neighbour)
        self.assertIsNotNone(boundary.check_paths(self.ROOTS, {"file_path": str(neighbour / "x")}))
        self.assertIsNone(boundary.check_paths(roots, {"file_path": str(neighbour / "x")}))
        self.assertIsNone(boundary.check_command(roots, f"ls {neighbour}"))

    def test_paths_outside_the_repository_are_rejected(self) -> None:
        outside = str(Path(ROOT.anchor) / "etc" / "hosts")
        self.assertIsNotNone(boundary.check_paths(self.ROOTS, {"file_path": outside}))
        self.assertIsNotNone(boundary.check_command(self.ROOTS, f"cat {outside}"))

    def test_slash_prefixed_arguments_are_not_treated_as_paths(self) -> None:
        """Regex patterns and flags starting with a slash are arguments, not filesystem paths."""
        self.assertIsNone(boundary.check_command(self.ROOTS, r"grep -n '/idea\|/plan' README.md"))
        self.assertIsNone(boundary.check_command(self.ROOTS, "sed 's|/old|/new|' workflow/backlog.md"))

    def test_standard_devices_are_allowed(self) -> None:
        """Shell staples like /dev/null are not a way out of the repository."""
        self.assertIsNone(boundary.check_command(self.ROOTS, "make check > /dev/null 2>&1"))
        self.assertIsNone(boundary.check_command(self.ROOTS, "make test 2>/dev/stderr"))

    def test_absolute_paths_inside_the_repository_are_allowed(self) -> None:
        """Naming a repository-local target by its absolute path is not an escape.

        Removing a granted root leaves the rest of the path behind, and that remainder must not read
        as a host path on its own: a repository-local temporary directory turns into a bare host one
        the moment the root in front of it is stripped away.
        """
        self.assertIsNone(boundary.check_command(self.ROOTS, f"make test > {ROOT / 'tmp' / 'red.log'}"))
        self.assertIsNone(boundary.check_command(self.ROOTS, f"cat {ROOT / 'workflow' / 'backlog.md'}"))

    def test_destructive_git_commands_are_rejected(self) -> None:
        for command in (
            "sudo make check",
            "git reset --hard HEAD~1",
            "git clean -fd",
            "git push --force origin master",
            "git tag v1.0.0",
            "git tag -a pong-1.0 cba2501 -m 'the accepted game'",
            "git tag -d pong-1.0",
        ):
            with self.subTest(command=command):
                self.assertIsNotNone(boundary.check_command(self.ROOTS, command))

    def test_reading_tags_is_not_creating_them(self) -> None:
        """The owner decides when a tag appears; looking at the ones that exist decides nothing."""
        for command in (
            "git tag",
            "git tag -l",
            "git tag --list 'pong-*'",
            "git tag -n5",
            "git tag --points-at HEAD",
            "git tag --sort=-creatordate",
        ):
            with self.subTest(command=command):
                self.assertIsNone(boundary.check_command(self.ROOTS, command))

    def test_urls_are_not_mistaken_for_host_paths(self) -> None:
        self.assertIsNone(boundary.check_command(self.ROOTS, "git remote -v"))
        self.assertIsNone(boundary.check_command(self.ROOTS, "echo https://example.com/home/x"))


class ProjectStatusTests(unittest.TestCase):
    """PROJECT_STATUS must describe the pointer it sits above, not drift away from it."""

    def test_the_working_statuses_are_accepted(self) -> None:
        self.assertEqual(context.project_status_errors("IN_PROGRESS", "T-AREA-1", "T-AREA-2"), [])
        self.assertEqual(context.project_status_errors("BLOCKED", "T-AREA-1", "T-AREA-2"), [])

    def test_an_unknown_status_is_rejected(self) -> None:
        self.assertNotEqual(context.project_status_errors("FINISHED", "none", "none"), [])

    def test_complete_requires_an_empty_queue_and_no_active_task(self) -> None:
        """COMPLETE is a claim about the whole repository, so it has to be earned."""
        self.assertEqual(context.project_status_errors("COMPLETE", "none", "none"), [])
        self.assertNotEqual(context.project_status_errors("COMPLETE", "T-AREA-1", "none"), [])
        self.assertNotEqual(context.project_status_errors("COMPLETE", "none", "T-AREA-2"), [])


class SessionContextTests(unittest.TestCase):
    def test_summary_reports_every_pointer_field(self) -> None:
        summary = session.summary(ROOT)
        for name in session.POINTER_FIELDS:
            with self.subTest(field=name):
                self.assertIn(f"- {name}: ", summary)
                self.assertNotIn(f"- {name}: missing", summary)

    def test_summary_briefs_the_manager_for_the_greeting(self) -> None:
        """The SessionStart context carries plan state and the first-reply greeting instruction."""
        summary = session.summary(ROOT)
        self.assertIn("- Roadmap direction: ", summary)
        self.assertIn("- Backlog queue: ", summary)
        self.assertIn("- Open ideas in workflow/source/: ", summary)
        self.assertIn("First-reply greeting:", summary)
        self.assertIn("invite the owner", summary)

    def test_statusline_is_one_line_with_task_and_dirt(self) -> None:
        line = session.statusline(ROOT)
        self.assertNotIn("\n", line)
        self.assertIn(f"{ROOT.name} · task:", line)
        self.assertIn("· dirty:", line)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("test_name", nargs="?", help="optional unittest name for make test-target")
    args = parser.parse_args()

    if args.test_name:
        suite = unittest.defaultTestLoader.loadTestsFromName(args.test_name, sys.modules[__name__])
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        return 0 if result.wasSuccessful() else 1

    return unittest.main(module=__name__, argv=[sys.argv[0]], verbosity=2, exit=False).result.wasSuccessful() is False


if __name__ == "__main__":
    raise SystemExit(main())
