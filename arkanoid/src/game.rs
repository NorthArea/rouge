use crate::ball::Ball;
use crate::brick::{self, Brick, BrickKind};
use crate::level;
use crate::paddle::Paddle;
use crate::score::Score;
use crate::Field;

/// Сколько раз игрок может потерять мяч, прежде чем игра закончится.
const LIVES: u32 = 3;

/// Состояние игры. Простой перечислимый тип и ничего больше: переходов немного, они видны в `update`
/// целиком, и никакой машины состояний для них не нужно.
#[derive(Debug, PartialEq)]
pub enum GameState {
    /// Мяч лежит на ракетке и ждёт сигнала старта.
    WaitingToStart,
    /// Раунд идёт: мяч летит, считаются столкновения и потеря мяча.
    Playing,
    /// Мяч только что потерян, жизнь списана. Состояние удерживается до `Space` (D-20).
    LifeLost,
    /// Жизни кончились. Из этого состояния игра сама не выходит.
    GameOver,
    /// Уровень пройден. Состояние удерживается до `Space` (D-20).
    LevelCompleted,
    /// Пройден последний уровень. Из этого состояния игра сама не выходит.
    GameWon,
}

/// Игра целиком: поле, ракетка, мяч, блоки, счёт, жизни и состояние. Это единственное место, где
/// игровые части встречаются вместе, поэтому переходы между состояниями живут здесь, а не в кадре.
pub struct Game {
    pub field: Field,
    pub paddle: Paddle,
    pub ball: Ball,
    pub bricks: Vec<Brick>,
    pub score: Score,
    pub lives: u32,
    /// Номер текущего уровня, начиная с первого. Его видит игрок, поэтому нумерация с единицы.
    pub level: usize,
    pub state: GameState,
}

impl Game {
    /// Начало игры: счёт нулевой, три жизни, полная раскладка блоков, мяч на ракетке.
    pub fn new(field: Field) -> Self {
        let paddle = Paddle::new(field);
        let ball = Ball::resting_on(&paddle);

        Self {
            field,
            paddle,
            ball,
            bricks: level::bricks(level::FIRST, field),
            score: Score::default(),
            lives: LIVES,
            level: level::FIRST,
            state: GameState::WaitingToStart,
        }
    }

    /// Сигнал старта раунда. Приходит из кадра нажатием `Space`; сама клавиша сюда не попадает,
    /// поэтому переход проверяется unit-тестом без окна и без Macroquad (D-10).
    pub fn start_round(&mut self) {
        match self.state {
            // Раунд начинается из ожидания — и в начале игры, и после потерянной жизни.
            GameState::WaitingToStart | GameState::LifeLost => self.state = GameState::Playing,
            // Пройденный уровень сменяется следующим, и тот снова ждёт подачи.
            GameState::LevelCompleted => self.advance_level(),
            // В идущем раунде сигнал ничего не значит, а законченную игру он не воскрешает.
            GameState::Playing | GameState::GameOver | GameState::GameWon => {}
        }
    }

    /// Сигнал полного рестарта. Как и старт раунда, приходит из кадра нажатием клавиши, а сюда
    /// попадает уже как сигнал, поэтому проверяется unit-тестом без окна (D-10).
    pub fn restart(&mut self) {
        // Начать игру заново — это ровно начало игры, поэтому рестарт выражен через конструктор:
        // правило исходного состояния живёт в одном месте и покрыто его тестами.
        *self = Self::new(self.field);
    }

    /// Следующий уровень: новая раскладка, мяч на ракетке, ожидание подачи. Счёт и жизни переходят
    /// с игроком — уровень сменился, а игра та же.
    fn advance_level(&mut self) {
        self.level += 1;
        self.bricks = level::bricks(self.level, self.field);
        self.ball.rest_on(&self.paddle);
        self.state = GameState::WaitingToStart;
    }

