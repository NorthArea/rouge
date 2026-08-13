use crate::ball::Ball;
use crate::paddle::Paddle;
use crate::score::Score;
use crate::Field;

/// Состояние игры. Простой перечислимый тип и ничего больше: переходов три, они видны в `update`
/// целиком, и никакой машины состояний для них не нужно.
#[derive(Debug, PartialEq)]
pub enum GameState {
    /// Мяч стоит в центре и ждёт сигнала старта.
    WaitingToStart,
    /// Раунд идёт: мяч летит, считаются столкновения и голы.
    Playing,
    /// Гол только что засчитан. Состояние живёт ровно один кадр и не удерживается таймером (D-14).
    PointScored,
}

/// Матч целиком: поле, обе ракетки, мяч, счёт и состояние. Это единственное место, где игровые части
/// встречаются вместе, поэтому переходы между состояниями живут здесь, а не в кадре.
pub struct Game {
    pub field: Field,
    pub left_paddle: Paddle,
    pub right_paddle: Paddle,
    pub ball: Ball,
    pub score: Score,
    pub state: GameState,
}

impl Game {
    /// Начало матча: счёт нулевой, мяч в центре, игра ждёт первой подачи.
    pub fn new(field: Field) -> Self {
        Self {
            field,
            left_paddle: Paddle::left(field),
            right_paddle: Paddle::right(field),
            ball: Ball::new(field),
            score: Score::default(),
            state: GameState::WaitingToStart,
        }
    }

    /// Сигнал старта раунда. Приходит из кадра нажатием `Space`; сама клавиша сюда не попадает,
    /// поэтому переход проверяется unit-тестом без окна и без Macroquad (D-10).
    pub fn start_round(&mut self) {
        // Раунд начинается только из ожидания. В идущем раунде сигнал ничего не значит, а в кадре
        // гола он пропустил бы возврат мяча в центр.
        if self.state == GameState::WaitingToStart {
            self.state = GameState::Playing;
        }
    }

    /// Сигнал рестарта матча. Как и старт раунда, приходит из кадра нажатием клавиши, а сюда попадает
    /// уже как сигнал, поэтому проверяется unit-тестом без окна (D-10).
    pub fn restart_match(&mut self) {
        // Начать матч заново — это ровно начало матча, поэтому рестарт выражен через конструктор:
        // правило исходных позиций, нулевого счёта и первой подачи живёт в одном месте.
        *self = Self::new(self.field);
    }

    /// Обновление за один кадр. Направления ракеток и delta time приходят аргументами, поэтому
    /// обновление вызывается из теста без окна и без Macroquad (D-10).
    pub fn update(&mut self, left_direction: f32, right_direction: f32, delta_time: f32) {
        // Ракетки обновляются в любом состоянии: игроки занимают позицию и до подачи, и в тот кадр,
        // когда очко уже засчитано.
        self.left_paddle
            .update(left_direction, self.field, delta_time);
        self.right_paddle
            .update(right_direction, self.field, delta_time);

        match self.state {
            // Мяч уже стоит в центре со скоростью будущей подачи и ждёт сигнала: до него в кадре
            // происходит только движение ракеток.
            GameState::WaitingToStart => {}
            GameState::Playing => self.play(delta_time),
            GameState::PointScored => self.begin_next_round(),
        }
    }

    /// Идущий раунд. Стадии кадра идут в том же порядке, что и раньше: перемещение мяча, затем
    /// проверка столкновений, затем проверка гола по уже перемещённому и отскочившему мячу.
    fn play(&mut self, delta_time: f32) {
        self.ball.update(delta_time);
        self.ball.bounce_off_field_edges(self.field);
        self.ball.bounce_off_paddle(&self.left_paddle);
        self.ball.bounce_off_paddle(&self.right_paddle);

        if self.score.count_goal(&self.ball, self.field) {
            self.state = GameState::PointScored;
        }
    }

