use macroquad::prelude::*;

use crate::asteroid::Asteroid;
use crate::bullet::{Bullet, BULLET_RADIUS};
use crate::collision;
use crate::score::Score;

/// **Ловушка позиции — добавление осколков во время обхода коллекции** (описана в backlog): осколки
/// собираются в локальный `Vec`, попавшие пули и астероиды отмечаются, после обхода — `retain` по
/// обеим коллекциям и `append` осколков. Одна пуля уничтожает не больше одного астероида за кадр
/// (`break` сразу после первого попадания).
///
/// Возвращает позиции уничтоженных астероидов — не для игровой логики, а для короткого визуального
/// эффекта разрушения (`T-AST-10`, `Game::effects`), чтобы эта функция осталась источником правды о
/// том, что именно было уничтожено в этом кадре.
///
/// СТАБ: возвращает пустой вектор — RED должен показать, что уничтоженные позиции не сообщаются.
pub fn resolve(
    bullets: &mut Vec<Bullet>,
    asteroids: &mut Vec<Asteroid>,
    score: &mut Score,
) -> Vec<Vec2> {
    let mut bullet_hit = vec![false; bullets.len()];
    let mut asteroid_hit = vec![false; asteroids.len()];
    let mut debris = Vec::new();
    let mut destroyed_positions = Vec::new();

    for (bullet_index, bullet) in bullets.iter().enumerate() {
        for (asteroid_index, asteroid) in asteroids.iter().enumerate() {
            if asteroid_hit[asteroid_index] {
                continue;
            }
            if collision::circles_overlap(
                bullet.position,
                BULLET_RADIUS,
                asteroid.position,
                asteroid.radius(),
            ) {
                bullet_hit[bullet_index] = true;
                asteroid_hit[asteroid_index] = true;
                score.add(asteroid.size.points());
                debris.extend(asteroid.split());
                destroyed_positions.push(asteroid.position);
                // Одна пуля уничтожает не больше одного астероида за кадр.
                break;
            }
        }
    }

    let mut bullet_index = 0;
    bullets.retain(|_| {
        let keep = !bullet_hit[bullet_index];
        bullet_index += 1;
        keep
    });
    let mut asteroid_index = 0;
    asteroids.retain(|_| {
        let keep = !asteroid_hit[asteroid_index];
        asteroid_index += 1;
        keep
    });
    asteroids.append(&mut debris);

    destroyed_positions
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::asteroid::AsteroidSize;

    fn bullet_at(position: Vec2) -> Bullet {
        Bullet::at(position, Vec2::ZERO)
    }

    #[test]
    fn a_hit_removes_the_bullet() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: Vec2::ZERO,
            size: AsteroidSize::Large,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert!(bullets.is_empty(), "попавшая пуля не исчезла");
    }

    #[test]
    fn hitting_a_large_asteroid_removes_it_and_adds_two_medium() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(30.0, 0.0),
            size: AsteroidSize::Large,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert_eq!(
            asteroids.len(),
            2,
            "после попадания в Large осталось {} астероидов, ожидалось 2 (Medium)",
            asteroids.len()
        );
        assert!(
            asteroids.iter().all(|a| a.size == AsteroidSize::Medium),
            "осколки Large — не все Medium"
        );
    }

    #[test]
    fn hitting_a_medium_asteroid_removes_it_and_adds_two_small() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(30.0, 0.0),
            size: AsteroidSize::Medium,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert_eq!(
            asteroids.len(),
            2,
            "после попадания в Medium осталось {} астероидов, ожидалось 2 (Small)",
            asteroids.len()
        );
        assert!(
            asteroids.iter().all(|a| a.size == AsteroidSize::Small),
            "осколки Medium — не все Small"
        );
    }

    #[test]
    fn hitting_a_small_asteroid_removes_it_and_adds_nothing() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(30.0, 0.0),
            size: AsteroidSize::Small,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert!(
            asteroids.is_empty(),
            "после попадания в Small осталось {} астероидов, ожидалось 0",
            asteroids.len()
        );
    }

    #[test]
    fn the_two_fragments_fly_apart_in_different_directions() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: vec2(30.0, 0.0),
            size: AsteroidSize::Large,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert!(
            (asteroids[0].velocity - asteroids[1].velocity).length() > 1.0,
            "осколки полетели в одну сторону: {:?} и {:?}",
            asteroids[0].velocity,
            asteroids[1].velocity
        );
    }

    #[test]
    fn score_is_credited_by_the_size_of_the_destroyed_asteroid() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: Vec2::ZERO,
            size: AsteroidSize::Small,
        }];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert_eq!(
            score.value(),
            AsteroidSize::Small.points(),
            "счёт после уничтожения Small — {}, ожидалось {}",
            score.value(),
            AsteroidSize::Small.points()
        );
    }

    #[test]
    fn destroying_a_small_asteroid_scores_more_than_a_large_one() {
        assert!(
            AsteroidSize::Small.points() > AsteroidSize::Large.points(),
            "Small ({}) не дороже Large ({})",
            AsteroidSize::Small.points(),
            AsteroidSize::Large.points()
        );
    }

    #[test]
    fn one_bullet_destroys_exactly_one_asteroid_even_when_two_overlap() {
        // Прецедент `a_ball_touching_two_bricks_bounces_once` из `T-ARK-5`: пуля на стыке двух
        // перекрывающихся астероидов не должна разбить оба.
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![
            Asteroid {
                position: vec2(100.0, 100.0),
                velocity: Vec2::ZERO,
                size: AsteroidSize::Small,
            },
            Asteroid {
                position: vec2(105.0, 100.0),
                velocity: Vec2::ZERO,
                size: AsteroidSize::Small,
            },
        ];
        let mut score = Score::new();

        resolve(&mut bullets, &mut asteroids, &mut score);

        assert_eq!(
            asteroids.len(),
            1,
            "после попадания на стыке двух Small осталось {} астероидов, ожидался 1",
            asteroids.len()
        );
    }

    #[test]
    fn a_hit_reports_the_destroyed_asteroids_position() {
        let mut bullets = vec![bullet_at(vec2(100.0, 100.0))];
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: Vec2::ZERO,
            size: AsteroidSize::Small,
        }];
        let mut score = Score::new();

        let destroyed = resolve(&mut bullets, &mut asteroids, &mut score);

        assert_eq!(
            destroyed,
            vec![vec2(100.0, 100.0)],
            "уничтоженные позиции — {:?}, ожидалось [(100, 100)]",
            destroyed
        );
    }

    #[test]
    fn a_frame_without_a_hit_reports_no_destroyed_asteroids() {
        let mut bullets: Vec<Bullet> = Vec::new();
        let mut asteroids = vec![Asteroid {
            position: vec2(100.0, 100.0),
            velocity: Vec2::ZERO,
            size: AsteroidSize::Small,
        }];
        let mut score = Score::new();

        let destroyed = resolve(&mut bullets, &mut asteroids, &mut score);

        assert!(
            destroyed.is_empty(),
            "кадр без попаданий сообщил уничтоженные позиции: {:?}",
            destroyed
        );
    }
}
