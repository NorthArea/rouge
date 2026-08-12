# Статус и ход выполнения задач

Текущий статус задач и указатель восстановления. Этот файл — единственный источник правды о статусе.

## Указатель восстановления

PROJECT_STATUS: IN_PROGRESS
CURRENT_TASK: T-PONG-1
NEXT_BACKLOG_ID: T-PONG-2
LAST_COMPLETED_TASK: none
LAST_GREEN_COMMAND: make check
LAST_CHECKPOINT: <short-sha>/<TASK_ID>
NEXT_ACTION: Владелец запускает `cargo run` в корне репозитория и подтверждает три наблюдения: открылось окно «Pong» 960×600; видно тёмное поле с рамкой и пунктирной центральной линией; нажатие `Esc` закрывает игру. После ответа владельца — закрыть T-PONG-1 (`DONE`, удалить запись из backlog, checkpoint-коммит, удалить карточку) либо исправить замечания.

## Реестр статусов задач

Допустимые статусы строк: `IN_PROGRESS`, `DONE`, `BLOCKED`. Статуса `PENDING` нет: неначатые задачи
остаются в backlog.

| Task | Status | Scope / result | Verification |
|------|--------|----------------|--------------|
| T-PONG-1 | IN_PROGRESS | Cargo-проект в корне, Macroquad, окно, game loop с явными стадиями кадра, отрисовка поля и пунктирной центральной линии, выход по `Esc`. Код готов и закоммичен; задача ждёт подтверждения специального gate владельцем. | RED наблюдался: `make test` → exit 2, `manifest path .../Cargo.toml does not exist`. GREEN: `make test` → exit 0. Полный гейт: `make check` → exit 0. Специальный gate `cargo run` — не запускался, ждёт наблюдения владельца. |
