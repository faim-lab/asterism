#![allow(dead_code)]
#![allow(clippy::single_match)]
use std::io::{self, Write};

use asterism::{
    collision::{AabbCollision, Vec2 as AstVec2},
    control::{KeyboardControl, MacroQuadKeyboardControl},
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
    GameState, Logic,
};
use macroquad::prelude::*;

mod data;
mod ids;

use data::*;
use ids::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

struct Paddle {
    pos: u8,
    speed: f32,
}

struct Ball {
    starting_pos: Vec2,
    pos: Vec2,
    vel: Vec2,
}

impl Ball {
    fn new(pos: Vec2) -> Self {
        Self {
            starting_pos: pos,
            pos,
            vel: Vec2::zero(),
        }
    }
}

struct Wall {
    pos: Vec2,
    size: Vec2,
}

impl Wall {
    fn new(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }
}

struct World {
    paddles: (Paddle, Paddle),
    balls: Vec<Ball>,
    walls: Vec<Wall>,
    serving: Option<Player>,
    score: (u8, u8),
}

impl GameState for World {}

struct Logics {
    control: MacroQuadKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,

    // i'm slightly scared of actually implementing this
    data: Data<World>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = MacroQuadKeyboardControl::new();
                control.add_key_map(0, KeyCode::Q, ActionID::MoveUp(Player::P1));
                control.add_key_map(0, KeyCode::A, ActionID::MoveDown(Player::P1));
                control.add_key_map(0, KeyCode::W, ActionID::Serve(Player::P1));
                control.add_key_map(1, KeyCode::O, ActionID::MoveUp(Player::P2));
                control.add_key_map(1, KeyCode::L, ActionID::MoveDown(Player::P2));
                control.add_key_map(1, KeyCode::I, ActionID::Serve(Player::P2));
                control.add_key_map(2, KeyCode::Escape, ActionID::Quit);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                // left
                collision.add_entity_as_xywh(
                    -2.0,
                    0.0,
                    2.0,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P1),
                );
                // right
                collision.add_entity_as_xywh(
                    WIDTH as f32,
                    0.0,
                    2.0,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P2),
                );
                // top
                collision.add_entity_as_xywh(
                    0.0,
                    -2.0,
                    WIDTH as f32,
                    2.0,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::BounceWall,
                );
                // bottom
                collision.add_entity_as_xywh(
                    0.0,
                    HEIGHT as f32,
                    WIDTH as f32,
                    2.0,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::BounceWall,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points(Player::P1), 0.0);
                resources.items.insert(PoolID::Points(Player::P2), 0.0);
                resources
            },
            data: Data::new(Vec::new()),
        }
    }
}

enum Variant {
    Paddles,
    TrickBall,
    TrickPaddle,
    // WallBreaker,
    // PaddleBallMania,
}

