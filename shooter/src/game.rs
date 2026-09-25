use macroquad::prelude::*;

use crate::bullet::{self, Bullet};
use crate::collision;
use crate::combat;
use crate::enemy::{self, Enemy};
use crate::player::Player;
use crate::score::Score;
use crate::Arena;

/// Максимальное здоровье игрока — число, а не три жизни (в отличие от Asteroids).
const PLAYER_MAX_HEALTH: i32 = 100;
/// Урон одного касания врага.
const CONTACT_DAMAGE: i32 = 20;
/// Неуязвимость после касания, секунд (D-40).
const INVULNERABILITY_DURATION: f32 = 1.0;
/// Начальный запас патронов — при cooldown выстрела 0.25с этого хватает примерно на 10 секунд
/// непрерывной стрельбы, чтобы расход до нуля укладывался в один сеанс игры в несколько минут.
const START_AMMO: i32 = 40;

/// Простой `enum` без state-machine framework (прямое требование задания). `WaveCompleted`
/// добавляется на `T-TDS-11`, когда для него появится поведение.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    WaitingToStart,
    Playing,
    GameOver,
}

/// Ввод одного кадра, уже прочитанный Macroquad-стадией `run()`: `Game` не читает клавиатуру и мышь
/// сама (D-10) и вызывается из тестов без окна.
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool,
    pub cursor_world: Vec2,
    pub space_pressed: bool,
}

pub struct Game {
    pub arena: Arena,
    pub player: Player,
    pub bullets: Vec<Bullet>,
    shoot_cooldown: f32,
    pub enemies: Vec<Enemy>,
    pub score: Score,
    pub state: GameState,
    pub health: i32,
    invulnerability_timer: f32,
    pub ammo: i32,
    /// Первый потребитель вне тестов — волны (`T-TDS-11`).
    #[allow(dead_code)]
    pub wave: u32,
}

