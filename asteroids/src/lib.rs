use macroquad::prelude::*;

mod ship;

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
    let mut ship = Ship::new(vec2(field.width / 2.0, field.height / 2.0));

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке: порядок стадий — часть
        // того, что этот проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let turn = ship::turn_direction(
            is_key_down(KeyCode::A) || is_key_down(KeyCode::Left),
            is_key_down(KeyCode::D) || is_key_down(KeyCode::Right),
        );
        let thrusting = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния.
        ship.rotate(turn, delta_time);
        // Тяга добавляется только пока клавиша зажата, но ограничение скорости внутри `apply_thrust`
        // выполняется каждый кадр (см. комментарий на `Ship::apply_thrust`) — иначе кадр без тяги не
        // проходил бы через `clamp_length_max`, а это ровно то место, которое обязано быть безопасным
        // на нулевом векторе.
        ship.apply_thrust(if thrusting { delta_time } else { 0.0 });
        ship.advance(delta_time);

        // 4. Проверка столкновений.
        //    Появится вместе с первым другим объектом (T-AST-7).

        // 5. Рендеринг.
        draw_field(field);
        draw_ship(&ship);
        if thrusting {
            draw_engine_flame(&ship);
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
