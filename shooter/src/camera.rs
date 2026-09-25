use macroquad::prelude::*;

use crate::arena::Arena;

/// Центр камеры: следует за игроком, но не показывает пространство за пределами арены. Единственное
/// место, где решается, где именно стоит камера — `screen_to_world`/`world_to_screen` пользуются этим
/// же значением, а не считают позицию камеры заново.
pub fn camera_center(player_position: Vec2, arena: Arena, screen_size: Vec2) -> Vec2 {
    let half_screen = screen_size / 2.0;
    vec2(
        player_position
            .x
            .clamp(half_screen.x, arena.width - half_screen.x),
        player_position
            .y
            .clamp(half_screen.y, arena.height - half_screen.y),
    )
}

/// Единственный на крейт перевод экранной точки в мировую (D-39). Первый потребитель вне тестов —
/// прицеливание мышью на `T-TDS-4`; до тех пор функция существует и проверена тестами, но не
/// вызывается из `run()`.
#[allow(dead_code)]
pub fn screen_to_world(screen_point: Vec2, camera_center: Vec2, screen_size: Vec2) -> Vec2 {
    camera_center - screen_size / 2.0 + screen_point
}

/// Обратный перевод: мировая точка в экранную. Обязан быть точным обратным преобразованием
/// `screen_to_world` при той же позиции камеры (round-trip) — тест на это ссылается прямо. Прямого
/// потребителя вне тестов в первой версии нет: UI (`T-TDS-12`) рисуется после `set_default_camera` в
/// уже экранных координатах и этим переводом не пользуется.
#[allow(dead_code)]
pub fn world_to_screen(world_point: Vec2, camera_center: Vec2, screen_size: Vec2) -> Vec2 {
    world_point - camera_center + screen_size / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 0.01;

    fn arena() -> Arena {
        Arena::new(1920.0, 1200.0)
    }

    fn screen() -> Vec2 {
        vec2(960.0, 600.0)
    }

    #[test]
    fn the_camera_follows_the_player_away_from_the_edges() {
        let player_position = vec2(960.0, 600.0);

        let center = camera_center(player_position, arena(), screen());

        assert!(
            (center - player_position).length() < TOLERANCE,
            "камера в {:?}, ожидалось совпадение с игроком {:?}",
            center,
            player_position
        );
    }

    #[test]
    fn the_camera_stops_at_the_left_edge() {
        let center = camera_center(vec2(0.0, 600.0), arena(), screen());

        assert!(
            (center.x - screen().x / 2.0).abs() < TOLERANCE,
            "камера у левого края в x = {}, ожидалось {}",
            center.x,
            screen().x / 2.0
        );
    }

    #[test]
    fn the_camera_stops_at_the_right_edge() {
        let center = camera_center(vec2(arena().width, 600.0), arena(), screen());

        let expected = arena().width - screen().x / 2.0;
        assert!(
            (center.x - expected).abs() < TOLERANCE,
            "камера у правого края в x = {}, ожидалось {}",
            center.x,
            expected
        );
    }

    #[test]
    fn the_camera_stops_at_the_top_edge() {
        let center = camera_center(vec2(960.0, 0.0), arena(), screen());

        assert!(
            (center.y - screen().y / 2.0).abs() < TOLERANCE,
            "камера у верхнего края в y = {}, ожидалось {}",
            center.y,
            screen().y / 2.0
        );
    }

    #[test]
    fn the_camera_stops_at_the_bottom_edge() {
        let center = camera_center(vec2(960.0, arena().height), arena(), screen());

        let expected = arena().height - screen().y / 2.0;
        assert!(
            (center.y - expected).abs() < TOLERANCE,
            "камера у нижнего края в y = {}, ожидалось {}",
            center.y,
            expected
        );
    }

    #[test]
    fn round_tripping_screen_to_world_and_back_returns_the_original_point() {
        let camera_center = vec2(700.0, 450.0);
        let original = vec2(300.0, 200.0);

        let world = screen_to_world(original, camera_center, screen());
        let back = world_to_screen(world, camera_center, screen());

        assert!(
            (back - original).length() < TOLERANCE,
            "round-trip дал {:?}, ожидался исходный {:?}",
            back,
            original
        );
    }

    #[test]
    fn the_center_of_the_window_maps_to_the_camera_center() {
        let camera_center_point = vec2(700.0, 450.0);

        let world = screen_to_world(screen() / 2.0, camera_center_point, screen());

        assert!(
            (world - camera_center_point).length() < TOLERANCE,
            "центр окна перевёлся в {:?}, ожидался центр камеры {:?}",
            world,
            camera_center_point
        );
    }
}
