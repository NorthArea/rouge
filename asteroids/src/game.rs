use macroquad::prelude::*;

use crate::asteroid::{self, Asteroid};
use crate::bullet::{self, Bullet};
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    WaitingToStart,
    Playing,
    PlayerDestroyed,
    WaveCompleted,
    GameOver,
}

impl GameState {
    /// Текст, который видит игрок в этом состоянии; `wave` — номер волны, которая начнётся следующей
    /// (нужен только для `WaveCompleted`). Латиница, потому что кириллические глифы встроенного
    /// шрифта Macroquad наблюдением не проверены (уже записано в Pong и Arkanoid).
    pub fn message(self, wave: u32) -> Option<String> {
        match self {
            GameState::WaitingToStart => Some("Press SPACE to start".to_string()),
            GameState::Playing => None,
            GameState::PlayerDestroyed => Some("Ship destroyed".to_string()),
            GameState::WaveCompleted => Some(format!("Wave {wave}")),
            GameState::GameOver => Some("GAME OVER\nPress R to restart".to_string()),
        }
    }
}

/// Короткий визуальный эффект на месте уничтоженного астероида — растущий и затухающий круг, ничего
/// общего с particle system (D-31 запрещает её первой версии): одна точка и один таймер жизни, тот же
/// приём, что у `Bullet::time_to_live`.
const EFFECT_DURATION: f32 = 0.3;

pub struct Effect {
    pub position: Vec2,
    time_remaining: f32,
}

impl Effect {
    /// Доля прожитого времени эффекта, `0.0` — только появился, `1.0` — вот-вот исчезнет; отрисовка
    /// использует её, чтобы растить радиус и гасить яркость (untested по D-10, сама отрисовка).
    pub fn progress(&self) -> f32 {
        1.0 - self.time_remaining / EFFECT_DURATION
    }
}

