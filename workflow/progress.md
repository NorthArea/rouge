# Статус и ход выполнения задач

Текущий статус задач и указатель восстановления. Этот файл — единственный источник правды о статусе.

## Указатель восстановления

PROJECT_STATUS: IN_PROGRESS
CURRENT_TASK: none
NEXT_BACKLOG_ID: T-PONG-1
LAST_COMPLETED_TASK: none
LAST_GREEN_COMMAND: <exact command>
LAST_CHECKPOINT: <short-sha>/<TASK_ID>
NEXT_ACTION: Выполнить /workflow:build — developer берёт T-PONG-1 (cargo-проект в корне, Macroquad, окно, game loop, поле, выход по Esc).

## Реестр статусов задач

Допустимые статусы строк: `IN_PROGRESS`, `DONE`, `BLOCKED`. Статуса `PENDING` нет: неначатые задачи
остаются в backlog.

| Task | Status | Scope / result | Verification |
|------|--------|----------------|--------------|
