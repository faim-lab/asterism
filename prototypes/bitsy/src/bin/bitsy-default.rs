use bitsy::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "that default bitsy game".to_owned(),
        window_width: GAME_SIZE as i32,
        window_height: GAME_SIZE as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {}
