# Статус и ход выполнения задач

Текущий статус задач и указатель восстановления. Этот файл — единственный источник правды о статусе.

## Указатель восстановления

PROJECT_STATUS: IN_PROGRESS
CURRENT_TASK: none
NEXT_BACKLOG_ID: T-PONG-2
LAST_COMPLETED_TASK: T-PONG-1
LAST_GREEN_COMMAND: make check
LAST_CHECKPOINT: aad74f3/T-PONG-1
NEXT_ACTION: Выполнить /workflow:build — developer берёт T-PONG-2 (ракетки: модель, движение через delta time, ограничение границами поля, управление `W`/`S` и `↑`/`↓`, отрисовка).

## Реестр статусов задач

Допустимые статусы строк: `IN_PROGRESS`, `DONE`, `BLOCKED`. Статуса `PENDING` нет: неначатые задачи
остаются в backlog.

| Task | Status | Scope / result | Verification |
|------|--------|----------------|--------------|
| T-PONG-1 | DONE | Cargo-проект Pong живёт в корне репозитория и открывает окно с полем: Macroquad подключён, game loop показывает стадии кадра явно, поле с рамкой и пунктирной центральной линией нарисовано, `Esc` закрывает игру. | RED: `make test` → exit 2, `manifest path .../Cargo.toml does not exist`. GREEN: `make test` → exit 0. Полный гейт: `make check` → exit 0. Unit-тестов нет по D-10 (рендеринг и ввод не тестируются). Специальный gate: прогон `cargo run` выполнял владелец, а не агент; gate подтверждён владельцем, дословный ответ — «ага, видно». |
