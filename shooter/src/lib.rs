use macroquad::prelude::*;

mod arena;

use arena::Arena;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const ARENA_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

const BORDER_THICKNESS: f32 = 6.0;

/// Арена задана явно больше окна (960×600): камера, которая появится в `T-TDS-3`, обязана
/// действительно следовать за игроком, а не показывать всю арену сразу. Числа выбраны так, чтобы
/// игрок доходил до каждого края за несколько секунд и видел, как край арены останавливает камеру.
const ARENA_WIDTH: f32 = 1920.0;
const ARENA_HEIGHT: f32 = 1200.0;

pub async fn run() {
    let arena = Arena::new(ARENA_WIDTH, ARENA_HEIGHT);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке (D-09) — порядок стадий
        // часть того, что этот проект показывает, даже когда позиция ещё не наполнила все стадии
        // поведением.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let _delta_time = get_frame_time();

        // 3. Обновление состояния. Пока нечего обновлять — игрок появится в `T-TDS-2`.

        // 4. Проверка столкновений. Пока нечего проверять.

        // 5. Рендеринг.
        draw_arena(arena);

        // 6. Следующий кадр.
        next_frame().await;
    }
}

fn draw_arena(arena: Arena) {
    clear_background(BACKGROUND_COLOR);
    draw_rectangle_lines(
        0.0,
        0.0,
        arena.width,
        arena.height,
        BORDER_THICKNESS * 2.0,
        ARENA_COLOR,
    );
}
