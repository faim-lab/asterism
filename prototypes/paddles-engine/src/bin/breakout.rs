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
    // left
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(-1.0, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    game.add_wall(wall);
    // right
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
    wall.set_size(Vec2::new(1.0, HEIGHT as f32));
    game.add_wall(wall);
    // top
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, -1.0));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    game.add_wall(wall);
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    let bottom_wall = game.add_wall(wall);

    // blocks
    let block_size = Vec2::new(32.0, 16.0);
    (0..5).for_each(|y| {
        (0..8).for_each(|x| {
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(x as f32 * 32.0, y as f32 * 16.0));
            wall.set_size(block_size);
            game.add_wall(wall);
        })
    });

    // paddle 1
    let mut paddle = Paddle::new();
    let left = paddle.add_control_map(KeyCode::Left, true);
    let right = paddle.add_control_map(KeyCode::Right, true);
    let action_serve = paddle.add_control_map(KeyCode::Space, true);
    paddle.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - PADDLE_WIDTH as f32 / 2.0,
        HEIGHT as f32 - PADDLE_OFF_X as f32,
    ));
    paddle.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    let paddle = game.add_paddle(paddle);

    let score = game.add_score(Score::new());

    game.add_ctrl_predicate(
        CtrlEvent {
            set: 0,
            action_id: left,
            event_type: ControlEventType::KeyHeld,
        },
        Box::new(|_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
            let mut paddle_col = logics.collision.get_synthesis(event.set);
            paddle_col.center.x -= 1.0;
            paddle_col.vel.x = (paddle_col.vel.x.abs() - 1.0).max(-1.0);
            logics.collision.update_synthesis(event.set, paddle_col);
        }),
    );

    game.add_ctrl_predicate(
        CtrlEvent {
            set: 0,
            action_id: right,
            event_type: ControlEventType::KeyHeld,
        },
        Box::new(|_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
            let mut paddle_col = logics.collision.get_synthesis(event.set);
            paddle_col.center.x += 1.0;
            paddle_col.vel.x = (paddle_col.vel.x.abs() + 1.0).min(1.0);
            logics.collision.update_synthesis(event.set, paddle_col);
        }),
    );

    let reset_game = move |state: &mut State, logics: &mut Logics| {
        // reset blocks? not sure how to do this

        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));

        logics
            .resources
            .handle_predicate(&(RsrcPool::Score(score), Transaction::Set(0)));

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
            .handle_predicate(&ControlReaction::SetKeyValid(0, action_serve));
    };

    game.add_collision_predicate(
        ColEvent::ByIdx(
            game.state.get_col_idx(ball.idx(), CollisionEnt::Ball),
            game.state
                .get_col_idx(bottom_wall.idx(), CollisionEnt::Wall),
        ),
        Box::new(move |state, logics, _| reset_game(state, logics)),
    );

    let bounce = move |state: &mut State, logics: &mut Logics, (i, j): &AColEvent| {
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

            let id = state.get_id(*j);
            if let EntID::Wall(wall_id) = id {
                if wall_id.idx() >= 4 {
                    state.queue_remove(EntID::Wall(wall_id));
                    logics
                        .resources
                        .handle_predicate(&(RsrcPool::Score(score), Transaction::Change(1)));
                }
            }
        }
    };

    game.add_collision_predicate(
        ColEvent::ByType(CollisionEnt::Ball, CollisionEnt::Wall),
        Box::new(bounce),
    );

    game.add_collision_predicate(
        ColEvent::ByType(CollisionEnt::Ball, CollisionEnt::Paddle),
        Box::new(bounce),
    );

    game.add_rsrc_ident_predicate(
        RsrcIdent {
            pool: Some(RsrcPool::Score(score)),
            threshold: 40,
            op: std::cmp::Ordering::Greater,
        },
        Box::new(move |state, logics, _| reset_game(state, logics)),
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
        CtrlEvent {
            set: paddle.idx(),
            action_id: action_serve,
            event_type: ControlEventType::KeyPressed,
        },
        Box::new(move_ball),
    );
}
