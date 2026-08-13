use crate::brick::{Brick, BrickKind};
use crate::Field;

/// Раскладки уровней заданы прямо в Rust-коде: ни внешнего редактора, ни JSON, ни загрузки из файлов.
/// Каждая строка — ряд блоков, каждый символ — блок: `N` обычный, `S` прочный, `I` неразрушимый,
/// `.` пустое место. Так уровень виден в исходнике таким же, каким игрок увидит его на экране, —
/// ради этого форматирование здесь отключено: собранные в одну строку ряды перестают быть картинкой.
#[rustfmt::skip]
const LAYOUTS: [&[&str]; 3] = [
    &[
        "NNNNNNNNNN",
        "NNNNNNNNNN",
        "NNNNNNNNNN",
    ],
    &[
        "SSSSSSSSSS",
        "NNNNNNNNNN",
        "I.NNNNNN.I",
        "NNNNNNNNNN",
    ],
    &[
        "ISSSSSSSSI",
        "SNNNNNNNNS",
        "NN.NNNN.NN",
        "I.NNNNNN.I",
        "NNNNNNNNNN",
    ],
];

/// Номер первого уровня. Уровни нумеруются с единицы, потому что этот номер видит игрок.
pub const FIRST: usize = 1;

const BRICK_HEIGHT: f32 = 24.0;
/// Зазор между блоками и отступ сетки от краёв поля и от его верхней границы.
const GAP: f32 = 6.0;
const SIDE_MARGIN: f32 = 40.0;
const TOP_MARGIN: f32 = 60.0;

/// Сколько уровней в игре. Последний из них заканчивается победой, а не следующим уровнем.
pub fn count() -> usize {
    LAYOUTS.len()
}

/// Блоки уровня. Ширина блока считается из ширины поля, поэтому ряд занимает её целиком при любом
/// размере окна.
pub fn bricks(number: usize, field: Field) -> Vec<Brick> {
    let rows = LAYOUTS[number - FIRST];
    let columns = rows[0].chars().count();
    let row_width = field.width - 2.0 * SIDE_MARGIN;
    let width = (row_width - GAP * (columns - 1) as f32) / columns as f32;

    let mut bricks = Vec::new();
    for (row, symbols) in rows.iter().enumerate() {
        for (column, symbol) in symbols.chars().enumerate() {
            let Some(kind) = kind_of(symbol) else {
                continue;
            };

            bricks.push(Brick::new(
                SIDE_MARGIN + column as f32 * (width + GAP),
                TOP_MARGIN + row as f32 * (BRICK_HEIGHT + GAP),
                width,
                BRICK_HEIGHT,
                kind,
            ));
        }
    }

    bricks
}

/// Символ раскладки в тип блока. Пустое место блока не создаёт.
fn kind_of(symbol: char) -> Option<BrickKind> {
    match symbol {
        'N' => Some(BrickKind::Normal),
        'S' => Some(BrickKind::Strong),
        'I' => Some(BrickKind::Indestructible),
        _ => None,
    }
}
