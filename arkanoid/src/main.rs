use macroquad::prelude::*;

mod ball;
mod paddle;

use ball::Ball;
use paddle::Paddle;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

const BORDER_THICKNESS: f32 = 6.0;

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
        window_title: "Arkanoid".to_owned(),
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
    let mut player = Paddle::new(field);
    let mut ball = Ball::new(field);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке: порядок стадий — часть
        // того, что этот проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let direction = read_direction();

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния.
        player.update(direction, field, delta_time);
        ball.update(delta_time);

        // 4. Проверка столкновений. Пока это только стены: нижней стены у поля нет, поэтому мяч,
        //    ушедший вниз, не возвращается — это временный тупик до появления жизней.
        ball.bounce_off_walls(field);

        // 5. Рендеринг.
        draw_field(field);
        draw_paddle(&player);
        draw_ball(&ball);

        // 6. Следующий кадр.
        next_frame().await;
    }
}

/// Ракетка ходит по горизонтали, поэтому влево смотрят `A` и `←`, вправо — `D` и `→`. Клавиша
/// считается зажатой, а не нажатой один раз: движение длится, пока клавишу держат. Обе стороны сразу
/// или ни одной — ракетка стоит.
fn read_direction() -> f32 {
    let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

    match (left, right) {
        (true, false) => paddle::LEFT,
        (false, true) => paddle::RIGHT,
        _ => paddle::STILL,
    }
}

fn draw_field(field: Field) {
    clear_background(BACKGROUND_COLOR);
    draw_borders(field);
}

/// Ракетка рисуется тем же светлым цветом, что и разметка поля: оформление минималистичное и
/// контрастное, один цвет на всё, кроме фона.
fn draw_paddle(paddle: &Paddle) {
    draw_rectangle(paddle.x, paddle.y, paddle.width, paddle.height, FIELD_COLOR);
}

/// Мяч рисуется квадратом, а не кругом: так изображение совпадает с прямоугольником, по которому
/// считаются столкновения.
fn draw_ball(ball: &Ball) {
    draw_rectangle(ball.x, ball.y, ball.size, ball.size, FIELD_COLOR);
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