    /// Очко уже начислено: мяч возвращается в центр, и игра сразу ждёт следующей подачи. Таймера
    /// между раундами нет (D-14), поэтому `PointScored` живёт ровно один кадр.
    fn begin_next_round(&mut self) {
        self.ball.reset(self.field);
        self.state = GameState::WaitingToStart;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paddle::{DOWN, STILL, UP};

    /// Кадр теста намеренно длинный и с круглой скоростью ракетки: ожидаемые положения в утверждениях
    /// должны читаться как арифметика, а не как результат подстановки игровых констант.
    const FRAME: f32 = 0.5;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Обе ракетки с круглыми скоростью и положением: игровой баланс к переходам состояний отношения
    /// не имеет.
    fn game_with_plain_paddles() -> Game {
        let mut game = Game::new(field());

        for paddle in [&mut game.left_paddle, &mut game.right_paddle] {
            paddle.y = 250.0;
            paddle.speed = 200.0;
        }

        game
    }

    /// Мяч, целиком ушедший за правую границу и остановленный: гол не должен зависеть от того,
    /// сколько мяч пролетит за проверяемый кадр.
    fn park_the_ball_past_the_right_edge(game: &mut Game) {
        game.ball.x = game.field.width + 1.0;
        game.ball.velocity_x = 0.0;
        game.ball.velocity_y = 0.0;
    }

    #[test]
    fn waiting_to_start_begins_playing_on_the_start_signal() {
        let mut game = Game::new(field());

        game.start_round();

        assert_eq!(
            game.state,
            GameState::Playing,
            "сигнал старта не начал раунд: состояние {:?}",
            game.state
        );
    }

    /// Решение этой задачи: ракетки управляются во всех состояниях, в том числе до подачи, чтобы
    /// игроки успевали занять позицию. Тест закрепляет именно это решение.
    #[test]
    fn paddles_move_while_waiting_for_the_serve() {
        let mut game = game_with_plain_paddles();

        game.update(UP, DOWN, FRAME);

        assert_eq!(
            game.left_paddle.y, 150.0,
            "левая ракетка не сдвинулась вверх до подачи: y = {}",
            game.left_paddle.y
        );
        assert_eq!(
            game.right_paddle.y, 350.0,
            "правая ракетка не сдвинулась вниз до подачи: y = {}",
            game.right_paddle.y
        );
    }

    #[test]
    fn a_goal_ends_the_round_with_the_point_scored() {
        let mut game = game_with_plain_paddles();
        game.state = GameState::Playing;
        park_the_ball_past_the_right_edge(&mut game);

        game.update(STILL, STILL, FRAME);

        assert_eq!(
            game.state,
            GameState::PointScored,
            "гол не завершил раунд: состояние {:?}",
            game.state
        );
        assert_eq!(
            (game.score.left, game.score.right),
            (1, 0),
            "гол не принёс очко левому игроку: счёт {}:{}",
            game.score.left,
            game.score.right
        );
    }

    /// Обратная сторона предыдущего перехода: раунд заканчивается голом и только им. Без этого теста
    /// безусловный выход из `Playing` набор пропускает — гол проверялся только со стороны гола.
    #[test]
    fn a_round_without_a_goal_keeps_playing() {
        let mut game = game_with_plain_paddles();
        game.state = GameState::Playing;
        // Мяч там, где его ставит начало матча: за проверяемый кадр он не доходит ни до боковой
        // границы, ни до границ поля по вертикали, ни до ракеток.
        let started_at = (game.ball.x, game.ball.y);

        game.update(STILL, STILL, FRAME);

        assert_eq!(
            game.state,
            GameState::Playing,
            "кадр раунда без гола завершил раунд: состояние {:?}",
            game.state
        );
        assert_eq!(
            (game.score.left, game.score.right),
            (0, 0),
            "кадр раунда без гола изменил счёт: {}:{}",
            game.score.left,
            game.score.right
        );
        // Раунд, в котором мяч стоит, раундом не был бы: кадр обязан его переместить.
        assert_ne!(
            (game.ball.x, game.ball.y),
            started_at,
            "кадр идущего раунда не сдвинул мяч: ({}, {})",
            game.ball.x,
            game.ball.y
        );
    }

    #[test]
    fn the_scored_point_starts_the_next_round_waiting() {
        let mut game = game_with_plain_paddles();
        game.state = GameState::PointScored;
        // Мяч остался там, где его застал гол: следующий кадр обязан вернуть его в центр.
        park_the_ball_past_the_right_edge(&mut game);

        game.update(STILL, STILL, FRAME);

        assert_eq!(
            game.state,
            GameState::WaitingToStart,
            "засчитанный гол не перевёл игру в ожидание подачи: состояние {:?}",
            game.state
        );
        // Центр считается от размера мяча, а не записан числом: положение мяча — его левый верхний
        // угол, и тест не должен повторять игровые константы своей копией.
        assert_eq!(
            game.ball.x,
            (game.field.width - game.ball.size) / 2.0,
            "мяч не вернулся в центр по горизонтали: x = {}",
            game.ball.x
        );
        assert_eq!(
            game.ball.y,
            (game.field.height - game.ball.size) / 2.0,
            "мяч не вернулся в центр по вертикали: y = {}",
            game.ball.y
        );
    }

    /// `PointScored` живёт один кадр, и сигнал старта может прийти именно в нём. Раунд с этого сигнала
    /// начаться не должен: иначе возврат мяча в центр был бы пропущен, мяч остался бы за границей и
    /// тут же принёс бы ещё одно очко.
    #[test]
    fn the_start_signal_does_not_skip_the_ball_returning_to_the_center() {
        let mut game = game_with_plain_paddles();
        game.state = GameState::PointScored;

        game.start_round();

        assert_eq!(
            game.state,
            GameState::PointScored,
            "сигнал старта в кадре гола пропустил возврат мяча в центр: состояние {:?}",
            game.state
        );
    }

    /// Рестарт доступен в любой момент матча, поэтому проверяются все три состояния: у матча нет
    /// условия победы (D-13), и `R` — единственный способ начать счёт заново.
    #[test]
    fn restarting_the_match_clears_the_score_and_waits() {
        for state in [
            GameState::WaitingToStart,
            GameState::Playing,
            GameState::PointScored,
        ] {
            let started_in = format!("{state:?}");
            let mut game = Game::new(field());
            game.state = state;

            // Матч в разгаре: счёт открыт, мяч и ракетки далеко от исходных позиций.
            game.score.left = 3;
            game.score.right = 5;
            game.ball.x = 100.0;
            game.ball.y = 40.0;
            game.left_paddle.y = 0.0;
            game.right_paddle.y = 500.0;

            game.restart_match();

            assert_eq!(
                (game.score.left, game.score.right),
                (0, 0),
                "рестарт из состояния {} не обнулил счёт: {}:{}",
                started_in,
                game.score.left,
                game.score.right
            );
            assert_eq!(
                game.state,
                GameState::WaitingToStart,
                "рестарт из состояния {} не привёл в ожидание подачи: состояние {:?}",
                started_in,
                game.state
            );
            assert_eq!(
                (game.ball.x, game.ball.y),
                (
                    (game.field.width - game.ball.size) / 2.0,
                    (game.field.height - game.ball.size) / 2.0
                ),
                "рестарт из состояния {} не вернул мяч в центр: ({}, {})",
                started_in,
                game.ball.x,
                game.ball.y
            );
            assert_eq!(
                (game.left_paddle.y, game.right_paddle.y),
                (
                    (game.field.height - game.left_paddle.height) / 2.0,
                    (game.field.height - game.right_paddle.height) / 2.0
                ),
                "рестарт из состояния {} не вернул ракетки в исходные позиции: {} и {}",
                started_in,
                game.left_paddle.y,
                game.right_paddle.y
            );
        }
    }

    #[test]
    fn waiting_to_start_keeps_the_ball_still() {
        let mut game = Game::new(field());
        // Мяч в центре и уже со скоростью подачи: до сигнала старта эта скорость его двигать не должна.
        let standing_at = (game.ball.x, game.ball.y);

        game.update(STILL, STILL, FRAME);

        assert_eq!(
            (game.ball.x, game.ball.y),
            standing_at,
            "мяч сдвинулся до подачи: ({}, {})",
            game.ball.x,
            game.ball.y
        );
    }
}
