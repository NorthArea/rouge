use macroquad::prelude::*;

use crate::arena::Arena;

/// Скорость врага, пикселей в секунду.
const ENEMY_SPEED: f32 = 90.0;
/// Радиус столкновения врага (D-38 — круг с кругом).
pub const ENEMY_RADIUS: f32 = 18.0;
/// Здоровье врага: больше одной единицы, чтобы `T-TDS-7` могло проверить смерть на N-м попадании,
/// а не на первом же.
pub const ENEMY_HEALTH: i32 = 3;
/// Минимальное расстояние от игрока, на котором может появиться враг — чтобы он не оказался у него
/// под ногами в момент спавна.
const MIN_SPAWN_DISTANCE: f32 = 300.0;
/// Отступ от края арены, чтобы враг не появлялся ровно на границе.
const SPAWN_MARGIN: f32 = 40.0;

pub struct Enemy {
    pub position: Vec2,
    /// Первый потребитель вне тестов — попадание пули (`T-TDS-7`).
    #[allow(dead_code)]
    pub health: i32,
    pub radius: f32,
}

impl Enemy {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            health: ENEMY_HEALTH,
            radius: ENEMY_RADIUS,
        }
    }

    /// Идёт прямо к цели — никакого обхода препятствий, никакого pathfinding (прямое требование
    /// задания). Враг ровно в позиции цели не даёт `NaN` и остаётся на месте: `try_normalize()` на
    /// нулевом векторе возвращает `None`, и шаг в этом кадре пропускается.
    pub fn chase(&mut self, target: Vec2, delta_time: f32) {
        let direction = target - self.position;
        let Some(normalized_direction) = direction.try_normalize() else {
            return;
        };
        self.position += normalized_direction * ENEMY_SPEED * delta_time;
    }
}

/// Раскладка волны детерминирована (D-37): позиции считаются по номеру волны и индексу врага, а не
/// через `rand`. Число врагов растёт с номером волны (`T-TDS-11` использует ту же формулу дальше).
pub fn enemy_count(wave: u32) -> u32 {
    3 + (wave - 1) * 2
}

/// Враги раскладываются по периметру арены циклически по четырём краям; отступ от края и зависящий от
/// волны и индекса сдвиг вдоль края дают разные, но воспроизводимые позиции. Позиции, оказавшиеся
/// ближе `MIN_SPAWN_DISTANCE` к игроку, отодвигаются от него вдоль той же прямой — расстояние остаётся
/// детерминированным, а не превращается в отбраковку и повторный бросок.
pub fn spawn_wave(wave: u32, arena: Arena, player_position: Vec2) -> Vec<Enemy> {
    (0..enemy_count(wave))
        .map(|index| {
            let position = position_on_perimeter(wave, index, arena);
            Enemy::new(push_away_from_player(position, player_position))
        })
        .collect()
}

fn position_on_perimeter(wave: u32, index: u32, arena: Arena) -> Vec2 {
    let edge = index % 4;
    // Детерминированный, но не тривиально совпадающий с равномерным шагом сдвиг вдоль края.
    let step = (wave * 7 + index * 13) % 97;
    let fraction = step as f32 / 97.0;
    match edge {
        0 => vec2(fraction * arena.width, SPAWN_MARGIN),
        1 => vec2(arena.width - SPAWN_MARGIN, fraction * arena.height),
        2 => vec2(
            arena.width - fraction * arena.width,
            arena.height - SPAWN_MARGIN,
        ),
        _ => vec2(SPAWN_MARGIN, arena.height - fraction * arena.height),
    }
}

fn push_away_from_player(position: Vec2, player_position: Vec2) -> Vec2 {
    let offset = position - player_position;
    let distance = offset.length();
    if distance >= MIN_SPAWN_DISTANCE || distance == 0.0 {
        return position;
    }
    player_position + offset / distance * MIN_SPAWN_DISTANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn arena() -> Arena {
        Arena::new(1920.0, 1200.0)
    }

    #[test]
    fn an_enemy_moves_closer_to_the_player_each_step() {
        let mut enemy = Enemy::new(vec2(0.0, 0.0));
        let player_position = vec2(500.0, 500.0);
        let distance_before = (player_position - enemy.position).length();

        enemy.chase(player_position, 1.0 / 60.0);

        let distance_after = (player_position - enemy.position).length();
        assert!(
            distance_after < distance_before,
            "расстояние после шага — {}, до шага было {}",
            distance_after,
            distance_before
        );
    }

    #[test]
    fn an_enemy_exactly_at_the_player_position_does_not_move_or_produce_nan() {
        let mut enemy = Enemy::new(vec2(300.0, 300.0));
        let start = enemy.position;

        enemy.chase(start, 1.0 / 60.0);

        assert!(
            (enemy.position - start).length() < TOLERANCE,
            "враг сдвинулся на нулевом расстоянии до цели: {:?}",
            enemy.position
        );
        assert!(
            enemy.position.x.is_finite() && enemy.position.y.is_finite(),
            "координаты врага стали нечисловыми: {:?}",
            enemy.position
        );
    }

    #[test]
    fn chasing_for_a_second_covers_the_same_distance_at_60_and_120_fps() {
        let target = vec2(10_000.0, 0.0); // далеко: направление не меняется за секунду погони
        let mut at_60_fps = Enemy::new(vec2(0.0, 0.0));
        for _ in 0..60 {
            at_60_fps.chase(target, 1.0 / 60.0);
        }
        let mut at_120_fps = Enemy::new(vec2(0.0, 0.0));
        for _ in 0..120 {
            at_120_fps.chase(target, 1.0 / 120.0);
        }

        assert!(
            (at_60_fps.position.x - ENEMY_SPEED).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось {}",
            at_60_fps.position.x,
            ENEMY_SPEED
        );
        assert!(
            (at_60_fps.position.x - at_120_fps.position.x).abs() < TOLERANCE,
            "60 FPS дало x = {}, 120 FPS дало x = {}",
            at_60_fps.position.x,
            at_120_fps.position.x
        );
    }

    #[test]
    fn spawning_the_same_wave_twice_gives_the_same_positions() {
        let player_position = vec2(960.0, 600.0);

        let first = spawn_wave(3, arena(), player_position);
        let second = spawn_wave(3, arena(), player_position);

        assert_eq!(first.len(), second.len(), "число врагов разошлось");
        for (a, b) in first.iter().zip(second.iter()) {
            assert!(
                (a.position - b.position).length() < TOLERANCE,
                "позиции разошлись: {:?} vs {:?}",
                a.position,
                b.position
            );
        }
    }

    #[test]
    fn no_enemy_spawns_closer_than_the_minimum_distance_or_outside_the_arena() {
        let player_position = vec2(960.0, 600.0);

        let enemies = spawn_wave(5, arena(), player_position);

        assert!(
            !enemies.is_empty(),
            "волна не дала врагов, тест некорректен"
        );
        for enemy in &enemies {
            let distance = (enemy.position - player_position).length();
            assert!(
                distance >= MIN_SPAWN_DISTANCE - TOLERANCE,
                "враг в {:?} появился в {} от игрока, ожидалось не меньше {}",
                enemy.position,
                distance,
                MIN_SPAWN_DISTANCE
            );
            assert!(
                enemy.position.x >= 0.0
                    && enemy.position.x <= arena().width
                    && enemy.position.y >= 0.0
                    && enemy.position.y <= arena().height,
                "враг появился вне арены: {:?}",
                enemy.position
            );
        }
    }
}
