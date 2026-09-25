use macroquad::prelude::*;

use crate::ship::Ship;
use crate::Field;

/// Скорость пули больше `MAX_SPEED` корабля (`ship.rs`), поэтому пуля всегда уходит вперёд, а не
/// тянется за разогнанным кораблём даже на максимальной скорости.
const BULLET_SPEED: f32 = 600.0;
/// Время жизни пули, секунд.
const BULLET_LIFETIME: f32 = 1.2;
/// Пауза между выстрелами, секунд.
const SHOOT_COOLDOWN: f32 = 0.3;

pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    time_to_live: f32,
}

impl Bullet {
    /// Пуля появляется у носа корабля, а не в его центре, и летит со скоростью корабля плюс
    /// `BULLET_SPEED` вдоль того же направления — относительно корабля, а не абсолютно.
    fn spawn(ship: &Ship) -> Self {
        Self {
            position: ship.nose_position(),
            velocity: ship.velocity + ship.facing() * BULLET_SPEED,
            time_to_live: BULLET_LIFETIME,
        }
    }

    pub fn advance(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
        self.time_to_live -= delta_time;
    }

    /// То же правило заворачивания, что у корабля (D-28); повторяется здесь, а не выносится в общий
    /// модуль (D-02).
    pub fn wrap(&mut self, field: Field) {
        self.position.x = self.position.x.rem_euclid(field.width);
        self.position.y = self.position.y.rem_euclid(field.height);
    }

    pub fn is_expired(&self) -> bool {
        self.time_to_live <= 0.0
    }
}

/// Выбывшие по времени жизни пули убираются из коллекции стадией обновления (D-28 — как и заворот,
/// живёт рядом с типом, а не в отдельном общем модуле).
pub fn remove_expired(bullets: &mut Vec<Bullet>) {
    bullets.retain(|bullet| !bullet.is_expired());
}

/// Кулдаун между выстрелами: единственное состояние, которое нужно для стрельбы помимо самих пуль.
pub struct Weapon {
    cooldown: f32,
}

impl Weapon {
    pub fn new() -> Self {
        Self { cooldown: 0.0 }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.cooldown = (self.cooldown - delta_time).max(0.0);
    }

    /// `None`, пока cooldown не истёк; иначе — новая пуля, и cooldown выставляется заново.
    pub fn shoot(&mut self, ship: &Ship) -> Option<Bullet> {
        if self.cooldown > 0.0 {
            return None;
        }
        self.cooldown = SHOOT_COOLDOWN;
        Some(Bullet::spawn(ship))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn ship() -> Ship {
        Ship::new(vec2(480.0, 300.0))
    }

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    #[test]
    fn shooting_adds_one_bullet_aimed_where_the_ship_faces() {
        let ship = ship();
        let mut weapon = Weapon::new();

        let bullet = weapon.shoot(&ship).expect("выстрел не дал пулю");

        let direction = bullet.velocity.normalize();
        assert!(
            (direction - vec2(1.0, 0.0)).length() < TOLERANCE,
            "пуля полетела в направлении {:?}, ожидалось (1, 0)",
            direction
        );
    }

    #[test]
    fn the_bullet_spawns_ahead_of_the_ship_not_at_its_center() {
        let ship = ship();
        let mut weapon = Weapon::new();

        let bullet = weapon.shoot(&ship).expect("выстрел не дал пулю");

        assert!(
            (bullet.position - ship.position).length() > TOLERANCE,
            "пуля появилась в центре корабля: {:?}",
            bullet.position
        );
    }

    #[test]
    fn a_second_shot_in_the_same_frame_is_blocked_by_cooldown() {
        let ship = ship();
        let mut weapon = Weapon::new();
        weapon.shoot(&ship);

        let second = weapon.shoot(&ship);

        assert!(
            second.is_none(),
            "второй выстрел в том же кадре прошёл, ожидался блок по cooldown"
        );
    }

    #[test]
    fn a_shot_after_the_cooldown_expires_goes_through() {
        let ship = ship();
        let mut weapon = Weapon::new();
        weapon.shoot(&ship);

        weapon.tick(SHOOT_COOLDOWN + 0.01);
        let after_cooldown = weapon.shoot(&ship);

        assert!(
            after_cooldown.is_some(),
            "выстрел после истечения cooldown не прошёл"
        );
    }

    #[test]
    fn cooldown_ticks_the_same_at_60_and_120_fps() {
        let ship = ship();
        let mut at_60_fps = Weapon::new();
        at_60_fps.shoot(&ship);
        for _ in 0..18 {
            // 18 кадров по 1/60 = 0.3с — ровно SHOOT_COOLDOWN.
            at_60_fps.tick(1.0 / 60.0);
        }
        let mut at_120_fps = Weapon::new();
        at_120_fps.shoot(&ship);
        for _ in 0..36 {
            at_120_fps.tick(1.0 / 120.0);
        }

        assert!(
            at_60_fps.shoot(&ship).is_some(),
            "60 FPS: cooldown не истёк за 0.3с"
        );
        assert!(
            at_120_fps.shoot(&ship).is_some(),
            "120 FPS: cooldown не истёк за 0.3с"
        );
    }

    #[test]
    fn an_expired_bullet_is_removed_while_a_live_one_stays() {
        let mut expired = Bullet::spawn(&ship());
        expired.time_to_live = -0.01;
        let live = Bullet::spawn(&ship());
        let mut bullets = vec![expired, live];

        remove_expired(&mut bullets);

        assert_eq!(
            bullets.len(),
            1,
            "после удаления истёкших пуль осталось {} штук, ожидалась 1",
            bullets.len()
        );
    }

    #[test]
    fn a_bullet_wraps_at_the_field_edge() {
        let mut bullet = Bullet::spawn(&ship());
        bullet.position.x = -5.0;

        bullet.wrap(field());

        assert!(
            (bullet.position.x - 955.0).abs() < TOLERANCE,
            "пуля за левой границей дала x = {}, ожидалось 955",
            bullet.position.x
        );
    }

    #[test]
    fn a_bullet_outruns_a_ship_at_its_speed_limit() {
        let mut ship = ship();
        // Пять секунд тяги — заведомо достаточно, чтобы разогнаться до предела скорости корабля
        // (прецедент `sustained_thrust_never_exceeds_the_speed_limit` в `ship.rs`).
        for _ in 0..300 {
            ship.apply_thrust(1.0 / 60.0);
        }
        let ship_speed = ship.velocity.length();

        let bullet = Bullet::spawn(&ship);

        assert!(
            bullet.velocity.length() > ship_speed,
            "скорость пули {} не превысила скорость разогнанного корабля {} — пуля отстанет",
            bullet.velocity.length(),
            ship_speed
        );
    }
}
