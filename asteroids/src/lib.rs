use macroquad::prelude::*;

mod asteroid;
mod bullet;
mod collision;
mod combat;
mod game;
mod score;
mod ship;

use asteroid::Asteroid;
use bullet::Bullet;
use game::{Effect, Game};
use ship::Ship;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);
const FLAME_COLOR: Color = Color::new(0.95, 0.55, 0.15, 1.0);

const BORDER_THICKNESS: f32 = 6.0;

/// Игровое поле. Размеры приходят значением из размеров окна, поэтому игровая логика не обращается
/// к состоянию Macroquad и остаётся вызываемой сама по себе. Дублирует `Field` из `pong` и `arkanoid`
/// — принятая цена независимости крейтов друг от друга (D-02).
#[derive(Clone, Copy)]
pub struct Field {
    pub width: f32,
    pub height: f32,
}

impl Field {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

pub async fn run() {
    // Окно не изменяет размер, поэтому поле — данные, заданные один раз, а не результат опроса
    // Macroquad в каждом кадре.
    let field = Field::new(screen_width(), screen_height());
    let mut game = Game::new(field);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке: порядок стадий — часть
        // того, что этот проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) {
            game.restart();
        }
        let turn = ship::turn_direction(
            is_key_down(KeyCode::A) || is_key_down(KeyCode::Left),
            is_key_down(KeyCode::D) || is_key_down(KeyCode::Right),
        );
        let thrusting = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        // `Space` — одна клавиша на два намерения, различённых внутри `Game::update` по состоянию
        // (начать раунд из `WaitingToStart` или выстрелить в `Playing`), по прецеденту `T-AST-5`.
        let space_pressed = is_key_pressed(KeyCode::Space);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния и 4. проверка столкновений — целиком внутри `Game::update`,
        //    переходы состояний видны там одним местом.
        game.update(turn, thrusting, space_pressed, delta_time);

        // 5. Рендеринг.
        draw_field(field);
        draw_ship(&game.ship);
        if thrusting {
            draw_engine_flame(&game.ship);
        }
        for bullet in &game.bullets {
            draw_bullet(bullet);
        }
        for asteroid in &game.asteroids {
            draw_asteroid(asteroid);
        }
        for effect in &game.effects {
            draw_effect(effect);
        }
        draw_status(&game);
        if let Some(message) = game.state.message(game.wave + 1) {
            draw_message(field, &message);
        }

        // 6. Следующий кадр.
        next_frame().await;
    }
}

fn draw_field(field: Field) {
    clear_background(BACKGROUND_COLOR);
    draw_borders(field);
}

/// Треугольник рисуется по тем же трём вершинам, по которым считается радиус столкновения (D-27
/// признаёт это расхождение для корабля: сталкивается кругом, но рисуется треугольником).
fn draw_ship(ship: &Ship) {
    let [nose, rear_left, rear_right] = ship.vertices();
    draw_triangle(nose, rear_left, rear_right, FIELD_COLOR);
}

/// Факел двигателя виден только в кадрах, когда тяга включена — иначе он показывал бы ускорение,
/// которого в этом кадре нет. Растёт из кормы назад от `facing()`; поперечная ось получена тем же
/// приёмом, что в `Ship::vertices()` (перпендикуляр к `facing()`), без повторного `sin`/`cos` (D-25).
fn draw_engine_flame(ship: &Ship) {
    let forward = ship.facing();
    let side = vec2(-forward.y, forward.x);
    let base_left = ship.position - forward * 10.0 + side * 6.0;
    let base_right = ship.position - forward * 10.0 - side * 6.0;
    let tip = ship.position - forward * 24.0;
    draw_triangle(base_left, base_right, tip, FLAME_COLOR);
}

/// Пуля рисуется квадратом по прецеденту мяча в Pong/Arkanoid (D-12).
fn draw_bullet(bullet: &Bullet) {
    let size = bullet::BULLET_RADIUS * 2.0;
    draw_rectangle(
        bullet.position.x - size / 2.0,
        bullet.position.y - size / 2.0,
        size,
        size,
        FIELD_COLOR,
    );
}

/// Окружностью, а не многоугольником: изображение и модель столкновений астероида совпадают точно
/// (столкновение придёт в `T-AST-7` как круг с кругом, D-27).
fn draw_asteroid(asteroid: &Asteroid) {
    draw_circle_lines(
        asteroid.position.x,
        asteroid.position.y,
        asteroid.radius(),
        2.0,
        FIELD_COLOR,
    );
}

/// Растущий и затухающий круг на месте уничтоженного астероида (`Effect::progress` — доля прожитого
/// времени эффекта); отрисовка не тестируется (D-10).
fn draw_effect(effect: &Effect) {
    let progress = effect.progress();
    let radius = 10.0 + progress * 30.0;
    let mut color = FIELD_COLOR;
    color.a = 1.0 - progress;
    draw_circle_lines(effect.position.x, effect.position.y, radius, 2.0, color);
}

/// Строка статуса в левом верхнем углу: счёт, жизни, номер волны. Отрисовка не тестируется (D-10);
/// сами значения читаются напрямую из `Game`.
fn draw_status(game: &Game) {
    let text = format!(
        "Score {}   Lives {}   Wave {}",
        game.score.value(),
        game.lives,
        game.wave
    );
    draw_text(&text, 16.0, 28.0, 24.0, FIELD_COLOR);
}

/// Сообщение состояния по центру поля (`GameState::message`); латиница — та же причина, что в
/// Pong и Arkanoid (кириллические глифы встроенного шрифта Macroquad наблюдением не проверены).
fn draw_message(field: Field, message: &str) {
    const FONT_SIZE: f32 = 32.0;
    for (line_index, line) in message.lines().enumerate() {
        let dimensions = measure_text(line, None, FONT_SIZE as u16, 1.0);
        let x = (field.width - dimensions.width) / 2.0;
        let y = field.height / 2.0 + line_index as f32 * (FONT_SIZE + 6.0);
        draw_text(line, x, y, FONT_SIZE, FIELD_COLOR);
    }
}

/// `draw_rectangle_lines` рисует рамку по центру контура, поэтому её внешняя половина ушла бы
/// за край окна. Рамка задаётся двойной толщиной, а видимой остаётся ровно `BORDER_THICKNESS`.
fn draw_borders(field: Field) {
    draw_rectangle_lines(
        0.0,
        0.0,
        field.width,
        field.height,
        BORDER_THICKNESS * 2.0,
        FIELD_COLOR,
    );
}
