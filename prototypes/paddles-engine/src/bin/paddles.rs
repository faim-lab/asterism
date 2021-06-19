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
    let action_q = paddle1.add_control_map(KeyCode::Q, true);
    let action_a = paddle1.add_control_map(KeyCode::A, true);
    let action_w = paddle1.add_control_map(KeyCode::W, true);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));
    let paddle1 = game.add_paddle(paddle1);

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

    let paddle2 = game.add_paddle(paddle2);

    let score1 = game.add_score(Score::new());
    let score2 = game.add_score(Score::new());

    game.add_ctrl_predicate(
        Box::new(move |ctrl: &CtrlEvent| -> bool {
            (ctrl.action_id == action_q || ctrl.action_id == action_o)
                && ctrl.event_type == ControlEventType::KeyHeld
        }),
        Box::new(
            |state: &mut State, logics: &mut Logics, event: &CtrlEvent| {
                let mut paddle_col = logics.collision.get_synthesis(event.set);
                paddle_col.center.y -= 1.0;
                paddle_col.vel.y = (paddle_col.vel.y.abs() - 1.0).max(-1.0);
                logics.collision.update_synthesis(event.set, paddle_col);
            },
        ),
    );

    game.add_ctrl_predicate(
        Box::new(move |ctrl: &CtrlEvent| -> bool {
            (ctrl.action_id == action_a || ctrl.action_id == action_l)
                && ctrl.event_type == ControlEventType::KeyHeld
        }),
        Box::new(
            |state: &mut State, logics: &mut Logics, event: &CtrlEvent| {
                let mut paddle_col = logics.collision.get_synthesis(event.set);
                paddle_col.center.y += 1.0;
                paddle_col.vel.y = (paddle_col.vel.y.abs() + 1.0).min(1.0);
                logics.collision.update_synthesis(event.set, paddle_col);
            },
        ),
    );

    let inc_score = |_: &mut State, logics: &mut Logics, event: &ColEvent| {
        let change_score = match event.1 {
            0 => 1,
            1 => 0,
            _ => {
                unreachable!()
            }
        };
        logics.resources.handle_predicate(&(
            RsrcPool::Score(ScoreID::new(change_score)),
            Transaction::Change(1),
        ));
    };

    game.add_collision_predicate(
        // gives an error about lifetimes. can you curry in rust??????
        Box::new(|contact: &ColEvent| -> bool {
            contact.0 == ball.idx()
                && (contact.1 == game.state.get_col_idx(left_wall.idx(), CollisionEnt::Wall)
                    || contact.1 == game.state.get_col_idx(right_wall.idx(), CollisionEnt::Wall))
        }),
        Box::new(inc_score),
    );

    let reset_ball = |state: &mut State, logics: &mut Logics, event: &RsrcEvent| {
        let RsrcPool::Score(score_id) = event.pool;
        logics
            .physics
            .handle_predicate(&PhysicsReaction::SetVel(0, Vec2::ZERO));
        logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(
                state.get_col_idx(0, CollisionEnt::Ball),
                Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                ),
            ));
        logics
            .control
            .handle_predicate(&ControlReaction::SetKeyValid(
                match score_id.idx() {
                    0 => 1,
                    1 => 0,
                    _ => unreachable!(),
                },
                ActionID::new(2), // eh....
            ));
    };

    game.add_rsrc_predicate(
        Box::new(|rsrc: &RsrcEvent| -> bool { rsrc.event_type == ResourceEventType::PoolUpdated }),
        Box::new(reset_ball),
    );

    let bounce_ball = |state: &mut State, logics: &mut Logics, (i, j): &ColEvent| {
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
    };

    game.add_collision_predicate(
        Box::new(|(i, j): &ColEvent| -> bool {
            game.logics.collision.metadata[*i].id == CollisionEnt::Ball
                && game.logics.collision.metadata[*j].id == CollisionEnt::Wall
        }),
        Box::new(bounce_ball),
    );

    let move_ball = |_: &mut State, logics: &mut Logics, event: &CtrlEvent| {
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
        Box::new(move |ctrl: &CtrlEvent| -> bool {
            (ctrl.action_id == action_w || ctrl.action_id == action_i)
                && ctrl.event_type == ControlEventType::KeyPressed
        }),
        Box::new(move_ball),
    );
}
