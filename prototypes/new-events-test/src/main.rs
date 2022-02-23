use asterism::physics::{PointPhysData, PointPhysics};
use asterism::tables::{ConditionTables, OutputTable};
use macroquad::prelude::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

fn window_conf() -> Conf {
    Conf {
        window_title: "physics test".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

struct Game {
    logics: Logics,
    tables: ConditionTables<QueryID>,
}

impl Game {
    fn new() -> Self {
        Self {
            logics: Logics {
                physics: PointPhysics::new(),
            },
            tables: ConditionTables::new(),
        }
    }
}

struct Logics {
    physics: PointPhysics,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, std::fmt::Debug)]
enum QueryID {
    Physics,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    // setup
    game.tables
        .add_query::<(usize, PointPhysData)>(QueryID::Physics, None);
    game.logics
        .physics
        .add_physics_entity(Vec2::new(50.0, 50.0), Vec2::new(1.0, 1.0), Vec2::ZERO);
    // game loop
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        game.logics.physics.update();
        let phys_data = game
            .tables
            .update_single(QueryID::Physics, game.logics.physics.get_table())
            .unwrap();
        clear_background(BLUE);
        next_frame().await;
    }
}
