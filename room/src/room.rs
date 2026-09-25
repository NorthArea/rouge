use macroquad::prelude::*;

/// Цвета сцены — примитивы Macroquad без текстур (D-46): пол, стены и кубы различаются только
/// оттенком, как во всех предыдущих играх репозитория.
const FLOOR_COLOR: Color = Color::new(0.30, 0.32, 0.36, 1.0);
const WALL_COLOR: Color = Color::new(0.55, 0.57, 0.62, 1.0);
const OBSTACLE_COLOR: Color = Color::new(0.80, 0.55, 0.20, 1.0);

const WALL_THICKNESS: f32 = 0.5;

/// Половинные размеры кубов-препятствий и их позиция на полу (координата Y — половина высоты, чтобы
/// куб стоял на полу, а не проваливался в него наполовину).
const OBSTACLE_HALF_HEIGHT: f32 = 0.75;
const OBSTACLES: [(f32, f32, f32); 3] = [
    (4.0, OBSTACLE_HALF_HEIGHT, 3.0),
    (-5.0, OBSTACLE_HALF_HEIGHT, -2.0),
    (2.0, OBSTACLE_HALF_HEIGHT, -6.0),
];
const OBSTACLE_SIZE: f32 = 1.5;

/// Комната, пока — только геометрия для отрисовки (D-44 допускает тип `Room`, здесь он ещё не
/// хранит коллизионные данные: `Aabb` и проверка пересечения появляются на `T-ROOM-7`, эта позиция
/// строит только то, что видно).
pub struct Room {
    pub width: f32,
    pub depth: f32,
    pub wall_height: f32,
}

impl Room {
    pub fn new() -> Self {
        Self {
            width: 20.0,
            depth: 20.0,
            wall_height: 4.0,
        }
    }

    /// Пол, четыре стены по периметру и кубы-препятствия — все из примитивов Macroquad
    /// (`draw_plane`, `draw_cube`), никаких моделей и текстур (D-46).
    pub fn draw(&self) {
        draw_plane(
            vec3(0.0, 0.0, 0.0),
            vec2(self.width / 2.0, self.depth / 2.0),
            None,
            FLOOR_COLOR,
        );

        let wall_center_y = self.wall_height / 2.0;
        // Северная и южная стены — вдоль оси X, полной ширины комнаты.
        draw_cube(
            vec3(0.0, wall_center_y, -self.depth / 2.0),
            vec3(self.width, self.wall_height, WALL_THICKNESS),
            None,
            WALL_COLOR,
        );
        draw_cube(
            vec3(0.0, wall_center_y, self.depth / 2.0),
            vec3(self.width, self.wall_height, WALL_THICKNESS),
            None,
            WALL_COLOR,
        );
        // Восточная и западная стены — вдоль оси Z, полной глубины комнаты.
        draw_cube(
            vec3(-self.width / 2.0, wall_center_y, 0.0),
            vec3(WALL_THICKNESS, self.wall_height, self.depth),
            None,
            WALL_COLOR,
        );
        draw_cube(
            vec3(self.width / 2.0, wall_center_y, 0.0),
            vec3(WALL_THICKNESS, self.wall_height, self.depth),
            None,
            WALL_COLOR,
        );

        for (x, y, z) in OBSTACLES {
            draw_cube(
                vec3(x, y, z),
                vec3(OBSTACLE_SIZE, OBSTACLE_SIZE, OBSTACLE_SIZE),
                None,
                OBSTACLE_COLOR,
            );
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