    /// Обновление за один кадр. Направление ракетки и delta time приходят аргументами, поэтому
    /// обновление вызывается из теста без окна и без Macroquad (D-10).
    pub fn update(&mut self, direction: f32, delta_time: f32) {
        // Ракетка слушается игрока в любом состоянии: до подачи он занимает ею позицию.
        self.paddle.update(direction, self.field, delta_time);

        match self.state {
            GameState::Playing => self.play(delta_time),
            // Игра закончена — проигрышем или победой: кадр больше ничего не двигает и не считает.
            GameState::GameOver | GameState::GameWon => {}
            // Мяч ждёт подачи и ездит на ракетке.
            GameState::WaitingToStart | GameState::LifeLost | GameState::LevelCompleted => {
                self.ball.rest_on(&self.paddle)
            }
        }
    }

    /// Идущий раунд. Стадии кадра идут в том же порядке, что и раньше: перемещение мяча, затем
    /// проверка столкновений.
    fn play(&mut self, delta_time: f32) {
        self.ball.update(delta_time);
        self.ball.bounce_off_walls(self.field);
        self.ball.bounce_off_paddle(&self.paddle);
        brick::bounce_off_bricks(&mut self.ball, &mut self.bricks, &mut self.score);

        if self.level_cleared() {
            // Последний уровень заканчивается победой сразу, без промежуточного ожидания: следующего
            // уровня, которого можно было бы дождаться, уже нет (D-20).
            self.state = if self.level == level::count() {
                GameState::GameWon
            } else {
                GameState::LevelCompleted
            };
            return;
        }

        // Нижняя граница поля — не стена, а потеря: мяч считается потерянным, когда ушёл за неё
        // целиком, то есть когда его верхний край оказался ниже края поля.
        if self.ball.y > self.field.height {
            self.lose_life();
        }
    }

    /// Уровень пройден, когда разрушаемых блоков не осталось. Неразрушимые в счёт не идут: убрать их
    /// с поля нельзя, и ожидание их уничтожения заперло бы игрока на уровне навсегда.
    fn level_cleared(&self) -> bool {
        self.bricks
            .iter()
            .filter(|brick| brick.kind != BrickKind::Indestructible)
            .all(|brick| brick.destroyed())
    }

