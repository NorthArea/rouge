use macroquad::prelude::*;

mod ship;

use ship::Ship;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

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

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния.
        ship.rotate(turn, delta_time);

        // 4. Проверка столкновений.
        //    Появится вместе с первым другим объектом (T-AST-7).

        // 5. Рендеринг.
        draw_field(field);
        draw_ship(&ship);

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
