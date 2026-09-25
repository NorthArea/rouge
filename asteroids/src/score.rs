/// Счёт как данные (по прецеденту `Score` в Pong).
pub struct Score(u32);

impl Score {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, points: u32) {
        self.0 += points;
    }

    /// Отображение счёта — задача `T-AST-10` (строка статуса `Score`/`Lives`/`Wave`), у неё здесь
    /// пока нет потребителя вне теста: `#[cfg(test)]` держит метод видимым только для тестов, чтобы
    /// не ловить `-D dead-code` под `cargo build --workspace` (тот же приём, что для `velocity`
    /// корабля на `T-AST-2` и `AsteroidSize::{Medium, Small}` на `T-AST-6`/`T-AST-7`).
    #[cfg(test)]
    pub(crate) fn value(&self) -> u32 {
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
