use macroquad::prelude::*;

mod arena;
mod bullet;
mod camera;
mod collision;
mod combat;
mod enemy;
mod game;
mod pickup;
mod player;
mod score;

use arena::Arena;
use bullet::Bullet;
use enemy::Enemy;
use game::{Effect, Game, Input};
use pickup::{Pickup, PickupKind};
use player::Player;

/// Цвет врага — другой, чтобы он отличался от игрока и пуль на глаз.
const ENEMY_COLOR: Color = Color::new(0.85, 0.25, 0.25, 1.0);
/// Цвета pickup: разные для здоровья и патронов, чтобы игрок различал их издалека.
const HEALTH_PICKUP_COLOR: Color = Color::new(0.3, 0.85, 0.4, 1.0);
const AMMO_PICKUP_COLOR: Color = Color::new(0.9, 0.8, 0.2, 1.0);
/// Игрок во время неуязвимости рисуется этим более тусклым цветом вместо обычного — видимый признак
/// того, что повторное касание сейчас не в счёт.
const PLAYER_INVULNERABLE_COLOR: Color = Color::new(0.88, 0.89, 0.93, 0.4);

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
    let mut game = Game::new(arena);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке (D-09) — порядок стадий
        // часть того, что этот проект показывает, даже когда позиция ещё не наполнила все стадии
        // поведением.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) {
            game.restart();
        }
        let up = is_key_down(KeyCode::W);
        let down = is_key_down(KeyCode::S);
        let left = is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::D);
        let fire = is_mouse_button_down(MouseButton::Left);
        let space_pressed = is_key_pressed(KeyCode::Space);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния. Камера считается здесь же, а не только в рендеринге: прицеливание
        //    нуждается в том же центре камеры, что и отрисовка, иначе курсор и то, что видит игрок,
        //    разойдутся (D-39). Переходы состояний и вся игровая логика кадра — внутри `Game::update`.
        let screen_size = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
        let center = camera::camera_center(game.player.position, arena, screen_size);
        let (mouse_x, mouse_y) = mouse_position();
        let cursor_world = camera::screen_to_world(vec2(mouse_x, mouse_y), center, screen_size);
        let input = Input {
            up,
            down,
            left,
            right,
            fire,
            cursor_world,
            space_pressed,
        };
        game.update(&input, delta_time);

        // 4. Проверка столкновений — уже выполнена внутри `Game::update`.

        // 5. Рендеринг.
        set_camera(&Camera2D {
            target: center,
            zoom: vec2(2.0 / screen_size.x, -2.0 / screen_size.y),
            ..Default::default()
        });
        draw_arena(arena);
        draw_player(&game.player, game.is_invulnerable());
        for bullet in &game.bullets {
            draw_bullet(bullet);
        }
        for enemy in &game.enemies {
            draw_enemy(enemy);
        }
        for pickup in &game.pickups {
            draw_pickup(pickup);
        }
        for effect in &game.effects {
            draw_effect(effect);
        }
        set_default_camera();
        draw_hud(&game);
        if let Some(message) = game.state.message(game.wave + 1) {
            draw_message(screen_size, &message);
        }

        // 6. Следующий кадр.
        next_frame().await;
    }
}

/// Кругом — та же форма, что у столкновений D-38, а не квадрат, как в Pong/Arkanoid/Asteroids: у
/// Shooter все подвижные объекты рисуются кругами (D-38).
fn draw_bullet(bullet: &Bullet) {
    draw_circle(
        bullet.position.x,
        bullet.position.y,
        bullet::BULLET_RADIUS,
        ARENA_COLOR,
    );
}

/// Кругом другого цвета, чем игрок — чтобы враг отличался на глаз.
fn draw_enemy(enemy: &Enemy) {
    draw_circle(
        enemy.position.x,
        enemy.position.y,
        enemy.radius,
        ENEMY_COLOR,
    );
}

/// Кругом, цвет зависит от вида pickup.
fn draw_pickup(pickup: &Pickup) {
    let color = match pickup.kind {
        PickupKind::Health => HEALTH_PICKUP_COLOR,
        PickupKind::Ammo => AMMO_PICKUP_COLOR,
    };
    draw_circle(pickup.position.x, pickup.position.y, pickup.radius, color);
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
/// показывающий, куда целится игрок. Во время неуязвимости (`invulnerable`) игрок рисуется тусклее —
/// видимый признак того, что повторное касание сейчас не в счёт (T-TDS-12).
fn draw_player(player: &Player, invulnerable: bool) {
    let color = if invulnerable {
        PLAYER_INVULNERABLE_COLOR
    } else {
        ARENA_COLOR
    };
    draw_circle_lines(
        player.position.x,
        player.position.y,
        player.radius,
        2.0,
        color,
    );
    draw_circle(player.position.x, player.position.y, 2.0, color);
    let barrel_tip = player.barrel_position();
    draw_line(
        player.position.x,
        player.position.y,
        barrel_tip.x,
        barrel_tip.y,
        3.0,
        color,
    );
}

/// Растущий и затухающий круг на месте уничтоженного врага (`Effect::progress` — доля прожитого
/// времени эффекта); отрисовка не тестируется (D-10).
fn draw_effect(effect: &Effect) {
    let progress = effect.progress();
    let radius = 8.0 + progress * 24.0;
    let mut color = ENEMY_COLOR;
    color.a = 1.0 - progress;
    draw_circle_lines(effect.position.x, effect.position.y, radius, 2.0, color);
}

/// HUD в левом верхнем углу — экранные координаты, стадия после `set_default_camera` (D-39): здесь
/// UI рисуется отдельно от мира и не уедет вместе с камерой.
fn draw_hud(game: &Game) {
    let text = format!(
        "Health {}   Ammo {}   Score {}   Wave {}",
        game.health.max(0),
        game.ammo,
        game.score.value(),
        game.wave
    );
    draw_text(&text, 16.0, 28.0, 24.0, ARENA_COLOR);
}

/// Сообщение состояния по центру окна (`GameState::message`); латиница — та же причина, что в
/// Pong/Arkanoid/Asteroids (кириллические глифы встроенного шрифта Macroquad наблюдением не
/// проверены). Центрируется по окну, а не по арене — сообщение рисуется уже в экранных координатах.
fn draw_message(screen_size: Vec2, message: &str) {
    const FONT_SIZE: f32 = 32.0;
    for (line_index, line) in message.lines().enumerate() {
        let dimensions = measure_text(line, None, FONT_SIZE as u16, 1.0);
        let x = (screen_size.x - dimensions.width) / 2.0;
        let y = screen_size.y / 2.0 + line_index as f32 * (FONT_SIZE + 6.0);
        draw_text(line, x, y, FONT_SIZE, ARENA_COLOR);
    }
}
