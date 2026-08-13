use crate::ball::Ball;
use crate::score::Score;
use crate::Field;

/// Очки за уничтоженный блок.
const POINTS: u32 = 100;

/// Раскладка блоков задана прямо в коде: ни внешнего редактора, ни загрузки из файлов. Пока раскладка
/// одна — сетка из рядов и столбцов под верхней границей поля.
const ROWS: u32 = 5;
const COLUMNS: u32 = 10;
const HEIGHT: f32 = 24.0;
/// Зазор между блоками и отступ сетки от краёв поля и от его верхней границы.
const GAP: f32 = 6.0;
const SIDE_MARGIN: f32 = 40.0;
const TOP_MARGIN: f32 = 60.0;

/// Блок игрового поля. Позиция — левый верхний угол прямоугольника, как и во всей отрисовке
/// Macroquad. Уничтоженный блок остаётся в коллекции, но выбывает из игры: так жизненный цикл
/// игрового объекта виден в одном месте и не требует перестройки коллекции посреди кадра.
pub struct Brick {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub destroyed: bool,
}

/// Раскладка блоков для поля: сетка `ROWS` × `COLUMNS`, растянутая по ширине поля. Ширина блока
/// считается из ширины поля, поэтому сетка занимает её целиком при любом размере окна.
pub fn layout(field: Field) -> Vec<Brick> {
    let row_width = field.width - 2.0 * SIDE_MARGIN;
    let width = (row_width - GAP * (COLUMNS - 1) as f32) / COLUMNS as f32;

    let mut bricks = Vec::new();
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            bricks.push(Brick {
                x: SIDE_MARGIN + column as f32 * (width + GAP),
                y: TOP_MARGIN + row as f32 * (HEIGHT + GAP),
                width,
                height: HEIGHT,
                destroyed: false,
            });
        }
    }

    bricks
}

/// Столкновение мяча с коллекцией блоков за один кадр.
pub fn bounce_off_bricks(ball: &mut Ball, bricks: &mut [Brick], score: &mut Score) {
    for brick in bricks {
        // Уничтоженный блок выбывает из игры целиком: он не сталкивается, не приносит очки и не
        // рисуется. Это и есть жизненный цикл игрового объекта в этой игре.
        if brick.destroyed || !ball.overlaps(brick.x, brick.y, brick.width, brick.height) {
            continue;
        }

        brick.destroyed = true;
        score.points += POINTS;
        flip_entry_axis(ball, brick);

        // За кадр мяч отскакивает не более одного раза: на стыке двух блоков второй отскок вернул бы
        // скорость к исходной, и мяч прошёл бы сквозь стену, не изменив направления.
        break;
    }
}

