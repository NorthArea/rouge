use macroquad::prelude::*;

/// Оформление минималистичное и целиком собрано из примитивов Macroquad: внешних assets в проекте нет.
const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const FIELD_COLOR: Color = Color::new(0.88, 0.89, 0.93, 1.0);

const BORDER_THICKNESS: f32 = 6.0;
const CENTER_LINE_WIDTH: f32 = 6.0;
const CENTER_LINE_DASH: f32 = 20.0;
const CENTER_LINE_GAP: f32 = 16.0;

/// Игровое поле. Размеры приходят значением из размеров окна, поэтому игровая логика не обращается
/// к состоянию Macroquad и остаётся вызываемой сама по себе.
#[derive(Clone, Copy)]
struct Field {
    width: f32,
    height: f32,
}

impl Field {
    fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Pong".to_owned(),
        window_width: 960,
        window_height: 600,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке. Двигать и сталкивать
        // пока нечего, но стадии уже занимают своё место: порядок стадий — часть того, что этот
        // проект показывает.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 2. Delta time — время предыдущего кадра, из которого считается любое перемещение.
        let delta_time = get_frame_time();

        // 3. Обновление состояния.
        let field = Field::new(screen_width(), screen_height());
        update(field, delta_time);

        // 4. Проверка столкновений.
        check_collisions(field);

        // 5. Рендеринг.
        draw_field(field);

        // 6. Следующий кадр.
        next_frame().await;
    }
}

fn update(_field: Field, _delta_time: f32) {}

fn check_collisions(_field: Field) {}

fn draw_field(field: Field) {
    clear_background(BACKGROUND_COLOR);
    draw_center_line(field);
    draw_borders(field);
}

/// Центральная линия — не сплошная линия, а столбик отдельных штрихов: так выглядит классический Pong.
fn draw_center_line(field: Field) {
    let x = (field.width - CENTER_LINE_WIDTH) / 2.0;
    let step = CENTER_LINE_DASH + CENTER_LINE_GAP;

    let mut y = CENTER_LINE_GAP;
    while y + CENTER_LINE_DASH <= field.height {
        draw_rectangle(x, y, CENTER_LINE_WIDTH, CENTER_LINE_DASH, FIELD_COLOR);
        y += step;
    }
}

/// `draw_rectangle_lines` рисует рамку по центру контура, поэтому её внешняя половина ушла бы
/// за край окна. Рамка задаётся двойной толщиной, а видимой остаётся ровно `BORDER_THICKNESS`.
fn draw_borders(field: Field) {
    draw_rectangle_lines(
        0.0,
        0.0,
        field.width,
        field.height,
        BORDER_THICKNESS * 2.0,
        FIELD_COLOR,
    );
}
