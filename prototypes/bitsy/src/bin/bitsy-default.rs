use bitsy::*;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::rand::srand(get_time().to_bits());
    let mut game = Game::new();
    init(&mut game);
    run(game).await;
}

fn init(game: &mut Game) {
    let mut player = Player::new();
    player.pos = IVec2::new(3, 3);
    player.color = PURPLE;
    let player_id = game.set_player(player);

    let mut character = Character::new();
    character.pos = IVec2::new(1, 2);
    character.color = PINK;
    let char_id = game.add_character(character);

    for _ in 0..3 {
        let mut tile = Tile::new();
        tile.solid = true;
        game.log_tile_info(tile);
    }

    #[rustfmt::skip]
    let map = r#"
00000000
0      0
0   1  0
0 1    0
0   2  0
0      0
0      0
00000000
    "#;

    game.load_tilemap_from_str(map).unwrap();

    game.add_collision_predicate(
        Contact::Ent(player_id.idx(), char_id.idx() + 1),
        Box::new(
            |_state: &mut State, logics: &mut Logics, event: &ColEvent| {
                if let Contact::Ent(i, _) = event {
                    logics
                        .collision
                        .handle_predicate(&CollisionReaction::SetEntPos(*i, IVec2::new(6, 6)));
                }
            },
        ),
    );
}
