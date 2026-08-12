use crate::Field;

/// Значения по умолчанию для мяча. Как и у ракетки, размер и скорость остаются данными: логика читает
/// их из самого мяча, поэтому тест задаёт свои значения и не зависит от игрового баланса.
const SIZE: f32 = 16.0;
/// Пикселей в секунду, а не за кадр: скорость задана во времени, поэтому не зависит от FPS.
const SPEED_X: f32 = 340.0;
const SPEED_Y: f32 = 220.0;

/// Мяч. Позиция — левый верхний угол прямоугольника, как и во всей отрисовке Macroquad. Мяч считается
/// прямоугольником (AABB) и рисуется квадратом, чтобы отображение совпадало с моделью столкновений.
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub size: f32,
}

impl Ball {
    /// Мяч в центре поля с начальной скоростью.
    pub fn new(field: Field) -> Self {
        Self {
            x: (field.width - SIZE) / 2.0,
            y: (field.height - SIZE) / 2.0,
            velocity_x: SPEED_X,
            velocity_y: SPEED_Y,
            size: SIZE,
        }
    }

    /// Перемещение за один кадр. Delta time приходит аргументом, поэтому вычисление ни к чему не
    /// обращается снаружи и вызывается из теста без окна и без Macroquad.
    pub fn update(&mut self, delta_time: f32) {
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
    }

