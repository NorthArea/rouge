use macroquad::prelude::*;

/// Скорость поворота, радиан в секунду.
const ROTATION_SPEED: f32 = std::f32::consts::PI;
/// Ускорение тяги, пикселей в секунду за секунду.
const ACCELERATION: f32 = 300.0;
/// Предел длины вектора скорости, пикселей в секунду.
const MAX_SPEED: f32 = 400.0;

/// Локальная форма корабля до поворота и переноса: нос вперёд по оси X, два кормовых угла. Используется
/// отрисовкой (`vertices()`) и, начиная с `T-AST-8`, вычислением радиуса столкновения по вписанной
/// окружности этого же треугольника (D-27) — той же тройкой точек, чтобы форма и радиус не могли
/// разойтись.
const NOSE: Vec2 = vec2(18.0, 0.0);
const REAR_LEFT: Vec2 = vec2(-12.0, 10.0);
const REAR_RIGHT: Vec2 = vec2(-12.0, -10.0);

pub struct Ship {
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
}

impl Ship {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            angle: 0.0,
        }
    }

    /// Единственный переход «угол → направление» во всём крейте (D-25): дальше через него
    /// выражаются тяга, нос корабля, направление пули, разлёт осколков и вершины треугольника.
    pub fn facing(&self) -> Vec2 {
        Vec2::from_angle(self.angle)
    }

    /// `direction`: `1.0` — по часовой (`D`/`→`), `-1.0` — против часовой (`A`/`←`), `0.0` — обе
    /// клавиши или ни одной. Ось Y в Macroquad направлена вниз, поэтому положительный угол
    /// поворачивает корабль по часовой стрелке на экране, а не против неё, как в обычной
    /// математической плоскости.
    pub fn rotate(&mut self, direction: f32, delta_time: f32) {
        self.angle += direction * ROTATION_SPEED * delta_time;
    }

    /// Тяга: скорость никогда не присваивается напрямую, только приращением вдоль `facing()` (D-09
    /// развёрнут, а не отменён — источник `velocity` стал результатом этой стадии). Предел скорости
    /// проверяется тем же вызовом, поэтому он действует и в кадрах без тяги, где вызывается с
    /// `delta_time = 0.0` (см. `Game::update`) — именно этот путь ловушки: `clamp_length_max`
    /// корректен на нулевом векторе, а ручной вариант через `normalize()` дал бы `NaN`.
    pub fn apply_thrust(&mut self, delta_time: f32) {
        self.velocity += self.facing() * ACCELERATION * delta_time;
        self.velocity = self.velocity.clamp_length_max(MAX_SPEED);
    }

    /// Перемещение по накопленной скорости — работает независимо от того, идёт ли тяга в этом кадре,
    /// поэтому корабль летит по инерции и после отпускания клавиши.
    pub fn advance(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }

    /// Вершины треугольника корабля в мировых координатах, для отрисовки. Поворот local-точек
    /// собран из `facing()` (продольная ось) и перпендикуляра к нему (поперечная ось) — без
    /// повторного вызова `sin`/`cos`, чтобы тригонометрия так и осталась ровно в `facing()` (D-25).
    pub fn vertices(&self) -> [Vec2; 3] {
        let forward = self.facing();
        let side = vec2(-forward.y, forward.x);
        [NOSE, REAR_LEFT, REAR_RIGHT]
            .map(|local| self.position + local.x * forward + local.y * side)
    }
}