/// Ровно четыре таймера *в игре* (D-29: cooldown выстрела, lifetime пули, пауза перед respawn,
/// неуязвимость) — `time_remaining` эффекта разрушения не входит в их число: это чисто визуальные
/// данные, отделённые от игровой логики (столкновений, respawn, волн), и D-29 их не считает.
pub struct Game {
    pub field: Field,
    pub ship: Ship,
    pub bullets: Vec<Bullet>,
    shoot_cooldown: f32,
    pub asteroids: Vec<Asteroid>,
    pub effects: Vec<Effect>,
    pub score: Score,
    pub lives: u32,
    pub wave: u32,
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
            shoot_cooldown: 0.0,
            asteroids: asteroid::spawn_wave(1, field),
            effects: Vec::new(),
            score: Score::new(),
            lives: LIVES,
            wave: 1,
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
                bullet::tick_cooldown(&mut self.shoot_cooldown, delta_time);
                if space_pressed {
                    if let Some(bullet) = bullet::shoot(&mut self.shoot_cooldown, &self.ship) {
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
            GameState::WaveCompleted => {
                if space_pressed {
                    self.advance_wave();
                }
                self.update_ship(turn, thrusting, delta_time);
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
        let destroyed = combat::resolve(&mut self.bullets, &mut self.asteroids, &mut self.score);
        for position in destroyed {
            self.effects.push(Effect {
                position,
                time_remaining: EFFECT_DURATION,
            });
        }
        for effect in &mut self.effects {
            effect.time_remaining -= delta_time;
        }
        self.effects.retain(|effect| effect.time_remaining > 0.0);

        if self.state == GameState::Playing {
            self.check_ship_collision();
        }
        // Пули с прошлой волны снимаются здесь же, в кадре, где астероиды опустели.
        if self.state == GameState::Playing && self.asteroids.is_empty() {
            self.bullets.clear();
            self.state = GameState::WaveCompleted;
        }
    }

    /// Следующая волна: номер растёт, счёт и жизни не трогаются — они не поля волны, а поля матча.
    fn advance_wave(&mut self) {
        self.wave += 1;
        self.asteroids = asteroid::spawn_wave(self.wave, self.field);
        self.bullets.clear();
        self.state = GameState::Playing;
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

    #[test]
    fn destroying_the_last_asteroid_completes_the_wave() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.asteroids = vec![]; // последний астероид уже уничтожен предыдущим кадром

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::WaveCompleted,
            "опустевшая коллекция не завершила волну: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn the_wave_does_not_end_while_an_asteroid_remains() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.asteroids = vec![stationary_asteroid_at(vec2(500.0, 500.0))];

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::Playing,
            "волна завершилась при оставшемся астероиде: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn the_next_wave_increases_the_wave_number() {
        let mut game = Game::new(field());
        game.state = GameState::WaveCompleted;
        let wave_before = game.wave;

        game.update(0.0, false, true, 1.0 / 60.0);

        assert_eq!(
            game.wave,
            wave_before + 1,
            "номер волны после перехода — {}, ожидалось {}",
            game.wave,
            wave_before + 1
        );
    }

    #[test]
    fn the_next_wave_has_more_large_asteroids_than_the_previous_one() {
        let mut game = Game::new(field());
        let first_wave_count = game.asteroids.len();
        game.state = GameState::WaveCompleted;

        game.update(0.0, false, true, 1.0 / 60.0);

        assert!(
            game.asteroids.len() > first_wave_count,
            "следующая волна дала {} астероидов, предыдущая — {}",
            game.asteroids.len(),
            first_wave_count
        );
    }

    #[test]
    fn advancing_to_the_next_wave_keeps_the_score_and_lives() {
        let mut game = Game::new(field());
        game.state = GameState::WaveCompleted;
        game.score.add(500);
        game.lives = 2;

        game.update(0.0, false, true, 1.0 / 60.0);

        assert_eq!(game.lives, 2, "жизни после перехода волны — {}", game.lives);
        assert_eq!(
            game.score.value(),
            500,
            "счёт после перехода волны — {}",
            game.score.value()
        );
        assert_eq!(
            game.state,
            GameState::Playing,
            "переход волны не вернул игру в Playing: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn each_state_shows_its_own_message() {
        assert_eq!(
            GameState::WaitingToStart.message(2),
            Some("Press SPACE to start".to_string()),
            "WaitingToStart"
        );
        assert_eq!(
            GameState::Playing.message(2),
            None,
            "Playing должен быть без сообщения"
        );
        assert_eq!(
            GameState::PlayerDestroyed.message(2),
            Some("Ship destroyed".to_string()),
            "PlayerDestroyed"
        );
        assert_eq!(
            GameState::WaveCompleted.message(2),
            Some("Wave 2".to_string()),
            "WaveCompleted"
        );
        assert_eq!(
            GameState::GameOver.message(2),
            Some("GAME OVER\nPress R to restart".to_string()),
            "GameOver"
        );
    }

    #[test]
    fn destroying_an_asteroid_spawns_an_effect_at_its_position() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.asteroids = vec![stationary_asteroid_at(vec2(50.0, 50.0))];
        game.bullets = vec![Bullet::at(vec2(50.0, 50.0), Vec2::ZERO)];

        game.update(0.0, false, false, 1.0 / 60.0);

        assert_eq!(
            game.effects.len(),
            1,
            "эффектов после уничтожения — {}, ожидался 1",
            game.effects.len()
        );
        assert!(
            (game.effects[0].position - vec2(50.0, 50.0)).length() < TOLERANCE,
            "эффект появился в {:?}, ожидалось (50, 50)",
            game.effects[0].position
        );
    }

    #[test]
    fn an_effect_disappears_after_its_duration() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.asteroids = vec![stationary_asteroid_at(vec2(50.0, 50.0))];
        game.bullets = vec![Bullet::at(vec2(50.0, 50.0), Vec2::ZERO)];
        game.update(0.0, false, false, 1.0 / 60.0);
        assert_eq!(
            game.effects.len(),
            1,
            "эффект не появился, тест некорректен"
        );

        game.update(0.0, false, false, EFFECT_DURATION + 0.01);

        assert!(
            game.effects.is_empty(),
            "эффект пережил свою длительность: осталось {}",
            game.effects.len()
        );
    }
}
