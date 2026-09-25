use macroquad::prelude::*;

use crate::arena::Arena;

/// Скорость игрока, пикселей в секунду.
const PLAYER_SPEED: f32 = 250.0;
/// Радиус столкновения игрока.
pub const PLAYER_RADIUS: f32 = 16.0;

pub struct Player {
    pub position: Vec2,
    pub radius: f32,
}

impl Player {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            radius: PLAYER_RADIUS,
        }
    }

    /// Скорость игрока **присваивается** из ввода, а не накапливается тягой (в отличие от корабля
    /// Asteroids): отпустил клавишу — остановился в том же кадре. У игрока нет инерции.
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
