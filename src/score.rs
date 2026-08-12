use crate::ball::Ball;
use crate::Field;

/// Счёт матча. У матча нет условия победы (D-13): счёт растёт без предела, а заканчивается матч только
/// рестартом или выходом, поэтому верхней границы у чисел здесь нет.
#[derive(Default)]
pub struct Score {
    pub left: u32,
    pub right: u32,
}

impl Score {
    /// Гол: мяч, ушедший за боковую границу, приносит очко противнику. Мяч и размеры поля приходят
    /// аргументами, поэтому проверка вызывается из теста без окна. Возвращает признак гола: раунд
    /// после него начинается заново, и кадру нужно об этом знать.
    pub fn count_goal(&mut self, ball: &Ball, field: Field) -> bool {
        // Гол засчитывается по полностью вышедшему мячу, а не по его переднему краю: иначе очко
        // начислялось бы раньше, чем мяч исчезнет с экрана.
        if ball.x + ball.size < 0.0 {
            self.right += 1;
            true
        } else if ball.x > field.width {
            self.left += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> Field {
        Field::new(960.0, 600.0)
    }

    /// Мяч теста строится игровым конструктором: счёту от мяча нужны только его положение и размер, а
    /// собственных размеров у счёта нет.
    fn ball_at(x: f32) -> Ball {
        let mut ball = Ball::new(field());
        ball.x = x;
        ball
    }

    /// Мяч ушёл за левую границу целиком: даже его правый край левее нуля.
    fn past_the_left_edge() -> Ball {
        let mut ball = ball_at(0.0);
        ball.x = -ball.size - 1.0;
        ball
    }

    /// Симметрично справа: даже левый край мяча правее поля.
    fn past_the_right_edge() -> Ball {
        ball_at(field().width + 1.0)
    }

    /// Мяч наполовину вышел за левую границу: он ещё виден на экране, поэтому гола ещё нет.
    fn half_past_the_left_edge() -> Ball {
        let mut ball = ball_at(0.0);
        ball.x = -ball.size / 2.0;
        ball
    }

    /// Симметрично справа: половина мяча ещё в поле.
    fn half_past_the_right_edge() -> Ball {
        let mut ball = ball_at(0.0);
        ball.x = field().width - ball.size / 2.0;
        ball
    }

    #[test]
    fn ball_past_the_left_edge_scores_for_the_right_player() {
        let mut score = Score::default();

        let goal = score.count_goal(&past_the_left_edge(), field());

        assert!(goal, "мяч за левой границей не засчитан как гол");
        assert_eq!(
            score.right, 1,
            "мяч за левой границей не принёс очко правому игроку: счёт {}:{}",
            score.left, score.right
        );
        assert_eq!(
            score.left, 0,
            "мяч за левой границей принёс очко и левому игроку: счёт {}:{}",
            score.left, score.right
        );
    }

    #[test]
    fn ball_past_the_right_edge_scores_for_the_left_player() {
        let mut score = Score::default();

        let goal = score.count_goal(&past_the_right_edge(), field());

        assert!(goal, "мяч за правой границей не засчитан как гол");
        assert_eq!(
            score.left, 1,
            "мяч за правой границей не принёс очко левому игроку: счёт {}:{}",
            score.left, score.right
        );
        assert_eq!(
            score.right, 0,
            "мяч за правой границей принёс очко и правому игроку: счёт {}:{}",
            score.left, score.right
        );
    }

    /// Мяч в поле и мяч, вышедший за границу наполовину, проверяются одним тестом: гол начисляется
    /// только за полностью ушедший мяч, поэтому «ещё виден» и «ещё не гол» — одно и то же условие.
    #[test]
    fn ball_inside_the_field_scores_nothing() {
        let mut score = Score::default();

        for ball in [
            ball_at(100.0),
            half_past_the_left_edge(),
            half_past_the_right_edge(),
        ] {
            let goal = score.count_goal(&ball, field());

            assert!(!goal, "мяч в точке x = {} засчитан как гол", ball.x);
            assert_eq!(
                (score.left, score.right),
                (0, 0),
                "мяч в точке x = {} изменил счёт: {}:{}",
                ball.x,
                score.left,
                score.right
            );
        }
    }
}
