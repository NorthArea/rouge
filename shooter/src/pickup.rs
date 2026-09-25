use macroquad::prelude::*;

use crate::collision;

/// Радиус столкновения pickup (D-38).
pub const PICKUP_RADIUS: f32 = 10.0;
/// Время жизни pickup, секунд (D-40) — арена не зарастает подборами, которые никто не поднял.
const PICKUP_LIFETIME: f32 = 8.0;
/// Сколько здоровья восстанавливает `Health`.
const HEALTH_RESTORE: i32 = 30;
/// Сколько патронов добавляет `Ammo`.
const AMMO_RESTORE: i32 = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickupKind {
    Health,
    Ammo,
}

pub struct Pickup {
    pub position: Vec2,
    pub kind: PickupKind,
    pub radius: f32,
    time_to_live: f32,
}

impl Pickup {
    pub fn new(position: Vec2, kind: PickupKind) -> Self {
        Self {
            position,
            kind,
            radius: PICKUP_RADIUS,
            time_to_live: PICKUP_LIFETIME,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.time_to_live -= delta_time;
    }

    pub fn is_expired(&self) -> bool {
        self.time_to_live <= 0.0
    }
}

/// Выпадение детерминировано от счётчика убитых врагов, а не от `rand` (D-37): каждый четвёртый
/// убитый враг оставляет `Health`, каждый второй (не считая уже покрытых четвёртым) — `Ammo`,
/// остальные — ничего. `kill_number` — единица для первого убитого врага, а не ноль.
pub fn pickup_for_kill(kill_number: u32) -> Option<PickupKind> {
    if kill_number.is_multiple_of(4) {
        Some(PickupKind::Health)
    } else if kill_number.is_multiple_of(2) {
        Some(PickupKind::Ammo)
    } else {
        None
    }
}

/// Подбор — circle-to-circle с игроком; `Health` не поднимает здоровье выше максимума, `Ammo`
/// добавляет запас без потолка. Подобранный pickup удаляется одним `retain`.
pub fn resolve(
    pickups: &mut Vec<Pickup>,
    player_position: Vec2,
    player_radius: f32,
    health: &mut i32,
    max_health: i32,
    ammo: &mut i32,
) {
    pickups.retain(|pickup| {
        let touched = collision::circles_overlap(
            player_position,
            player_radius,
            pickup.position,
            pickup.radius,
        );
        if touched {
            match pickup.kind {
                PickupKind::Health => *health = (*health + HEALTH_RESTORE).min(max_health),
                PickupKind::Ammo => *ammo += AMMO_RESTORE,
            }
        }
        !touched
    });
}

/// Pickups с истёкшим lifetime исчезают одним `retain`, без ручных индексов.
pub fn remove_expired(pickups: &mut Vec<Pickup>) {
    pickups.retain(|pickup| !pickup.is_expired());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fourth_kill_drops_health() {
        assert_eq!(pickup_for_kill(4), Some(PickupKind::Health));
    }

    #[test]
    fn the_second_kill_drops_ammo() {
        assert_eq!(pickup_for_kill(2), Some(PickupKind::Ammo));
    }

    #[test]
    fn the_first_kill_drops_nothing() {
        assert_eq!(pickup_for_kill(1), None);
    }

    #[test]
    fn the_rule_is_deterministic() {
        assert_eq!(pickup_for_kill(7), pickup_for_kill(7));
        assert_eq!(pickup_for_kill(12), pickup_for_kill(12));
    }

    #[test]
    fn a_health_pickup_increases_health_without_exceeding_the_maximum() {
        let mut pickups = vec![Pickup::new(vec2(100.0, 100.0), PickupKind::Health)];
        let mut health = 90;
        let mut ammo = 0;

        resolve(
            &mut pickups,
            vec2(100.0, 100.0),
            16.0,
            &mut health,
            100,
            &mut ammo,
        );

        assert_eq!(
            health, 100,
            "здоровье после подбора — {}, ожидалось 100 (потолок)",
            health
        );
    }

    #[test]
    fn a_health_pickup_below_the_maximum_restores_the_full_amount() {
        let mut pickups = vec![Pickup::new(vec2(100.0, 100.0), PickupKind::Health)];
        let mut health = 50;
        let mut ammo = 0;

        resolve(
            &mut pickups,
            vec2(100.0, 100.0),
            16.0,
            &mut health,
            100,
            &mut ammo,
        );

        assert_eq!(
            health,
            50 + HEALTH_RESTORE,
            "здоровье после подбора — {}",
            health
        );
    }

    #[test]
    fn an_ammo_pickup_increases_the_ammo_supply() {
        let mut pickups = vec![Pickup::new(vec2(100.0, 100.0), PickupKind::Ammo)];
        let mut health = 100;
        let mut ammo = 5;

        resolve(
            &mut pickups,
            vec2(100.0, 100.0),
            16.0,
            &mut health,
            100,
            &mut ammo,
        );

        assert_eq!(ammo, 5 + AMMO_RESTORE, "запас после подбора — {}", ammo);
    }

    #[test]
    fn a_picked_up_pickup_is_removed_and_not_picked_up_again() {
        let mut pickups = vec![Pickup::new(vec2(100.0, 100.0), PickupKind::Ammo)];
        let mut health = 100;
        let mut ammo = 0;

        resolve(
            &mut pickups,
            vec2(100.0, 100.0),
            16.0,
            &mut health,
            100,
            &mut ammo,
        );
        assert!(pickups.is_empty(), "подобранный pickup не удалился");

        resolve(
            &mut pickups,
            vec2(100.0, 100.0),
            16.0,
            &mut health,
            100,
            &mut ammo,
        );

        assert_eq!(
            ammo, AMMO_RESTORE,
            "pickup подобран повторно: ammo = {}",
            ammo
        );
    }

    #[test]
    fn a_pickup_disappears_after_its_lifetime() {
        let mut pickup = Pickup::new(vec2(0.0, 0.0), PickupKind::Health);
        pickup.tick(PICKUP_LIFETIME - 0.1);
        assert!(!pickup.is_expired(), "pickup истёк раньше своего lifetime");

        pickup.tick(0.2);
        assert!(pickup.is_expired(), "pickup пережил свой lifetime");
    }
}
