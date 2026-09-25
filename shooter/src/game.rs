use macroquad::prelude::*;

use crate::bullet::{self, Bullet};
use crate::collision;
use crate::combat;
use crate::enemy::{self, Enemy};
use crate::pickup::{self, Pickup};
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

/// Простой `enum` без state-machine framework (прямое требование задания).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    WaitingToStart,
    Playing,
    WaveCompleted,
    GameOver,
}

impl GameState {
    /// Текст, который видит игрок в этом состоянии; `wave` — номер волны, которая начнётся
    /// следующей (нужен только для `WaveCompleted`). Латиница — кириллические глифы встроенного
    /// шрифта Macroquad наблюдением не проверены (тот же прецедент, что в Pong/Arkanoid/Asteroids).
    pub fn message(self, wave: u32) -> Option<String> {
        match self {
            GameState::WaitingToStart => Some("Press SPACE to start".to_string()),
            GameState::Playing => None,
            GameState::WaveCompleted => Some(format!("Wave {wave}\nPress SPACE to continue")),
            GameState::GameOver => Some("GAME OVER\nPress R to restart".to_string()),
        }
    }
}

/// Короткий визуальный эффект на месте убитого врага — растущий и затухающий круг, ничего общего с
/// системой частиц (D-41 её запрещает первой версии): одна точка и один таймер жизни, тот же приём,
/// что у `Bullet::time_to_live`.
const EFFECT_DURATION: f32 = 0.3;

pub struct Effect {
    pub position: Vec2,
    time_remaining: f32,
}

