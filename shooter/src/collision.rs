use macroquad::prelude::*;

/// Круг с кругом (D-38): сравнение квадрата расстояния с квадратом суммы радиусов, без извлечения
/// корня. Единственная проверка столкновений на весь крейт — ею пользуются попадание пули во врага,
/// контактный урон игроку и подбор pickup.
pub fn circles_overlap(a_position: Vec2, a_radius: f32, b_position: Vec2, b_radius: f32) -> bool {
    let distance_squared = a_position.distance_squared(b_position);
    let radius_sum = a_radius + b_radius;
    distance_squared <= radius_sum * radius_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_circles_are_detected() {
        assert!(circles_overlap(vec2(0.0, 0.0), 5.0, vec2(8.0, 0.0), 5.0));
    }

    #[test]
    fn circles_touching_exactly_at_the_sum_of_radii_count_as_overlapping() {
        assert!(circles_overlap(vec2(0.0, 0.0), 5.0, vec2(10.0, 0.0), 5.0));
    }

    #[test]
    fn circles_farther_apart_than_the_sum_of_radii_do_not_overlap() {
        assert!(!circles_overlap(vec2(0.0, 0.0), 5.0, vec2(10.01, 0.0), 5.0));
    }
}
