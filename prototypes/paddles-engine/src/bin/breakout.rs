use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const BALL_SIZE: u8 = 10;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_WIDTH: u8 = 48;
const PADDLE_HEIGHT: u8 = 8;

fn window_conf() -> Conf {
    Conf {
        window_title: "breakout".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // initialize game
    let mut game = Game::new();
    init(&mut game);
    run(game).await;
}

fn init(game: &mut Game) {
    // ball
    let mut ball = Ball::new();
    ball.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        HEIGHT as f32 - PADDLE_OFF_X as f32 * 2.0,
    ));
    ball.set_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));
    let ball = game.add_ball(ball);

    // walls
    let walls = [
        {
            // left
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(-1.0, 0.0));
            wall.set_size(Vec2::new(1.0, HEIGHT as f32));
            game.add_wall(wall)
        },
        {
            // right
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
            wall.set_size(Vec2::new(1.0, HEIGHT as f32));
            game.add_wall(wall)
        },
        {
            // top
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(0.0, -1.0));
            wall.set_size(Vec2::new(WIDTH as f32, 1.0));
            game.add_wall(wall)
        },
    ];
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let bottom_wall = game.add_wall(wall);

    // blocks
    let block_size = Vec2::new(32.0, 16.0);
    let blocks = (0..5)
        .map(|y| {
            (0..8)
                .map(|x| {
                    let mut wall = Wall::new();
                    wall.set_pos(Vec2::new(x as f32 * 32.0, y as f32 * 16.0));
                    wall.set_size(block_size);
                    game.add_wall(wall)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // paddle 1
    let mut paddle = Paddle::new();
    paddle.add_control_map(KeyCode::Left, true);
    paddle.add_control_map(KeyCode::Right, true);
    let action_serve = paddle.add_control_map(KeyCode::Space, true);
    paddle.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - PADDLE_WIDTH as f32 / 2.0,
        HEIGHT as f32 - PADDLE_OFF_X as f32,
    ));
    paddle.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    let paddle = game.add_paddle(paddle);

    game.set_paddle_col_synthesis(Box::new(|mut paddle: Paddle| {
        if paddle.controls[0].3.value > 0.0 {
            paddle.pos.x -= 1.0;
            // paddle.vel.y -= 1.0;
        }
        if paddle.controls[1].3.value > 0.0 {
            paddle.pos.x += 1.0;
            // paddle.vel.y = (-1.0).min(paddle.vel.y);
        }
        paddle
    }));

    let reset_ball = |state: &mut State, logics: &mut Logics, _: &ColEvent| {
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));
        logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(
                state.get_col_idx(0, CollisionEnt::Ball),
                Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    HEIGHT as f32 - PADDLE_OFF_X as f32 * 2.0,
                ),
            ));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyValid(
                0,
                ActionID::new(2), // eh....
            ));
    };

    game.add_collision_predicate(
        (ball.idx(), CollisionEnt::Ball),
        (bottom_wall.idx(), CollisionEnt::Wall),
        Box::new(reset_ball),
    );

    let bounce = |state: &mut State, logics: &mut Logics, event: &ColEvent| {
        if let ColEvent::ByIndex(i, j) = event {
            let id = state.get_id(*i);
            if let EntID::Ball(ball_id) = id {
                let sides_touched = logics.collision.sides_touched(*i, *j);
                let mut vals = logics.physics.get_synthesis(ball_id.idx());
                if sides_touched.y != 0.0 {
                    vals.vel.y *= -1.0;
                }
                if sides_touched.x != 0.0 {
                    vals.vel.x *= -1.0;
                }
                logics.physics.update_synthesis(ball_id.idx(), vals);
            }
        }
    };

    let bounce_inc_remove = |state: &mut State, logics: &mut Logics, event: &ColEvent| {
        if let ColEvent::ByIndex(i, j) = event {
            let id = state.get_id(*j);
            if let EntID::Wall(wall_id) = id {
                let id = state.get_id(*i);
                if let EntID::Ball(ball_id) = id {
                    let sides_touched = logics.collision.sides_touched(*i, *j);
                    let mut vals = logics.physics.get_synthesis(ball_id.idx());
                    if sides_touched.y != 0.0 {
                        vals.vel.y *= -1.0;
                    }
                    if sides_touched.x != 0.0 {
                        vals.vel.x *= -1.0;
                    }
                    logics.physics.update_synthesis(ball_id.idx(), vals);
                    state.queue_remove(EntID::Wall(wall_id));
                    logics.resources.handle_predicate(&(
                        RsrcPool::Score(ScoreID::new(0)),
                        Transaction::Change(1),
                    ));
                }
            }
        }
    };

    for wall in walls.iter() {
        game.add_collision_predicate(
            (ball.idx(), CollisionEnt::Ball),
            (wall.idx(), CollisionEnt::Wall),
            Box::new(bounce),
        );
    }

    for block in blocks.iter().flatten() {
        game.add_collision_predicate(
            (ball.idx(), CollisionEnt::Ball),
            (block.idx(), CollisionEnt::Wall),
            Box::new(bounce_inc_remove),
        );
    }

    game.add_collision_predicate(
        (ball.idx(), CollisionEnt::Ball),
        (paddle.idx(), CollisionEnt::Paddle),
        Box::new(bounce),
    );

    let move_ball = |_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::new(1.0, 1.0)));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyInvalid(event.set, event.action_id));
    };

    game.add_ctrl_predicate(
        paddle,
        action_serve,
        ControlEventType::KeyPressed,
        Box::new(move_ball),
    );
}
