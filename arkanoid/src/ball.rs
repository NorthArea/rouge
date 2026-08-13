use crate::paddle::Paddle;
use crate::Field;

/// Значения по умолчанию для мяча. Как и у ракетки, размер и скорость остаются данными: логика читает
/// их из самого мяча, поэтому тест задаёт свои значения и не зависит от игрового баланса.
const SIZE: f32 = 14.0;
/// Пикселей в секунду, а не за кадр: скорость задана во времени, поэтому не зависит от FPS.
const SPEED_X: f32 = 260.0;
const SPEED_Y: f32 = 320.0;

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
    /// Мяч, лежащий на ракетке и готовый к подаче. Раунд всегда начинается отсюда — и в начале игры,
    /// и после потерянной жизни, — поэтому правило живёт в одном месте.
    pub fn resting_on(paddle: &Paddle) -> Self {
        let mut ball = Self {
            x: 0.0,
            y: 0.0,
            velocity_x: SPEED_X,
            // Ось Y направлена вниз, поэтому подача вверх — отрицательная вертикальная скорость.
            velocity_y: -SPEED_Y,
            size: SIZE,
        };

        ball.rest_on(paddle);
        ball
    }

    /// Мяч ставится по центру ракетки и вплотную над ней. Пока раунд не начат, мяч ездит вместе с
    /// ракеткой, поэтому это же правило вызывается каждый кадр ожидания.
    pub fn rest_on(&mut self, paddle: &Paddle) {
        self.x = paddle.x + (paddle.width - self.size) / 2.0;
        self.y = paddle.y - self.size;
    }

    /// Перемещение за один кадр. Delta time приходит аргументом, поэтому вычисление ни к чему не
    /// обращается снаружи и вызывается из теста без окна и без Macroquad.
    pub fn update(&mut self, delta_time: f32) {
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
    }

    /// Столкновение со стенами поля и отражение от них. Размеры поля приходят аргументом, поэтому
    /// проверка вызывается из теста без окна. Нижняя граница стеной не является: ушедший вниз мяч
    /// не возвращается — его подхватит потеря жизни.
    pub fn bounce_off_walls(&mut self, field: Field) {
        // Позиция мяча — его левый край, поэтому правая граница ограничивает не `x`, а правый край.
        let rightmost_x = field.width - self.size;

        if self.x < 0.0 {
            // Мяч возвращается на границу, а не остаётся за ней: иначе он остался бы в состоянии
            // столкновения и отражался бы в каждом следующем кадре, дребезжа у границы.
            self.x = 0.0;
            self.velocity_x = -self.velocity_x;
        } else if self.x > rightmost_x {
            self.x = rightmost_x;
            self.velocity_x = -self.velocity_x;
        }

        // Верхняя граница — единственная горизонтальная стена: нижней в Arkanoid нет, и ушедший
        // вниз мяч не возвращается.
        if self.y < 0.0 {
            self.y = 0.0;
            self.velocity_y = -self.velocity_y;
        }
    }

    /// Столкновение с ракеткой и отражение от неё. Ракетка приходит аргументом, поэтому проверка
    /// вызывается из теста без окна. Отражение не сводится к смене знака вертикальной скорости:
    /// горизонтальная составляющая зависит от того, куда по ракетке пришёлся удар.
    pub fn bounce_off_paddle(&mut self, paddle: &Paddle) {
        // Одного пересечения для отскока мало: ракетка в Arkanoid одна и стоит внизу, поэтому мяч
        // отбивается только на пути вниз. Уже отбитый мяч, ещё не успевший выйти из ракетки, иначе
        // разворачивался бы в каждом кадре и залип бы в ней.
        if !self.overlaps(paddle.x, paddle.y, paddle.width, paddle.height) || self.velocity_y <= 0.0
        {
            return;
        }

        let speed_y = self.velocity_y.abs();

        self.velocity_y = -self.velocity_y;
        // Горизонтальная составляющая задана долей той же вертикальной скорости, а не отдельной
        // константой: величина `velocity_y` при отскоке не меняется, поэтому мяч не может разогнаться
        // от удара к удару, а на самом краю ракетки уходит ровно под 45°.
        self.velocity_x = self.hit_offset(paddle) * speed_y;

        // Мяч ставится вплотную к верхней стороне ракетки: проверка направления и так не даст ему
        // отразиться второй раз, но оставленный внутри ракетки мяч выглядел бы залипшим в ней.
        self.y = paddle.y - self.size;
    }

    /// Место попадания по горизонтали: -1 — левый край ракетки, 0 — её центр, 1 — правый край. Мяч
    /// может задеть ракетку самым краем, и тогда его центр окажется за пределами ракетки, поэтому
    /// смещение ограничивается диапазоном.
    fn hit_offset(&self, paddle: &Paddle) -> f32 {
        let ball_center = self.x + self.size / 2.0;
        let paddle_center = paddle.x + paddle.width / 2.0;

        ((ball_center - paddle_center) / (paddle.width / 2.0)).clamp(-1.0, 1.0)
    }

    /// Пересечение прямоугольника мяча с любым другим прямоугольником (AABB), написанное вручную:
    /// physics engine в проекте нет. Прямоугольники не пересекаются, если один целиком левее, правее,
    /// выше или ниже другого, — значит пересекаются они тогда, когда неверно всё это сразу. Ракетка и
    /// блок проверяются одной и той же функцией: для столкновений оба — просто прямоугольники.
    pub fn overlaps(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        self.x < x + width
            && self.x + self.size > x
            && self.y < y + height
            && self.y + self.size > y
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

    /// Ракетка теста тоже с круглыми размерами: она занимает 400..500 по горизонтали и 540..556 по
    /// вертикали, её центр по горизонтали — 450.
    fn paddle() -> Paddle {
        Paddle {
            x: 400.0,
            y: 540.0,
            width: 100.0,
            height: 16.0,
            speed: 200.0,
        }
    }

    /// Мяч падает вертикально ровно на центр ракетки: его центр по горизонтали 440 + 20 / 2 = 450, а
    /// по вертикали он занимает 530..550 против ракетки 540..556, то есть уже пересекается с ней.
    /// Горизонтальной скорости у него нет, поэтому всё, что появится по этой оси, — работа отскока.
    fn hitting_the_paddle_center() -> Ball {
        Ball {
            x: 440.0,
            y: 530.0,
            velocity_x: 0.0,
            velocity_y: 300.0,
            ..ball()
        }
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
    fn ball_bounces_off_the_left_edge() {
        let mut going_left = Ball {
            x: 10.0,
            velocity_x: -400.0,
            ..ball()
        };

        // Кадр уводит мяч за левую границу: 10 - 400 * 0.1 = -30.
        going_left.update(0.1);
        going_left.bounce_off_walls(field());

        assert!(
            (going_left.velocity_x - 400.0).abs() < TOLERANCE,
            "левая граница не отразила мяч вправо: velocity_x = {}, ожидалось 400",
            going_left.velocity_x
        );
        assert!(
            (going_left.velocity_y - 100.0).abs() < TOLERANCE,
            "отражение изменило вертикальную скорость: velocity_y = {}, ожидалось 100",
            going_left.velocity_y
        );
        // Мяч, оставленный за границей, столкнулся бы с ней и в следующем кадре и задребезжал бы.
        assert!(
            going_left.x >= 0.0,
            "после отражения мяч не внутри поля: x = {}",
            going_left.x
        );
    }

    #[test]
    fn ball_bounces_off_the_right_edge() {
        let mut going_right = Ball {
            x: 930.0,
            velocity_x: 400.0,
            ..ball()
        };

        // Кадр уводит мяч за правую границу: правый край 930 + 400 * 0.1 + 20 = 990 при ширине 960.
        going_right.update(0.1);
        going_right.bounce_off_walls(field());

        assert!(
            (going_right.velocity_x + 400.0).abs() < TOLERANCE,
            "правая граница не отразила мяч влево: velocity_x = {}, ожидалось -400",
            going_right.velocity_x
        );
        assert!(
            (going_right.velocity_y - 100.0).abs() < TOLERANCE,
            "отражение изменило вертикальную скорость: velocity_y = {}, ожидалось 100",
            going_right.velocity_y
        );
        assert!(
            going_right.x + going_right.size <= field().width,
            "после отражения мяч не внутри поля: правый край = {}",
            going_right.x + going_right.size
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
        rising.bounce_off_walls(field());

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
        assert!(
            rising.y >= 0.0,
            "после отражения мяч не внутри поля: y = {}",
            rising.y
        );
    }

    /// Все три стены проверяются одним тестом: мяч, летящий внутри поля к любой из них, не должен
    /// отражаться только потому, что он движется в её сторону.
    #[test]
    fn ball_inside_the_field_does_not_bounce() {
        for (velocity_x, velocity_y) in [(-100.0, 0.0), (100.0, 0.0), (0.0, -100.0)] {
            let mut moving = Ball {
                velocity_x,
                velocity_y,
                ..ball()
            };
            let expected = (moving.x + velocity_x * 0.5, moving.y + velocity_y * 0.5);

            moving.update(0.5);
            moving.bounce_off_walls(field());

            assert!(
                (moving.velocity_x - velocity_x).abs() < TOLERANCE
                    && (moving.velocity_y - velocity_y).abs() < TOLERANCE,
                "мяч внутри поля отразился: velocity = ({}, {}), ожидалось ({}, {})",
                moving.velocity_x,
                moving.velocity_y,
                velocity_x,
                velocity_y
            );
            assert!(
                (moving.x - expected.0).abs() < TOLERANCE
                    && (moving.y - expected.1).abs() < TOLERANCE,
                "мяч внутри поля сдвинут стеной: ({}, {}), ожидалось ({}, {})",
                moving.x,
                moving.y,
                expected.0,
                expected.1
            );
        }
    }

    #[test]
    fn hit_at_the_paddle_center_sends_the_ball_up() {
        let mut hitting = hitting_the_paddle_center();

        hitting.bounce_off_paddle(&paddle());

        assert!(
            (hitting.velocity_y + 300.0).abs() < TOLERANCE,
            "ракетка не отправила мяч вверх: velocity_y = {}, ожидалось -300",
            hitting.velocity_y
        );
        assert!(
            hitting.velocity_x.abs() < TOLERANCE,
            "попадание в центр ракетки увело мяч в сторону: velocity_x = {}, ожидалось около 0",
            hitting.velocity_x
        );
    }

    #[test]
    fn hit_left_of_the_paddle_center_sends_the_ball_left() {
        // Центр мяча 415 + 20 / 2 = 425 против центра ракетки 450: половина левой половины ракетки,
        // то есть смещение -0.5, а горизонтальная скорость -0.5 * 300 = -150.
        let mut hitting = Ball {
            x: 415.0,
            ..hitting_the_paddle_center()
        };

        hitting.bounce_off_paddle(&paddle());

        assert!(
            (hitting.velocity_x + 150.0).abs() < TOLERANCE,
            "попадание левее центра не увело мяч влево: velocity_x = {}, ожидалось -150",
            hitting.velocity_x
        );
    }

    #[test]
    fn hit_right_of_the_paddle_center_sends_the_ball_right() {
        // Зеркально предыдущему тесту: центр мяча 465 + 20 / 2 = 475, смещение 0.5, скорость 150.
        let mut hitting = Ball {
            x: 465.0,
            ..hitting_the_paddle_center()
        };

        hitting.bounce_off_paddle(&paddle());

        assert!(
            (hitting.velocity_x - 150.0).abs() < TOLERANCE,
            "попадание правее центра не увело мяч вправо: velocity_x = {}, ожидалось 150",
            hitting.velocity_x
        );
    }

    #[test]
    fn ball_missing_the_paddle_keeps_its_velocity() {
        // Промах по горизонтали (мяч 100..120 против ракетки 400..500) и промах по вертикали
        // (мяч 100..120 против ракетки 540..556): пересечения нет ни по одной из осей.
        let misses = [
            Ball {
                x: 100.0,
                ..hitting_the_paddle_center()
            },
            Ball {
                y: 100.0,
                ..hitting_the_paddle_center()
            },
        ];

        for mut missing in misses {
            let (x, y) = (missing.x, missing.y);

            missing.bounce_off_paddle(&paddle());

            assert!(
                missing.velocity_x.abs() < TOLERANCE
                    && (missing.velocity_y - 300.0).abs() < TOLERANCE,
                "промах мимо ракетки из ({}, {}) изменил скорость: velocity = ({}, {}), ожидалось (0, 300)",
                x,
                y,
                missing.velocity_x,
                missing.velocity_y
            );
        }
    }

    #[test]
    fn ball_already_moving_up_does_not_bounce_again() {
        // Мяч всё ещё пересекается с ракеткой, но уже отскочил и летит вверх. Второй отскок развернул
        // бы его обратно в ракетку, и мяч залип бы в ней, разворачиваясь в каждом кадре.
        let mut leaving = Ball {
            velocity_y: -300.0,
            ..hitting_the_paddle_center()
        };

        leaving.bounce_off_paddle(&paddle());

        assert!(
            (leaving.velocity_y + 300.0).abs() < TOLERANCE,
            "мяч, летящий от ракетки, отразился второй раз: velocity_y = {}, ожидалось -300",
            leaving.velocity_y
        );
    }

    #[test]
    fn bounce_puts_the_ball_above_the_paddle() {
        let mut hitting = hitting_the_paddle_center();

        hitting.bounce_off_paddle(&paddle());

        // Мяч, оставленный внутри ракетки, выглядел бы залипшим в ней, пока из неё выбирается.
        assert!(
            hitting.y + hitting.size <= paddle().y + TOLERANCE,
            "мяч остался внутри ракетки: нижний край = {}, ожидалось не ниже 540",
            hitting.y + hitting.size
        );
    }
}
