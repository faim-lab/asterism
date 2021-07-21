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
    game.set_background(BLACK);
    let mut player = Player::new();
    player.pos = IVec2::new(3, 3);
    player.color = PURPLE;

    let rocks = game.log_rsrc();
    let num_rocks = Resource::new();
    player.add_inventory_item(rocks, num_rocks);

    game.set_player(player);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    game.log_tile_info(Tile::new());

    #[rustfmt::skip]
    let maps = [r#"
00000000
0      0
0   2  0
0      0
0      0
0  3   0
0      0
00000000
    "#,

r#"
00000000
0      0
0      0
0   1  0
0      0
0      0
0      0
00000000
    "#
    ];

    for map in maps.iter() {
        game.add_room_from_str(map).unwrap();
    }

    let mut character = Character::new();
    character.pos = IVec2::new(1, 2);
    character.color = BROWN;
    let char_id = game.add_character(character, 0);

    game.add_collision_predicate(
        Contact::Ent(0, char_id.idx() + 1),
        0,
        Box::new(
            |state: &mut State, logics: &mut Logics, _: &(ColEvent, usize)| {
                logics
                    .resources
                    .handle_predicate(&(state.resources[0], Transaction::Change(1)));
            },
        ),
    );

    game.add_link((0, IVec2::new(3, 5)), (1, IVec2::new(1, 1)));

    game.add_rsrc_predicate(
        game.state.resources[0],
        ResourceEventType::PoolUpdated,
        Box::new(|_: &mut State, logics: &mut Logics, event: &RsrcEvent| {
            println!(
                "got a rock (total rocks: {})",
                logics.resources.get_ident_data(event.pool).0
            );
        }),
    );
}