/// Комбинирует одновременное удержание двух клавиш поворота в единственное направление: `-1.0` против
/// часовой (`A`/`←`), `1.0` по часовой (`D`/`→`), `0.0` при обеих или ни одной. Чистая функция от уже
/// прочитанного состояния клавиш, поэтому вызывается из теста без окна — само чтение клавиш (D-10)
/// остаётся в `main.rs`.
pub fn turn_direction(left: bool, right: bool) -> f32 {
    match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn ship() -> Ship {
        Ship::new(vec2(480.0, 300.0))
    }

    #[test]
    fn angle_zero_faces_right() {
        let ship = ship();

        let facing = ship.facing();

        assert!(
            (facing - vec2(1.0, 0.0)).length() < TOLERANCE,
            "угол 0 дал направление {:?}, ожидалось (1, 0)",
            facing
        );
    }

    #[test]
    fn a_quarter_turn_faces_down_the_screen() {
        let mut ship = ship();
        ship.angle = std::f32::consts::FRAC_PI_2;

        let facing = ship.facing();

        // Ось Y в Macroquad направлена вниз, поэтому угол π/2 — это низ экрана, а не верх
        // математической плоскости.
        assert!(
            (facing - vec2(0.0, 1.0)).length() < TOLERANCE,
            "угол π/2 дал направление {:?}, ожидалось (0, 1)",
            facing
        );
    }

    #[test]
    fn rotating_for_a_second_is_the_same_at_60_and_120_fps() {
        let mut at_60_fps = ship();
        for _ in 0..60 {
            at_60_fps.rotate(1.0, 1.0 / 60.0);
        }
        let mut at_120_fps = ship();
        for _ in 0..120 {
            at_120_fps.rotate(1.0, 1.0 / 120.0);
        }

        // Поворот по часовой стрелке (direction = 1.0) на скорости `ROTATION_SPEED = π` в течение
        // одной секунды — ровно π радиан.
        assert!(
            (at_60_fps.angle - std::f32::consts::PI).abs() < TOLERANCE,
            "60 кадров по 1/60: angle = {}, ожидалось π",
            at_60_fps.angle
        );
        assert!(
            (at_120_fps.angle - std::f32::consts::PI).abs() < TOLERANCE,
            "120 кадров по 1/120: angle = {}, ожидалось π",
            at_120_fps.angle
        );
        assert!(
            (at_60_fps.angle - at_120_fps.angle).abs() < TOLERANCE,
            "60 FPS дало angle = {}, 120 FPS дало angle = {}",
            at_60_fps.angle,
            at_120_fps.angle
        );
    }

    #[test]
    fn holding_both_turn_keys_leaves_the_angle_unchanged() {
        let mut ship = ship();
        let direction = turn_direction(true, true);

        ship.rotate(direction, 1.0);

        assert!(
            (ship.angle - 0.0).abs() < TOLERANCE,
            "одновременное удержание обеих клавиш повернуло корабль: angle = {}, ожидалось 0",
            ship.angle
        );
    }

    #[test]
    fn thrust_along_the_x_axis_increases_velocity_x_only() {
        let mut ship = ship();

        ship.apply_thrust(1.0);

        // Один "кадр" длиной в целую секунду: velocity.x = ACCELERATION * 1.0 = 300, ниже MAX_SPEED,
        // так что ограничение ещё не срабатывает и значение видно как чистая арифметика.
        assert!(
            (ship.velocity.x - 300.0).abs() < TOLERANCE,
            "тяга вдоль оси X дала velocity.x = {}, ожидалось 300",
            ship.velocity.x
        );
        assert!(
            (ship.velocity.y - 0.0).abs() < TOLERANCE,
            "тяга вдоль оси X изменила velocity.y = {}, ожидалось 0",
            ship.velocity.y
        );
    }

    #[test]
    fn the_ship_keeps_moving_after_thrust_stops() {
        let mut ship = ship();
        ship.apply_thrust(0.5); // velocity.x = 150
        let position_after_thrust = ship.position;

        // Тяги больше нет — только инерция.
        ship.advance(1.0);

        assert!(
            (ship.position.x - (position_after_thrust.x + 150.0)).abs() < TOLERANCE,
            "корабль не пролетел по инерции: x = {}, ожидалось {}",
            ship.position.x,
            position_after_thrust.x + 150.0
        );
    }

    fn coast(velocity: Vec2, steps: u32, delta_time: f32) -> Ship {
        let mut ship = ship();
        ship.velocity = velocity;
        for _ in 0..steps {
            ship.advance(delta_time);
        }
        ship
    }

    #[test]
    fn coasting_covers_the_same_distance_regardless_of_frame_rate() {
        // Инерция без тяги — чистое `position += velocity * delta_time`, поэтому проверка того же
        // вида, что у ракетки и мяча Pong: путь за секунду не должен зависеть от числа кадров.
        let at_60_fps = coast(vec2(200.0, 0.0), 60, 1.0 / 60.0);
        let at_120_fps = coast(vec2(200.0, 0.0), 120, 1.0 / 120.0);

        assert!(
            (at_60_fps.position.x - (480.0 + 200.0)).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось {}",
            at_60_fps.position.x,
            480.0 + 200.0
        );
        assert!(
            (at_120_fps.position.x - (480.0 + 200.0)).abs() < TOLERANCE,
            "120 кадров по 1/120: x = {}, ожидалось {}",
            at_120_fps.position.x,
            480.0 + 200.0
        );
        assert!(
            (at_60_fps.position.x - at_120_fps.position.x).abs() < TOLERANCE,
            "60 FPS дало x = {}, 120 FPS дало x = {}",
            at_60_fps.position.x,
            at_120_fps.position.x
        );
    }

    #[test]
    fn sustained_thrust_never_exceeds_the_speed_limit() {
        let mut ship = ship();
        // Пять секунд тяги — заведомо больше, чем нужно, чтобы разогнаться выше MAX_SPEED = 400 без
        // ограничения (300 * 5 = 1500).
        for _ in 0..300 {
            ship.apply_thrust(1.0 / 60.0);
        }

        assert!(
            ship.velocity.length() <= 400.0 + TOLERANCE,
            "скорость превысила предел: {}, ожидалось не больше 400",
            ship.velocity.length()
        );
    }

    #[test]
    fn clamping_the_speed_does_not_change_its_direction() {
        let mut ship = ship();
        ship.velocity = vec2(300.0, 400.0); // длина 500 — уже выше MAX_SPEED
        let direction_before = ship.velocity.normalize();

        // delta_time = 0.0: тяги не добавляется, срабатывает только ограничение — тот самый путь,
        // который у неподвижного корабля (см. следующий тест) должен остаться безопасным на NaN.
        ship.apply_thrust(0.0);
        let direction_after = ship.velocity.normalize();

        assert!(
            (direction_before - direction_after).length() < TOLERANCE,
            "ограничение скорости изменило направление: было {:?}, стало {:?}",
            direction_before,
            direction_after
        );
        assert!(
            ship.velocity.length() <= 400.0 + TOLERANCE,
            "ограничение не сработало: длина осталась {}",
            ship.velocity.length()
        );
    }

    #[test]
    fn a_still_ship_without_thrust_stays_put_and_finite() {
        let mut ship = ship();
        let start = ship.position;

        // Тяги нет (delta_time = 0.0 у apply_thrust), но вызов ограничения скорости всё равно
        // происходит каждый кадр — ловушка позиции: на нулевой скорости ручной `normalize()` дал бы
        // `NaN`, после чего `advance` унесла бы корабль в `NaN`-координаты навсегда.
        ship.apply_thrust(0.0);
        ship.advance(1.0 / 60.0);

        assert!(
            ship.position.x.is_finite() && ship.position.y.is_finite(),
            "координаты корабля стали нечисловыми: {:?}",
            ship.position
        );
        assert!(
            (ship.position - start).length() < TOLERANCE,
            "неподвижный корабль без тяги сдвинулся: {:?}, ожидалось {:?}",
            ship.position,
            start
        );
    }
}