    /// Столкновение с верхней и нижней границами поля и отражение по вертикали. Размеры поля приходят
    /// аргументом, поэтому проверка вызывается из теста без окна.
    pub fn bounce_off_field_edges(&mut self, field: Field) {
        // Позиция мяча — его верхний край, поэтому нижняя граница ограничивает не `y`, а низ мяча.
        let lowest_y = field.height - self.size;

        if self.y < 0.0 {
            // Мяч возвращается на границу, а не остаётся за ней: иначе он остался бы в состоянии
            // столкновения и отражался бы в каждом следующем кадре, дребезжа у границы.
            self.y = 0.0;
            self.velocity_y = -self.velocity_y;
        } else if self.y > lowest_y {
            self.y = lowest_y;
            self.velocity_y = -self.velocity_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Сравнение положений идёт с погрешностью: f32 округляет результат на каждом шаге, а тесты
    /// сравнивают прогоны с разным числом шагов, где число округлений разное.
    const TOLERANCE: f32 = 0.01;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Мяч теста намеренно с круглыми размерами и скоростью: ожидаемые положения в утверждениях
    /// должны читаться как арифметика, а не как результат подстановки игровых констант.
    fn ball() -> Ball {
        Ball {
            x: 100.0,
            y: 200.0,
            velocity_x: 300.0,
            velocity_y: 100.0,
            size: 20.0,
        }
    }

    fn advance(steps: u32, delta_time: f32) -> Ball {
        let mut moving = ball();
        for _ in 0..steps {
            moving.update(delta_time);
        }
        moving
    }

    #[test]
    fn ball_moves_by_velocity_times_delta_time() {
        let mut moving = ball();

        moving.update(0.5);

        // Полшага секунды: 100 + 300 * 0.5 = 250 по горизонтали и 200 + 100 * 0.5 = 250 по вертикали.
        assert!(
            (moving.x - 250.0).abs() < TOLERANCE,
            "шаг не сдвинул мяч по горизонтали: x = {}, ожидалось 250",
            moving.x
        );
        assert!(
            (moving.y - 250.0).abs() < TOLERANCE,
            "шаг не сдвинул мяч по вертикали: y = {}, ожидалось 250",
            moving.y
        );
    }

    #[test]
    fn ball_moves_the_same_distance_regardless_of_frame_rate() {
        let at_60_fps = advance(60, 1.0 / 60.0);
        let at_120_fps = advance(120, 1.0 / 120.0);

        // Одна секунда со скоростью (300, 100) пикселей в секунду: 100 + 300 = 400 и 200 + 100 = 300.
        assert!(
            (at_60_fps.x - 400.0).abs() < TOLERANCE && (at_60_fps.y - 300.0).abs() < TOLERANCE,
            "60 кадров по 1/60: мяч в ({}, {}), ожидалось (400, 300)",
            at_60_fps.x,
            at_60_fps.y
        );
        assert!(
            (at_120_fps.x - 400.0).abs() < TOLERANCE && (at_120_fps.y - 300.0).abs() < TOLERANCE,
            "120 кадров по 1/120: мяч в ({}, {}), ожидалось (400, 300)",
            at_120_fps.x,
            at_120_fps.y
        );
        assert!(
            (at_60_fps.x - at_120_fps.x).abs() < TOLERANCE
                && (at_60_fps.y - at_120_fps.y).abs() < TOLERANCE,
            "60 FPS дало ({}, {}), 120 FPS дало ({}, {})",
            at_60_fps.x,
            at_60_fps.y,
            at_120_fps.x,
            at_120_fps.y
        );
    }

    #[test]
    fn ball_bounces_off_the_top_edge() {
        let mut rising = Ball {
            y: 10.0,
            velocity_y: -400.0,
            ..ball()
        };

        // Кадр уводит мяч за верхнюю границу: 10 - 400 * 0.1 = -30.
        rising.update(0.1);
        rising.bounce_off_field_edges(field());

        assert!(
            (rising.velocity_y - 400.0).abs() < TOLERANCE,
            "верхняя граница не отразила мяч вниз: velocity_y = {}, ожидалось 400",
            rising.velocity_y
        );
        assert!(
            (rising.velocity_x - 300.0).abs() < TOLERANCE,
            "отражение изменило горизонтальную скорость: velocity_x = {}, ожидалось 300",
            rising.velocity_x
        );
        // Мяч, оставленный за границей, столкнулся бы с ней и в следующем кадре и задребезжал бы.
        assert!(
            rising.y >= 0.0 && rising.y + rising.size <= field().height,
            "после отражения мяч не внутри поля: y = {}",
            rising.y
        );
    }

    #[test]
    fn ball_bounces_off_the_bottom_edge() {
        let mut falling = Ball {
            y: 570.0,
            velocity_y: 400.0,
            ..ball()
        };

        // Кадр уводит мяч за нижнюю границу: низ мяча 570 + 400 * 0.1 + 20 = 630 при высоте поля 600.
        falling.update(0.1);
        falling.bounce_off_field_edges(field());

        assert!(
            (falling.velocity_y + 400.0).abs() < TOLERANCE,
            "нижняя граница не отразила мяч вверх: velocity_y = {}, ожидалось -400",
            falling.velocity_y
        );
        assert!(
            (falling.velocity_x - 300.0).abs() < TOLERANCE,
            "отражение изменило горизонтальную скорость: velocity_x = {}, ожидалось 300",
            falling.velocity_x
        );
        assert!(
            falling.y >= 0.0 && falling.y + falling.size <= field().height,
            "после отражения мяч не внутри поля: y = {}",
            falling.y
        );
    }

    /// Обе ветки границы проверяются одним тестом: мяч, летящий внутри поля вверх или вниз, не должен
    /// отражаться только потому, что он движется в сторону границы.
    #[test]
    fn ball_inside_the_field_does_not_bounce() {
        for velocity_y in [-100.0, 100.0] {
            let mut moving = Ball {
                velocity_y,
                ..ball()
            };
            let expected_y = moving.y + velocity_y * 0.5;

            moving.update(0.5);
            moving.bounce_off_field_edges(field());

            assert!(
                (moving.velocity_y - velocity_y).abs() < TOLERANCE,
                "мяч внутри поля отразился: velocity_y = {}, ожидалось {}",
                moving.velocity_y,
                velocity_y
            );
            assert!(
                (moving.y - expected_y).abs() < TOLERANCE,
                "мяч внутри поля сдвинут границей: y = {}, ожидалось {}",
                moving.y,
                expected_y
            );
        }
    }
}
