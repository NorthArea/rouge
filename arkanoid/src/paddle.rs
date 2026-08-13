use crate::Field;

/// Направление горизонтального движения ракетки в текущем кадре. Ось X направлена вправо, поэтому
/// движение влево — отрицательное.
pub const LEFT: f32 = -1.0;
pub const RIGHT: f32 = 1.0;
pub const STILL: f32 = 0.0;

/// Значения по умолчанию для ракетки. В структуре размер и скорость остаются данными: логика читает
/// их из самой ракетки, поэтому тест задаёт свои значения и не зависит от игрового баланса.
const WIDTH: f32 = 120.0;
const HEIGHT: f32 = 16.0;
/// Пикселей в секунду, а не за кадр: скорость задана во времени, поэтому не зависит от FPS.
const SPEED: f32 = 520.0;
/// Отступ ракетки от нижней границы поля.
const BOTTOM_MARGIN: f32 = 40.0;

/// Ракетка игрока. Позиция — левый верхний угол прямоугольника, как и во всей отрисовке Macroquad.
pub struct Paddle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
}

impl Paddle {
    /// Ракетка внизу поля, по центру по горизонтали. В Arkanoid ракетка одна, поэтому у неё один
    /// конструктор, а не пара «левая/правая», как в Pong.
    pub fn new(field: Field) -> Self {
        Self {
            x: (field.width - WIDTH) / 2.0,
            y: field.height - BOTTOM_MARGIN - HEIGHT,
            width: WIDTH,
            height: HEIGHT,
            speed: SPEED,
        }
    }

    /// Обновление положения за один кадр. Размеры поля и delta time приходят аргументами, поэтому
    /// вычисление ни к чему не обращается снаружи и вызывается из теста без окна и без Macroquad.
    pub fn update(&mut self, direction: f32, field: Field, delta_time: f32) {
        let moved = self.x + direction * self.speed * delta_time;

        // Правая граница ограничивает правый край ракетки, а `x` — её левый, поэтому из ширины поля
        // вычитается ширина ракетки.
        self.x = moved.clamp(0.0, field.width - self.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Сравнение положений идёт с погрешностью: f32 округляет результат на каждом шаге, а тест
    /// сравнивает прогоны с разным числом шагов, где число округлений разное.
    const TOLERANCE: f32 = 0.01;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Ракетка теста намеренно с круглыми размерами и скоростью: ожидаемые положения в утверждениях
    /// должны читаться как арифметика, а не как результат подстановки игровых констант.
    fn paddle() -> Paddle {
        Paddle {
            x: 400.0,
            y: 540.0,
            width: 100.0,
            height: 16.0,
            speed: 200.0,
        }
    }

    fn advance(direction: f32, steps: u32, delta_time: f32) -> Paddle {
        let mut paddle = paddle();
        for _ in 0..steps {
            paddle.update(direction, field(), delta_time);
        }
        paddle
    }

    #[test]
    fn paddle_moves_the_same_distance_regardless_of_frame_rate() {
        let at_60_fps = advance(RIGHT, 60, 1.0 / 60.0);
        let at_120_fps = advance(RIGHT, 120, 1.0 / 120.0);

        // Одна секунда вправо со скоростью 200 пикселей в секунду: 400 + 200 = 600.
        assert!(
            (at_60_fps.x - 600.0).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось 600",
            at_60_fps.x
        );
        assert!(
            (at_120_fps.x - 600.0).abs() < TOLERANCE,
            "120 кадров по 1/120: x = {}, ожидалось 600",
            at_120_fps.x
        );
        assert!(
            (at_60_fps.x - at_120_fps.x).abs() < TOLERANCE,
            "60 FPS дало x = {}, 120 FPS дало x = {}",
            at_60_fps.x,
            at_120_fps.x
        );
    }

    #[test]
    fn paddle_without_input_does_not_move() {
        let untouched = advance(STILL, 60, 1.0 / 60.0);

        assert!(
            (untouched.x - 400.0).abs() < TOLERANCE,
            "секунда без ввода сдвинула ракетку: x = {}, ожидалось 400",
            untouched.x
        );
    }

    #[test]
    fn paddle_stops_at_the_left_edge() {
        // Три секунды влево — заведомо больше, чем 400 пикселей до левой границы.
        let pressed_left = advance(LEFT, 180, 1.0 / 60.0);

        assert!(
            (pressed_left.x - 0.0).abs() < TOLERANCE,
            "ракетка ушла за левую границу: x = {}, ожидалось 0",
            pressed_left.x
        );
    }

    #[test]
    fn paddle_stops_at_the_right_edge() {
        // Правая граница ограничивает не левый край ракетки, а её правый: 960 - 100 = 860.
        let pressed_right = advance(RIGHT, 180, 1.0 / 60.0);

        assert!(
            (pressed_right.x - 860.0).abs() < TOLERANCE,
            "ракетка ушла за правую границу: x = {}, ожидалось 860",
            pressed_right.x
        );
    }
}
