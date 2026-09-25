use macroquad::prelude::*;

use crate::arena::Arena;
use crate::player::Player;

/// Скорость пули, пикселей в секунду.
const BULLET_SPEED: f32 = 700.0;
/// Время жизни пули, секунд — на арене 1920×1200 этого достаточно, чтобы пересечь её по диагонали
/// с запасом, даже если пуля не встретит границу раньше.
const BULLET_LIFETIME: f32 = 1.5;
/// Радиус столкновения пули (D-38 — круг с кругом).
pub const BULLET_RADIUS: f32 = 4.0;
/// Пауза между выстрелами, секунд.
const SHOOT_COOLDOWN: f32 = 0.25;

pub struct Bullet {
    pub position: Vec2,
    pub direction: Vec2,
    time_to_live: f32,
}

impl Bullet {
    pub fn new(position: Vec2, direction: Vec2) -> Self {
        Self {
            position,
            direction,
            time_to_live: BULLET_LIFETIME,
        }
    }

    /// Направление зафиксировано в момент выстрела (`new`) и не пересчитывается здесь — пуля летит по
    /// прямой независимо от того, куда потом смотрит игрок.
    pub fn advance(&mut self, delta_time: f32) {
        self.position += self.direction * BULLET_SPEED * delta_time;
        self.time_to_live -= delta_time;
    }

    pub fn is_expired(&self) -> bool {
        self.time_to_live <= 0.0
    }

    /// В отличие от Asteroids, у арены Shooter есть край, а не заворачивание (D-41): пуля, ушедшая за
    /// него, удаляется, а не появляется с другой стороны.
    pub fn is_outside(&self, arena: Arena) -> bool {
        self.position.x < 0.0
            || self.position.x > arena.width
            || self.position.y < 0.0
            || self.position.y > arena.height
    }
}

/// Уменьшает cooldown на delta time, не уходя в отрицательные значения дальше нуля — та же форма, что
/// `tick_cooldown` в Asteroids.
pub fn tick_cooldown(cooldown: &mut f32, delta_time: f32) {
    if *cooldown > 0.0 {
        *cooldown -= delta_time;
    }
}

/// Выстрел: `None`, если cooldown ещё не истёк — ЛКМ в этом кадре ничего не создаёт. Иначе cooldown
/// сбрасывается и пуля появляется у дульного среза, а не в центре игрока.
pub fn shoot(cooldown: &mut f32, player: &Player) -> Option<Bullet> {
    if *cooldown > 0.0 {
        return None;
    }
    *cooldown = SHOOT_COOLDOWN;
    Some(Bullet::new(player.barrel_position(), player.facing()))
}

/// Пули, у которых истёк lifetime или которые вышли за арену, одним `retain` без ручных индексов.
pub fn remove_expired(bullets: &mut Vec<Bullet>, arena: Arena) {
    bullets.retain(|bullet| !bullet.is_expired() && !bullet.is_outside(arena));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    const TOLERANCE: f32 = 0.01;

    fn arena() -> Arena {
        Arena::new(1920.0, 1200.0)
    }

    #[test]
    fn holding_the_trigger_does_not_shoot_faster_than_the_cooldown() {
        let mut cooldown = 0.0;
        let player = Player::new(vec2(500.0, 500.0));

        let first = shoot(&mut cooldown, &player);
        let second = shoot(&mut cooldown, &player);

        assert!(first.is_some(), "первый выстрел не создал пулю");
        assert!(
            second.is_none(),
            "второй выстрел в пределах cooldown создал пулю"
        );
    }

    #[test]
    fn the_cooldown_expiring_allows_the_next_shot() {
        let mut cooldown = 0.0;
        let player = Player::new(vec2(500.0, 500.0));
        shoot(&mut cooldown, &player);

        tick_cooldown(&mut cooldown, SHOOT_COOLDOWN + 0.01);
        let after_cooldown = shoot(&mut cooldown, &player);

        assert!(
            after_cooldown.is_some(),
            "выстрел после истечения cooldown не создал пулю"
        );
    }

    #[test]
    fn a_bullet_is_alive_before_its_lifetime_and_expired_after() {
        let mut bullet = Bullet::new(vec2(0.0, 0.0), vec2(1.0, 0.0));
        bullet.advance(BULLET_LIFETIME - 0.1);
        assert!(!bullet.is_expired(), "пуля истекла раньше своего lifetime");

        bullet.advance(0.2);
        assert!(bullet.is_expired(), "пуля пережила свой lifetime");
    }

    #[test]
    fn a_bullet_past_the_arena_edge_is_outside() {
        let bullet = Bullet::new(vec2(-5.0, 500.0), vec2(-1.0, 0.0));

        assert!(
            bullet.is_outside(arena()),
            "пуля за левым краем арены не распознана как вышедшая"
        );
    }

    #[test]
    fn a_bullet_inside_the_arena_is_not_outside() {
        let bullet = Bullet::new(vec2(500.0, 500.0), vec2(1.0, 0.0));

        assert!(
            !bullet.is_outside(arena()),
            "пуля внутри арены распознана как вышедшая"
        );
    }

    #[test]
    fn a_bullet_travels_the_same_distance_at_60_and_120_fps() {
        let mut at_60_fps = Bullet::new(vec2(0.0, 0.0), vec2(1.0, 0.0));
        for _ in 0..60 {
            at_60_fps.advance(1.0 / 60.0);
        }
        let mut at_120_fps = Bullet::new(vec2(0.0, 0.0), vec2(1.0, 0.0));
        for _ in 0..120 {
            at_120_fps.advance(1.0 / 120.0);
        }

        assert!(
            (at_60_fps.position.x - BULLET_SPEED).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось {}",
            at_60_fps.position.x,
            BULLET_SPEED
        );
        assert!(
            (at_60_fps.position.x - at_120_fps.position.x).abs() < TOLERANCE,
            "60 FPS дало x = {}, 120 FPS дало x = {}",
            at_60_fps.position.x,
            at_120_fps.position.x
        );
    }

    #[test]
    fn shooting_a_new_bullet_does_not_retarget_earlier_bullets() {
        let mut cooldown = 0.0;
        let mut player = Player::new(vec2(500.0, 500.0));
        player.aim_at(player.position + vec2(1.0, 0.0));
        let first = shoot(&mut cooldown, &player).expect("первый выстрел должен создать пулю");

        tick_cooldown(&mut cooldown, SHOOT_COOLDOWN + 0.01);
        player.aim_at(player.position + vec2(0.0, 1.0));
        shoot(&mut cooldown, &player);

        assert!(
            (first.direction - vec2(1.0, 0.0)).length() < TOLERANCE,
            "направление первой пули изменилось после смены прицела: {:?}",
            first.direction
        );
    }
}
