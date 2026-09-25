use macroquad::prelude::*;

use crate::Field;

/// Число крупных астероидов первой волны.
const START_ASTEROIDS: u32 = 4;
/// Потолок числа крупных астероидов волны, чтобы поздние волны не превращались в кашу.
const MAX_WAVE_ASTEROIDS: u32 = 12;
/// Минимальное расстояние от центра поля (стартовой позиции корабля) до центра заспавненного
/// астероида: сумма примерного радиуса корабля, радиуса `Large` и запаса, чтобы волна не убивала
/// игрока в момент старта. Радиус столкновения корабля появится только в `T-AST-8` (D-27), поэтому
/// здесь используется консервативная оценка, а не публичная константа корабля.
const MIN_SPAWN_DISTANCE: f32 = 180.0;
/// Угол отклонения осколка от направления родителя, симметрично в обе стороны.
const SPLIT_ANGLE: f32 = std::f32::consts::FRAC_PI_6;

/// Обычное перечисление без иерархий и trait'ов (D-02, D-25): радиус, скорость и очки — чистые
/// функции размера, а не отдельные структуры-стратегии. `Medium` и `Small` добавлены здесь, в
/// `T-AST-7`, вместе с первым потребителем — разбиением (см. `Asteroid::split`); на `T-AST-6` у них
/// не было бы потребителя вне теста, что `cargo build --workspace` ловит как dead code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    pub fn radius(self) -> f32 {
        match self {
            AsteroidSize::Large => 40.0,
            AsteroidSize::Medium => 24.0,
            AsteroidSize::Small => 12.0,
        }
    }

    /// Скорость растёт при уменьшении размера — мелкий осколок отскакивает быстрее крупного
    /// родителя (классическое поведение Asteroids).
    pub fn speed(self) -> f32 {
        match self {
            AsteroidSize::Large => 60.0,
            AsteroidSize::Medium => 100.0,
            AsteroidSize::Small => 150.0,
        }
    }

    /// Мелкую цель труднее сбить, поэтому она дороже — классическое Asteroids, а не опечатка
    /// (объясняется в README, `T-AST-12`).
    pub fn points(self) -> u32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }
}

pub struct Asteroid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: AsteroidSize,
}

impl Asteroid {
    pub fn radius(&self) -> f32 {
        self.size.radius()
    }

    pub fn advance(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }

    /// То же правило заворачивания, что у корабля и пули (D-28); повторяется здесь, а не выносится в
    /// общий модуль (D-02).
    pub fn wrap(&mut self, field: Field) {
        self.position.x = self.position.x.rem_euclid(field.width);
        self.position.y = self.position.y.rem_euclid(field.height);
    }

    /// `Large` → два `Medium`, `Medium` → два `Small`, `Small` — пустой вектор. Осколки летят из
    /// той же точки, что и родитель, отклонённые от направления родителя на фиксированный угол в
    /// разные стороны (детерминированно, без RNG — D-30) и на повышенной скорости своего размера.
    pub fn split(&self) -> Vec<Asteroid> {
        let child_size = match self.size {
            AsteroidSize::Large => AsteroidSize::Medium,
            AsteroidSize::Medium => AsteroidSize::Small,
            AsteroidSize::Small => return Vec::new(),
        };
        let direction = if self.velocity.length() > 0.0 {
            self.velocity.normalize()
        } else {
            Vec2::X
        };

        [SPLIT_ANGLE, -SPLIT_ANGLE]
            .into_iter()
            .map(|angle| Asteroid {
                position: self.position,
                velocity: Vec2::from_angle(angle).rotate(direction) * child_size.speed(),
                size: child_size,
            })
            .collect()
    }
}