/// Сторона входа определяется тем, по какой оси перекрытие меньше: мяч, влетевший снизу, перекрывает
/// блок по вертикали на меньшую глубину, чем по горизонтали, и наоборот. Меняет знак та составляющая
/// скорости, вдоль которой мяч и вошёл.
fn flip_entry_axis(ball: &mut Ball, brick: &Brick) {
    let overlap_x = (ball.x + ball.size).min(brick.x + brick.width) - ball.x.max(brick.x);
    let overlap_y = (ball.y + ball.size).min(brick.y + brick.height) - ball.y.max(brick.y);

    if overlap_x < overlap_y {
        ball.velocity_x = -ball.velocity_x;
    } else {
        ball.velocity_y = -ball.velocity_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    /// Блок теста намеренно с круглыми размерами: он занимает 300..380 по горизонтали и 100..124 по
    /// вертикали.
    fn brick() -> Brick {
        Brick {
            x: 300.0,
            y: 100.0,
            width: 80.0,
            height: 24.0,
            destroyed: false,
        }
    }

    /// Мяч, подошедший к блоку снизу: он занимает 330..350 по горизонтали (целиком в пределах блока)
    /// и 114..134 по вертикали, то есть перекрывает блок на 20 по горизонтали и на 10 по вертикали.
    /// Меньшее перекрытие вертикальное, значит вошёл он снизу.
    fn hitting_from_below() -> Ball {
        Ball {
            x: 330.0,
            y: 114.0,
            velocity_x: 0.0,
            velocity_y: -300.0,
            size: 20.0,
        }
    }

    #[test]
    fn a_hit_destroys_the_brick() {
        let mut ball = hitting_from_below();
        let mut bricks = vec![brick()];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        assert!(
            bricks[0].destroyed,
            "попадание не уничтожило блок: destroyed = {}",
            bricks[0].destroyed
        );
    }

    #[test]
    fn a_frame_without_a_collision_changes_nothing() {
        // Мяч далеко от блока: 100..120 против 300..380 по горизонтали и 400..420 против 100..124
        // по вертикали.
        let mut ball = Ball {
            x: 100.0,
            y: 400.0,
            ..hitting_from_below()
        };
        let mut bricks = vec![brick()];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        assert!(!bricks[0].destroyed, "кадр без столкновения уничтожил блок");
        assert!(
            (ball.velocity_y + 300.0).abs() < TOLERANCE && ball.velocity_x.abs() < TOLERANCE,
            "кадр без столкновения изменил скорость: velocity = ({}, {}), ожидалось (0, -300)",
            ball.velocity_x,
            ball.velocity_y
        );
        assert_eq!(
            score.points, 0,
            "кадр без столкновения изменил счёт: {}",
            score.points
        );
    }

    #[test]
    fn destroying_a_brick_adds_one_hundred_points() {
        let mut ball = hitting_from_below();
        let mut bricks = vec![brick()];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        assert_eq!(
            score.points, 100,
            "уничтоженный блок не принёс очков: счёт {}, ожидалось 100",
            score.points
        );
    }

    /// Обе горизонтальные стороны блока проверяются одним тестом: снизу и сверху вход одинаково
    /// разворачивает мяч по вертикали, отличается только знак.
    #[test]
    fn a_hit_from_below_or_above_flips_the_vertical_velocity() {
        // Снизу: мяч 114..134 против блока 100..124 — перекрытие 10 по вертикали и 20 по горизонтали.
        // Сверху: мяч 90..110 — перекрытие тоже 10 по вертикали, но мяч летит вниз.
        let entries = [(114.0, -300.0, 300.0), (90.0, 300.0, -300.0)];

        for (y, velocity_y, expected_y) in entries {
            let mut ball = Ball {
                y,
                velocity_y,
                ..hitting_from_below()
            };
            let mut bricks = vec![brick()];
            let mut score = Score::default();

            bounce_off_bricks(&mut ball, &mut bricks, &mut score);

            assert!(
                (ball.velocity_y - expected_y).abs() < TOLERANCE,
                "вход с y = {} не развернул мяч по вертикали: velocity_y = {}, ожидалось {}",
                y,
                ball.velocity_y,
                expected_y
            );
            assert!(
                ball.velocity_x.abs() < TOLERANCE,
                "вход с y = {} изменил горизонтальную скорость: velocity_x = {}, ожидалось 0",
                y,
                ball.velocity_x
            );
        }
    }

    #[test]
    fn a_hit_from_the_side_flips_the_horizontal_velocity() {
        // Мяч подошёл слева: 290..310 против блока 300..380 — перекрытие 10 по горизонтали, а по
        // вертикали 105..125 против 100..124 даёт 19. Меньшее перекрытие горизонтальное.
        let mut ball = Ball {
            x: 290.0,
            y: 105.0,
            velocity_x: 300.0,
            velocity_y: 0.0,
            ..hitting_from_below()
        };
        let mut bricks = vec![brick()];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        assert!(
            (ball.velocity_x + 300.0).abs() < TOLERANCE,
            "вход сбоку не развернул мяч по горизонтали: velocity_x = {}, ожидалось -300",
            ball.velocity_x
        );
        assert!(
            ball.velocity_y.abs() < TOLERANCE,
            "вход сбоку изменил вертикальную скорость: velocity_y = {}, ожидалось 0",
            ball.velocity_y
        );
    }

    /// Жизненный цикл игрового объекта: уничтоженный блок остаётся в коллекции, но выбывает из игры.
    /// Второй проход мяча через его место не должен ни отражать мяч, ни приносить очки.
    #[test]
    fn a_destroyed_brick_takes_no_further_part() {
        let mut ball = hitting_from_below();
        let mut bricks = vec![brick()];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        // Мяч возвращается на то же место с той же скоростью: блока там больше нет.
        let mut passing_again = hitting_from_below();
        bounce_off_bricks(&mut passing_again, &mut bricks, &mut score);

        assert!(
            (passing_again.velocity_y + 300.0).abs() < TOLERANCE,
            "уничтоженный блок отразил мяч: velocity_y = {}, ожидалось -300",
            passing_again.velocity_y
        );
        assert_eq!(
            score.points, 100,
            "уничтоженный блок начислил очки повторно: счёт {}, ожидалось 100",
            score.points
        );
    }

    /// Этого пункта нет в списке D-18, но без него коллекция ведёт себя хуже одиночного блока: два
    /// отскока за кадр гасят друг друга, и мяч проходит сквозь стык блоков, не изменив направления.
    #[test]
    fn a_ball_touching_two_bricks_bounces_once() {
        // Мяч 365..385 задевает и блок 300..380, и соседний 380..460, подходя к ним снизу.
        let mut ball = Ball {
            x: 365.0,
            velocity_x: 100.0,
            ..hitting_from_below()
        };
        let mut bricks = vec![
            brick(),
            Brick {
                x: 380.0,
                ..brick()
            },
        ];
        let mut score = Score::default();

        bounce_off_bricks(&mut ball, &mut bricks, &mut score);

        assert!(
            (ball.velocity_y - 300.0).abs() < TOLERANCE
                && (ball.velocity_x - 100.0).abs() < TOLERANCE,
            "мяч на стыке блоков отразился дважды: velocity = ({}, {}), ожидалось (100, 300)",
            ball.velocity_x,
            ball.velocity_y
        );
        assert_eq!(
            bricks.iter().filter(|brick| brick.destroyed).count(),
            1,
            "кадр уничтожил больше одного блока"
        );
    }
}
