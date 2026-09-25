/// Счёт матча. По прецеденту Pong/Asteroids: простое число, обёрнутое типом, а не голый `u32` в
/// `Game` — прибавление очков остаётся одним именованным действием.
pub struct Score {
    value: u32,
}

impl Score {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn add(&mut self, points: u32) {
        self.value += points;
    }

    /// Первый потребитель вне тестов — UI (`T-TDS-12`).
    #[allow(dead_code)]
    pub fn value(&self) -> u32 {
        self.value
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_score_starts_at_zero() {
        assert_eq!(Score::new().value(), 0, "новый счёт не нулевой");
    }

    #[test]
    fn adding_points_accumulates() {
        let mut score = Score::new();
        score.add(10);
        score.add(25);

        assert_eq!(
            score.value(),
            35,
            "счёт после двух начислений — {}, ожидалось 35",
            score.value()
        );
    }
}