impl Game {
    pub fn new(arena: Arena) -> Self {
        let player = Player::new(vec2(arena.width / 2.0, arena.height / 2.0));
        let enemies = enemy::spawn_wave(1, arena, player.position);
        Self {
            arena,
            player,
            bullets: Vec::new(),
            shoot_cooldown: 0.0,
            enemies,
            score: Score::new(),
            state: GameState::WaitingToStart,
            health: PLAYER_MAX_HEALTH,
            invulnerability_timer: 0.0,
            ammo: START_AMMO,
            wave: 1,
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: f32) {
        if self.state == GameState::GameOver {
            // Мир не обновляется в GameOver, и сигнал старта не воскрешает законченную игру.
            return;
        }

        match self.state {
            GameState::WaitingToStart => {
                if input.space_pressed {
                    self.state = GameState::Playing;
                }
                self.update_player(input, delta_time);
            }
            GameState::Playing => {
                self.update_player(input, delta_time);
                bullet::tick_cooldown(&mut self.shoot_cooldown, delta_time);
                // При нуле патронов cooldown не трогается: проверка запаса стоит раньше вызова
                // `bullet::shoot`, а не внутри него — ЛКМ без патронов не запускает cooldown впустую.
                if input.fire && self.ammo > 0 {
                    if let Some(new_bullet) = bullet::shoot(&mut self.shoot_cooldown, &self.player)
                    {
                        self.bullets.push(new_bullet);
                        self.ammo -= 1;
                    }
                }
                for bullet in &mut self.bullets {
                    bullet.advance(delta_time);
                }
                bullet::remove_expired(&mut self.bullets, self.arena);
                for enemy in &mut self.enemies {
                    enemy.chase(self.player.position, delta_time);
                }
                combat::resolve(&mut self.bullets, &mut self.enemies, &mut self.score);
                tick(&mut self.invulnerability_timer, delta_time);
                self.check_player_collision();
            }
            GameState::GameOver => unreachable!("обработан выше отдельным `return`"),
        }
    }

    fn update_player(&mut self, input: &Input, delta_time: f32) {
        self.player
            .update(input.up, input.down, input.left, input.right, delta_time);
        self.player.clamp_to_arena(self.arena);
        self.player.aim_at(input.cursor_world);
    }

    /// Контакт с врагом наносит урон не каждый кадр: пока идёт неуязвимость (`invulnerability_timer`
    /// уже уменьшен в `update` до вызова этой функции), касание не проверяется. Несколько врагов,
    /// коснувшихся игрока в одном кадре, засчитываются как одно касание — `any()` останавливается на
    /// первом найденном.
    fn check_player_collision(&mut self) {
        if self.invulnerability_timer > 0.0 {
            return;
        }
        let touched = self.enemies.iter().any(|enemy| {
            collision::circles_overlap(
                self.player.position,
                self.player.radius,
                enemy.position,
                enemy.radius,
            )
        });
        if !touched {
            return;
        }
        self.health -= CONTACT_DAMAGE;
        self.invulnerability_timer = INVULNERABILITY_DURATION;
        if self.health <= 0 {
            self.state = GameState::GameOver;
        }
    }

    /// Полный рестарт из любого состояния — новый матч через тот же конструктор, а не ручной сброс
    /// полей по одному.
    pub fn restart(&mut self) {
        *self = Game::new(self.arena);
    }
}

/// Уменьшает таймер на delta time, не поднимая его обратно к нулю раньше срока: при большом
/// `delta_time` (когда неуязвимость истекает в этом же кадре) значение уходит в отрицательное, и
/// последующая проверка `> 0.0` в том же кадре уже пропускает касание.
fn tick(timer: &mut f32, delta_time: f32) {
    if *timer > 0.0 {
        *timer -= delta_time;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arena() -> Arena {
        Arena::new(1920.0, 1200.0)
    }

    fn still_input() -> Input {
        Input {
            up: false,
            down: false,
            left: false,
            right: false,
            fire: false,
            cursor_world: vec2(0.0, 0.0),
            space_pressed: false,
        }
    }

    fn game_with_enemy_on_player() -> Game {
        let mut game = Game::new(arena());
        game.state = GameState::Playing;
        game.enemies = vec![Enemy::new(game.player.position)];
        game
    }

    fn fire_input() -> Input {
        let mut input = still_input();
        input.fire = true;
        input
    }

    fn playing_game() -> Game {
        let mut game = Game::new(arena());
        game.state = GameState::Playing;
        game.enemies = Vec::new(); // без врагов, чтобы контактный урон не мешал тестам боеприпасов
        game
    }

    #[test]
    fn touching_an_enemy_reduces_the_players_health() {
        let mut game = game_with_enemy_on_player();

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.health,
            PLAYER_MAX_HEALTH - CONTACT_DAMAGE,
            "здоровье после касания — {}, ожидалось {}",
            game.health,
            PLAYER_MAX_HEALTH - CONTACT_DAMAGE
        );
    }

    #[test]
    fn a_second_touch_within_the_cooldown_deals_no_extra_damage() {
        let mut game = game_with_enemy_on_player();
        game.update(&still_input(), 1.0 / 60.0);
        let health_after_first_hit = game.health;

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.health, health_after_first_hit,
            "повторное касание в пределах cooldown нанесло урон: {}",
            game.health
        );
    }

    #[test]
    fn a_touch_after_the_cooldown_expires_deals_damage_again() {
        let mut game = game_with_enemy_on_player();
        game.update(&still_input(), 1.0 / 60.0);
        let health_after_first_hit = game.health;

        game.update(&still_input(), INVULNERABILITY_DURATION + 0.01);

        assert_eq!(
            game.health,
            health_after_first_hit - CONTACT_DAMAGE,
            "касание после истечения cooldown не нанесло урон: {}",
            game.health
        );
    }

