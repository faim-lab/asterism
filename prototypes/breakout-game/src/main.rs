use asterism::{
    collision::AabbCollision,
    control::{KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
};
use macroquad::prelude::*;
use std::io::{self, Write};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_Y: u8 = 237;
const PADDLE_WIDTH: u8 = 48;
const PADDLE_HEIGHT: u8 = 8;
const BALL_SIZE: u8 = 8;
const BLOCK_WIDTH: u8 = 32;
const BLOCK_HEIGHT: u8 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveRight,
    MoveLeft,
    Serve,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum CollisionID {
    Paddle,
    Ball,
    EndWall,
    TopWall,
    SideWall,
    Block(usize, usize),
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Ball
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum PoolID {
    Points,
}

#[derive(Clone, Copy)]
struct Block {
    color: Color,
    visible: bool,
}

struct Logics {
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
    resources: QueuedResources<PoolID, u8>,
}

struct World {
    paddle: u8,
    ball: Vec2,
    ball_vel: Vec2,
    blocks: [[Block; 8]; 5],
    score: u8,
}

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

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = KeyboardControl::new();
                control.add_key_map(0, KeyCode::Right, ActionID::MoveRight, true);
                control.add_key_map(0, KeyCode::Left, ActionID::MoveLeft, true);
                control.add_key_map(0, KeyCode::Space, ActionID::Serve, true);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit, true);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                // left
                collision.add_entity_as_xywh(
                    Vec2::new(-2.0, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::SideWall,
                );
                // right
                collision.add_entity_as_xywh(
                    Vec2::new(WIDTH as f32, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::SideWall,
                );
                // top
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, -2.0),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::TopWall,
                );
                // bottom
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, HEIGHT as f32),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::EndWall,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources
                    .items
                    .insert(PoolID::Points, (0, u8::MIN, u8::MAX));
                resources
            },
        }
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddle: WIDTH / 2 - PADDLE_WIDTH / 2,
            ball: Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, (HEIGHT / 4 * 3) as f32),
            ball_vel: Vec2::new(0.0, 0.0),
            blocks: [
                [Block::new(BLUE, true); 8],
                [Block::new(PINK, true); 8],
                [Block::new(WHITE, true); 8],
                [Block::new(PINK, true); 8],
                [Block::new(BLUE, true); 8],
            ],
            score: 0,
        }
    }

    fn update(&mut self, logics: &mut Logics) -> bool {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        if logics.control.values[0][3].value != 0.0 {
            return false;
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        let mut block_broken = false;
        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Ball, CollisionID::EndWall) => {
                    println!("\ngame over");
                    self.reset();
                    // i don't know if i like this
                    logics.resources.items.clear();
                }
                (CollisionID::Ball, CollisionID::Paddle) => {
                    let sides_touched = logics.collision.sides_touched(contact.i, contact.j);
                    dbg!(sides_touched);
                    self.ball_vel.y *= -1.0;
                    if sides_touched.y < 0.0 {
                        // self.change_angle();
                    } else if sides_touched.x != 0.0 {
                        self.ball_vel.x *= -1.0;
                    }
                    self.ball_vel *= 1.1;
                }
                (CollisionID::Ball, CollisionID::SideWall) => {
                    self.ball_vel.x *= -1.0;
                }
                (CollisionID::Ball, CollisionID::TopWall) => self.ball_vel.y *= -1.0,
                (CollisionID::Ball, CollisionID::Block(i, j)) => {
                    if !block_broken {
                        let sides_touched = logics.collision.sides_touched(contact.i, contact.j);
                        if sides_touched.x != 0.0 {
                            self.ball_vel.x *= -1.0;
                        } else if sides_touched.y != 0.0 {
                            self.ball_vel.y *= -1.0;
                        }
                        self.blocks[i][j].visible = false;
                        logics
                            .resources
                            .transactions
                            .push((PoolID::Points, Transaction::Change(1)));
                        block_broken = true;
                    }
                }
                _ => {}
            }
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_type) => match item_type {
                    PoolID::Points => {
                        print!("current score: {}\r", self.score);
                        io::stdout().flush().unwrap();
                        if self.score >= 40 {
                            println!("\nyou win!");
                            self.reset();
                        }
                    }
                },
                Err(_) => {}
            }
        }

        true
    }

    fn project_control(&self, control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        if self.ball_vel.x == 0.0 && self.ball_vel.y == 0.0 {
            control.mapping[0][2].is_valid = true;
        }
        control.mapping[0][3].is_valid = true;
    }

    fn unproject_control(&mut self, control: &KeyboardControl<ActionID, MacroquadInputWrapper>) {
        self.paddle = ((self.paddle as i16
            + control.get_action(ActionID::MoveRight).unwrap().value as i16
            - control.get_action(ActionID::MoveLeft).unwrap().value as i16)
            .max(0) as u8)
            .min(WIDTH - PADDLE_WIDTH);

        if self.ball_vel.x == 0.0 && self.ball_vel.y == 0.0 {
            let values = control.get_action(ActionID::Serve).unwrap();
            if values.changed_by > 0.0 && values.value != 0.0 {
                self.ball_vel = Vec2::new(1.0, 1.0);
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        physics.add_physics_entity(self.ball, self.ball_vel, Vec2::new(0.0, 0.0));
    }

    fn unproject_physics(&mut self, physics: &PointPhysics) {
        self.ball.x = physics.positions[0].x;
        self.ball.y = physics.positions[0].y;
        self.ball_vel = physics.velocities[0];
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID>,
        control: &KeyboardControl<ActionID, MacroquadInputWrapper>,
    ) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            self.ball,
            Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32),
            self.ball_vel,
            true,
            false,
            CollisionID::Ball,
        );

        let paddle_size = Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32);
        collision.add_entity_as_xywh(
            Vec2::new(self.paddle as f32, PADDLE_OFF_Y as f32),
            paddle_size,
            Vec2::new(0.0, control.values[0][1].value - control.values[0][0].value),
            true,
            true,
            CollisionID::Paddle,
        );

        for (i, row) in self.blocks.iter().enumerate() {
            for (j, Block { visible, .. }) in row.iter().enumerate() {
                if *visible {
                    collision.add_entity_as_xywh(
                        Vec2::new(
                            j as f32 * BLOCK_WIDTH as f32,
                            i as f32 * BLOCK_HEIGHT as f32,
                        ),
                        Vec2::new(BLOCK_WIDTH as f32, BLOCK_HEIGHT as f32),
                        Vec2::ZERO,
                        true,
                        true,
                        CollisionID::Block(i, j),
                    );
                }
            }
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.ball = collision.centers[4] - collision.half_sizes[4];
    }

    fn _change_angle(&mut self) {
        let paddle_center = (self.paddle + PADDLE_WIDTH / 2) as f32;
        let angle: f32 = (((self.ball.x + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-((PADDLE_WIDTH / 2) as f32))
            .min((PADDLE_WIDTH / 2) as f32)
            / (PADDLE_WIDTH / 2) as f32)
            .abs()
            * 70.0;
        let magnitude = f32::sqrt(self.ball_vel.x.powi(2) + self.ball_vel.y.powi(2));
        self.ball_vel.x =
            angle.to_radians().sin() * magnitude * if self.ball_vel.x < 0.0 { -1.0 } else { 1.0 };
        self.ball_vel.y =
            angle.to_radians().cos() * magnitude * if self.ball_vel.y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID, u8>) {
        if !resources.items.contains_key(&PoolID::Points) {
            resources
                .items
                .insert(PoolID::Points, (0, u8::MIN, u8::MAX));
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID, u8>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_type) => {
                    let value = resources.get_value_by_itemtype(item_type).unwrap() as u8;
                    match item_type {
                        PoolID::Points => self.score = value,
                    }
                }
                Err(_) => {}
            }
        }
    }

    fn reset(&mut self) {
        self.ball = Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, (HEIGHT / 4 * 3) as f32);
        self.ball_vel = Vec2::new(0.0, 0.0);
        self.score = 0;
        self.blocks = [
            [Block::new(BLUE, true); 8],
            [Block::new(PINK, true); 8],
            [Block::new(WHITE, true); 8],
            [Block::new(PINK, true); 8],
            [Block::new(BLUE, true); 8],
        ];
    }

    fn draw(&self) {
        clear_background(Color::new(0., 0., 0.5, 1.));
        draw_rectangle(
            self.paddle as f32,
            PADDLE_OFF_Y as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            self.ball.x,
            self.ball.y,
            BALL_SIZE as f32,
            BALL_SIZE as f32,
            Color::new(1., 0.75, 0., 1.),
        );

        for (i, row) in self.blocks.iter().enumerate() {
            for (j, Block { color, visible }) in row.iter().enumerate() {
                if *visible {
                    draw_rectangle(
                        j as f32 * BLOCK_WIDTH as f32,
                        i as f32 * BLOCK_HEIGHT as f32,
                        BLOCK_WIDTH as f32,
                        BLOCK_HEIGHT as f32,
                        *color,
                    );
                }
            }
        }
    }
}

impl Block {
    fn new(color: Color, visible: bool) -> Self {
        Self { color, visible }
    }
}
