use macroquad::prelude::*;

use crate::aabb::Aabb;

/// Цвета сцены — примитивы Macroquad без текстур (D-46): пол, стены и кубы различаются только
/// оттенком, как во всех предыдущих играх репозитория.
const FLOOR_COLOR: Color = Color::new(0.30, 0.32, 0.36, 1.0);
const WALL_COLOR: Color = Color::new(0.55, 0.57, 0.62, 1.0);
const OBSTACLE_COLOR: Color = Color::new(0.80, 0.55, 0.20, 1.0);

const WALL_THICKNESS: f32 = 0.5;

/// Центры кубов-препятствий на полу (координата Y — половина высоты, чтобы куб стоял на полу, а не
/// проваливался в него наполовину).
const OBSTACLE_HALF_HEIGHT: f32 = 0.75;
const OBSTACLE_CENTERS: [(f32, f32, f32); 3] = [
    (4.0, OBSTACLE_HALF_HEIGHT, 3.0),
    (-5.0, OBSTACLE_HALF_HEIGHT, -2.0),
    (2.0, OBSTACLE_HALF_HEIGHT, -6.0),
];
const OBSTACLE_HALF_SIZE: f32 = 0.75;

/// Комната: геометрия для отрисовки и та же геометрия как данные для столкновений (D-44) — одна
/// коллекция `Aabb` вместо двух независимых копий, которые расходятся при первой же правке
/// (`T-ROOM-7`: отрисовка `T-ROOM-2`, ранее рисовавшая по разрозненным константам, переведена сюда).
pub struct Room {
    pub width: f32,
    pub depth: f32,
    pub floor_level: f32,
    pub walls: Vec<Aabb>,
    pub obstacles: Vec<Aabb>,
}

impl Room {
    pub fn new() -> Self {
        let width = 20.0;
        let depth = 20.0;
        let wall_height = 4.0;
        let floor_level = 0.0;
        let wall_center_y = floor_level + wall_height / 2.0;

        let walls = vec![
            // Северная и южная стены — вдоль оси X, полной ширины комнаты.
            Aabb::from_center_half_extents(
                vec3(0.0, wall_center_y, -depth / 2.0),
                vec3(width / 2.0, wall_height / 2.0, WALL_THICKNESS / 2.0),
            ),
            Aabb::from_center_half_extents(
                vec3(0.0, wall_center_y, depth / 2.0),
                vec3(width / 2.0, wall_height / 2.0, WALL_THICKNESS / 2.0),
            ),
            // Восточная и западная стены — вдоль оси Z, полной глубины комнаты.
            Aabb::from_center_half_extents(
                vec3(-width / 2.0, wall_center_y, 0.0),
                vec3(WALL_THICKNESS / 2.0, wall_height / 2.0, depth / 2.0),
            ),
            Aabb::from_center_half_extents(
                vec3(width / 2.0, wall_center_y, 0.0),
                vec3(WALL_THICKNESS / 2.0, wall_height / 2.0, depth / 2.0),
            ),
        ];

        let obstacles = OBSTACLE_CENTERS
            .into_iter()
            .map(|(x, y, z)| {
                Aabb::from_center_half_extents(
                    vec3(x, y, z),
                    vec3(OBSTACLE_HALF_SIZE, OBSTACLE_HALF_SIZE, OBSTACLE_HALF_SIZE),
                )
            })
            .collect();

        Self {
            width,
            depth,
            floor_level,
            walls,
            obstacles,
        }
    }

    /// Стены и кубы-препятствия вместе — то, с чем сталкивается игрок (D-43); пол в эту коллекцию
    /// не входит, у него собственная проверка по высоте (`Player::apply_gravity`).
    pub fn colliders(&self) -> Vec<Aabb> {
        self.walls
            .iter()
            .chain(self.obstacles.iter())
            .copied()
            .collect()
    }

    /// Пол — примитивом Macroquad; стены и кубы-препятствия рисуются из той же коллекции `Aabb`,
    /// которой пользуется столкновение (`T-ROOM-8`), чтобы нарисованное и просчитываемое не могли
    /// разойтись.
    pub fn draw(&self) {
        draw_plane(
            vec3(0.0, self.floor_level, 0.0),
            vec2(self.width / 2.0, self.depth / 2.0),
            None,
            FLOOR_COLOR,
        );

        for wall in &self.walls {
            draw_aabb(wall, WALL_COLOR);
        }
        for obstacle in &self.obstacles {
            draw_aabb(obstacle, OBSTACLE_COLOR);
        }
    }
}

fn draw_aabb(aabb: &Aabb, color: Color) {
    let center = (aabb.min + aabb.max) / 2.0;
    let size = aabb.max - aabb.min;
    draw_cube(center, size, None, color);
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
