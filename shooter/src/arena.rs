/// Арена. Размеры заданы константами в `lib.rs`, поэтому игровая логика не обращается к состоянию
/// Macroquad и остаётся вызываемой сама по себе. Дублирует `Field`/`Arena` трёх других крейтов —
/// принятая цена независимости крейтов друг от друга (D-02).
#[derive(Clone, Copy)]
pub struct Arena {
    pub width: f32,
    pub height: f32,
}

impl Arena {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
