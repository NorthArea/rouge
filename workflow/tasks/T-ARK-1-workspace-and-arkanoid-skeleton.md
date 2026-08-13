# T-ARK-1: workspace, переезд Pong и каркас Arkanoid

## Статус

Главный статус, выбор задачи и состояние восстановления находятся в `workflow/progress.md`. Эта карточка
нужна для работы над одной задачей и удаляется после статуса `DONE`.

## Реализует

- `T-ARK-1` — перевод репозитория в cargo-workspace, переезд Pong в крейт `pong` без изменения его
  кода, крейт `arkanoid` с окном, полем, явными стадиями кадра и выходом по `Esc`.

## Цель

Репозиторий становится cargo-workspace из двух крейтов; `cargo run -p arkanoid` открывает окно с
игровым полем, в коде явно виден цикл кадра, `Esc` закрывает игру; принятый Pong продолжает работать
и остаётся зелёным на новом месте.

## Scope

- Корневой `Cargo.toml` — виртуальный манифест workspace: `[workspace]`, `resolver = "2"`,
  `members = ["pong", "arkanoid"]`, без секции `[package]` (D-16).
- Переезд Pong: `git mv src pong/src`, прежний корневой манифест становится `pong/Cargo.toml`
  (пакет `pong`, та же зависимость `macroquad`). Содержимое файлов игры не редактируется.
- `git mv README.md pong/README.md`; единственная допустимая правка внутри — команда запуска
  `cargo run` → `cargo run -p pong`.
- Новый корневой `README.md` — короткий указатель по D-15.
- Новый крейт `arkanoid`: `arkanoid/Cargo.toml` (macroquad той же версии) и `arkanoid/src/main.rs`
  с окном, отрисовкой поля и рамки, выходом по `Esc` и явными стадиями кадра из D-09.
- Размеры поля читаются из размеров окна и передаются значениями, а не глобальным состоянием (D-10).

## Out Of Scope

- Ракетка, мяч, блоки, счёт, жизни, состояния, бонусы, UI, README Arkanoid.
- Любые изменения игровой логики Pong.
- Правка `Makefile`: она governance и выполнена управляющим до начала позиции (коммит `0f7e688`).

## Критерии приёмки

- [x] Корневой `Cargo.toml` — виртуальный манифест workspace с двумя участниками.
- [x] 30 тестов Pong зелёные из нового каталога `pong/src`.
- [x] sha256 каждого файла `pong/src/*.rs` совпадает с состоянием до переезда.
- [x] `cargo test -p arkanoid` находит пакет (было: не находил).
- [x] `arkanoid/src/main.rs` показывает стадии кадра D-09 как структуру кода и выходит по `Esc`.
- [ ] Специальный gate: наблюдаемый прогон обеих игр — подтверждает владелец.

## Проверка

Поведения задача не добавляет: это перемещение файлов и создание манифестов. Новых unit-тестов нет —
всё, что создаётся здесь, это окно и отрисовка, а их тестировать запрещено (D-10). Наблюдаемый RED
существует без написания теста: пакета `arkanoid` в репозитории ещё нет.

```text
RED: cargo test --manifest-path ./Cargo.toml -p arkanoid
  - пакет arkanoid → exit 101, "error: package ID specification `arkanoid` did not match any packages"
BASELINE (до переезда): make test → exit 0, 30 passed
GREEN: cargo test --manifest-path ./Cargo.toml -p arkanoid → exit 0
GREEN (переезд): make test → exit 0, 30 passed из pong/src
TARGETED: make test-target TEST=game
FULL: make check
SPECIAL: cargo run -p arkanoid (окно, поле, Esc закрывает) и cargo run -p pong (принятая игра
  работает) — подтверждает владелец, агент окон не видит.
```

### sha256 файлов Pong до и после переезда

Совпали все семь, снято `shasum -a 256` до `git mv` и после него:

```text
c1cacf04238e83541bac76b6176f675cdd0e3dda85b62855ccc4e03a569656de  src/ball.rs   → pong/src/ball.rs
35f703a6c200f3a69ccba200dea3fcd1ef43b4427b31ed727890e46d651091b0  src/game.rs   → pong/src/game.rs
98fc62160db5192de666a06da287be4e28376f451e6602d20cc5f26b2c251f10  src/main.rs   → pong/src/main.rs
2a1c07c8b3d6ccbacca45d18f23a0b23fc99da082ff156e44a47bf0082aa7834  src/paddle.rs → pong/src/paddle.rs
f164ea1768d181cfd97dbe14550ef5fb327fb50b35e253964649d43d973f210d  src/score.rs  → pong/src/score.rs
78ab8c554288d517ae5a24a77161e781016858cb7d9bd37bbde30ad3626cbb91  README.md     → pong/README.md (до правки строки запуска)
e84e27a8531b7f5b72e7028beb5a021da0c72734e6a6d43723595a9ae34eb782  Cargo.toml    → pong/Cargo.toml
```

`git diff -M --stat HEAD` подтверждает то же со стороны git: пять файлов `pong/src/*.rs` и
`pong/Cargo.toml` — переименования с нулём изменённых строк, `README.md => pong/README.md` — ровно
`2 +-`, то есть единственная разрешённая правка команды запуска.

## Результат проверок

```text
RED:      cargo test --manifest-path ./Cargo.toml -p arkanoid → exit 101
          error: package ID specification `arkanoid` did not match any packages
BASELINE: make test (до переезда) → exit 0, 30 passed
GREEN:    cargo test --manifest-path ./Cargo.toml -p arkanoid → exit 0 (0 тестов — их здесь и не должно быть)
GREEN:    make test → exit 0, 30 passed из pong/src + 0 из arkanoid
TARGETED: make test-target TEST=game → exit 0, 8 passed, 22 filtered out
FULL:     make check → exit 0 с первого прогона
SPECIAL:  ожидает владельца
```

## Риски

- Переезд ломает пути в `Makefile` — снимается тем, что `CARGO_MANIFEST` продолжает указывать на
  корневой `Cargo.toml`, а команды идут с `--workspace`; ловится `make check`.
- Незамеченная правка кода Pong — снимается сверкой sha256 и `git diff -M` (чистые переименования).
- Пустые стадии кадра в новом крейте могут дать предупреждение под `-D warnings` — ловится `make check`.

## Отчёт о завершении

Передать владельцу, затем удалить карточку. Постоянная запись остаётся в строке `progress.md`.

- изменённые файлы:
- команды проверки и коды завершения:
- следующая задача и точное следующее действие:
