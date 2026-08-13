use crate::Field;

/// Направление вертикального движения ракетки в текущем кадре. Ось Y в Macroquad направлена вниз,
/// поэтому движение вверх — отрицательное.
pub const UP: f32 = -1.0;
pub const DOWN: f32 = 1.0;
pub const STILL: f32 = 0.0;

/// Значения по умолчанию для обеих ракеток. В структуре размер и скорость остаются данными: логика
/// читает их из самой ракетки, поэтому тест задаёт свои значения и не зависит от игрового баланса.
const WIDTH: f32 = 16.0;
const HEIGHT: f32 = 96.0;
/// Пикселей в секунду, а не за кадр: скорость задана во времени, поэтому не зависит от FPS.
const SPEED: f32 = 420.0;
/// Отступ ракетки от боковой границы поля.
const EDGE_MARGIN: f32 = 24.0;

/// Ракетка игрока. Позиция — левый верхний угол прямоугольника, как и во всей отрисовке Macroquad.
pub struct Paddle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
}

impl Paddle {
    /// Левая ракетка: у левой границы поля, по центру по вертикали.
    pub fn left(field: Field) -> Self {
        Self::new(EDGE_MARGIN, field)
    }

    /// Правая ракетка: симметрично у правой границы.
    pub fn right(field: Field) -> Self {
        Self::new(field.width - EDGE_MARGIN - WIDTH, field)
    }

    fn new(x: f32, field: Field) -> Self {
        Self {
            x,
            y: (field.height - HEIGHT) / 2.0,
            width: WIDTH,
            height: HEIGHT,
            speed: SPEED,
        }
    }

    /// Обновление положения за один кадр. Размеры поля и delta time приходят аргументами, поэтому
    /// вычисление ни к чему не обращается снаружи и вызывается из теста без окна и без Macroquad.
    pub fn update(&mut self, direction: f32, field: Field, delta_time: f32) {
        let moved = self.y + direction * self.speed * delta_time;

        // Нижняя граница ограничивает низ ракетки, а `y` — её верх, поэтому из высоты поля
        // вычитается высота ракетки.
        self.y = moved.clamp(0.0, field.height - self.height);
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
            x: 40.0,
            y: 250.0,
            width: 16.0,
            height: 100.0,
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
        let at_60_fps = advance(DOWN, 60, 1.0 / 60.0);
        let at_120_fps = advance(DOWN, 120, 1.0 / 120.0);

        // Одна секунда вниз со скоростью 200 пикселей в секунду: 250 + 200 = 450.
        assert!(
            (at_60_fps.y - 450.0).abs() < TOLERANCE,
            "60 кадров по 1/60: y = {}, ожидалось 450",
            at_60_fps.y
        );
        assert!(
            (at_120_fps.y - 450.0).abs() < TOLERANCE,
            "120 кадров по 1/120: y = {}, ожидалось 450",
            at_120_fps.y
        );
        assert!(
            (at_60_fps.y - at_120_fps.y).abs() < TOLERANCE,
            "60 FPS дало y = {}, 120 FPS дало y = {}",
            at_60_fps.y,
            at_120_fps.y
        );
    }

    #[test]
    fn paddle_without_input_does_not_move() {
        let untouched = advance(STILL, 60, 1.0 / 60.0);

        assert!(
            (untouched.y - 250.0).abs() < TOLERANCE,
            "секунда без ввода сдвинула ракетку: y = {}, ожидалось 250",
            untouched.y
        );
    }

    #[test]
    fn paddle_stops_at_the_top_edge() {
        // Две секунды вверх — заведомо больше, чем 250 пикселей до верхней границы.
        let pressed_up = advance(UP, 120, 1.0 / 60.0);

        assert!(
            (pressed_up.y - 0.0).abs() < TOLERANCE,
            "ракетка ушла за верхнюю границу: y = {}, ожидалось 0",
            pressed_up.y
        );
    }

    #[test]
    fn paddle_stops_at_the_bottom_edge() {
        // Нижняя граница ограничивает не верхний край ракетки, а её низ: 600 - 100 = 500.
        let pressed_down = advance(DOWN, 120, 1.0 / 60.0);

        assert!(
            (pressed_down.y - 500.0).abs() < TOLERANCE,
            "ракетка ушла за нижнюю границу: y = {}, ожидалось 500",
            pressed_down.y
        );
    }
}