/// Детерминированная раскладка волны (D-30): кольцо вокруг центра поля с равным шагом угла, без
/// генератора случайных чисел. Смещение угла зависит от номера волны, чтобы последующие волны
/// (`T-AST-9`) не ложились ровно поверх предыдущей раскладки.
pub fn spawn_wave(wave: u32, field: Field) -> Vec<Asteroid> {
    let center = vec2(field.width / 2.0, field.height / 2.0);
    let count = (START_ASTEROIDS + wave.saturating_sub(1)).min(MAX_WAVE_ASTEROIDS);
    let ring_radius = MIN_SPAWN_DISTANCE + 60.0;
    let wave_offset = (wave.saturating_sub(1)) as f32 * std::f32::consts::FRAC_PI_6;

    (0..count)
        .map(|i| {
            let angle = wave_offset + (i as f32) * std::f32::consts::TAU / (count as f32);
            let direction = vec2(angle.cos(), angle.sin());
            Asteroid {
                position: center + direction * ring_radius,
                // Разлетаются от центра, чтобы новая волна сразу расходилась по полю, а не стояла
                // на месте до первого кадра со случайным направлением (которого в проекте нет, D-30).
                velocity: direction * AsteroidSize::Large.speed(),
                size: AsteroidSize::Large,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    #[test]
    fn moving_covers_the_same_distance_regardless_of_frame_rate() {
        let mut at_60_fps = Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(120.0, 0.0),
            size: AsteroidSize::Large,
        };
        for _ in 0..60 {
            at_60_fps.advance(1.0 / 60.0);
        }
        let mut at_120_fps = Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(120.0, 0.0),
            size: AsteroidSize::Large,
        };
        for _ in 0..120 {
            at_120_fps.advance(1.0 / 120.0);
        }

        assert!(
            (at_60_fps.position.x - 220.0).abs() < TOLERANCE,
            "60 кадров по 1/60: x = {}, ожидалось 220",
            at_60_fps.position.x
        );
        assert!(
            (at_60_fps.position.x - at_120_fps.position.x).abs() < TOLERANCE,
            "60 FPS дало x = {}, 120 FPS дало x = {}",
            at_60_fps.position.x,
            at_120_fps.position.x
        );
    }

    #[test]
    fn wraps_at_the_field_edge() {
        let mut asteroid = Asteroid {
            position: vec2(-5.0, 300.0),
            velocity: Vec2::ZERO,
            size: AsteroidSize::Large,
        };

        asteroid.wrap(field());

        assert!(
            (asteroid.position.x - 955.0).abs() < TOLERANCE,
            "выход за левую границу дал x = {}, ожидалось 955",
            asteroid.position.x
        );
    }

    #[test]
    fn the_first_wave_has_the_expected_number_of_large_asteroids() {
        let wave = spawn_wave(1, field());

        assert_eq!(
            wave.len(),
            START_ASTEROIDS as usize,
            "первая волна дала {} астероидов, ожидалось {}",
            wave.len(),
            START_ASTEROIDS
        );
        assert!(
            wave.iter().all(|a| a.size == AsteroidSize::Large),
            "не все астероиды первой волны — Large"
        );
    }

    #[test]
    fn a_later_wave_has_more_large_asteroids_than_the_first() {
        let first = spawn_wave(1, field());
        let later = spawn_wave(3, field());

        assert!(
            later.len() > first.len(),
            "волна 3 дала {} астероидов, волна 1 — {}, ожидался рост",
            later.len(),
            first.len()
        );
    }

    #[test]
    fn wave_asteroid_count_never_exceeds_the_cap() {
        let far_wave = spawn_wave(50, field());

        assert!(
            far_wave.len() as u32 <= MAX_WAVE_ASTEROIDS,
            "волна 50 дала {} астероидов, потолок {}",
            far_wave.len(),
            MAX_WAVE_ASTEROIDS
        );
    }

    #[test]
    fn no_asteroid_in_the_starting_wave_spawns_too_close_to_the_center() {
        let center = vec2(field().width / 2.0, field().height / 2.0);
        let wave = spawn_wave(1, field());

        for asteroid in &wave {
            let distance = (asteroid.position - center).length();
            assert!(
                distance >= MIN_SPAWN_DISTANCE,
                "астероид в {:?} стоит в {} от центра, ожидалось не меньше {}",
                asteroid.position,
                distance,
                MIN_SPAWN_DISTANCE
            );
        }
    }
}
