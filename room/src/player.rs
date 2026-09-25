use macroquad::prelude::*;

use crate::input::Input;

/// Предел pitch — чуть меньше строгой вертикали. Ровно вертикаль исключена намеренно: в ней
/// направление взгляда вырождается (горизонтальная составляющая обнуляется) и камера теряет
/// ориентацию вокруг своей оси.
pub const PITCH_LIMIT: f32 = 89.0 / 180.0 * std::f32::consts::PI;

/// Игрок как данные (D-44): позиция и взгляд. Вертикальная скорость и признак «на земле» приходят
/// на `T-ROOM-5`.
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

    /// Двигает игрока по полу относительно направления взгляда — `pitch` в горизонтальное движение
    /// не входит нарочно (D-48): если бы входил, игрок, посмотрев в пол, поехал бы в пол, а
    /// посмотрев в потолок — взлетел. Скорость не зависит от FPS (умножение на `delta_time`), а
    /// движение по диагонали не быстрее прямого — намерение нормализуется, если оно не нулевое
    /// (нулевой вектор не нормализуется, чтобы не получить `NaN`, когда клавиши не нажаты или
    /// противоположные пары гасят друг друга).
    pub fn move_on_floor(&mut self, input: &Input, speed: f32, delta_time: f32) {
        let (forward, right) = horizontal_axes(self.yaw);
        let mut intent = Vec3::ZERO;
        if input.forward {
            intent += forward;
        }
        if input.back {
            intent -= forward;
        }
        if input.right {
            intent += right;
        }
        if input.left {
            intent -= right;
        }
        if intent.length_squared() > 0.0 {
            intent = intent.normalize();
        }
        self.position += intent * speed * delta_time;
    }
}

/// Единственный в крейте переход «yaw → горизонтальное направление» (D-48): даёт пару `forward` и
/// `right` в горизонтальной плоскости. `pitch` в него сознательно не входит. При `yaw = 0` `forward`
/// совпадает с направлением взгляда при `pitch = 0` (`-Z`), `right` ему перпендикулярен.
fn horizontal_axes(yaw: f32) -> (Vec3, Vec3) {
    let forward = vec3(yaw.sin(), 0.0, -yaw.cos());
    let right = vec3(yaw.cos(), 0.0, yaw.sin());
    (forward, right)
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

    fn no_movement() -> Input {
        Input {
            mouse_delta: Vec2::ZERO,
            forward: false,
            back: false,
            left: false,
            right: false,
        }
    }

    #[test]
    fn forward_at_zero_yaw_matches_the_expected_world_axis_and_right_is_perpendicular() {
        let (forward, right) = horizontal_axes(0.0);
        assert!(
            (forward - vec3(0.0, 0.0, -1.0)).length() < 1e-6,
            "forward при yaw=0 — {:?}, ожидалось (0, 0, -1)",
            forward
        );
        assert!(
            forward.dot(right).abs() < 1e-6,
            "right не перпендикулярен forward: скалярное произведение {}",
            forward.dot(right)
        );
    }

    #[test]
    fn forward_and_right_are_unit_length_and_horizontal_for_an_arbitrary_yaw() {
        let (forward, right) = horizontal_axes(1.234);
        assert!(
            (forward.length() - 1.0).abs() < 1e-6,
            "forward не единичной длины: {}",
            forward.length()
        );
        assert!(
            (right.length() - 1.0).abs() < 1e-6,
            "right не единичной длины: {}",
            right.length()
        );
        assert_eq!(forward.y, 0.0, "forward имеет вертикальную составляющую");
        assert_eq!(right.y, 0.0, "right имеет вертикальную составляющую");
    }

    #[test]
    fn a_right_angle_turn_puts_forward_where_right_used_to_be() {
        let (_, right_before) = horizontal_axes(0.0);
        let (forward_after, _) = horizontal_axes(std::f32::consts::FRAC_PI_2);
        assert!(
            (forward_after - right_before).length() < 1e-5,
            "forward после поворота на 90° — {:?}, ожидалось прежнее right {:?}",
            forward_after,
            right_before
        );
    }

    #[test]
    fn the_path_over_a_second_is_the_same_at_60_and_120_fps() {
        let mut player_60fps = Player::new(vec3(0.0, 0.0, 0.0));
        let input = Input {
            forward: true,
            ..no_movement()
        };
        for _ in 0..60 {
            player_60fps.move_on_floor(&input, 5.0, 1.0 / 60.0);
        }

        let mut player_120fps = Player::new(vec3(0.0, 0.0, 0.0));
        for _ in 0..120 {
            player_120fps.move_on_floor(&input, 5.0, 1.0 / 120.0);
        }

        assert!(
            (player_60fps.position - player_120fps.position).length() < 1e-4,
            "путь за секунду разошёлся: 60fps {:?}, 120fps {:?}",
            player_60fps.position,
            player_120fps.position
        );
    }

    #[test]
    fn diagonal_movement_is_not_faster_than_straight_movement() {
        let mut diagonal = Player::new(vec3(0.0, 0.0, 0.0));
        let diagonal_input = Input {
            forward: true,
            right: true,
            ..no_movement()
        };
        diagonal.move_on_floor(&diagonal_input, 5.0, 1.0);

        let mut straight = Player::new(vec3(0.0, 0.0, 0.0));
        let straight_input = Input {
            forward: true,
            ..no_movement()
        };
        straight.move_on_floor(&straight_input, 5.0, 1.0);

        assert!(
            diagonal.position.length() <= straight.position.length() + 1e-5,
            "диагональное движение оказалось быстрее прямого: {} > {}",
            diagonal.position.length(),
            straight.position.length()
        );
    }

    #[test]
    fn opposite_keys_together_leave_the_player_in_place_without_nan() {
        let mut player = Player::new(vec3(0.0, 0.0, 0.0));
        let input = Input {
            forward: true,
            back: true,
            left: true,
            right: true,
            ..no_movement()
        };
        player.move_on_floor(&input, 5.0, 1.0);
        assert_eq!(
            player.position,
            vec3(0.0, 0.0, 0.0),
            "противоположные клавиши сдвинули игрока: {:?}",
            player.position
        );
        assert!(
            player.position.is_finite(),
            "позиция содержит NaN/inf: {:?}",
            player.position
        );
    }
}
