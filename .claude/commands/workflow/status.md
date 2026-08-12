---
description: "Workflow: the manager's dashboard — recovery pointer, queue, ideas, agents, commits; changes nothing"
allowed-tools: Bash(make context-check:*), Bash(git status:*), Bash(git log:*), Bash(ls:*), Read, Glob, ListAgents
---

# Workflow status

Recovery pointer and registry:

@workflow/progress.md

Open queue:

@workflow/backlog.md

Worktree state: !`git status --short`

Recent commits: !`git log --oneline -5`

Active task cards: !`ls -1 workflow/tasks/ 2>/dev/null | grep '^T-' || echo none`

Open ideas (name and age): !`ls -lt workflow/source/ 2>/dev/null | awk 'NR>1 && $NF !~ /example$/ {print $NF, "(" $6, $7 ")"}'`

Context consistency: !`make context-check`

Also check `ListAgents` — which subagents are alive right now.

Assemble a short manager dashboard, not a report:

1. `CURRENT_TASK`, `NEXT_BACKLOG_ID`, `NEXT_ACTION` (quoted exactly), worktree cleanliness.
2. Active subagents and what each is doing.
3. Open ideas with their age — an idea hanging without a verdict is part of the project state.
4. Whether the pointer, registry, cards and queue agree; name any disagreement instead of silently
   fixing it, plus the `make context-check` result.
5. One sentence: what to do next.

This command is read-only. Do not create a card, edit `workflow/` or start work.
