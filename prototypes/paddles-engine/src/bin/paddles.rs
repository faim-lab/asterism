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
    game.add_ball(ball);

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
    game.add_wall(wall);
    // bottom
    let mut wall = Wall::new();
    wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
    wall.set_size(Vec2::new(WIDTH as f32, 1.0));
    game.add_wall(wall);

    // paddle 1
    let mut paddle1 = Paddle::new();
    let action_q = paddle1.add_control_map(KeyCode::Q, true);
    let action_a = paddle1.add_control_map(KeyCode::A, true);
    let action_w = paddle1.add_control_map(KeyCode::W, true);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    game.add_paddle(paddle1);

    // paddle 2
    let mut paddle2 = Paddle::new();
    let action_o = paddle2.add_control_map(KeyCode::O, true);
    let action_l = paddle2.add_control_map(KeyCode::L, true);
    let action_i = paddle2.add_control_map(KeyCode::I, false);
    paddle2.set_pos(Vec2::new(
        WIDTH as f32 - PADDLE_OFF_X as f32 - PADDLE_WIDTH as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle2.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    game.add_paddle(paddle2);

    let score1 = game.add_score(Score::new());
    let score2 = game.add_score(Score::new());

    // expands to a match statement mapping the values 0, 1 to the two inputs given
    //
    // example:
    // match_set!(set, 1, 0)
    //
    // expands to
    //
    // match set {
    //     0 => 1,
    //     1 => 0,
    //     _ => unreachable!(),
    // }
    macro_rules! match_set {
        ($which_set:expr, $if1:expr, $if2:expr) => {
            match $which_set {
                0 => $if1,
                1 => $if2,
                _ => unreachable!(),
            }
        };
    }

    // paddle movement
    let move_down = |logics: &mut Logics, set| {
        let mut paddle_col = logics.collision.get_ident_data(set);
        paddle_col.center.y += 1.0;
        paddle_col.vel.y = (paddle_col.vel.y.abs() + 1.0).min(1.0);
        logics.collision.update_ident_data(set, paddle_col);
    };

    let move_up = |logics: &mut Logics, set| {
        let mut paddle_col = logics.collision.get_ident_data(set);
        paddle_col.center.y -= 1.0;
        paddle_col.vel.y = (paddle_col.vel.y.abs() - 1.0).max(-1.0);
        logics.collision.update_ident_data(set, paddle_col);
    };

    // serving
    let serve_ball = move |logics: &mut Logics, set: usize| {
        let vel = match_set!(set, Vec2::splat(1.0), Vec2::splat(-1.0));
        let action_id = match_set!(set, action_w, action_i);
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, vel));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyInvalid(set, action_id));
    };

    // increase score on collision with side wall
    let inc_score = move |logics: &mut Logics, set: usize| {
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyValid(
                set,
                match_set!(set, action_w, action_i),
            ));
        logics.resources.handle_predicate(&(
            RsrcPool::Score(match_set!(set, score1, score2)),
            Transaction::Change(1),
        ));
    };

    let bounce_ball = |(i, j): &ColEvent, state: &mut State, logics: &mut Logics| {
        let id = state.get_id(*i);
        if let EntID::Ball(ball_id) = id {
            let sides_touched = logics.collision.sides_touched(*i, *j);
            let mut vals = logics.physics.get_ident_data(ball_id.idx());
            if sides_touched.y != 0.0 {
                vals.vel.y *= -1.0;
            }
            if sides_touched.x != 0.0 {
                vals.vel.x *= -1.0;
            }
            logics.physics.update_ident_data(ball_id.idx(), vals);
        }
    };

    let move_paddle = QueryType::User(game.add_query());
    let serve = QueryType::User(game.add_query());
    let bounce = QueryType::User(game.add_query());
    let score = QueryType::User(game.add_query());
    let score_increased = QueryType::User(game.add_query());

    paddles_engine::rules!(game =>
        control: [
            {
                filter move_paddle,
                QueryType::CtrlEvent => CtrlEvent,
                |ctrl, _, _| {
                    ctrl.event_type == ControlEventType::KeyHeld
                },
                foreach |ctrl, _, logics| {
                    if ctrl.action_id == action_q || ctrl.action_id == action_o {
                        move_up(logics, ctrl.set);
                    } else if ctrl.action_id == action_a || ctrl.action_id == action_l {
                        move_down(logics, ctrl.set);
                    }
                }
            },
            {
                filter serve,
                QueryType::CtrlEvent => CtrlEvent,
                |ctrl, _, _| {
                    ctrl.event_type == ControlEventType::KeyPressed && (ctrl.action_id == action_w || ctrl.action_id == action_i)
                },
                foreach |ctrl, _, logics| {
                    serve_ball(logics, ctrl.set);
                }
            }
        ]

        physics: []

        collision: [
            {
                filter bounce,
                QueryType::ColEvent => ColEvent,
                |(i, j), _, logics| {
                    let i_id = logics.collision.metadata[*i].id;
                    let j_id = logics.collision.metadata[*j].id;
                    i_id == CollisionEnt::Ball &&
                        (j_id == CollisionEnt::Wall || j_id == CollisionEnt::Paddle)
                },
                foreach |col, state, logics| {
                    bounce_ball(col, state, logics);
                }
            },
            {
                filter score,
                QueryType::ColEvent => ColEvent,
                |(i, j), state, logics| {
                    let i_id = logics.collision.metadata[*i].id;
                    i_id == CollisionEnt::Ball &&
                        (*j == state.get_col_idx(left_wall.idx(), CollisionEnt::Wall) || *j == state.get_col_idx(right_wall.idx(), CollisionEnt::Wall))
                },
                foreach |(_, j), state, logics| {
                    if *j == state.get_col_idx(left_wall.idx(), CollisionEnt::Wall) {
                        inc_score(logics, 1);
                    } else if *j == state.get_col_idx(right_wall.idx(), CollisionEnt::Wall) {
                        inc_score(logics, 0);
                    } else {
                        unreachable!();
                    }
                }
            }
        ]

        resources: [
            {
                filter score_increased,
                QueryType::RsrcEvent => RsrcEvent,
                |pool, _, _| {
                    pool.event_type == ResourceEventType::PoolUpdated
                },
                foreach |event, _, logics| {
                    let RsrcPool::Score(score) = event.pool;

                    println!(
                        "p{} scored: {}",
                        score.idx() + 1,
                        logics.resources.get_ident_data(event.pool).0
                    );
                    logics
                        .physics
                        .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));

                    logics.physics.handle_predicate(&PhysicsReaction::SetPos(
                        0,
                        Vec2::splat(WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0),
                    ));

                    logics
                        .control
                        .handle_predicate(&ControlReaction::SetKeyValid(
                            score.idx(),
                            match_set!(score.idx(), action_w, action_i),
                        ));
                }
            }
        ]
    );
}
