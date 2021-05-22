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
    let mut game = Game::new()
        .with_ball(
            Ball::new()
                .with_pos(Vec2::new(
                    WIDTH as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    HEIGHT as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                ))
                .with_size(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32)),
        )
        .with_paddle(
            Paddle::new()
                .with_pos(Vec2::new(
                    PADDLE_OFF_X as f32,
                    HEIGHT as f32 / 2.0 - PADDLE_HEIGHT as f32 / 2.0,
                ))
                .with_size(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32)),
        );
    game.run().await;
}