    /// Мяч потерян: жизнь списывается, мяч возвращается на ракетку, раунд ждёт нового `Space`.
    /// Если жизней не осталось, ждать больше нечего — игра закончена.
    fn lose_life(&mut self) {
        self.lives -= 1;
        self.ball.rest_on(&self.paddle);
        self.state = if self.lives == 0 {
            GameState::GameOver
        } else {
            GameState::LifeLost
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paddle::{RIGHT, STILL};

    /// Кадр теста намеренно длинный: ожидаемые положения в утверждениях должны читаться как
    /// арифметика, а не как результат подстановки игровых констант.
    const FRAME: f32 = 0.5;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Идущий раунд с единственным блоком, в который мяч попадает в ближайшем кадре: поле уровня
    /// заменено целиком, чтобы завершение уровня не зависело от его настоящей раскладки.
    fn round_with_one_brick_left(kind: BrickKind) -> Game {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.bricks = vec![Brick::new(300.0, 100.0, 80.0, 24.0, kind)];
        game.ball.x = 330.0;
        game.ball.y = 114.0;
        game.ball.velocity_x = 0.0;
        game.ball.velocity_y = 0.0;
        game
    }

    /// Идущий раунд, в котором мяч уже целиком ушёл вниз и остановлен: потеря жизни не должна
    /// зависеть от того, сколько мяч пролетит за проверяемый кадр.
    fn round_with_the_ball_lost() -> Game {
        let mut game = Game::new(field());
        game.state = GameState::Playing;
        game.ball.y = game.field.height + 1.0;
        game.ball.velocity_x = 0.0;
        game.ball.velocity_y = 0.0;
        game
    }

    #[test]
    fn the_start_signal_begins_the_round() {
        let mut game = Game::new(field());

        game.start_round();

        assert_eq!(
            game.state,
            GameState::Playing,
            "сигнал старта не начал раунд: состояние {:?}",
            game.state
        );
    }

    /// До подачи мяч не летит сам, а ездит на ракетке: раунд ещё не начался.
    #[test]
    fn waiting_to_start_keeps_the_ball_on_the_paddle() {
        let mut game = Game::new(field());
        let started_at = game.paddle.x;

        game.update(RIGHT, FRAME);

        assert!(
            game.paddle.x > started_at,
            "ракетка не сдвинулась до подачи: x = {}",
            game.paddle.x
        );
        // Центр мяча над центром ракетки, а его низ — вплотную к её верху.
        assert_eq!(
            game.ball.x,
            game.paddle.x + (game.paddle.width - game.ball.size) / 2.0,
            "мяч не последовал за ракеткой: x = {}",
            game.ball.x
        );
        assert_eq!(
            game.ball.y,
            game.paddle.y - game.ball.size,
            "мяч не лежит на ракетке: y = {}",
            game.ball.y
        );
    }

    #[test]
    fn a_lost_ball_costs_a_life_and_returns_to_the_paddle() {
        let mut game = round_with_the_ball_lost();

        game.update(STILL, FRAME);

        assert_eq!(game.lives, 2, "потеря мяча не отняла жизнь: {}", game.lives);
        assert_eq!(
            game.state,
            GameState::LifeLost,
            "потеря мяча не остановила раунд: состояние {:?}",
            game.state
        );
        assert_eq!(
            game.ball.y,
            game.paddle.y - game.ball.size,
            "мяч не вернулся на ракетку: y = {}",
            game.ball.y
        );
    }

    #[test]
    fn a_ball_inside_the_field_costs_no_life() {
        let mut game = Game::new(field());
        game.state = GameState::Playing;

        game.update(STILL, FRAME);

        assert_eq!(
            game.lives, 3,
            "мяч в игре отнял жизнь: осталось {}",
            game.lives
        );
        assert_eq!(
            game.state,
            GameState::Playing,
            "мяч в игре остановил раунд: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn losing_the_last_life_ends_the_game() {
        let mut game = round_with_the_ball_lost();
        game.lives = 1;

        game.update(STILL, FRAME);

        assert_eq!(game.lives, 0, "жизнь не списана: осталось {}", game.lives);
        assert_eq!(
            game.state,
            GameState::GameOver,
            "потеря последней жизни не завершила игру: состояние {:?}",
            game.state
        );
    }

    /// Из `GameOver` игра сама не выходит, и кадр в нём ничего не меняет: ни мяч, ни счёт.
    #[test]
    fn game_over_freezes_the_frame() {
        let mut game = Game::new(field());
        game.state = GameState::GameOver;
        game.lives = 0;
        game.score.points = 500;
        game.ball.x = 100.0;
        game.ball.y = 200.0;
        let standing_at = (game.ball.x, game.ball.y);

        game.update(RIGHT, FRAME);

        assert_eq!(
            (game.ball.x, game.ball.y),
            standing_at,
            "кадр после конца игры сдвинул мяч: ({}, {})",
            game.ball.x,
            game.ball.y
        );
        assert_eq!(
            game.score.points, 500,
            "кадр после конца игры изменил счёт: {}",
            game.score.points
        );
    }

    /// `LifeLost` удерживается до `Space` и по нему возобновляет раунд (D-20): иначе игра встала бы
    /// после первой же потерянной жизни.
    #[test]
    fn the_start_signal_resumes_the_round_after_a_lost_life() {
        let mut game = Game::new(field());
        game.state = GameState::LifeLost;

        game.start_round();

        assert_eq!(
            game.state,
            GameState::Playing,
            "сигнал старта не возобновил раунд после потери жизни: состояние {:?}",
            game.state
        );
    }

    /// Обратная сторона того же правила: из `GameOver` сигнал старта игру не воскрешает.
    #[test]
    fn the_start_signal_does_not_revive_a_finished_game() {
        let mut game = Game::new(field());
        game.state = GameState::GameOver;
        game.lives = 0;

        game.start_round();

        assert_eq!(
            game.state,
            GameState::GameOver,
            "сигнал старта воскресил законченную игру: состояние {:?}",
            game.state
        );
    }

    /// Уровень заканчивается, когда бить больше не по чему.
    #[test]
    fn clearing_the_last_breakable_brick_completes_the_level() {
        let mut game = round_with_one_brick_left(BrickKind::Normal);

        game.update(STILL, FRAME);

        assert!(
            game.bricks[0].destroyed(),
            "кадр не уничтожил последний блок"
        );
        assert_eq!(
            game.state,
            GameState::LevelCompleted,
            "уничтожение последнего блока не завершило уровень: состояние {:?}",
            game.state
        );
    }

    /// Неразрушимый блок нельзя убрать с поля, поэтому ждать его уничтожения — значит запереть игру
    /// на уровне навсегда.
    #[test]
    fn a_field_left_with_only_indestructible_bricks_is_complete() {
        let mut game = round_with_one_brick_left(BrickKind::Normal);
        game.bricks.push(Brick::new(
            600.0,
            100.0,
            80.0,
            24.0,
            BrickKind::Indestructible,
        ));

        game.update(STILL, FRAME);

        assert_eq!(
            game.state,
            GameState::LevelCompleted,
            "оставшийся неразрушимый блок не дал завершить уровень: состояние {:?}",
            game.state
        );
    }

    #[test]
    fn advancing_loads_the_next_level_and_keeps_score_and_lives() {
        let mut game = Game::new(field());
        game.state = GameState::LevelCompleted;
        game.score.points = 700;
        game.lives = 2;

        game.start_round();

        assert_eq!(
            game.level, 2,
            "переход не сменил уровень: номер {}",
            game.level
        );
        assert_eq!(
            game.state,
            GameState::WaitingToStart,
            "новый уровень не начался с ожидания подачи: состояние {:?}",
            game.state
        );
        // Новая раскладка, а не остатки пройденной: на поле снова есть что разбивать.
        assert!(
            game.bricks.iter().any(|brick| !brick.destroyed()),
            "новый уровень пришёл без блоков"
        );
        assert_eq!(
            (game.score.points, game.lives),
            (700, 2),
            "переход сбросил счёт или жизни: {} очков, {} жизней",
            game.score.points,
            game.lives
        );
    }

    #[test]
    fn clearing_the_last_level_wins_the_game() {
        let mut game = round_with_one_brick_left(BrickKind::Normal);
        game.level = level::count();

        game.update(STILL, FRAME);

        assert_eq!(
            game.state,
            GameState::GameWon,
            "последний уровень не привёл к победе: состояние {:?}",
            game.state
        );
    }

    /// Победа так же окончательна, как проигрыш: кадр в ней ничего не меняет.
    #[test]
    fn game_won_freezes_the_frame() {
        let mut game = Game::new(field());
        game.state = GameState::GameWon;
        game.score.points = 1500;
        game.ball.x = 100.0;
        game.ball.y = 200.0;
        let standing_at = (game.ball.x, game.ball.y);

        game.update(RIGHT, FRAME);

        assert_eq!(
            (game.ball.x, game.ball.y),
            standing_at,
            "кадр после победы сдвинул мяч: ({}, {})",
            game.ball.x,
            game.ball.y
        );
        assert_eq!(
            game.score.points, 1500,
            "кадр после победы изменил счёт: {}",
            game.score.points
        );
    }

    /// Рестарт доступен в любой момент, поэтому проверяются все состояния: это единственный способ
    /// начать игру заново, не выходя из неё.
    #[test]
    fn a_full_restart_returns_the_game_to_its_start() {
        for state in [
            GameState::WaitingToStart,
            GameState::Playing,
            GameState::LifeLost,
            GameState::GameOver,
            GameState::LevelCompleted,
            GameState::GameWon,
        ] {
            let started_in = format!("{state:?}");
            let mut game = Game::new(field());
            let full_field = game.bricks.len();

            // Игра в разгаре: счёт открыт, жизней меньше, уровень не первый, поле выбито.
            game.state = state;
            game.score.points = 900;
            game.lives = 1;
            game.level = 3;
            game.bricks.clear();

            game.restart();

            assert_eq!(
                (game.score.points, game.lives, game.level),
                (0, 3, level::FIRST),
                "рестарт из состояния {} не вернул игру к началу: {} очков, {} жизней, уровень {}",
                started_in,
                game.score.points,
                game.lives,
                game.level
            );
            assert_eq!(
                game.bricks.len(),
                full_field,
                "рестарт из состояния {} не вернул полную раскладку: блоков {}",
                started_in,
                game.bricks.len()
            );
            assert_eq!(
                game.state,
                GameState::WaitingToStart,
                "рестарт из состояния {} не привёл в ожидание подачи: состояние {:?}",
                started_in,
                game.state
            );
        }
    }
}
