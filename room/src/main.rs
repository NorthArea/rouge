use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Room".to_owned(),
        window_width: 960,
        window_height: 600,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    room::run().await;
}
