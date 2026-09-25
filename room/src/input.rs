use macroquad::prelude::*;

/// Ввод одного кадра — то же основание, по которому `Input` появился в Game 04 (D-35, распространено
/// на Game 05 решением D-44): чтобы движение игрока вызывалось из тестов без окна, а не только через
/// реальное чтение клавиатуры и мыши.
pub struct Input {
    pub mouse_delta: Vec2,
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}
