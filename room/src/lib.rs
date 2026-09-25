use macroquad::prelude::*;

mod aabb;
mod input;
mod player;
mod room;

use input::Input;
use player::Player;
use room::Room;

const BACKGROUND_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);

/// FOV по вертикали. Ориентир из задания — 45–75°; 60° даёт широкий обзор комнаты без заметного
/// "рыбьего глаза" искажения по краям экрана (первая позиция репозитория, где картинка строится
/// перспективной проекцией, а не прямым переводом мировых координат в экранные, D-43).
const FOV_Y_DEGREES: f32 = 60.0;

/// Ускорение свободного падения, единиц в секунду². Подобрано так, чтобы падение с высоты в
/// несколько метров ощущалось быстрым, но не мгновенным (точная высота прыжка настраивается на
/// `T-ROOM-6`).
const GRAVITY: f32 = 20.0;

/// Чувствительность мыши — множитель между смещением мыши за кадр (`mouse_delta_position`, в
/// локальных экранных единицах) и приращением угла в радианах. Подбирается на глаз на приёмке
/// (D-46 прямо исключает настройку чувствительности в интерфейсе).
const MOUSE_SENSITIVITY: f32 = 6.0;

/// Скорость движения по полу, метров в секунду. Комната — 20×20, значение выбрано так, чтобы её
/// можно было пересечь за несколько секунд (тот же ориентир, что у arena предыдущих игр).
const MOVE_SPEED: f32 = 5.0;

pub async fn run() {
    let room = Room::new();
    let mut player = Player::new(vec3(
        0.0,
        room.floor_level + player::EYE_HEIGHT,
        room.depth * 0.35,
    ));

    // Захват курсора включается при входе в игру и снимается при выходе по Esc, чтобы меню не
    // осталось без курсора (D-23 через захват мыши). Не тестируется (D-10) — это состояние окна.
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        // Кадр всегда проходит одни и те же стадии в одном и том же порядке (D-09), даже когда
        // позиция ещё не наполнила все стадии поведением.

        // 1. Чтение ввода.
        if is_key_pressed(KeyCode::Escape) {
            set_cursor_grab(false);
            show_mouse(true);
            break;
        }
        let input = Input {
            mouse_delta: mouse_delta_position(),
            forward: is_key_down(KeyCode::W),
            back: is_key_down(KeyCode::S),
            left: is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::D),
            // Нажатие, а не удержание (см. комментарий у поля `Input::jump`).
            jump: is_key_pressed(KeyCode::Space),
        };

        // 2. Delta time — из него считается и движение по полу.
        let delta_time = get_frame_time();

        // 3. Обновление состояния. Взгляд поворачивается прямо от смещения мыши за кадр — mouse
        //    look не умножается на delta time, он уже per-frame величина; движение по полу — умножается,
        //    иначе скорость зависела бы от частоты кадров. Прыжок — до гравитации: он лишь задаёт
        //    вертикальную скорость, а гравитация в этом же кадре уже начинает её гасить.
        player.look(input.mouse_delta, MOUSE_SENSITIVITY);
        let colliders = room.colliders();
        player.move_on_floor(&input, &colliders, MOVE_SPEED, delta_time);
        player.jump(&input);
        player.apply_gravity(GRAVITY, delta_time, room.floor_level, &colliders);

        // 4. Проверка столкновений — уже выполнена внутри стадии 3: горизонтальное движение и
        //    падение/приземление разрешаются по осям против `colliders` в том же вызове, что их
        //    применяет (D-43).

        // 5. Рендеринг. Камера стоит в позиции игрока и смотрит по направлению его взгляда — та же
        //    привязка камеры к игроку, что и в Asteroids/Shooter, только в трёх измерениях.
        clear_background(BACKGROUND_COLOR);
        set_camera(&Camera3D {
            position: player.position,
            target: player.position + player.look_direction(),
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
