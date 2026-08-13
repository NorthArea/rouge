use crate::paddle::Paddle;
use crate::Field;

/// Бонус — падающий прямоугольник. Позиция — левый верхний угол, как и во всей отрисовке Macroquad.
const WIDTH: f32 = 28.0;
const HEIGHT: f32 = 12.0;
/// Пикселей в секунду, а не за кадр: скорость падения задана во времени и не зависит от FPS.
const FALL_SPEED: f32 = 190.0;

/// Временный игровой объект: он появляется из уничтоженного блока, живёт, пока падает, и исчезает —
/// пойманным ракеткой или ушедшим за нижнюю границу поля.
pub struct Bonus {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bonus {
    /// Бонус, выпавший из уничтоженного блока: он появляется там, где блок был разбит.
    pub fn dropped_at(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

/// Падение бонусов за один кадр вместе с их концом: пойманный ракеткой расширяет её и уходит с поля,
/// а не пойманный исчезает, покинув поле снизу.
pub fn update(bonuses: &mut Vec<Bonus>, paddle: &mut Paddle, field: Field, delta_time: f32) {
    bonuses.retain_mut(|bonus| {
        bonus.y += FALL_SPEED * delta_time;

        if paddle.overlaps(bonus.x, bonus.y, bonus.width, bonus.height) {
            paddle.widen();
            return false;
        }

        bonus.y <= field.height
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Ракетка теста с круглыми размерами: она занимает 400..500 по горизонтали и 540..556 по
    /// вертикали.
    fn paddle() -> Paddle {
        Paddle {
            x: 400.0,
            y: 540.0,
            width: 100.0,
            height: 16.0,
            speed: 200.0,
            wide_time_left: 0.0,
        }
    }

    /// Бонус прямо над ракеткой: за проверяемый кадр он опустится ей на верхний край.
    fn falling_onto_the_paddle() -> Bonus {
        Bonus {
            x: 440.0,
            y: 530.0,
            width: 28.0,
            height: 12.0,
        }
    }

    #[test]
    fn a_caught_bonus_widens_the_paddle_and_leaves_the_field() {
        let mut bonuses = vec![falling_onto_the_paddle()];
        let mut paddle = paddle();
        let started_at = paddle.width;

        update(&mut bonuses, &mut paddle, field(), 0.1);

        assert!(
            paddle.width > started_at,
            "пойманный бонус не расширил ракетку: ширина {}, была {}",
            paddle.width,
            started_at
        );
        assert!(
            bonuses.is_empty(),
            "пойманный бонус остался на поле: {} штук",
            bonuses.len()
        );
    }

    /// Бонус падает через delta time, как и всё остальное в игре: 60 шагов по `1/60` дают то же, что
    /// 120 шагов по `1/120`.
    #[test]
    fn a_bonus_falls_the_same_distance_regardless_of_frame_rate() {
        let mut at_60_fps = vec![Bonus::dropped_at(100.0, 100.0)];
        let mut at_120_fps = vec![Bonus::dropped_at(100.0, 100.0)];
        let mut paddle = paddle();

        for _ in 0..60 {
            update(&mut at_60_fps, &mut paddle, field(), 1.0 / 60.0);
        }
        for _ in 0..120 {
            update(&mut at_120_fps, &mut paddle, field(), 1.0 / 120.0);
        }

        // Секунда падения со скоростью FALL_SPEED: 100 + FALL_SPEED. Ожидание считается из
        // продакшн-константы, а не из её копии в тесте.
        let expected = 100.0 + FALL_SPEED;
        assert!(
            (at_60_fps[0].y - expected).abs() < TOLERANCE,
            "60 кадров по 1/60: y = {}, ожидалось {}",
            at_60_fps[0].y,
            expected
        );
        assert!(
            (at_60_fps[0].y - at_120_fps[0].y).abs() < TOLERANCE,
            "60 FPS дало y = {}, 120 FPS дало y = {}",
            at_60_fps[0].y,
            at_120_fps[0].y
        );
    }

    #[test]
    fn a_missed_bonus_leaves_the_field_without_widening_the_paddle() {
        // Бонус мимо ракетки по горизонтали (100..128 против 400..500) и уже у нижней границы.
        let mut bonuses = vec![Bonus::dropped_at(100.0, 595.0)];
        let mut paddle = paddle();
        let started_at = paddle.width;

        update(&mut bonuses, &mut paddle, field(), 0.1);

        assert!(
            bonuses.is_empty(),
            "упущенный бонус остался на поле: {} штук",
            bonuses.len()
        );
        assert_eq!(
            paddle.width, started_at,
            "упущенный бонус расширил ракетку: ширина {}",
            paddle.width
        );
    }
}
