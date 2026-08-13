use macroquad::prelude::*;

mod ball;
mod bonus;
mod brick;
mod game;
mod level;
mod paddle;
mod score;

use ball::Ball;
use bonus::Bonus;
use brick::{Brick, BrickKind};
use game::Game;
use paddle::Paddle;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);
/// Цвет блока показывает его тип, а повреждённый прочный блок отличается от целого: различие живёт
/// в данных блока, поэтому цвет — лишь его отображение.
const NORMAL_BRICK_COLOR: Color = Color::new(0.35, 0.62, 0.86, 1.0);
const STRONG_BRICK_COLOR: Color = Color::new(0.90, 0.65, 0.25, 1.0);
const DAMAGED_BRICK_COLOR: Color = Color::new(0.55, 0.40, 0.18, 1.0);
const INDESTRUCTIBLE_BRICK_COLOR: Color = Color::new(0.45, 0.47, 0.52, 1.0);
/// Бонус выделен собственным цветом: игрок должен отличать его от блока с первого взгляда.
const BONUS_COLOR: Color = Color::new(0.42, 0.83, 0.45, 1.0);

const BORDER_THICKNESS: f32 = 6.0;

/// Строка состояния: счёт, жизни и номер уровня одной строкой над полем блоков.
const STATUS_FONT_SIZE: u16 = 28;
const STATUS_MARGIN: f32 = 24.0;
const STATUS_BASELINE: f32 = 38.0;

/// Сообщение состояния — крупная надпись по центру поля.
const MESSAGE_FONT_SIZE: u16 = 44;

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
    let mut game = Game::new(field);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке: порядок стадий — часть
        // того, что этот проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        // Старт раунда — разовое нажатие, а не удержание: игра реагирует на нажатие один раз,
        // сколько бы кадров клавишу ни держали.
        if is_key_pressed(KeyCode::Space) {
            game.start_round();
        }
        if is_key_pressed(KeyCode::R) {
            game.restart();
        }
        let direction = read_direction();

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния и 4. проверка столкновений. Обе стадии зависят от состояния игры,
        //    поэтому они перешли внутрь `Game::update` и перечислены в том же порядке там: до подачи
        //    мяч лежит на ракетке, в раунде летит, сталкивается и может быть потерян.
        game.update(direction, delta_time);

        // 5. Рендеринг.
        draw_field(field);
        draw_bricks(&game.bricks);
        draw_paddle(&game.paddle);
        draw_ball(&game.ball);
        draw_bonuses(&game.bonuses);
        draw_status(&game);
        draw_message(&game, field);

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

/// Уничтоженный блок не рисуется: он выбыл из игры целиком, а не только из проверки столкновений.
fn draw_bricks(bricks: &[Brick]) {
    for brick in bricks.iter().filter(|brick| !brick.destroyed()) {
        draw_rectangle(
            brick.x,
            brick.y,
            brick.width,
            brick.height,
            brick_color(brick),
        );
    }
}

/// Цвет выводится из данных блока: тип и оставшаяся прочность. Повреждённый прочный блок темнее
/// целого, поэтому игрок видит, что удар засчитан.
fn brick_color(brick: &Brick) -> Color {
    match brick.kind {
        BrickKind::Normal => NORMAL_BRICK_COLOR,
        BrickKind::Strong if brick.hits_left > 1 => STRONG_BRICK_COLOR,
        BrickKind::Strong => DAMAGED_BRICK_COLOR,
        BrickKind::Indestructible => INDESTRUCTIBLE_BRICK_COLOR,
    }
}

/// Падающие бонусы. Пойманный или упущенный бонус уже удалён из коллекции, поэтому рисовать нечего.
fn draw_bonuses(bonuses: &[Bonus]) {
    for bonus in bonuses {
        draw_rectangle(bonus.x, bonus.y, bonus.width, bonus.height, BONUS_COLOR);
    }
}

/// Счёт, жизни и уровень — одна строка в левом верхнем углу поля.
fn draw_status(game: &Game) {
    let text = format!(
        "SCORE {}    LIVES {}    LEVEL {}",
        game.score.points, game.lives, game.level
    );

    draw_text(
        &text,
        STATUS_MARGIN,
        STATUS_BASELINE,
        f32::from(STATUS_FONT_SIZE),
        FIELD_COLOR,
    );
}

/// Сообщение текущего состояния по центру поля. В идущем раунде сообщения нет, и рисовать нечего:
/// какое именно сообщение показывает состояние, решает сама игра, а здесь остаётся только вывод.
fn draw_message(game: &Game, field: Field) {
    let Some(message) = game.state.message() else {
        return;
    };

    // Ширину строки нужно измерить: центрировать текст иначе нечем.
    let width = measure_text(message, None, MESSAGE_FONT_SIZE, 1.0).width;

    draw_text(
        message,
        (field.width - width) / 2.0,
        field.height / 2.0,
        f32::from(MESSAGE_FONT_SIZE),
        FIELD_COLOR,
    );
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
