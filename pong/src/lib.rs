use macroquad::prelude::*;

mod ball;
mod game;
mod paddle;
mod score;

use ball::Ball;
use game::{Game, GameState};
use paddle::Paddle;
use score::Score;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

const BORDER_THICKNESS: f32 = 6.0;
const CENTER_LINE_WIDTH: f32 = 6.0;
const CENTER_LINE_DASH: f32 = 20.0;
const CENTER_LINE_GAP: f32 = 16.0;

const SCORE_FONT_SIZE: u16 = 56;
/// Отступ счёта от центральной линии и высота его строки от верхней границы поля.
const SCORE_MARGIN: f32 = 48.0;
const SCORE_BASELINE: f32 = 96.0;

/// Подсказка о том, как начать раунд. Показывается только в ожидании подачи, поэтому не мешает игре.
/// Текст латиницей: встроенный шрифт Macroquad не гарантирует кириллических глифов, а проверить это
/// автотестом нельзя — рендеринг не тестируется.
const HINT_TEXT: &str = "PRESS SPACE TO SERVE";
const HINT_FONT_SIZE: u16 = 28;
const HINT_BASELINE_FROM_BOTTOM: f32 = 64.0;

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
        // Старт раунда и рестарт матча — разовые нажатия, а не удержание: игра реагирует на нажатие
        // один раз, сколько бы кадров клавишу ни держали.
        if is_key_pressed(KeyCode::Space) {
            game.start_round();
        }
        if is_key_pressed(KeyCode::R) {
            game.restart_match();
        }
        let left_direction = read_direction(KeyCode::W, KeyCode::S);
        let right_direction = read_direction(KeyCode::Up, KeyCode::Down);

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния и 4. проверка столкновений. Обе стадии зависят от состояния игры,
        // поэтому они перешли внутрь `Game::update` и перечислены в том же порядке там: до подачи
        // двигаются только ракетки, в раунде мяч летит, сталкивается и приносит гол, а в кадре гола
        // возвращается в центр.
        game.update(left_direction, right_direction, delta_time);

        // 5. Рендеринг.
        draw_field(field);
        draw_score(&game.score, field);
        draw_paddle(&game.left_paddle);
        draw_paddle(&game.right_paddle);
        draw_ball(&game.ball);
        if game.state == GameState::WaitingToStart {
            draw_hint(field);
        }

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

/// Счёт — текст по обе стороны от центральной линии. Левый счёт выравнивается по правому краю, для
/// чего его ширину нужно измерить: двузначный счёт иначе наехал бы на линию.
fn draw_score(score: &Score, field: Field) {
    let left = score.left.to_string();
    let right = score.right.to_string();
    let center_x = field.width / 2.0;
    let left_width = measure_text(&left, None, SCORE_FONT_SIZE, 1.0).width;

    draw_text(
        &left,
        center_x - SCORE_MARGIN - left_width,
        SCORE_BASELINE,
        f32::from(SCORE_FONT_SIZE),
        FIELD_COLOR,
    );
    draw_text(
        &right,
        center_x + SCORE_MARGIN,
        SCORE_BASELINE,
        f32::from(SCORE_FONT_SIZE),
        FIELD_COLOR,
    );
}

/// Подсказка про `Space` внизу поля, по центру. Ширину строки нужно измерить: центрировать текст
/// иначе нечем.
fn draw_hint(field: Field) {
    let width = measure_text(HINT_TEXT, None, HINT_FONT_SIZE, 1.0).width;

    draw_text(
        HINT_TEXT,
        (field.width - width) / 2.0,
        field.height - HINT_BASELINE_FROM_BOTTOM,
        f32::from(HINT_FONT_SIZE),
        FIELD_COLOR,
    );
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
