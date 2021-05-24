#![allow(unused)]
use macroquad::prelude::*;
use paddles_engine::*;

const WIDTH: u16 = 500;
const HEIGHT: u16 = 500;
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
    // clippy's annoyed about an iterator but i can't seem to silence the warning on that line [shrug emoji]
    #![allow(clippy::needless_collect)]

    // initialize game
    let mut game = Game::new();

    let mut ball = Ball::new();
    ball.set_pos(Vec2::new(
        WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
        HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
    ));
    ball.set_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32));

    let ball = game.add_ball(ball);

    let walls = vec![
        {
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(-1.0, 0.0));
            wall.set_size(Vec2::new(1.0, HEIGHT as f32));
            wall
        },
        {
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(WIDTH as f32, 0.0));
            wall.set_size(Vec2::new(1.0, HEIGHT as f32));
            wall
        },
        {
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(0.0, -1.0));
            wall.set_size(Vec2::new(WIDTH as f32, 1.0));
            wall
        },
        {
            let mut wall = Wall::new();
            wall.set_pos(Vec2::new(0.0, HEIGHT as f32));
            wall.set_size(Vec2::new(WIDTH as f32, 1.0));
            wall
        },
    ];

    let walls = walls
        .into_iter()
        .map(|wall| game.add_wall(wall))
        .collect::<Vec<_>>();

    let mut paddle1 = Paddle::new();
    let action_o = paddle1.add_control_map(KeyCode::O);
    let action_l = paddle1.add_control_map(KeyCode::L);
    paddle1.set_pos(Vec2::new(
        PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle1.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    let paddle1 = game.add_paddle(paddle1);

    let mut paddle2 = Paddle::new();
    let action_w = paddle2.add_control_map(KeyCode::W);
    let action_s = paddle2.add_control_map(KeyCode::S);
    paddle2.set_pos(Vec2::new(
        WIDTH as f32 - PADDLE_OFF_X as f32,
        HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
    ));
    paddle2.set_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32));

    let paddle2 = game.add_paddle(paddle2);

    let score1 = Score::new();
    let score1 = game.add_score(score1);

    // idk
    // game.add_collision_react(
    //     CollisionEnt::Ball(ball),
    //     CollisionEnt::Wall(walls[0]),
    //     Box::new(/* ??? */),
    // );

    // i know we talked about controls "not being physical" about 70 times but i don't know how else to represent this in a logic. CollisionEvent::ChangePos and CollisionEvent::ChangeVel? but that doesn't feel intuitive.
    // i guess collision presents the idea of "objects in space"? but it doesn't provide the concept of "move an object by x amount"
    // .with_key_event(
    //     action_o,
    //     ControlEvent::KeyHeld,
    //     Box::new(PhysicsReaction::SetVel(Vec2::new(0.0, -1.0))),
    // )
    // .with_key_event(
    //     action_l,
    //     ControlEvent::KeyHeld,
    //     Box::new(PhysicsReaction::SetVel(Vec2::new(0.0, 1.0))),
    // )

    run(game).await;
}