    #[test]
    fn two_enemies_touching_at_once_deal_a_single_touchs_damage() {
        let mut game = Game::new(arena());
        game.state = GameState::Playing;
        game.enemies = vec![
            Enemy::new(game.player.position),
            Enemy::new(game.player.position),
        ];

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.health,
            PLAYER_MAX_HEALTH - CONTACT_DAMAGE,
            "два одновременных касания нанесли не одно попадание: здоровье {}",
            game.health
        );
    }

    #[test]
    fn zero_health_ends_the_game_and_freezes_the_world() {
        let mut game = game_with_enemy_on_player();
        game.health = CONTACT_DAMAGE;

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::GameOver,
            "здоровье ноль не дало GameOver: состояние {:?}",
            game.state
        );

        let enemy_position_before = game.enemies[0].position;
        let mut moving_input = still_input();
        moving_input.right = true;
        game.update(&moving_input, 1.0);

        assert_eq!(
            game.enemies[0].position, enemy_position_before,
            "мир обновился в GameOver: враг сдвинулся"
        );
    }

    #[test]
    fn restart_resets_health_score_wave_bullets_and_enemies_from_game_over() {
        let mut game = game_with_enemy_on_player();
        game.state = GameState::GameOver;
        game.health = 0;
        game.score.add(500);
        game.wave = 3;
        game.bullets = vec![Bullet::new(vec2(1.0, 1.0), Vec2::X)];

        game.restart();

        assert_eq!(game.health, PLAYER_MAX_HEALTH, "здоровье после рестарта");
        assert_eq!(game.score.value(), 0, "счёт после рестарта");
        assert_eq!(game.wave, 1, "номер волны после рестарта");
        assert!(game.bullets.is_empty(), "пули пережили рестарт");
        assert_eq!(
            game.state,
            GameState::WaitingToStart,
            "состояние после рестарта — {:?}",
            game.state
        );
    }

    #[test]
    fn restart_also_works_from_playing() {
        let mut game = game_with_enemy_on_player();
        game.score.add(100);

        game.restart();

        assert_eq!(game.score.value(), 0, "счёт после рестарта из Playing");
        assert_eq!(
            game.state,
            GameState::WaitingToStart,
            "состояние после рестарта из Playing — {:?}",
            game.state
        );
    }

    #[test]
    fn firing_a_shot_reduces_ammo_by_exactly_one() {
        let mut game = playing_game();
        let ammo_before = game.ammo;

        game.update(&fire_input(), 1.0 / 60.0);

        assert_eq!(
            game.ammo,
            ammo_before - 1,
            "запас после выстрела — {}, ожидалось {}",
            game.ammo,
            ammo_before - 1
        );
    }

    #[test]
    fn firing_with_zero_ammo_does_not_create_a_bullet() {
        let mut game = playing_game();
        game.ammo = 0;

        game.update(&fire_input(), 1.0 / 60.0);

        assert!(
            game.bullets.is_empty(),
            "выстрел при нуле патронов создал пулю"
        );
    }

    #[test]
    fn ammo_never_goes_negative() {
        let mut game = playing_game();
        game.ammo = 1;

        // Два выстрела подряд: cooldown между ними выдержан явно, чтобы проверить именно ammo,
        // а не cooldown.
        game.update(&fire_input(), 1.0 / 60.0);
        game.update(&still_input(), 1.0);
        game.update(&fire_input(), 1.0 / 60.0);

        assert!(game.ammo >= 0, "запас патронов ушёл в минус: {}", game.ammo);
    }

    #[test]
    fn the_last_bullet_fires_and_the_next_click_does_not() {
        let mut game = playing_game();
        game.ammo = 1;

        game.update(&fire_input(), 1.0 / 60.0);
        assert_eq!(game.bullets.len(), 1, "последний патрон не выстрелил");

        game.update(&still_input(), 1.0);
        game.update(&fire_input(), 1.0 / 60.0);

        assert_eq!(
            game.bullets.len(),
            1,
            "выстрел при нуле патронов после последнего создал вторую пулю"
        );
    }
}
