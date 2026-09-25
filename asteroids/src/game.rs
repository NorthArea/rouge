use macroquad::prelude::*;

use crate::asteroid::{self, Asteroid};
use crate::bullet::{self, Bullet, Weapon};
use crate::collision;
use crate::combat;
use crate::score::Score;
use crate::ship::Ship;
use crate::Field;

/// Три жизни (пункт чек-листа 11).
const LIVES: u32 = 3;
/// Пауза перед respawn, секунд.
const RESPAWN_DELAY: f32 = 2.0;
/// Неуязвимость после respawn, секунд.
const INVULNERABILITY_DURATION: f32 = 2.0;

/// `WaveCompleted` здесь не объявлен: у него не было бы потребителя до `T-AST-9` (первый потребитель —
/// завершение волны), а необъявленный вариант — dead code под `-D warnings` (тот же случай, что
/// `LevelCompleted` в `T-ARK-7`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    WaitingToStart,
    Playing,
    PlayerDestroyed,
    GameOver,
}

/// Ровно четыре таймера во всей игре (D-29): cooldown выстрела и lifetime пули (в `Bullet`/`Weapon`,
/// `T-AST-5`), пауза перед respawn и неуязвимость (здесь, поля `f32`, убывающие на `delta_time`).
///
/// **Решение по scope, принятое при реализации `T-AST-8`** (тот же приём, что для `velocity`
/// корабля на `T-AST-2`, `AsteroidSize` на `T-AST-6`/`T-AST-7` и `Score::value` здесь же): номер
/// волны не хранится полем `Game` — читать и увеличивать его начинает только завершение волны
/// (`T-AST-9`), а поле без чтения вне теста — `-D dead-code` под `cargo build --workspace`. Волна
/// пока живёт неявно: `spawn_wave(1, field)` — единственный вызов, и он уже есть в `T-AST-6`.
pub struct Game {
    pub field: Field,
    pub ship: Ship,
    pub bullets: Vec<Bullet>,
    weapon: Weapon,
    pub asteroids: Vec<Asteroid>,
    pub score: Score,
    pub lives: u32,
    pub state: GameState,
    respawn_timer: f32,
    invulnerability_timer: f32,
}

impl Game {
    pub fn new(field: Field) -> Self {
        Self {
            field,
            ship: Ship::new(center(field)),
            bullets: Vec::new(),
            weapon: Weapon::new(),
            asteroids: asteroid::spawn_wave(1, field),
            score: Score::new(),
            lives: LIVES,
            state: GameState::WaitingToStart,
            respawn_timer: 0.0,
            invulnerability_timer: 0.0,
        }
    }

