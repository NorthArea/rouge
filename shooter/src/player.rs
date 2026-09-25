use macroquad::prelude::*;

use crate::arena::Arena;

/// Скорость игрока, пикселей в секунду.
const PLAYER_SPEED: f32 = 250.0;
/// Радиус столкновения игрока.
pub const PLAYER_RADIUS: f32 = 16.0;
/// Расстояние от края игрока до дульного среза — общее для отрисовки ствола и точки спавна пули,
/// чтобы обе точки не могли разойтись.
const BARREL_LENGTH: f32 = 14.0;

pub struct Player {
    pub position: Vec2,
    pub radius: f32,
    /// Угол ориентации — в сторону последней ненулевой позиции курсора. Инициализируется нулём (вправо
    /// по оси X), как `Ship::angle` в Asteroids.
    pub aim_angle: f32,
}

impl Player {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            radius: PLAYER_RADIUS,
            aim_angle: 0.0,
        }
    }

    /// Направление, в котором смотрит игрок — единственный переход «угол → направление» в крейте,
    /// по прецеденту `Ship::facing()` в Asteroids: отсюда дальше выражаются и ствол, и (в `T-TDS-5`)
    /// направление выстрела.
    pub fn facing(&self) -> Vec2 {
        Vec2::from_angle(self.aim_angle)
    }

    /// Точка у дульного среза в мировых координатах — место появления пули (`T-TDS-5`) и отрисовки
    /// ствола (`T-TDS-4`), выраженные одной точкой, чтобы они не могли разойтись.
    pub fn barrel_position(&self) -> Vec2 {
        self.position + self.facing() * (self.radius + BARREL_LENGTH)
    }

    /// Поворот игрока к курсору. `direction`, `normalized_direction` и `angle` названы отдельно, как
    /// того требует задание. Курсор ровно на игроке даёт нулевой `direction`, а `try_normalize()` на
    /// нулевом векторе возвращает `None` — ориентация в этом кадре не пересчитывается и остаётся
    /// прежней, а не превращается в `NaN` и не сбрасывается в произвольную сторону.
    pub fn aim_at(&mut self, cursor_world: Vec2) {
        let direction = cursor_world - self.position;
        let Some(normalized_direction) = direction.try_normalize() else {
            return;
        };
        let angle = normalized_direction.y.atan2(normalized_direction.x);
        self.aim_angle = angle;
    }

    /// Скорость игрока **присваивается** из ввода, а не накапливается тягой (в отличие от корабля
    /// Asteroids): отпустил клавишу — остановился в том же кадре. У игрока нет инерции.
    pub fn update(&mut self, up: bool, down: bool, left: bool, right: bool, delta_time: f32) {
        let intent = movement_intent(up, down, left, right);
        self.position += intent * PLAYER_SPEED * delta_time;
    }

    /// Удерживает игрока внутри арены с поправкой на радиус: ограничивается позиция игрока, а не его
    /// центр без поправки — иначе край игрока пересекал бы границу арены.
    pub fn clamp_to_arena(&mut self, arena: Arena) {
        self.position.x = self
            .position
            .x
            .clamp(self.radius, arena.width - self.radius);
        self.position.y = self
            .position
            .y
            .clamp(self.radius, arena.height - self.radius);
    }
}

