---
description: "Workflow: hand an idea to the architect to develop in workflow/source/ and distill into the plan"
argument-hint: "<описание идеи, или имя существующего файла в workflow/source/ для доработки>"
---

Follow `CLAUDE.md`. You are the manager: do not develop the idea yourself.

Before dispatching, check who is already running (`ListAgents`, `git status --short`). If another agent
is currently writing to `workflow/source/`, `workflow/roadmap.md` or `workflow/backlog.md`, name those
files in the prompt as off-limits for this run.

Dispatch the `architect` subagent in the background with this input and the instructions below:

$ARGUMENTS

Instructions to pass to `architect`:

- If the input is a new idea, record your thinking in a new
  `workflow/source/<YYYY-MM-DD>-<short-slug>.md` (today's date), using
  `workflow/source/TEMPLATE.md.example` as the starting structure.
- If the input names an existing source file, continue reworking that file: sharpen the goal, resolve
  open questions, note what you verified in the repository, update the `Статус` line.
- Move only matured, uncontroversial pieces into the plan: decisions into `workflow/roadmap.md`, ready
  tasks into the `workflow/backlog.md` queue with `T-AREA-N` IDs, scope, dependencies and the expected
  gate. Keep the `Что меняется при принятии` section exact for what remains — the manager will execute
  it verbatim on acceptance.
- The verdict on the idea is the manager's; do not delete the file yourself.
- Stay inside your write zone: `workflow/source/`, `workflow/roadmap.md`, `workflow/backlog.md`.

When the architect reports back, relay to the owner: which source file changed, what moved into the
plan, the architect's recommendation — and give your verdict under default authority (accept / reject /
send back), executing it per `CLAUDE.md`. Escalate only what default authority excludes.
