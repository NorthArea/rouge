/// Счёт как данные (по прецеденту `Score` в Pong).
pub struct Score(u32);

impl Score {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, points: u32) {
        self.0 += points;
    }

    /// Читает строка статуса (`draw_status`, `T-AST-10`) — первый потребитель вне теста.
    pub fn value(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_points_increases_the_score() {
        let mut score = Score::new();

        score.add(20);
        score.add(50);

        assert_eq!(
            score.value(),
            70,
            "счёт после начисления 20 и 50 — {}, ожидалось 70",
            score.value()
        );
    }
}
