use macroquad::prelude::*;

use crate::input::Input;

/// Предел pitch — чуть меньше строгой вертикали. Ровно вертикаль исключена намеренно: в ней
/// направление взгляда вырождается (горизонтальная составляющая обнуляется) и камера теряет
/// ориентацию вокруг своей оси.
pub const PITCH_LIMIT: f32 = 89.0 / 180.0 * std::f32::consts::PI;

/// Высота глаз игрока над полом — приземление ставит `position.y` не на уровень пола, а на эту
/// высоту над ним, чтобы камера (которая стоит в `position`) оказалась на высоте глаз, а не в полу.
pub const EYE_HEIGHT: f32 = 1.6;

/// Импульс прыжка — вертикальная скорость, которую получает игрок при отрыве от земли.
pub const JUMP_IMPULSE: f32 = 8.0;

/// Игрок как данные (D-44): позиция, взгляд, вертикальная скорость и признак «на земле».
pub struct Player {
    pub position: Vec3,
    pub velocity_y: f32,
    pub on_ground: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity_y: 0.0,
            on_ground: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Падение под гравитацией и приземление. Порядок операций фиксирован и больше не меняется:
    /// сперва скорость, потом позиция, потом приземление — обратный порядок даёт игрока, который на
    /// один кадр проваливается под пол и выталкивается обратно (визуально дрожание у земли).
    /// `floor_level` — мировая координата Y пола; приземление ставит `position.y` на
    /// `floor_level + EYE_HEIGHT`, а не на сам пол (высота глаз, см. `EYE_HEIGHT`).
    pub fn apply_gravity(&mut self, gravity: f32, delta_time: f32, floor_level: f32) {
        self.velocity_y -= gravity * delta_time;
        self.position.y += self.velocity_y * delta_time;

        let eye_level_on_floor = floor_level + EYE_HEIGHT;
        if self.position.y <= eye_level_on_floor {
            self.position.y = eye_level_on_floor;
            self.velocity_y = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }
    }

    /// Прыжок — только с земли. `Input::jump` уже читается как «нажатие» (`is_key_pressed`), не
    /// «удержание» (D-47: запрет второго прыжка в воздухе держит признак «на земле», а не счётчик
    /// или таймер — у Game 05 таймеров нет вовсе).
    pub fn jump(&mut self, input: &Input) {
        if input.jump && self.on_ground {
            self.velocity_y = JUMP_IMPULSE;
            self.on_ground = false;
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
            jump: false,
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

    #[test]
    fn a_player_above_the_floor_falls_over_a_frame_and_speeds_up() {
        let mut player = Player::new(vec3(0.0, 20.0, 0.0));
        let position_before = player.position.y;
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        assert!(
            player.position.y < position_before,
            "игрок не опустился за кадр: было {}, стало {}",
            position_before,
            player.position.y
        );
        assert!(
            player.velocity_y < 0.0,
            "вертикальная скорость не стала отрицательной: {}",
            player.velocity_y
        );
    }

    #[test]
    fn falling_accelerates_over_two_frames_in_a_row() {
        let mut player = Player::new(vec3(0.0, 20.0, 0.0));
        let start = player.position.y;
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        let after_first_frame = start - player.position.y;
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        let after_second_frame = start - player.position.y;
        assert!(
            after_second_frame > after_first_frame * 2.0,
            "падение не ускоряется: за первый кадр {}, за два — {}",
            after_first_frame,
            after_second_frame
        );
    }

    #[test]
    fn falling_stops_exactly_at_the_floor_level_and_not_below() {
        let mut player = Player::new(vec3(0.0, 20.0, 0.0));
        for _ in 0..600 {
            player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        }
        assert_eq!(
            player.position.y, EYE_HEIGHT,
            "игрок остановился не на уровне пола: {}",
            player.position.y
        );
    }

    #[test]
    fn landing_zeroes_the_vertical_velocity_and_sets_on_ground() {
        let mut player = Player::new(vec3(0.0, EYE_HEIGHT + 0.001, 0.0));
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        assert_eq!(
            player.velocity_y, 0.0,
            "скорость после приземления не нулевая"
        );
        assert!(
            player.on_ground,
            "признак «на земле» не поднят после приземления"
        );
    }

    #[test]
    fn being_airborne_clears_on_ground() {
        let mut player = Player::new(vec3(0.0, 20.0, 0.0));
        player.on_ground = true;
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        assert!(
            !player.on_ground,
            "признак «на земле» остался истинным в воздухе"
        );
    }

    #[test]
    fn jumping_from_the_ground_makes_the_vertical_velocity_positive_and_clears_on_ground() {
        let mut player = Player::new(vec3(0.0, EYE_HEIGHT, 0.0));
        player.on_ground = true;
        let input = Input {
            jump: true,
            ..no_movement()
        };
        player.jump(&input);
        assert!(
            player.velocity_y > 0.0,
            "скорость после прыжка не положительная: {}",
            player.velocity_y
        );
        assert!(
            !player.on_ground,
            "признак «на земле» остался истинным после прыжка"
        );
    }

    #[test]
    fn jumping_in_the_air_does_not_change_the_vertical_velocity() {
        let mut player = Player::new(vec3(0.0, 20.0, 0.0));
        player.on_ground = false;
        player.velocity_y = -3.0;
        let input = Input {
            jump: true,
            ..no_movement()
        };
        player.jump(&input);
        assert_eq!(
            player.velocity_y, -3.0,
            "прыжок в воздухе изменил вертикальную скорость: {}",
            player.velocity_y
        );
    }

    #[test]
    fn gravity_returns_the_player_to_the_ground_after_a_jump() {
        let mut player = Player::new(vec3(0.0, EYE_HEIGHT, 0.0));
        player.on_ground = true;
        let jump_input = Input {
            jump: true,
            ..no_movement()
        };
        player.jump(&jump_input);
        for _ in 0..600 {
            player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        }
        assert!(
            player.on_ground,
            "игрок после прыжка не вернулся на землю через 10 секунд падения"
        );
        assert_eq!(player.position.y, EYE_HEIGHT);
    }

    #[test]
    fn holding_jump_does_not_grant_a_second_jump_before_landing() {
        let mut player = Player::new(vec3(0.0, EYE_HEIGHT, 0.0));
        player.on_ground = true;
        let jump_input = Input {
            jump: true,
            ..no_movement()
        };
        player.jump(&jump_input);
        let velocity_after_first_jump = player.velocity_y;
        // Клавиша всё ещё "нажата" в следующем кадре — Input.jump тоже true, как если бы вызов
        // is_key_pressed по ошибке заменили на is_key_down.
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        player.jump(&jump_input);
        assert!(
            player.velocity_y <= velocity_after_first_jump,
            "второй прыжок в воздухе увеличил скорость: было {}, стало {}",
            velocity_after_first_jump,
            player.velocity_y
        );
    }

    #[test]
    fn a_player_resting_on_the_floor_does_not_sink_or_jitter_over_a_frame() {
        let mut player = Player::new(vec3(0.0, EYE_HEIGHT, 0.0));
        player.on_ground = true;
        player.apply_gravity(9.8, 1.0 / 60.0, 0.0);
        assert_eq!(
            player.position.y, EYE_HEIGHT,
            "игрок на полу сдвинулся за кадр без ввода: {}",
            player.position.y
        );
    }
}