    /// Одна клавиша `Space` (`space_pressed`) делает два дела в зависимости от состояния — тот же
    /// приём, что в `T-AST-5`, только теперь состояние есть по-настоящему: в `WaitingToStart`
    /// начинает раунд и не стреляет, в `Playing` стреляет. `PlayerDestroyed` возвращается в
    /// `Playing` по истечении паузы таймером, а не нажатием клавиши — respawn автоматический.
    pub fn update(&mut self, turn: f32, thrusting: bool, space_pressed: bool, delta_time: f32) {
        if self.state == GameState::GameOver {
            // Кадр ничего не меняет, и сигнал старта не воскрешает законченную игру.
            return;
        }

        match self.state {
            GameState::WaitingToStart => {
                if space_pressed {
                    self.state = GameState::Playing;
                }
                self.update_ship(turn, thrusting, delta_time);
            }
            GameState::Playing => {
                self.update_ship(turn, thrusting, delta_time);
                self.weapon.tick(delta_time);
                if space_pressed {
                    if let Some(bullet) = self.weapon.shoot(&self.ship) {
                        self.bullets.push(bullet);
                    }
                }
            }
            GameState::PlayerDestroyed => {
                self.respawn_timer -= delta_time;
                if self.respawn_timer <= 0.0 {
                    self.ship.position = center(self.field);
                    self.ship.velocity = Vec2::ZERO;
                    self.ship.angle = 0.0;
                    self.invulnerability_timer = INVULNERABILITY_DURATION;
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => unreachable!("обработан выше отдельным `return`"),
        }

        for bullet in &mut self.bullets {
            bullet.advance(delta_time);
            bullet.wrap(self.field);
        }
        bullet::remove_expired(&mut self.bullets);
        for asteroid in &mut self.asteroids {
            asteroid.advance(delta_time);
            asteroid.wrap(self.field);
        }
        combat::resolve(&mut self.bullets, &mut self.asteroids, &mut self.score);

        if self.state == GameState::Playing {
            self.check_ship_collision();
        }
    }

    fn update_ship(&mut self, turn: f32, thrusting: bool, delta_time: f32) {
        self.ship.rotate(turn, delta_time);
        self.ship
            .apply_thrust(if thrusting { delta_time } else { 0.0 });
        self.ship.advance(delta_time);
        self.ship.wrap(self.field);
        if self.invulnerability_timer > 0.0 {
            self.invulnerability_timer -= delta_time;
        }
    }

    fn check_ship_collision(&mut self) {
        if self.invulnerability_timer > 0.0 {
            return;
        }
        let hit = self.asteroids.iter().any(|asteroid| {
            collision::circles_overlap(
                self.ship.position,
                self.ship.collision_radius(),
                asteroid.position,
                asteroid.radius(),
            )
        });
        if !hit {
            return;
        }
        self.lives -= 1;
        if self.lives == 0 {
            self.state = GameState::GameOver;
        } else {
            self.state = GameState::PlayerDestroyed;
            self.respawn_timer = RESPAWN_DELAY;
        }
    }

    /// Полный рестарт из любого состояния — новый матч через тот же конструктор.
    pub fn restart(&mut self) {
        *self = Game::new(self.field);
    }
}

fn center(field: Field) -> Vec2 {
    vec2(field.width / 2.0, field.height / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asteroid::AsteroidSize;

    const TOLERANCE: f32 = 0.01;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    fn stationary_asteroid_at(position: Vec2) -> Asteroid {
        Asteroid {
            position,
            velocity: Vec2::ZERO,
            size: AsteroidSize::Large,
        }
    }

    #[test]
    fn a_collision_with_an_asteroid_costs_a_life_and_destroys_the_player() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        let ship_position = game.ship.position;
        game.asteroids = vec![stationary_asteroid_at(ship_position)];

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(
            game.lives,
            LIVES - 1,
            "жизней после столкновения — {}, ожидалось {}",
            game.lives,
            LIVES - 1
        );
        assert_eq!(
            game.state,
            GameState::PlayerDestroyed,
            "состояние после столкновения — {:?}, ожидалось PlayerDestroyed",
            game.state
        );
    }

    #[test]
    fn an_invulnerable_ship_does_not_lose_a_life_to_the_same_collision() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.invulnerability_timer = 1.0;
        let ship_position = game.ship.position;
        game.asteroids = vec![stationary_asteroid_at(ship_position)];

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(
            game.lives, LIVES,
            "неуязвимый корабль потерял жизнь: осталось {}, ожидалось {}",
            game.lives, LIVES
        );
        assert_eq!(
            game.state,
            GameState::Playing,
            "неуязвимый корабль был уничтожен: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn invulnerability_expires_and_the_next_collision_costs_a_life() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.invulnerability_timer = 0.05;
        let ship_position = game.ship.position;
        game.asteroids = vec![stationary_asteroid_at(ship_position)];

        // delta_time больше оставшейся неуязвимости: таймер истекает в этом же кадре, и то же
        // столкновение уже должно быть засчитано.
        game.update(0.0, false, false, 0.1);

        assert_eq!(
            game.lives,
            LIVES - 1,
            "жизней после истечения неуязвимости и столкновения — {}, ожидалось {}",
            game.lives,
            LIVES - 1
        );
    }

    #[test]
    fn the_respawn_pause_counts_down_by_delta_time_and_resets_the_ship() {
        let mut game = Game::new(field());
        game.state = GameState::PlayerDestroyed;
        game.respawn_timer = RESPAWN_DELAY;
        game.ship.position = vec2(10.0, 10.0);
        game.ship.velocity = vec2(50.0, -50.0);
        game.ship.angle = 1.0;

        game.update(0.0, false, false, RESPAWN_DELAY - 0.1);
        assert_eq!(
            game.state,
            GameState::PlayerDestroyed,
            "пауза истекла раньше срока: состояние {:?}",
            game.state
        );

        game.update(0.0, false, false, 0.2);
        assert_eq!(
            game.state,
            GameState::Playing,
            "по истечении паузы состояние — {:?}, ожидалось Playing",
            game.state
        );
        let expected_center = center(field());
        assert!(
            (game.ship.position - expected_center).length() < TOLERANCE,
            "корабль после respawn в {:?}, ожидался центр {:?}",
            game.ship.position,
            expected_center
        );
        assert!(
            game.ship.velocity.length() < TOLERANCE,
            "скорость корабля после respawn — {:?}, ожидался ноль",
            game.ship.velocity
        );
        assert!(
            game.ship.angle.abs() < TOLERANCE,
            "угол корабля после respawn — {}, ожидался 0",
            game.ship.angle
        );
    }

    #[test]
    fn losing_the_last_life_ends_the_game() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.lives = 1;
        let ship_position = game.ship.position;
        game.asteroids = vec![stationary_asteroid_at(ship_position)];

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(game.lives, 0, "жизней осталось {}, ожидалось 0", game.lives);
        assert_eq!(
            game.state,
            GameState::GameOver,
            "потеря последней жизни не дала GameOver: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn game_over_freezes_the_frame_and_ignores_the_start_signal() {
        let mut game = Game::new(field());
        game.state = GameState::GameOver;
        let ship_before = game.ship.position;
        game.asteroids = vec![Asteroid {
            position: vec2(500.0, 500.0),
            velocity: vec2(50.0, 50.0),
            size: AsteroidSize::Large,
        }];

        game.update(1.0, true, true, 1.0);

        assert!(
            (game.ship.position - ship_before).length() < TOLERANCE,
            "корабль сдвинулся в GameOver: {:?}",
            game.ship.position
        );
        assert!(
            (game.asteroids[0].position - vec2(500.0, 500.0)).length() < TOLERANCE,
            "астероид сдвинулся в GameOver: {:?}",
            game.asteroids[0].position
        );
        assert_eq!(
            game.state,
            GameState::GameOver,
            "сигнал старта воскресил законченную игру: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn the_start_signal_begins_the_round_from_waiting_to_start() {
        let mut game = Game::new(field());

        game.update(0.0, false, true, 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::Playing,
            "сигнал старта не начал раунд: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn a_full_restart_returns_everything_to_the_start() {
        let mut game = Game::new(field());
        game.state = GameState::GameOver;
        game.lives = 0;
        game.score.add(900);
        game.ship.position = vec2(5.0, 5.0);
        game.bullets = vec![Bullet::at(vec2(1.0, 1.0), Vec2::ZERO)];
        game.asteroids = vec![stationary_asteroid_at(vec2(1.0, 1.0))];

        game.restart();

        assert_eq!(game.lives, LIVES, "жизни после рестарта — {}", game.lives);
        assert_eq!(
            game.state,
            GameState::WaitingToStart,
            "состояние после рестарта — {:?}",
            game.state
        );
        assert!(game.bullets.is_empty(), "пули пережили рестарт");
        let expected_center = center(field());
        assert!(
            (game.ship.position - expected_center).length() < TOLERANCE,
            "корабль после рестарта не в центре: {:?}",
            game.ship.position
        );
    }
}
