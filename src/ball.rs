use crate::paddle::Paddle;
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

    /// Столкновение с ракеткой и отражение от неё. Ракетка приходит аргументом, поэтому проверка
    /// вызывается из теста без окна. Отражение не сводится к смене знака горизонтальной скорости:
    /// вертикальная составляющая зависит от того, куда по ракетке пришёлся удар.
    pub fn bounce_off_paddle(&mut self, paddle: &Paddle) {
        if !self.overlaps(paddle) || !self.moves_toward(paddle) {
            return;
        }

        let speed_x = self.velocity_x.abs();

        self.velocity_x = -self.velocity_x;
        // Вертикальная составляющая задана долей той же горизонтальной скорости, а не отдельной
        // константой: величина `velocity_x` при отскоке не меняется, поэтому мяч не может разогнаться
        // от удара к удару, а на самом краю ракетки уходит ровно под 45°.
        self.velocity_y = self.hit_offset(paddle) * speed_x;

        // Мяч ставится вплотную к отбившей стороне ракетки: проверка направления и так не даст ему
        // отразиться второй раз, но оставленный внутри ракетки мяч выглядел бы залипшим в ней.
        self.x = if self.velocity_x > 0.0 {
            paddle.x + paddle.width
        } else {
            paddle.x - self.size
        };
    }

    /// Место попадания по вертикали: -1 — верхний край ракетки, 0 — её центр, 1 — нижний край. Мяч
    /// может задеть ракетку самым краем, и тогда его центр окажется за пределами ракетки, поэтому
    /// смещение ограничивается диапазоном.
    fn hit_offset(&self, paddle: &Paddle) -> f32 {
        let ball_center = self.y + self.size / 2.0;
        let paddle_center = paddle.y + paddle.height / 2.0;

        ((ball_center - paddle_center) / (paddle.height / 2.0)).clamp(-1.0, 1.0)
    }

    /// Пересечение двух прямоугольников (AABB), написанное вручную: physics engine в проекте нет.
    /// Прямоугольники не пересекаются, если один целиком левее, правее, выше или ниже другого, —
    /// значит пересекаются они тогда, когда неверно всё это сразу.
    fn overlaps(&self, paddle: &Paddle) -> bool {
        self.x < paddle.x + paddle.width
            && self.x + self.size > paddle.x
            && self.y < paddle.y + paddle.height
            && self.y + self.size > paddle.y
    }

    /// Мяч летит к ракетке, а не от неё. Одного пересечения для отскока мало: мяч, который уже
    /// отбит и ещё не успел выйти из ракетки, иначе отражался бы в каждом кадре и залип бы в ней.
    fn moves_toward(&self, paddle: &Paddle) -> bool {
        let ball_center = self.x + self.size / 2.0;
        let paddle_center = paddle.x + paddle.width / 2.0;

        if paddle_center > ball_center {
            self.velocity_x > 0.0
        } else {
            self.velocity_x < 0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paddle::Paddle;

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

    /// Ракетки теста тоже с круглыми размерами: левая занимает 40..56 по горизонтали и 250..350 по
    /// вертикали, её центр по вертикали — 300.
    fn left_paddle() -> Paddle {
        Paddle {
            x: 40.0,
            y: 250.0,
            width: 16.0,
            height: 100.0,
            speed: 200.0,
        }
    }

    /// Правая ракетка отличается только положением по горизонтали: 900..916.
    fn right_paddle() -> Paddle {
        Paddle {
            x: 900.0,
            ..left_paddle()
        }
    }

    /// Мяч уже пересекается с левой ракеткой (50..70 против 40..56) и летит влево, а по вертикали
    /// его центр совпадает с центром ракетки: 290 + 20 / 2 = 300.
    fn hitting_left_paddle() -> Ball {
        Ball {
            x: 50.0,
            y: 290.0,
            velocity_x: -300.0,
            ..ball()
        }
    }

    /// Симметрично для правой ракетки: мяч 890..910 против 900..916 и летит вправо.
    fn hitting_right_paddle() -> Ball {
        Ball {
            x: 890.0,
            y: 290.0,
            ..ball()
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

    #[test]
    fn ball_bounces_right_off_the_left_paddle() {
        let mut hitting = hitting_left_paddle();

        hitting.bounce_off_paddle(&left_paddle());

        assert!(
            (hitting.velocity_x - 300.0).abs() < TOLERANCE,
            "левая ракетка не отправила мяч вправо: velocity_x = {}, ожидалось 300",
            hitting.velocity_x
        );
    }

    #[test]
    fn ball_bounces_left_off_the_right_paddle() {
        let mut hitting = hitting_right_paddle();

        hitting.bounce_off_paddle(&right_paddle());

        assert!(
            (hitting.velocity_x + 300.0).abs() < TOLERANCE,
            "правая ракетка не отправила мяч влево: velocity_x = {}, ожидалось -300",
            hitting.velocity_x
        );
    }

    #[test]
    fn hit_at_the_paddle_center_sends_the_ball_almost_level() {
        let mut hitting = hitting_left_paddle();

        hitting.bounce_off_paddle(&left_paddle());

        assert!(
            hitting.velocity_y.abs() < TOLERANCE,
            "попадание в центр ракетки увело мяч по вертикали: velocity_y = {}, ожидалось около 0",
            hitting.velocity_y
        );
    }

    #[test]
    fn hit_above_the_paddle_center_sends_the_ball_up() {
        // Центр мяча 265 + 20 / 2 = 275 против центра ракетки 300: половина верхней половины ракетки,
        // то есть смещение -0.5, а вертикальная скорость -0.5 * 300 = -150.
        let mut hitting = Ball {
            y: 265.0,
            ..hitting_left_paddle()
        };

        hitting.bounce_off_paddle(&left_paddle());

        assert!(
            (hitting.velocity_y + 150.0).abs() < TOLERANCE,
            "попадание выше центра не отправило мяч вверх: velocity_y = {}, ожидалось -150",
            hitting.velocity_y
        );
    }

    #[test]
    fn hit_below_the_paddle_center_sends_the_ball_down() {
        // Зеркально предыдущему тесту: центр мяча 315 + 20 / 2 = 325, смещение 0.5, скорость 150.
        let mut hitting = Ball {
            y: 315.0,
            ..hitting_left_paddle()
        };

        hitting.bounce_off_paddle(&left_paddle());

        assert!(
            (hitting.velocity_y - 150.0).abs() < TOLERANCE,
            "попадание ниже центра не отправило мяч вниз: velocity_y = {}, ожидалось 150",
            hitting.velocity_y
        );
    }

    #[test]
    fn ball_missing_the_paddle_keeps_its_velocity() {
        // Промах по вертикали (мяч 100..120 против ракетки 250..350) и промах по горизонтали
        // (мяч 200..220 против ракетки 40..56): пересечения нет ни по одной из осей.
        let misses = [
            Ball {
                y: 100.0,
                ..hitting_left_paddle()
            },
            Ball {
                x: 200.0,
                ..hitting_left_paddle()
            },
        ];

        for mut missing in misses {
            let (x, y) = (missing.x, missing.y);

            missing.bounce_off_paddle(&left_paddle());

            assert!(
                (missing.velocity_x + 300.0).abs() < TOLERANCE
                    && (missing.velocity_y - 100.0).abs() < TOLERANCE,
                "промах мимо ракетки из ({}, {}) изменил скорость: velocity = ({}, {}), ожидалось (-300, 100)",
                x,
                y,
                missing.velocity_x,
                missing.velocity_y
            );
        }
    }

    #[test]
    fn ball_already_moving_away_from_the_paddle_does_not_bounce_again() {
        // Мяч всё ещё пересекается с левой ракеткой, но уже отскочил и летит вправо. Второй отскок
        // развернул бы его обратно в ракетку, и мяч залип бы в ней, разворачиваясь в каждом кадре.
        let mut leaving = Ball {
            velocity_x: 300.0,
            ..hitting_left_paddle()
        };

        leaving.bounce_off_paddle(&left_paddle());

        assert!(
            (leaving.velocity_x - 300.0).abs() < TOLERANCE
                && (leaving.velocity_y - 100.0).abs() < TOLERANCE,
            "мяч, летящий от ракетки, отразился второй раз: velocity = ({}, {}), ожидалось (300, 100)",
            leaving.velocity_x,
            leaving.velocity_y
        );
    }

    #[test]
    fn bounce_puts_the_ball_outside_the_paddle() {
        let mut from_the_left = hitting_left_paddle();
        let mut from_the_right = hitting_right_paddle();

        from_the_left.bounce_off_paddle(&left_paddle());
        from_the_right.bounce_off_paddle(&right_paddle());

        // Мяч, оставленный внутри ракетки, выглядел бы залипшим в ней, пока из неё выбирается.
        assert!(
            from_the_left.x >= left_paddle().x + left_paddle().width - TOLERANCE,
            "мяч остался внутри левой ракетки: x = {}, ожидалось не левее 56",
            from_the_left.x
        );
        assert!(
            from_the_right.x + from_the_right.size <= right_paddle().x + TOLERANCE,
            "мяч остался внутри правой ракетки: правый край = {}, ожидалось не правее 900",
            from_the_right.x + from_the_right.size
        );
    }
}
