use asterism::tables::*;
use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

fn window_conf() -> Conf {
    Conf {
        window_title: "test".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    test().await;
}

async fn test() {
    let mut game = Game::new();

    let mut tables = ConditionTables::new();
    let ball_wall = tables.add_query();
    let score_check = tables.add_query();

    let ball_wall_condition = Compose::Just(ball_wall, ProcessOutput::ForEach);
    let score_check_condition = Compose::Just(score_check, ProcessOutput::IfAny);
    let both = Compose::And(
        Box::new(ball_wall_condition.clone()),
        Box::new(score_check_condition.clone()),
    );

    let ball_wall_condition = tables.add_condition(ball_wall_condition);
    let ball_wall_score_condition = tables.add_condition(both);

    let mut ball = Ball::new();
    ball.pos = Vec2::new(0.0, 10.0);
    ball.size = Vec2::new(10.0, 10.0);
    ball.vel = Vec2::X;
    let _ = game.add_ball(ball);

    let mut wall = Wall::new();
    wall.pos = Vec2::new(50.0, 0.0);
    wall.size = Vec2::new(10.0, 100.0);
    let _ = game.add_wall(wall);

    let score = game.add_score(Score::new());

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);
        game.logics.physics.update();
        let phys = game.logics.physics.get_synthesis(0);
        let mut col = game.logics.collision.get_synthesis(1);
        col.center = phys.pos + col.half_size;
        game.logics.collision.update_synthesis(1, col);

        game.logics.collision.update();

        let mut phys = game.logics.physics.get_synthesis(0);
        let col = game.logics.collision.get_synthesis(1);
        phys.pos = col.center - col.half_size;
        game.logics.physics.update_synthesis(0, phys);

        tables.update_query(
            ball_wall,
            game.logics
                .collision
                .check_predicate(|(i, j): &(usize, usize)| {
                    game.logics.collision.metadata[*i].id == CollisionEnt::Ball
                        && game.logics.collision.metadata[*j].id == CollisionEnt::Wall
                }),
        );

        for (contact_idx, _) in tables
            .check_condition(ball_wall_condition)
            .iter()
            .enumerate()
            .filter(|(_, val)| **val)
        {
            let contact = game.logics.collision.contacts[contact_idx];
            let ball_idx = contact.i - 1;
            game.logics
                .physics
                .handle_predicate(&PhysicsReaction::SetPos(ball_idx, Vec2::new(0.0, 10.0)));
            game.logics
                .resources
                .handle_predicate(&(RsrcPool::Score(score), Transaction::Change(1)))
        }

        game.logics.resources.update();

        tables.update_query(
            score_check,
            game.logics
                .resources
                .check_predicate(|(_, (val, ..)): &(RsrcPool, (u16, u16, u16))| *val > 5),
        );

        for (contact_idx, _) in tables
            .check_condition(ball_wall_score_condition)
            .iter()
            .enumerate()
            .filter(|(_, val)| **val)
        {
            let contact = game.logics.collision.contacts[contact_idx];
            let ball_idx = contact.i - 1;
            game.logics
                .physics
                .handle_predicate(&PhysicsReaction::SetPos(ball_idx, Vec2::new(100.0, 10.0)));
            game.logics
                .physics
                .handle_predicate(&PhysicsReaction::SetVel(ball_idx, -Vec2::X));
        }

        tables.check_condition(ball_wall_score_condition);

        next_frame().await;
    }
}
