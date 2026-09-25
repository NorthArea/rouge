use macroquad::prelude::*;

mod arena;
mod camera;
mod player;

use arena::Arena;
use player::Player;

/// Размер окна, тот же, что в `window_conf` (`main.rs`). Используется камерой, чтобы вычислить, какая
/// часть арены видна, поэтому число не может разойтись с настройками окна незаметно.
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 600.0;

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
    let mut player = Player::new(vec2(arena.width / 2.0, arena.height / 2.0));

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке (D-09) — порядок стадий
        // часть того, что этот проект показывает, даже когда позиция ещё не наполнила все стадии
        // поведением.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let up = is_key_down(KeyCode::W);
        let down = is_key_down(KeyCode::S);
        let left = is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::D);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния. Камера считается здесь же, а не только в рендеринге: прицеливание
        //    нуждается в том же центре камеры, что и отрисовка, иначе курсор и то, что видит игрок,
        //    разойдутся (D-39).
        player.update(up, down, left, right, delta_time);
        player.clamp_to_arena(arena);
        let screen_size = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
        let center = camera::camera_center(player.position, arena, screen_size);
        let (mouse_x, mouse_y) = mouse_position();
        let cursor_world = camera::screen_to_world(vec2(mouse_x, mouse_y), center, screen_size);
        player.aim_at(cursor_world);

        // 4. Проверка столкновений. Пока нечего проверять.

        // 5. Рендеринг.
        set_camera(&Camera2D {
            target: center,
            zoom: vec2(2.0 / screen_size.x, -2.0 / screen_size.y),
            ..Default::default()
        });
        draw_arena(arena);
        draw_player(&player);
        set_default_camera();

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

/// Кругом с видимым центром: сам круг — форма столкновения (D-38), точка в центре — чтобы позиция
/// читалась однозначно, а не только по контуру. Короткая линия от центра в сторону `facing()` — ствол,
/// показывающий, куда целится игрок.
fn draw_player(player: &Player) {
    draw_circle_lines(
        player.position.x,
        player.position.y,
        player.radius,
        2.0,
        ARENA_COLOR,
    );
    draw_circle(player.position.x, player.position.y, 2.0, ARENA_COLOR);
    let barrel_tip = player.position + player.facing() * (player.radius + 14.0);
    draw_line(
        player.position.x,
        player.position.y,
        barrel_tip.x,
        barrel_tip.y,
        3.0,
        ARENA_COLOR,
    );
}
