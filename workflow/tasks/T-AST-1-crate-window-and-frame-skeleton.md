# T-AST-1: Крейт, окно и каркас кадра

## Статус

Главный статус, выбор задачи и состояние восстановления находятся в `workflow/progress.md`. Эта карточка
нужна для работы над одной задачей и удаляется после статуса `DONE`.

## Реализует

- `T-AST-1` — первая позиция очереди Game 03 — Asteroids.

## Цель

`cargo run -p asteroids` открывает окно с игровым полем, в коде явно видны стадии кадра, `Esc` выходит.

## Scope

- Новый крейт `asteroids` четвёртым участником workspace (`members` в корневом `Cargo.toml`).
- `asteroids/Cargo.toml` с зависимостью `macroquad = "0.4.16"`, та же форма, что у `pong`/`arkanoid`.
- `asteroids/src/lib.rs` с единственным публичным элементом `pub async fn run()`; `Field` объявлен
  прямо в этом крейте (дублирование — принятая цена D-02), стадии кадра D-09 видны как структура кода:
  чтение ввода → delta time → обновление → столкновения → рендеринг → следующий кадр; отрисовка фона и
  рамки поля примитивами Macroquad.
- `asteroids/src/main.rs` — `window_conf` + `#[macroquad::main]`, вызывающий `asteroids::run().await`
  (16 строк, форма `pong/src/main.rs`).

## Out Of Scope

- Корабль, пули, астероиды, состояния, счёт.
- Пункт меню и правка корневого `Cargo.toml` в части `default-members` (`T-AST-11`).
- README (`T-AST-12`).

## Критерии приёмки

- [ ] `cargo test --manifest-path ./Cargo.toml -p asteroids` проходит (крейт существует, компилируется).
- [ ] `make check` зелёный по всему workspace.
- [ ] Специальный gate: наблюдаемый прогон `cargo run -p asteroids` — окно с полем, `Esc` выходит.
  Подтверждает владелец.

## Проверка

```text
RED: cargo test --manifest-path ./Cargo.toml -p asteroids
  - наблюдаемая ошибка (не тест): package ID specification 'asteroids' did not match any packages
GREEN: cargo test --manifest-path ./Cargo.toml -p asteroids
TARGETED: make test-target TEST=game (30 Pong + 52 Arkanoid + 8 menu продолжают проходить)
FULL: make check
SPECIAL: cargo run -p asteroids — окно с полем открывается, Esc выходит. Ждёт подтверждения владельца;
задача не закрывается до ответа.
```

Unit-тестов на этой позиции нет: всё поведение — окно и отрисовка, а они не тестируются (D-10).

## Риски

- Ломка сборки Pong/Arkanoid/Menu правкой корневого `Cargo.toml` — обнаружит `make check` (workspace-сборка
  и тесты всех крейтов).

## Отчёт о завершении

Передать владельцу, затем удалить карточку. Постоянная запись остаётся в строке `progress.md`.

- изменённые файлы: `Cargo.toml`, `asteroids/Cargo.toml`, `asteroids/src/main.rs`, `asteroids/src/lib.rs`,
  `workflow/progress.md`, `workflow/tasks/T-AST-1-crate-window-and-frame-skeleton.md`.
- команды проверки и коды завершения: RED `cargo test --manifest-path ./Cargo.toml -p asteroids` → 101;
  GREEN та же команда → 0; TARGETED `make test-target TEST=game` → 0; FULL `make check` → 0;
  `make boundary-check` → 0; `make context-check` → 0.
- задача не закрыта: ждёт специального gate — наблюдаемый прогон `cargo run -p asteroids` подтверждает
  владелец. Карточка остаётся до его ответа.
