---
description: "Workflow: dispatch the developer — next task by default, a given TASK_ID, or `all` for the whole queue"
argument-hint: "[TASK_ID | all] — пусто: продолжить текущую или взять следующую"
---

Follow `CLAUDE.md`. You are the manager: do not implement anything yourself — dispatch the `developer`
subagent in the background and relay its reports.

Before dispatching, check `ListAgents` and `git status --short`: if a developer is already running, do
not start a second one — report what is in flight instead.

Argument:

$ARGUMENTS

Pick the mode from the argument and the recovery pointer in `workflow/progress.md`:

- **No argument, `CURRENT_TASK` is set** — resume: the developer rebuilds state from `NEXT_ACTION` and
  the card — not from git history — and continues the lifecycle from that exact point. It must not
  switch to a different task.
- **No argument, `CURRENT_TASK: none`** — start next: the developer takes `NEXT_BACKLOG_ID` (or reports
  that the queue is empty — never invent a task ID) through the full task lifecycle.
- **A `TASK_ID`** — take exactly that task. If it is absent from the backlog queue, report and stop. If
  a different task is already `IN_PROGRESS`, stop: closing or abandoning it is an owner decision.
- **`all`** — work the queue continuously: close each task completely before the next, report each one
  separately, and end the run on a special gate, a red full gate, an unsatisfied dependency, a premise
  the repository contradicts, or an empty queue.

If the task's queue position carries a special gate, tell the developer to announce it on pickup and
stop after its own verification; the run/skip decision is yours under default authority — record it
with the exit code in the progress row, and escalate only a gate that is destructive or irreversible.

After dispatching, set a pulse monitor — a persistent background watch over the registry and commits
(poll every 2 minutes, emit only on state change):

```bash
cd "$CLAUDE_PROJECT_DIR"; prev=""; while true; do \
  commit=$(git log --oneline -1 2>/dev/null | cut -c1-60); \
  task=$(awk -F': ' '/^CURRENT_TASK:/{print $2}' workflow/progress.md); \
  cards=$(ls workflow/tasks/ 2>/dev/null | grep -c '^T-'); \
  dirty=$(git status --short 2>/dev/null | wc -l | tr -d ' '); \
  cur="task=$task cards=$cards dirty=$dirty last-commit: $commit"; \
  if [ "$cur" != "$prev" ]; then echo "$cur"; prev="$cur"; fi; \
  sleep 120; done
```

Reading the events: `task=<ID> cards=1` — work in progress; `task=none cards=1` — close-out phase;
a new commit naming the task ID — the task landed; `task=none cards=0 dirty=0` — lifecycle complete:
stop the monitor and run acceptance («пройди приёмку»). If events stall for 15–20 minutes without the
developer finishing, check whether it is waiting on a permission prompt and tell the owner.

Relay each report to the owner as it arrives: task ID and scope, changed files, RED/GREEN evidence,
exact commands with exit codes, progress state, next action — or the blocker's four facts.
