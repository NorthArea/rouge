use macroquad::prelude::*;

mod ball;
mod paddle;

use ball::Ball;
use paddle::Paddle;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

const BORDER_THICKNESS: f32 = 6.0;
const CENTER_LINE_WIDTH: f32 = 6.0;
const CENTER_LINE_DASH: f32 = 20.0;
const CENTER_LINE_GAP: f32 = 16.0;

/// Игровое поле. Размеры приходят значением из размеров окна, поэтому игровая логика не обращается
/// к состоянию Macroquad и остаётся вызываемой сама по себе.
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

fn window_conf() -> Conf {
    Conf {
        window_title: "Pong".to_owned(),
        window_width: 960,
        window_height: 600,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Окно не изменяет размер, поэтому поле — данные, заданные один раз, а не результат опроса
    // Macroquad в каждом кадре.
    let field = Field::new(screen_width(), screen_height());
    let mut left_paddle = Paddle::left(field);
    let mut right_paddle = Paddle::right(field);
    let mut ball = Ball::new(field);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке: порядок стадий — часть
        // того, что этот проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let left_direction = read_direction(KeyCode::W, KeyCode::S);
        let right_direction = read_direction(KeyCode::Up, KeyCode::Down);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния.
        left_paddle.update(left_direction, field, delta_time);
        right_paddle.update(right_direction, field, delta_time);
        ball.update(delta_time);

        // 4. Проверка столкновений. Ракетки ограничены полем при обновлении, а мяч отражается от
        // верхней и нижней границ именно здесь: это столкновение, а не перемещение.
        ball.bounce_off_field_edges(field);

        // 5. Рендеринг.
        draw_field(field);
        draw_paddle(&left_paddle);
        draw_paddle(&right_paddle);
        draw_ball(&ball);

        // 6. Следующий кадр.
        next_frame().await;
    }
}

/// Обе ракетки читают ввод одинаково. Клавиша считается зажатой, а не нажатой один раз: движение
/// длится, пока клавишу держат. Обе клавиши сразу или ни одной — ракетка стоит.
fn read_direction(up: KeyCode, down: KeyCode) -> f32 {
    match (is_key_down(up), is_key_down(down)) {
        (true, false) => paddle::UP,
        (false, true) => paddle::DOWN,
        _ => paddle::STILL,
    }
}

fn draw_field(field: Field) {
    clear_background(BACKGROUND_COLOR);
    draw_center_line(field);
    draw_borders(field);
}

/// Ракетки и мяч рисуются тем же светлым цветом, что и разметка поля: классический Pong контрастен и
/// обходится одним цветом на всё, кроме фона.
fn draw_paddle(paddle: &Paddle) {
    draw_rectangle(paddle.x, paddle.y, paddle.width, paddle.height, FIELD_COLOR);
}

/// Мяч рисуется квадратом, а не кругом: так изображение совпадает с прямоугольником, по которому
/// считаются столкновения.
fn draw_ball(ball: &Ball) {
    draw_rectangle(ball.x, ball.y, ball.size, ball.size, FIELD_COLOR);
}

/// Центральная линия — не сплошная линия, а столбик отдельных штрихов: так выглядит классический Pong.
fn draw_center_line(field: Field) {
    let x = (field.width - CENTER_LINE_WIDTH) / 2.0;
    let step = CENTER_LINE_DASH + CENTER_LINE_GAP;

    let mut y = CENTER_LINE_GAP;
    while y + CENTER_LINE_DASH <= field.height {
        draw_rectangle(x, y, CENTER_LINE_WIDTH, CENTER_LINE_DASH, FIELD_COLOR);
        y += step;
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
