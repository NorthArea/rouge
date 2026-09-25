use macroquad::prelude::Vec2;

use crate::bullet::{Bullet, BULLET_RADIUS};
use crate::collision;
use crate::enemy::Enemy;
use crate::score::Score;

/// Урон одной пули.
const BULLET_DAMAGE: i32 = 1;
/// Очки за смерть врага.
const ENEMY_KILL_POINTS: u32 = 10;

/// Попадания пуль по врагам за кадр. Каждая пуля обрабатывается не более одного раза (`consumed`) —
/// пуля, уже попавшая в одного врага, не проверяется против следующих; уже погибший в этом же кадре
/// враг не обрабатывается повторно, потому что удаляется из коллекции только один раз, в конце.
pub fn resolve(
    bullets: &mut Vec<Bullet>,
    enemies: &mut Vec<Enemy>,
    score: &mut Score,
) -> Vec<Vec2> {
    let mut consumed = vec![false; bullets.len()];

    for enemy in enemies.iter_mut() {
        for (bullet_index, bullet) in bullets.iter().enumerate() {
            if consumed[bullet_index] {
                continue;
            }
            let hit = collision::circles_overlap(
                bullet.position,
                BULLET_RADIUS,
                enemy.position,
                enemy.radius,
            );
            if hit {
                consumed[bullet_index] = true;
                enemy.health -= BULLET_DAMAGE;
            }
        }
    }

    let mut destroyed_positions = Vec::new();
    enemies.retain(|enemy| {
        if enemy.health <= 0 {
            destroyed_positions.push(enemy.position);
            false
        } else {
            true
        }
    });
    score.add(destroyed_positions.len() as u32 * ENEMY_KILL_POINTS);

    let mut consumed_iter = consumed.into_iter();
    bullets.retain(|_| !consumed_iter.next().unwrap());

    destroyed_positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::*;

    fn bullet_at(position: Vec2) -> Bullet {
        Bullet::new(position, Vec2::X)
    }

    #[test]
    fn a_hit_reduces_the_enemys_health_by_one_bullets_damage() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut enemies = vec![Enemy::new(vec2(100.0, 100.0))];
        let mut score = Score::new();
        let health_before = enemies[0].health;

        resolve(&mut bullets, &mut enemies, &mut score);

        assert_eq!(
            enemies[0].health,
            health_before - BULLET_DAMAGE,
            "здоровье после попадания — {}, ожидалось {}",
            enemies[0].health,
            health_before - BULLET_DAMAGE
        );
    }

    #[test]
    fn an_enemy_with_several_health_dies_on_the_nth_hit_not_the_first() {
        let mut enemies = vec![Enemy::new(vec2(100.0, 100.0))];
        let mut score = Score::new();
        let hits_to_kill = enemies[0].health;

        for _ in 0..hits_to_kill - 1 {
            let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
            resolve(&mut bullets, &mut enemies, &mut score);
        }
        assert_eq!(enemies.len(), 1, "враг погиб раньше последнего попадания");

        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        resolve(&mut bullets, &mut enemies, &mut score);

        assert!(enemies.is_empty(), "враг пережил своё N-е попадание");
    }

    #[test]
    fn an_enemys_death_awards_points_exactly_once() {
        let mut enemies = vec![Enemy::new(vec2(100.0, 100.0))];
        let hits_to_kill = enemies[0].health;
        let mut score = Score::new();

        for _ in 0..hits_to_kill {
            let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
            resolve(&mut bullets, &mut enemies, &mut score);
        }

        assert_eq!(
            score.value(),
            ENEMY_KILL_POINTS,
            "счёт после одной смерти — {}, ожидалось {}",
            score.value(),
            ENEMY_KILL_POINTS
        );
    }

    #[test]
    fn one_bullet_in_range_of_two_enemies_damages_only_one() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut enemies = vec![
            Enemy::new(vec2(100.0, 100.0)),
            Enemy::new(vec2(105.0, 100.0)),
        ];
        let health_before = enemies[0].health;
        let mut score = Score::new();

        resolve(&mut bullets, &mut enemies, &mut score);

        let damaged_count = enemies
            .iter()
            .filter(|enemy| enemy.health < health_before)
            .count();
        assert_eq!(
            damaged_count, 1,
            "одна пуля повредила {} врагов, ожидался 1",
            damaged_count
        );
    }

    #[test]
    fn a_bullet_is_removed_after_a_hit_and_does_not_hit_again_next_frame() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut enemies = vec![Enemy::new(vec2(100.0, 100.0))];
        let mut score = Score::new();

        resolve(&mut bullets, &mut enemies, &mut score);
        assert!(bullets.is_empty(), "пуля пережила попадание");

        let health_after_first_frame = enemies[0].health;
        resolve(&mut bullets, &mut enemies, &mut score);

        assert_eq!(
            enemies[0].health, health_after_first_frame,
            "удалённая пуля нанесла урон во втором кадре"
        );
    }

    #[test]
    fn a_kill_reports_the_dead_enemys_position() {
        let mut bullets = vec![bullet_at(vec2(200.0, 300.0))];
        let mut enemies = vec![Enemy::new(vec2(200.0, 300.0))];
        enemies[0].health = 1; // одно попадание убивает
        let mut score = Score::new();

        let destroyed_positions = resolve(&mut bullets, &mut enemies, &mut score);

        assert_eq!(
            destroyed_positions,
            vec![vec2(200.0, 300.0)],
            "позиции убитых — {:?}, ожидалась [(200, 300)]",
            destroyed_positions
        );
    }

    #[test]
    fn a_frame_without_a_kill_reports_no_positions() {
        let mut bullets: Vec<Bullet> = Vec::new();
        let mut enemies = vec![Enemy::new(vec2(200.0, 300.0))];
        let mut score = Score::new();

        let destroyed_positions = resolve(&mut bullets, &mut enemies, &mut score);

        assert!(
            destroyed_positions.is_empty(),
            "кадр без убийств сообщил позиции: {:?}",
            destroyed_positions
        );
    }

    #[test]
    fn a_near_miss_just_outside_the_combined_radius_is_not_a_hit() {
        let separation = BULLET_RADIUS + crate::enemy::ENEMY_RADIUS + 1.0;
        let mut bullets = vec![bullet_at(vec2(100.0 + separation, 100.0))];
        let mut enemies = vec![Enemy::new(vec2(100.0, 100.0))];
        let health_before = enemies[0].health;
        let mut score = Score::new();

        resolve(&mut bullets, &mut enemies, &mut score);

        assert_eq!(
            enemies[0].health, health_before,
            "промах мимо врага всё равно нанёс урон"
        );
        assert_eq!(bullets.len(), 1, "промахнувшаяся пуля была удалена");
    }
}
