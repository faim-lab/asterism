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
async fn main() {
    let mut game = Game::new();
    init(&mut game);
    run(game).await;
}

#[allow(unused)]
fn init(game: &mut Game) {
    let mut player = Player::new();
    player.pos = IVec2::new(3, 3);
    player.color = PURPLE;
    let up = player.add_control_map(KeyCode::Up, true);
    let down = player.add_control_map(KeyCode::Down, true);
    let left = player.add_control_map(KeyCode::Left, true);
    let right = player.add_control_map(KeyCode::Right, true);
    let player_id = game.set_player(player);
}