fn get_variant() -> Variant {
    println!("please enter which paddles variant you want to play.\npaddles: 1\ntrick-ball: 2\ntrick-paddle: 3"); // \nwall-breaker: 4\npaddle-ball-mania: 5");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => return Variant::Paddles,
            "2" => return Variant::TrickBall,
            "3" => return Variant::TrickPaddle,
            // "4" => return Variant::WallBreaker,
            // "5" => return Variant::PaddleBallMania,
            _ => {
                println!("please enter a valid input...");
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // let variant = get_variant();

    let mut world = World::new();
    let mut logics = Logics::new();

    loop {
        if !world.update(&mut logics) {
            break;
        }
        world.draw();
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (
                Paddle {
                    pos: HEIGHT / 2 - PADDLE_HEIGHT / 2,
                    speed: 1.0,
                },
                Paddle {
                    pos: HEIGHT / 2 - PADDLE_HEIGHT / 2,
                    speed: 1.0,
                },
            ),
            balls: vec![Ball::new(Vec2::new(
                (WIDTH / 2 - BALL_SIZE / 2) as f32,
                (HEIGHT / 2 - BALL_SIZE / 2) as f32,
            ))],
            walls: Vec::new(),
            serving: Some(Player::P1),
            score: (0, 0),
        }
    }

    fn update(&mut self, logics: &mut Logics) -> bool {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        if logics.control.values[2][0].value != 0.0 {
            return false;
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        // let mut remove_walls = Vec::new();

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Ball(i), CollisionID::ScoreWall(player)) => {
                    self.balls[i].pos = self.balls[i].starting_pos;
                    self.balls[i].vel = Vec2::zero();
                    match player {
                        Player::P1 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P2), Transaction::Change(1.0))]);
                            self.serving = Some(Player::P2);
                        }
                        Player::P2 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P1), Transaction::Change(1.0))]);
                            self.serving = Some(Player::P1);
                        }
                    }
                }

                (CollisionID::Ball(i), CollisionID::BounceWall) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(&contact, &CollisionID::Ball(i));
                    if sides_touched.y != 0.0 {
                        self.balls[i].vel.y *= -1.0;
                    }
                    if sides_touched.x != 0.0 {
                        self.balls[i].vel.x *= -1.0;
                    }
                }

                (CollisionID::Ball(i), CollisionID::Ball(j)) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));
                    if sides_touched.x != 0.0 {
                        // restituted left/right
                        self.balls[i].vel.x *= -1.0;
                        self.balls[j].vel.x *= -1.0;
                    }
                    if sides_touched.y != 0.0 {
                        // restituted up/down
                        self.balls[i].vel.y *= -1.0;
                        self.balls[j].vel.y *= -1.0;
                    }
                }

                (CollisionID::Ball(i), CollisionID::Paddle(player)) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));
                    match player {
                        Player::P1 => {
                            if sides_touched.x > 0.0 {
                                self.balls[i].vel.x *= -1.0;
                            }
                        }
                        Player::P2 => {
                            if sides_touched.x < 0.0 {
                                self.balls[i].vel.x *= -1.0;
                            }
                        }
                    }
                    if sides_touched.y != 0.0 {
                        self.balls[i].vel.y *= -1.0;
                    }

                    self.change_angle(i, player);
                    if self.balls[i].vel.magnitude() < 5.0 {
                        self.balls[i].vel *= 1.1;
                    }
                }

                _ => {}
            }
        }

        /* for idx in remove_walls.iter().rev() {
            self.walls.remove(*idx);
        } */

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        match item_type {
                            PoolID::Points(player) => {
                                match player {
                                    Player::P1 => print!("p1"),
                                    Player::P2 => print!("p2"),
                                }
                                println!(" scores! p1: {}, p2: {}", self.score.0, self.score.1);
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }

        true
    }

    fn project_control(&self, control: &mut MacroQuadKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[1][0].is_valid = true;
        control.mapping[1][1].is_valid = true;
        control.mapping[2][0].is_valid = true;

        for Ball { vel, .. } in self.balls.iter() {
            if (vel.x, vel.y) == (0.0, 0.0) {
                match self.serving {
                    Some(Player::P1) => control.mapping[0][2].is_valid = true,
                    Some(Player::P2) => control.mapping[1][2].is_valid = true,
                    None => {}
                }
            } else {
                control.mapping[0][2].is_valid = false;
                control.mapping[1][2].is_valid = false;
            }
        }
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.paddles.0.pos = ((self.paddles.0.pos as i16
            + ((control
                .get_action_in_set(0, ActionID::MoveDown(Player::P1))
                .unwrap()
                .value
                - control
                    .get_action_in_set(0, ActionID::MoveUp(Player::P1))
                    .unwrap()
                    .value)
                * self.paddles.0.speed) as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);
        self.paddles.1.pos = ((self.paddles.1.pos as i16
            + ((control
                .get_action_in_set(1, ActionID::MoveDown(Player::P2))
                .unwrap()
                .value
                - control
                    .get_action_in_set(1, ActionID::MoveUp(Player::P2))
                    .unwrap()
                    .value)
                * self.paddles.1.speed) as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);

        for Ball { vel, .. } in self.balls.iter_mut() {
            if (vel.x, vel.y) == (0.0, 0.0) {
                match self.serving {
                    Some(Player::P1) => {
                        let values = control
                            .get_action_in_set(0, ActionID::Serve(Player::P1))
                            .unwrap();
                        if values.changed_by >= 1.0 && values.value != 0.0 {
                            vel.x = 1.0;
                            vel.y = 1.0;
                        }
                    }
                    Some(Player::P2) => {
                        let values = control
                            .get_action_in_set(1, ActionID::Serve(Player::P2))
                            .unwrap();
                        if values.changed_by >= 1.0 && values.value != 0.0 {
                            vel.x = -1.0;
                            vel.y = -1.0;
                        }
                    }
                    None => {}
                }
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        for Ball { pos, vel, .. } in self.balls.iter() {
            physics.add_physics_entity(*pos, *vel, Vec2::zero());
        }
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        for (i, Ball { pos, vel, .. }) in self.balls.iter_mut().enumerate() {
            *pos = physics.positions[i];
            *vel = physics.velocities[i];
        }
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID, Vec2>) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        for (i, Ball { pos, vel, .. }) in self.balls.iter().enumerate() {
            collision.add_entity_as_xywh(
                pos.x,
                pos.y,
                BALL_SIZE as f32,
                BALL_SIZE as f32,
                *vel,
                true,
                false,
                CollisionID::Ball(i),
            );
        }

        for (i, Wall { pos, size }) in self.walls.iter().enumerate() {
            collision.add_entity_as_xywh(
                pos.x,
                pos.y,
                size.x,
                size.y,
                Vec2::zero(),
                true,
                true,
                CollisionID::BreakWall(i),
            );
        }

        collision.add_entity_as_xywh(
            PADDLE_OFF_X as f32,
            self.paddles.0.pos as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0, self.paddles.0.speed),
            true,
            true,
            CollisionID::Paddle(Player::P1),
        );

        collision.add_entity_as_xywh(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
            self.paddles.1.pos as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0, self.paddles.1.speed),
            true,
            true,
            CollisionID::Paddle(Player::P2),
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        for (i, mut ball) in self.balls.iter_mut().enumerate() {
            ball.pos = collision
                .get_xy_pos_for_entity(CollisionID::Ball(i))
                .unwrap();
        }
    }

    fn change_angle(&mut self, ball_idx: usize, player: Player) {
        let ball = &mut self.balls[ball_idx];
        let paddle_center = match player {
            Player::P1 => self.paddles.0.pos + PADDLE_HEIGHT / 2,
            Player::P2 => self.paddles.1.pos + PADDLE_HEIGHT / 2,
        } as f32;
        let angle: f32 = (((ball.pos.y + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-(PADDLE_HEIGHT as f32) / 2.0)
            .min(PADDLE_HEIGHT as f32 / 2.0)
            / PADDLE_HEIGHT as f32)
            .abs()
            * 80.0;
        let magnitude = ball.vel.magnitude();
        ball.vel.x =
            angle.to_radians().cos() * magnitude * if ball.vel.x < 0.0 { -1.0 } else { 1.0 };
        ball.vel.y =
            angle.to_radians().sin() * magnitude * if ball.vel.y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        resources
            .items
            .entry(PoolID::Points(Player::P1))
            .or_insert(0.0);
        resources
            .items
            .entry(PoolID::Points(Player::P2))
            .or_insert(0.0);
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types.iter() {
                        let value = resources.get_value_by_itemtype(item_type).unwrap() as u8;
                        match item_type {
                            PoolID::Points(player) => match player {
                                Player::P1 => self.score.0 = value,
                                Player::P2 => self.score.1 = value,
                            },
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    fn draw(&self) {
        clear_background(Color::new(0., 0., 0.5, 1.));

        // paddles
        draw_rectangle(
            PADDLE_OFF_X as f32,
            self.paddles.0.pos as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
            self.paddles.1.pos as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            WHITE,
        );

        for Ball { pos, .. } in self.balls.iter() {
            draw_rectangle(
                pos.x,
                pos.y,
                BALL_SIZE as f32,
                BALL_SIZE as f32,
                Color::new(1., 0.75, 0., 1.),
            );
        }

        for Wall { pos, size } in self.walls.iter() {
            draw_rectangle(pos.x, pos.y, size.x, size.y, Color::new(0.8, 0.8, 0.8, 1.));
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "paddles variants".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}
