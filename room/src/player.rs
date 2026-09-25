use macroquad::prelude::*;

/// Предел pitch — чуть меньше строгой вертикали. Ровно вертикаль исключена намеренно: в ней
/// направление взгляда вырождается (горизонтальная составляющая обнуляется) и камера теряет
/// ориентацию вокруг своей оси.
pub const PITCH_LIMIT: f32 = 89.0 / 180.0 * std::f32::consts::PI;

/// Игрок как данные (D-44): позиция и взгляд. На этой позиции неподвижен — движение приходит на
/// `T-ROOM-4`.
pub struct Player {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Поворачивает взгляд на смещение мыши за кадр, умноженное на чувствительность. Горизонтальное
    /// смещение меняет yaw, вертикальное — pitch (в пределе `PITCH_LIMIT`). yaw приводится к
    /// диапазону `(-π, π]`: само по себе накопление без предела не портит направление, синус и
    /// косинус периодичны, но делает значение в debug overlay (`T-ROOM-9`) нечитаемым при долгом
    /// вращении в одну сторону — решение зафиксировано тестом, а не оставлено случайным.
    pub fn look(&mut self, mouse_delta: Vec2, sensitivity: f32) {
        self.yaw = wrap_angle(self.yaw + mouse_delta.x * sensitivity);
        self.pitch = clamp_pitch(self.pitch - mouse_delta.y * sensitivity);
    }

    /// Направление взгляда — единственное место в крейте, переводящее yaw и pitch в вектор (D-48
    /// говорит о переходе "yaw → горизонтальное направление"; это его трёхмерный аналог для взгляда,
    /// которым пользуется камера как точка, куда она смотрит: `position + look_direction()`).
    pub fn look_direction(&self) -> Vec3 {
        direction_from_angles(self.yaw, self.pitch)
    }
}

fn clamp_pitch(pitch: f32) -> f32 {
    pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT)
}

/// Приводит угол к диапазону `(-π, π]`, чтобы yaw не рос неограниченно при долгом вращении в одну
/// сторону.
fn wrap_angle(angle: f32) -> f32 {
    let turn = std::f32::consts::TAU;
    let wrapped = angle.rem_euclid(turn);
    if wrapped > std::f32::consts::PI {
        wrapped - turn
    } else {
        wrapped
    }
}

/// При нулевых yaw и pitch направление — мировая ось `-Z` (тот же выбор, что в собственной камере
/// крейта; в Macroquad `draw_grid`/`draw_plane` кладут "вперёд по полу" в плоскость XZ, Y — вверх).
/// Положительный yaw поворачивает взгляд в сторону `+X`, положительный pitch — вверх (`+Y`).
fn direction_from_angles(yaw: f32, pitch: f32) -> Vec3 {
    vec3(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_looking(yaw: f32, pitch: f32) -> Player {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        player.yaw = yaw;
        player.pitch = pitch;
        player
    }

    #[test]
    fn pitch_above_the_limit_is_clamped_to_the_limit() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        player.look(vec2(0.0, -1000.0), 1.0);
        assert!(
            (player.pitch - PITCH_LIMIT).abs() < 1e-6,
            "pitch после большого поворота вверх — {}, ожидался предел {}",
            player.pitch,
            PITCH_LIMIT
        );
    }

    #[test]
    fn pitch_below_the_limit_is_clamped_to_the_lower_limit() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        player.look(vec2(0.0, 1000.0), 1.0);
        assert!(
            (player.pitch - (-PITCH_LIMIT)).abs() < 1e-6,
            "pitch после большого поворота вниз — {}, ожидался предел {}",
            player.pitch,
            -PITCH_LIMIT
        );
    }

    #[test]
    fn pitch_within_the_limit_is_not_clamped() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        player.look(vec2(0.0, -0.1), 1.0);
        assert!(
            (player.pitch - 0.1).abs() < 1e-6,
            "pitch внутри предела изменился неожиданно: {}",
            player.pitch
        );
    }

    #[test]
    fn yaw_accumulates_from_successive_mouse_deltas() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        player.look(vec2(0.2, 0.0), 1.0);
        player.look(vec2(0.3, 0.0), 1.0);
        assert!(
            (player.yaw - 0.5).abs() < 1e-6,
            "yaw после двух смещений — {}, ожидалось 0.5",
            player.yaw
        );
    }

    #[test]
    fn yaw_wraps_instead_of_growing_without_bound() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        // Много оборотов подряд в одну сторону — без приведения к диапазону значение росло бы
        // неограниченно.
        for _ in 0..1000 {
            player.look(vec2(1.0, 0.0), 1.0);
        }
        assert!(
            player.yaw > -std::f32::consts::PI && player.yaw <= std::f32::consts::PI,
            "yaw вышел за диапазон (-π, π]: {}",
            player.yaw
        );
    }

    #[test]
    fn the_look_direction_at_zero_yaw_and_pitch_matches_the_expected_world_axis() {
        let player = player_looking(0.0, 0.0);
        let direction = player.look_direction();
        assert!(
            (direction - vec3(0.0, 0.0, -1.0)).length() < 1e-6,
            "направление взгляда при нулевых углах — {:?}, ожидалось (0, 0, -1)",
            direction
        );
    }
}