impl Effect {
    /// Доля прожитого времени эффекта, `0.0` — только появился, `1.0` — вот-вот исчезнет; отрисовка
    /// использует её, чтобы растить радиус и гасить яркость (не тестируется — сама отрисовка, D-10).
    pub fn progress(&self) -> f32 {
        1.0 - self.time_remaining / EFFECT_DURATION
    }
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
    pub pickups: Vec<Pickup>,
    pub effects: Vec<Effect>,
    kills: u32,
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
            pickups: Vec::new(),
            effects: Vec::new(),
            kills: 0,
            wave: 1,
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: f32) {
        if self.state == GameState::GameOver {
            // Мир не обновляется в GameOver, и сигнал старта не воскрешает законченную игру.
            return;
        }

        for effect in &mut self.effects {
            effect.time_remaining -= delta_time;
        }
        self.effects.retain(|effect| effect.time_remaining > 0.0);

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
                // Контактный урон проверяется до `combat::resolve`, на ещё не уменьшенной коллекции
                // врагов: враг, который в этом же кадре умрёт от пули, успевает нанести касание —
                // GameOver из-за этого урона решается раньше, чем опустевшая коллекция могла бы
                // переключить игру в WaveCompleted (см. проверку ниже).
                tick(&mut self.invulnerability_timer, delta_time);
                self.check_player_collision();
                let destroyed_positions =
                    combat::resolve(&mut self.bullets, &mut self.enemies, &mut self.score);
                for position in destroyed_positions {
                    self.kills += 1;
                    if let Some(kind) = pickup::pickup_for_kill(self.kills) {
                        self.pickups.push(Pickup::new(position, kind));
                    }
                    self.effects.push(Effect {
                        position,
                        time_remaining: EFFECT_DURATION,
                    });
                }
                for pickup in &mut self.pickups {
                    pickup.tick(delta_time);
                }
                pickup::remove_expired(&mut self.pickups);
                pickup::resolve(
                    &mut self.pickups,
                    self.player.position,
                    self.player.radius,
                    &mut self.health,
                    PLAYER_MAX_HEALTH,
                    &mut self.ammo,
                );
                // Проверяется последней: если касание того же кадра уже перевело игру в `GameOver`,
                // опустевшая после `combat::resolve` коллекция врагов не переводит её ещё и в
                // `WaveCompleted` — `GameOver` побеждает, потому что `self.state` уже не `Playing`.
                if self.state == GameState::Playing && self.enemies.is_empty() {
                    self.bullets.clear();
                    self.state = GameState::WaveCompleted;
                }
            }
            GameState::WaveCompleted => {
                if input.space_pressed {
                    self.advance_wave();
                }
                self.update_player(input, delta_time);
            }
            GameState::GameOver => unreachable!("обработан выше отдельным `return`"),
        }
    }

    /// Следующая волна: номер растёт, `enemy::spawn_wave` с новым номером даёт больше врагов
    /// (`enemy::enemy_count`), счёт/здоровье/патроны не трогаются — это поля матча, а не волны.
    fn advance_wave(&mut self) {
        self.wave += 1;
        self.enemies = enemy::spawn_wave(self.wave, self.arena, self.player.position);
        self.bullets.clear();
        self.state = GameState::Playing;
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

    /// Видимый признак неуязвимости после касания — отрисовка использует его, чтобы притушить или
    /// мигнуть игроком; сама отрисовка не тестируется (D-10), поэтому тестом покрыт только сам факт,
    /// что после касания это возвращает `true` (см. тесты контактного урона выше).
    pub fn is_invulnerable(&self) -> bool {
        self.invulnerability_timer > 0.0
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

    fn fire_input_with_space() -> Input {
        let mut input = still_input();
        input.space_pressed = true;
        input
    }

    fn playing_game() -> Game {
        let mut game = Game::new(arena());
        game.state = GameState::Playing;
        // Один далёкий враг — не касается игрока (не мешает тестам боеприпасов), но не даёт
        // опустевшей коллекции враг случайно завершить волну (T-TDS-11).
        game.enemies = vec![Enemy::new(vec2(10.0, 10.0))];
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

    #[test]
    fn the_fourth_kill_leaves_a_pickup_at_the_dead_enemys_position() {
        let mut game = playing_game();
        // Разные позиции на каждой итерации, чтобы проверка не могла случайно совпасть с pickup от
        // более раннего убийства (второе убийство тоже даёт pickup — `Ammo`, D-37).
        let kill_positions: [Vec2; 4] = [
            vec2(200.0, 200.0),
            vec2(300.0, 200.0),
            vec2(400.0, 200.0),
            vec2(500.0, 200.0),
        ];

        for kill_position in kill_positions {
            // Явный возврат в Playing: каждая итерация убивает единственного врага, а с T-TDS-11
            // это само по себе завершает волну — тест здесь проверяет выпадение pickup, а не переход
            // волны (он проверен отдельно), поэтому состояние переустанавливается вручную.
            game.state = GameState::Playing;
            game.enemies = vec![Enemy::new(kill_position)];
            game.enemies[0].health = 1; // одно попадание убивает
            game.bullets = vec![Bullet::new(kill_position, Vec2::X)];
            game.update(&still_input(), 1.0 / 60.0);
        }

        let health_pickups: Vec<_> = game
            .pickups
            .iter()
            .filter(|pickup| pickup.kind == pickup::PickupKind::Health)
            .collect();
        assert_eq!(
            health_pickups.len(),
            1,
            "pickups вида Health после четырёх убийств: {}, ожидался 1",
            health_pickups.len()
        );
        // Допуск больше обычного: враг успевает сделать шаг chase AI в том же кадре, где его убивает
        // пуля (`chase` идёт раньше `combat::resolve` в `Game::update`).
        assert!(
            (health_pickups[0].position - kill_positions[3]).length() < 5.0,
            "pickup появился в {:?}, ожидалась позиция четвёртого убитого врага {:?}",
            health_pickups[0].position,
            kill_positions[3]
        );
    }

    #[test]
    fn destroying_the_last_enemy_completes_the_wave() {
        let mut game = playing_game();
        game.enemies = Vec::new(); // последний враг уже уничтожен предыдущим кадром

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::WaveCompleted,
            "опустевшая коллекция не завершила волну: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn the_wave_does_not_end_while_an_enemy_remains() {
        let mut game = playing_game();
        game.enemies = vec![Enemy::new(vec2(500.0, 500.0))];

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::Playing,
            "волна завершилась при оставшемся враге: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn enemies_do_not_spawn_on_their_own_in_wave_completed() {
        let mut game = playing_game();
        game.state = GameState::WaveCompleted;
        game.enemies = Vec::new();

        game.update(&still_input(), 1.0 / 60.0);

        assert!(
            game.enemies.is_empty(),
            "враги появились в WaveCompleted без подтверждения игрока"
        );
    }

    #[test]
    fn the_next_wave_increases_the_wave_number() {
        let mut game = playing_game();
        game.state = GameState::WaveCompleted;
        let wave_before = game.wave;

        game.update(&fire_input_with_space(), 1.0 / 60.0);

        assert_eq!(
            game.wave,
            wave_before + 1,
            "номер волны после перехода — {}, ожидалось {}",
            game.wave,
            wave_before + 1
        );
    }

    #[test]
    fn the_next_wave_has_more_enemies_than_the_previous_one() {
        let mut game = playing_game();
        let first_wave_count = game.enemies.len();
        game.state = GameState::WaveCompleted;

        game.update(&fire_input_with_space(), 1.0 / 60.0);

        assert!(
            game.enemies.len() > first_wave_count,
            "следующая волна дала {} врагов, предыдущая — {}",
            game.enemies.len(),
            first_wave_count
        );
    }

    #[test]
    fn advancing_the_wave_keeps_score_health_and_ammo() {
        let mut game = playing_game();
        game.state = GameState::WaveCompleted;
        game.score.add(500);
        game.health = 40;
        game.ammo = 7;

        game.update(&fire_input_with_space(), 1.0 / 60.0);

        assert_eq!(game.score.value(), 500, "счёт после перехода волны");
        assert_eq!(game.health, 40, "здоровье после перехода волны");
        assert_eq!(game.ammo, 7, "патроны после перехода волны");
        assert_eq!(
            game.state,
            GameState::Playing,
            "переход волны не вернул игру в Playing: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn game_over_at_the_same_moment_as_the_last_kill_beats_wave_completed() {
        let mut game = playing_game();
        game.health = CONTACT_DAMAGE; // следующее касание убивает игрока
                                      // Единственный оставшийся враг гибнет от пули в этом же кадре и одновременно касается игрока.
        game.enemies = vec![Enemy::new(game.player.position)];
        game.enemies[0].health = 1;
        game.bullets = vec![Bullet::new(game.player.position, Vec2::X)];

        game.update(&still_input(), 1.0 / 60.0);

        assert_eq!(
            game.state,
            GameState::GameOver,
            "смерть последнего врага и урон игроку в одном кадре не дали GameOver: состояние {:?}",
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
            GameState::WaveCompleted.message(2),
            Some("Wave 2\nPress SPACE to continue".to_string()),
            "WaveCompleted"
        );
        assert_eq!(
            GameState::GameOver.message(2),
            Some("GAME OVER\nPress R to restart".to_string()),
            "GameOver"
        );
    }

    #[test]
    fn is_invulnerable_reflects_the_cooldown_after_a_touch() {
        let mut game = game_with_enemy_on_player();
        assert!(
            !game.is_invulnerable(),
            "новая игра уже неуязвима без касания"
        );

        game.update(&still_input(), 1.0 / 60.0);

        assert!(
            game.is_invulnerable(),
            "касание не включило неуязвимость: is_invulnerable() = false"
        );
    }

    #[test]
    fn a_kill_leaves_a_death_effect_that_fades_and_disappears() {
        let mut game = playing_game();
        let kill_position = vec2(300.0, 300.0);
        game.enemies = vec![Enemy::new(kill_position)];
        game.enemies[0].health = 1;
        game.bullets = vec![Bullet::new(kill_position, Vec2::X)];

        game.update(&still_input(), 1.0 / 60.0);
        assert_eq!(game.effects.len(), 1, "убийство не оставило эффект");
        assert!(
            (game.effects[0].position - kill_position).length() < 5.0,
            "эффект появился в {:?}, ожидалась позиция убитого врага {:?}",
            game.effects[0].position,
            kill_position
        );

        game.update(&still_input(), EFFECT_DURATION + 0.01);
        assert!(
            game.effects.is_empty(),
            "эффект пережил свою длительность: осталось {}",
            game.effects.len()
        );
    }
}
