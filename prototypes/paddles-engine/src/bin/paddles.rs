use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const BALL_SIZE: u8 = 10;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;

fn window_conf() -> Conf {
    Conf {
        window_title: "paddles".to_owned(),
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
        HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
    ));
    ball.set_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));
    let ball = game.add_ball(ball);

    // walls
    // left
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(-1.0, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    let left_wall = game.add_wall(wall);
    // right
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    let right_wall = game.add_wall(wall);
    // top
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, -1.0));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let top_wall = game.add_wall(wall);
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let bottom_wall = game.add_wall(wall);

    // paddle 1
    let mut paddle1 = Paddle::new();
    paddle1.add_control_map(KeyCode::Q, true);
    paddle1.add_control_map(KeyCode::A, true);
    let action_w = paddle1.add_control_map(KeyCode::W, true);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    let paddle1 = game.add_paddle(paddle1);

    // paddle 2
    let mut paddle2 = Paddle::new();
    paddle2.add_control_map(KeyCode::O, true);
    paddle2.add_control_map(KeyCode::L, true);
    let action_i = paddle2.add_control_map(KeyCode::I, false);
    paddle2.set_pos(Vec2::new(
        WIDTH as f32 - PADDLE_OFF_X as f32 - PADDLE_WIDTH as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle2.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    let paddle2 = game.add_paddle(paddle2);

    let score1 = game.add_score(Score::new());
    let score2 = game.add_score(Score::new());

    game.set_paddle_col_synthesis(Box::new(|mut paddle: Paddle| {
        if paddle.controls[0].3.value > 0.0 {
            paddle.pos.y -= 1.0;
            // paddle.vel.y -= 1.0;
        }
        if paddle.controls[1].3.value > 0.0 {
            paddle.pos.y += 1.0;
            // paddle.vel.y = (-1.0).min(paddle.vel.y);
        }
        paddle
    }));

    let inc_score = |_: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
        println!("collided");
        if let CollisionEnt::Wall(wall_id) = event.1 {
            logics.resources.handle_predicate(&(
                RsrcPool::Score(ScoreID::new(wall_id.idx())),
                Transaction::Change(1),
            ));
        }
    };

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Wall(left_wall),
        Box::new(inc_score),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Wall(right_wall),
        Box::new(inc_score),
    );

    let reset_ball = |state: &mut State, logics: &mut Logics, event: &ResourceEvent<RsrcPool>| {
        let RsrcPool::Score(score_id) = event.pool;
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));
        logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(
                state.get_col_idx(CollisionEnt::Ball(BallID::new(0))),
                Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                ),
            ));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyValid(
                score_id.idx(),
                ActionID::new(2), // eh....
            ));
    };

    game.add_rsrc_predicate(
        RsrcPool::Score(score1),
        ResourceEventType::PoolUpdated,
        Box::new(reset_ball),
    );
    game.add_rsrc_predicate(
        RsrcPool::Score(score2),
        ResourceEventType::PoolUpdated,
        Box::new(reset_ball),
    );

    let bounce_ball_y =
        |_: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
            if let CollisionEnt::Ball(ball_id) = event.0 {
                let mut vals = logics.physics.get_synthesis(ball_id.idx());
                vals.vel.y *= -1.0;
                logics.physics.update_synthesis(ball_id.idx(), vals);
            }
        };

    let bounce_ball_x =
        |_: &mut State, logics: &mut Logics, event: &CollisionEvent<CollisionEnt>| {
            if let CollisionEnt::Ball(ball_id) = event.0 {
                let mut vals = logics.physics.get_synthesis(ball_id.idx());
                vals.vel.x *= -1.0;
                logics.physics.update_synthesis(ball_id.idx(), vals);
            }
        };

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Wall(top_wall),
        Box::new(bounce_ball_y),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Wall(bottom_wall),
        Box::new(bounce_ball_y),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Paddle(paddle1),
        Box::new(bounce_ball_x),
    );

    game.add_collision_predicate(
        CollisionEnt::Ball(ball),
        CollisionEnt::Paddle(paddle2),
        Box::new(bounce_ball_x),
    );

    let move_ball = |_: &mut State, logics: &mut Logics, event: &ControlEvent<ActionID>| {
        let vel = match event.set {
            0 => Vec2::new(1.0, 1.0),
            1 => Vec2::new(-1.0, -1.0),
            _ => unreachable!(),
        };
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, vel));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyInvalid(event.set, event.action_id));
    };

    game.add_ctrl_predicate(
        paddle1,
        action_w,
        ControlEventType::KeyPressed,
        Box::new(move_ball),
    );

    game.add_ctrl_predicate(
        paddle2,
        action_i,
        ControlEventType::KeyPressed,
        Box::new(move_ball),
    );
}
