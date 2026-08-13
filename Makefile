SHELL := /bin/sh

.DEFAULT_GOAL := help

FRAMEWORK_MAKEFILE := $(firstword $(MAKEFILE_LIST))
PROJECT_ROOT := $(abspath $(dir $(FRAMEWORK_MAKEFILE)))
PYTHON ?= uv run

# Override these variables in a consuming project's Makefile or on the command line. Each shipped
# default below is also the framework's own self-test command — a consuming project that leaves it
# unset gets a gate that runs (and passes) without ever touching the project's own tests, so `test`,
# `verify` and `format-check` warn on stderr, without failing, whenever a command still equals its
# shipped default.
UNIT_TEST_DEFAULT := $(PYTHON) "$(PROJECT_ROOT)/scripts/framework_test.py"
FULL_VERIFY_DEFAULT := $(PYTHON) "$(PROJECT_ROOT)/scripts/framework_test.py"
FORMAT_CHECK_DEFAULT := git -C "$(PROJECT_ROOT)" diff --check

CARGO_MANIFEST := "$(PROJECT_ROOT)/Cargo.toml"

# The repository holds one crate per game, so every command covers the whole workspace: a gate that
# checked only the default members would go quietly green while another game was broken.
UNIT_TEST_COMMAND ?= cargo test --manifest-path $(CARGO_MANIFEST) --workspace
TARGETED_TEST_COMMAND ?= cargo test --manifest-path $(CARGO_MANIFEST) --workspace
# The project contract requires four commands to pass with no warnings left behind, so the full gate
# runs all four rather than duplicating UNIT_TEST_COMMAND. Keeping clippy and build here means they
# are part of the ordinary gate, not a special gate repeated on every queue position.
FULL_VERIFY_COMMAND ?= cargo fmt --manifest-path $(CARGO_MANIFEST) --all -- --check && \
	cargo clippy --manifest-path $(CARGO_MANIFEST) --workspace --all-targets --all-features -- -D warnings && \
	cargo test --manifest-path $(CARGO_MANIFEST) --workspace && \
	cargo build --manifest-path $(CARGO_MANIFEST) --workspace
FORMAT_CHECK_COMMAND ?= cargo fmt --manifest-path $(CARGO_MANIFEST) --all -- --check
FORMAT_COMMAND ?= cargo fmt --manifest-path $(CARGO_MANIFEST) --all
BOUNDARY_CHECK_COMMAND ?= $(PYTHON) "$(PROJECT_ROOT)/scripts/boundary.py" --scan --root "$(PROJECT_ROOT)"
CONTEXT_CHECK_COMMAND ?= $(PYTHON) "$(PROJECT_ROOT)/scripts/context.py" --check --root "$(PROJECT_ROOT)"

.PHONY: help test test-target verify check format-check format boundary-check context-check diff-check \
	framework-test hook-session hook-boundary hook-post-bash hook-stop statusline

help: ## Show the canonical command interface
	@awk 'BEGIN {FS = ":.*##"; print "Usage: make <target>\n"} /^[a-zA-Z0-9_-]+:.*##/ {printf "  %-16s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test: ## Run the project's unit tests
	@test -n "$(UNIT_TEST_COMMAND)" || (printf '%s\n' 'UNIT_TEST_COMMAND is required' >&2; exit 1)
	@test "$(UNIT_TEST_COMMAND)" != "$(UNIT_TEST_DEFAULT)" || \
		printf '%s\n' 'test: UNIT_TEST_COMMAND is still the framework default — override it in the project Makefile' >&2
	$(UNIT_TEST_COMMAND)

test-target: ## Run one targeted test, for example: make test-target TEST=SomeTest
	@test -n "$(TEST)" || (printf '%s\n' 'TEST is required' >&2; exit 1)
	@test -n "$(TARGETED_TEST_COMMAND)" || (printf '%s\n' 'TARGETED_TEST_COMMAND is required' >&2; exit 1)
	$(TARGETED_TEST_COMMAND) "$(TEST)"

verify: ## Run the complete project verification suite
	@test -n "$(FULL_VERIFY_COMMAND)" || (printf '%s\n' 'FULL_VERIFY_COMMAND is required' >&2; exit 1)
	@test "$(FULL_VERIFY_COMMAND)" != "$(FULL_VERIFY_DEFAULT)" || \
		printf '%s\n' 'verify: FULL_VERIFY_COMMAND is still the framework default — override it in the project Makefile' >&2
	$(FULL_VERIFY_COMMAND)

format-check: ## Check formatting
	@test -n "$(FORMAT_CHECK_COMMAND)" || (printf '%s\n' 'FORMAT_CHECK_COMMAND is required' >&2; exit 1)
	@test "$(FORMAT_CHECK_COMMAND)" != "$(FORMAT_CHECK_DEFAULT)" || \
		printf '%s\n' 'format-check: FORMAT_CHECK_COMMAND is still the framework default — override it in the project Makefile' >&2
	$(FORMAT_CHECK_COMMAND)

format: ## Apply the project's formatter, so a red format-check has a fix behind the same interface
	@test -n "$(FORMAT_COMMAND)" || (printf '%s\n' 'FORMAT_COMMAND is required' >&2; exit 1)
	$(FORMAT_COMMAND)

boundary-check: ## Check that agent-facing files stay inside the repository
	@test -n "$(BOUNDARY_CHECK_COMMAND)" || (printf '%s\n' 'BOUNDARY_CHECK_COMMAND is required' >&2; exit 1)
	$(BOUNDARY_CHECK_COMMAND)

context-check: ## Check that recovery context is internally consistent
	@test -n "$(CONTEXT_CHECK_COMMAND)" || (printf '%s\n' 'CONTEXT_CHECK_COMMAND is required' >&2; exit 1)
	$(CONTEXT_CHECK_COMMAND)

diff-check: ## Check whitespace errors in the Git diff
	git diff --check

framework-test: ## Run the framework's own self-test (hooks, boundary, context, agent wiring)
	$(PYTHON) "$(PROJECT_ROOT)/scripts/framework_test.py"

check: ## Run the complete local verification gate
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" format-check
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" boundary-check
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" context-check
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" diff-check
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" framework-test
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" test
	$(MAKE) --no-print-directory -f "$(FRAMEWORK_MAKEFILE)" verify

# Claude Code adapters: .claude/settings.json points its hooks and status line at these targets,
# so the enforcement logic lives behind the same interface as every other repository command.

hook-session: ## SessionStart hook: inject the workflow recovery pointer
	@$(PYTHON) "$(PROJECT_ROOT)/scripts/session.py" --hook

hook-boundary: ## PreToolUse hook: deny tool calls that leave the repository
	@$(PYTHON) "$(PROJECT_ROOT)/scripts/boundary.py" --hook

hook-post-bash: ## PostToolUse hook: context-check after a git commit
	@$(PYTHON) "$(PROJECT_ROOT)/scripts/context.py" --post-commit-hook

hook-stop: ## Stop hook: refuse to end a session with inconsistent workflow context
	@$(PYTHON) "$(PROJECT_ROOT)/scripts/context.py" --stop-hook

statusline: ## Status line: repo · current task · next task · dirty files
	@$(PYTHON) "$(PROJECT_ROOT)/scripts/session.py" --statusline
