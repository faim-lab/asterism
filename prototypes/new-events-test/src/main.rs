use asterism::physics::PointPhysics;
// use asterism::tables::{ConditionTables, OutputTable};
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

impl Logics {
    fn new() -> Self {
        Self {
            physics: PointPhysics::new(),
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
    let mut logics = Logics::new();
    // setup
    // game.tables
    //     .add_query::<(usize, PointPhysData)>(QueryID::Physics, None);
    logics
        .physics
        .add_physics_entity(Vec2::new(50.0, 50.0), Vec2::new(1.0, 1.0), Vec2::ZERO);

    // game loop
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        logics.physics.update();
        for (i, phys) in logics.physics.iter_mut().enumerate() {
            if phys.vel.length_squared() > 0.0 {
                *phys.vel = Vec2::ZERO;
                println!("physics entity {}, please stop", i);
            }
        }
        clear_background(BLUE);
        next_frame().await;
    }
}
