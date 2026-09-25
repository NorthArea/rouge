use macroquad::prelude::*;

mod room;

use room::Room;

const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);

/// FOV по вертикали. Ориентир из задания — 45–75°; 60° даёт широкий обзор комнаты без заметного
/// "рыбьего глаза" искажения по краям экрана (первая позиция репозитория, где картинка строится
/// перспективной проекцией, а не прямым переводом мировых координат в экранные, D-43).
const FOV_Y_DEGREES: f32 = 60.0;

pub async fn run() {
    let room = Room::new();

    // Игрока ещё нет (он появляется на T-ROOM-3): камера стоит в фиксированной точке комнаты и
    // смотрит в её центр — этого достаточно, чтобы увидеть, что сцена построена и читается.
    // Привязка к игроку заменяет эти числа его полями, не меняя структуры кадра.
    let camera_position = vec3(0.0, room.wall_height * 0.7, room.depth * 0.45);
    let camera_target = vec3(0.0, room.wall_height * 0.25, 0.0);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке (D-09), даже когда
        // позиция ещё не наполнила все стадии поведением.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 2. Delta time — здесь пока не потребляется: ни игрока, ни движения ещё нет.
        let _delta_time = get_frame_time();

        // 3. Обновление состояния — пусто: игрок появляется на T-ROOM-3.

        // 4. Проверка столкновений — пусто: геометрия как данные для столкновений появляется на
        //    T-ROOM-7.

        // 5. Рендеринг.
        clear_background(BACKGROUND_COLOR);
        set_camera(&Camera3D {
            position: camera_position,
            target: camera_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: FOV_Y_DEGREES.to_radians(),
            ..Default::default()
        });
        room.draw();
        set_default_camera();

        // 6. Следующий кадр.
        next_frame().await;
    }
}
