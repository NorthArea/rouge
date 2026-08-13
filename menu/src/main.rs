use macroquad::prelude::*;

mod selection;

use selection::{MenuItem, Selection, ITEMS};

/// Оформление минималистичное, тот же язык, что в играх: тёмный фон, светлый текст, выделенный
/// пункт отличается только цветом — без внешних assets (D-01).
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const TEXT_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);
const SELECTED_COLOR: Color = Color::new(0.95, 0.75, 0.20, 1.0);

const WINDOW_WIDTH: f32 = 960.0;
const WINDOW_HEIGHT: f32 = 600.0;

/// Заголовок и пункты латиницей: кириллица встроенного шрифта Macroquad наблюдением не проверена,
/// та же причина, что и в подсказке Pong.
const TITLE_TEXT: &str = "SELECT A GAME";
const TITLE_FONT_SIZE: u16 = 40;
const TITLE_Y: f32 = 220.0;

const ITEM_FONT_SIZE: u16 = 48;
const FIRST_ITEM_Y: f32 = 320.0;
const ITEM_SPACING: f32 = 80.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Menu".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Курсор живёт вне цикла: он должен пережить возврат из игры и указывать туда же, куда
    // указывал до входа (D-24).
    let mut selection = Selection::new();

    loop {
        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Up) {
            selection.move_up();
        }
        if is_key_pressed(KeyCode::Down) {
            selection.move_down();
        }
        let confirmed = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        let escaped = is_key_pressed(KeyCode::Escape);

        // 2. Намерение игрока: подтверждение выбора и Esc сходятся в одном и том же трёхветочном
        // match — Esc даёт то же намерение "выйти", что и пункт QUIT (D-23).
        let intent = if escaped {
            Some(selection.escape())
        } else if confirmed {
            Some(selection.confirm())
        } else {
            None
        };

        if let Some(item) = intent {
            match item {
                MenuItem::Pong => {
                    pong::run().await;
                    // Ловушка кадра возврата (D-23): Pong вышел из своего цикла по тому же Esc, не
                    // дождавшись next_frame(). Если меню тут же снова прочитает ввод, оно увидит то
                    // же самое нажатие и немедленно закроет программу. Поэтому меню сначала ждёт
                    // новый кадр и только потом возвращается к чтению клавиатуры.
                    next_frame().await;
                    continue;
                }
                MenuItem::Arkanoid => {
                    arkanoid::run().await;
                    next_frame().await;
                    continue;
                }
                MenuItem::Quit => break,
            }
        }

        // 3. Рендеринг.
        draw_menu(&selection);

        // 4. Следующий кадр.
        next_frame().await;
    }
}

fn draw_menu(selection: &Selection) {
    clear_background(BACKGROUND_COLOR);
    draw_centered_text(TITLE_TEXT, TITLE_Y, TITLE_FONT_SIZE, TEXT_COLOR);

    for (index, item) in ITEMS.iter().enumerate() {
        let color = if index == selection.cursor() {
            SELECTED_COLOR
        } else {
            TEXT_COLOR
        };
        let y = FIRST_ITEM_Y + index as f32 * ITEM_SPACING;
        draw_centered_text(item.label(), y, ITEM_FONT_SIZE, color);
    }
}

/// Ширину строки нужно измерить: центрировать текст иначе нечем, тот же приём, что в счёте Pong.
fn draw_centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let width = measure_text(text, None, font_size, 1.0).width;
    draw_text(
        text,
        (WINDOW_WIDTH - width) / 2.0,
        y,
        f32::from(font_size),
        color,
    );
}
