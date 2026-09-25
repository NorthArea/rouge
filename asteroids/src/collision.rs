use macroquad::prelude::*;

/// Столкновение круг-круг, написанное вручную (D-27): расстояние между центрами против суммы
/// радиусов. Касание строго на сумме радиусов трактуется как столкновение — `<=`, а не `<`.
///
pub fn circles_overlap(a_position: Vec2, a_radius: f32, b_position: Vec2, b_radius: f32) -> bool {
    (a_position - b_position).length() <= a_radius + b_radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circles_closer_than_the_radius_sum_overlap() {
        let overlaps = circles_overlap(vec2(0.0, 0.0), 10.0, vec2(15.0, 0.0), 10.0);

        assert!(
            overlaps,
            "круги на расстоянии 15 при сумме радиусов 20 не посчитаны столкнувшимися"
        );
    }

    #[test]
    fn circles_farther_than_the_radius_sum_do_not_overlap() {
        let overlaps = circles_overlap(vec2(0.0, 0.0), 10.0, vec2(25.0, 0.0), 10.0);

        assert!(
            !overlaps,
            "круги на расстоянии 25 при сумме радиусов 20 посчитаны столкнувшимися"
        );
    }

    #[test]
    fn circles_touching_exactly_at_the_radius_sum_overlap() {
        // Пограничный случай, для которого позиция явно фиксирует один способ трактовки: `<=`.
        let overlaps = circles_overlap(vec2(0.0, 0.0), 10.0, vec2(20.0, 0.0), 10.0);

        assert!(
            overlaps,
            "круги, касающиеся ровно на сумме радиусов (20), не посчитаны столкнувшимися"
        );
    }
}