/// Вектор намерения движения из четырёх направлений, уже нормализованный. Нормализация нулевого
/// вектора даёт `NaN` (та же ловушка, что у корабля Asteroids и у ракетки Pong), поэтому используется
/// `normalize_or_zero` — на пустом или взаимно гасящем вводе намерение остаётся нулевым и числовым.
fn movement_intent(up: bool, down: bool, left: bool, right: bool) -> Vec2 {
    let x = (right as i32 - left as i32) as f32;
    let y = (down as i32 - up as i32) as f32;
    vec2(x, y).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn player() -> Player {
        Player::new(vec2(960.0, 600.0))
    }

    fn arena() -> Arena {
        Arena::new(1920.0, 1200.0)
    }

    #[test]
    fn a_cursor_to_the_right_of_the_player_gives_a_rightward_direction() {
        let mut player = player();

        player.aim_at(player.position + vec2(200.0, 0.0));

        let facing = player.facing();
        assert!(
            (facing - vec2(1.0, 0.0)).length() < TOLERANCE,
            "курсор справа дал направление {:?}, ожидалось (1, 0)",
            facing
        );
    }

    #[test]
    fn a_cursor_below_the_player_gives_a_downward_direction() {
        let mut player = player();

        // Ось Y в Macroquad направлена вниз, поэтому курсор с большим Y — это низ экрана.
        player.aim_at(player.position + vec2(0.0, 200.0));

        let facing = player.facing();
        assert!(
            (facing - vec2(0.0, 1.0)).length() < TOLERANCE,
            "курсор снизу дал направление {:?}, ожидалось (0, 1)",
            facing
        );
    }

    #[test]
    fn the_facing_direction_has_unit_length_for_an_arbitrary_cursor() {
        let mut player = player();

        player.aim_at(player.position + vec2(37.0, -481.0));

        assert!(
            (player.facing().length() - 1.0).abs() < TOLERANCE,
            "длина направления — {}, ожидалась 1",
            player.facing().length()
        );
    }

    #[test]
    fn a_cursor_on_the_player_does_not_corrupt_the_orientation() {
        let mut player = player();
        player.aim_at(player.position + vec2(0.0, 1.0));
        let angle_before = player.aim_angle;

        player.aim_at(player.position);

        assert!(
            (player.aim_angle - angle_before).abs() < TOLERANCE,
            "курсор на игроке изменил ориентацию: было {}, стало {}",
            angle_before,
            player.aim_angle
        );
        assert!(
            player.aim_angle.is_finite(),
            "ориентация стала нечисловой: {}",
            player.aim_angle
        );
    }

    #[test]
    fn the_angle_and_the_facing_direction_are_consistent() {
        let mut player = player();
        let cursor_direction = vec2(-150.0, 90.0).normalize();

        player.aim_at(player.position + cursor_direction * 300.0);

        assert!(
            (player.facing() - cursor_direction).length() < TOLERANCE,
            "facing() дал {:?}, ожидалось направление курсора {:?}",
            player.facing(),
            cursor_direction
        );
    }

    #[test]
    fn moving_right_for_a_second_is_the_same_at_60_and_120_fps() {
        let mut at_60_fps = player();
        for _ in 0..60 {
            at_60_fps.update(false, false, false, true, 1.0 / 60.0);
        }
        let mut at_120_fps = player();
        for _ in 0..120 {
            at_120_fps.update(false, false, false, true, 1.0 / 120.0);
        }

        let expected_x = 960.0 + PLAYER_SPEED;
        assert!(
            (at_60_fps.position.x - expected_x).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось {}",
            at_60_fps.position.x,
            expected_x
        );
        assert!(
            (at_120_fps.position.x - expected_x).abs() < TOLERANCE,
            "120 кадров по 1/120: x = {}, ожидалось {}",
            at_120_fps.position.x,
            expected_x
        );
    }

    #[test]
    fn diagonal_movement_is_not_faster_than_straight_movement() {
        let mut diagonal = player();
        diagonal.update(true, false, false, true, 1.0);
        let straight = vec2(960.0, 600.0) + vec2(1.0, 0.0) * PLAYER_SPEED;
        let start = vec2(960.0, 600.0);

        let diagonal_distance = (diagonal.position - start).length();
        let straight_distance = (straight - start).length();

        assert!(
            diagonal_distance <= straight_distance + TOLERANCE,
            "диагональное движение прошло {}, прямое — {}",
            diagonal_distance,
            straight_distance
        );
    }

    #[test]
    fn opposite_keys_together_leave_the_player_in_place_and_finite() {
        let mut player = player();
        let start = player.position;

        player.update(true, true, true, true, 1.0);

        assert!(
            (player.position - start).length() < TOLERANCE,
            "противоположные клавиши сдвинули игрока: {:?}",
            player.position
        );
        assert!(
            player.position.x.is_finite() && player.position.y.is_finite(),
            "координаты игрока стали нечисловыми: {:?}",
            player.position
        );
    }

    #[test]
    fn the_player_stops_at_the_left_edge() {
        let mut player = player();
        player.position.x = -50.0;

        player.clamp_to_arena(arena());

        assert!(
            (player.position.x - PLAYER_RADIUS).abs() < TOLERANCE,
            "игрок ушёл за левую границу: x = {}, ожидалось {}",
            player.position.x,
            PLAYER_RADIUS
        );
    }

    #[test]
    fn the_player_stops_at_the_right_edge() {
        let mut player = player();
        player.position.x = 3000.0;

        player.clamp_to_arena(arena());

        let expected = arena().width - PLAYER_RADIUS;
        assert!(
            (player.position.x - expected).abs() < TOLERANCE,
            "игрок ушёл за правую границу: x = {}, ожидалось {}",
            player.position.x,
            expected
        );
    }

    #[test]
    fn the_player_stops_at_the_top_edge() {
        let mut player = player();
        player.position.y = -50.0;

        player.clamp_to_arena(arena());

        assert!(
            (player.position.y - PLAYER_RADIUS).abs() < TOLERANCE,
            "игрок ушёл за верхнюю границу: y = {}, ожидалось {}",
            player.position.y,
            PLAYER_RADIUS
        );
    }

    #[test]
    fn the_player_stops_at_the_bottom_edge() {
        let mut player = player();
        player.position.y = 3000.0;

        player.clamp_to_arena(arena());

        let expected = arena().height - PLAYER_RADIUS;
        assert!(
            (player.position.y - expected).abs() < TOLERANCE,
            "игрок ушёл за нижнюю границу: y = {}, ожидалось {}",
            player.position.y,
            expected
        );
    }
}
