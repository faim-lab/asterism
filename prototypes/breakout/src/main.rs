use asterism::{
    collision::AabbCollision, control::KeyboardControl, control::MacroQuadKeyboardControl,
    physics::PointPhysics, resources::QueuedResources, resources::Transaction,
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

impl Default for ActionID {
    fn default() -> Self {
        Self::Serve
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum CollisionID {
    Paddle,
    Ball,
    End,
    Wall(Side),
    Block(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum Side {
    Top,
    Left,
    Right,
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Ball
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points,
}

#[derive(Clone, Copy)]
struct Block {
    color: Color,
    visible: bool,
}

struct Logics {
    control: MacroQuadKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
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
        if let Ok(cont) = world.update(&mut logics) {
            if !cont {
                break;
            }
        }
        world.draw();
        next_frame().await;
    }
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = MacroQuadKeyboardControl::new();
                control.add_key_map(0, KeyCode::Right, ActionID::MoveRight);
                control.add_key_map(0, KeyCode::Left, ActionID::MoveLeft);
                control.add_key_map(0, KeyCode::Space, ActionID::Serve);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit);
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
                    CollisionID::Wall(Side::Left),
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
                    CollisionID::Wall(Side::Right),
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
                    CollisionID::Wall(Side::Top),
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
                    CollisionID::End,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points, 0.0);
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

    fn update(&mut self, logics: &mut Logics) -> Result<bool, ()> {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        if logics.control.values[0][3].value != 0.0 {
            return Ok(false);
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        let mut block_broken = false;
        for (idx, contact) in logics.collision.contacts.iter().enumerate() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Ball, CollisionID::End) => {
                    println!("\ngame over");
                    self.reset();
                    // i don't know if i like this
                    logics.resources.items.clear();
                }
                (CollisionID::Ball, CollisionID::Paddle) => {
                    let sides_touched = logics.collision.sides_touched(idx);
                    self.ball_vel.set_y(-self.ball_vel.y());
                    if sides_touched.y() < 0.0 {
                        self.change_angle();
                    } else if sides_touched.x() != 0.0 {
                        self.ball_vel.set_x(-self.ball_vel.x());
                    }
                    self.ball_vel *= 1.1;
                }
                (CollisionID::Ball, CollisionID::Wall(side)) => match side {
                    Side::Right | Side::Left => {
                        self.ball_vel.set_x(-self.ball_vel.x());
                    }
                    Side::Top => {
                        self.ball_vel.set_y(-self.ball_vel.y());
                    }
                },
                (CollisionID::Ball, CollisionID::Block(i, j)) => {
                    if !block_broken {
                        let sides_touched = logics.collision.sides_touched(idx);
                        if sides_touched.x() != 0.0 {
                            self.ball_vel.set_x(-self.ball_vel.x());
                        } else if sides_touched.y() != 0.0 {
                            self.ball_vel.set_y(-self.ball_vel.y());
                        }
                        self.blocks[i][j].visible = false;
                        logics
                            .resources
                            .transactions
                            .push(vec![(PoolID::Points, Transaction::Change(1.0))]);
                        block_broken = true;
                    }
                }
                _ => {}
            }
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for (completed, item_types) in logics.resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    match item_type {
                        PoolID::Points => {
                            print!("current score: {}\r", self.score);
                            io::stdout().flush().unwrap();
                            if self.score >= 40 {
                                println!("\nyou win!");
                                self.reset();
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn project_control(&self, control: &mut MacroQuadKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        if self.ball_vel.x() == 0.0 && self.ball_vel.y() == 0.0 {
            control.mapping[0][2].is_valid = true;
        }
        control.mapping[0][3].is_valid = true;
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.paddle = ((self.paddle as i16 + control.values[0][0].value as i16 * 2
            - control.values[0][1].value as i16 * 2)
            .max(0) as u8)
            .min(WIDTH - PADDLE_WIDTH);

        if self.ball_vel.x() == 0.0 && self.ball_vel.y() == 0.0 {
            let values = &control.values[0][2];
            if values.changed_by == 1.0 && values.value != 0.0 {
                self.ball_vel = Vec2::new(1.0, 1.0);
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        physics.add_physics_entity(self.ball, self.ball_vel, Vec2::new(0.0, 0.0));
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        self.ball.set_x(
            physics.positions[0]
                .x()
                .max(0.0)
                .min((WIDTH - BALL_SIZE) as f32),
        );
        self.ball.set_y(
            physics.positions[0]
                .y()
                .max(0.0)
                .min((HEIGHT - BALL_SIZE) as f32),
        );
        self.ball_vel = physics.velocities[0];
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID, Vec2>,
        control: &MacroQuadKeyboardControl<ActionID>,
    ) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            self.ball.x(),
            self.ball.y(),
            BALL_SIZE as f32,
            BALL_SIZE as f32,
            self.ball_vel,
            true,
            false,
            CollisionID::Ball,
        );

        collision.add_entity_as_xywh(
            self.paddle as f32,
            PADDLE_OFF_Y as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0, control.values[0][1].value - control.values[0][0].value),
            true,
            true,
            CollisionID::Paddle,
        );

        for (i, row) in self.blocks.iter().enumerate() {
            for (j, Block { visible, .. }) in row.iter().enumerate() {
                if *visible {
                    collision.add_entity_as_xywh(
                        j as f32 * BLOCK_WIDTH as f32,
                        i as f32 * BLOCK_HEIGHT as f32,
                        BLOCK_WIDTH as f32,
                        BLOCK_HEIGHT as f32,
                        Vec2::new(0.0, 0.0),
                        true,
                        true,
                        CollisionID::Block(i, j),
                    );
                }
            }
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        self.ball
            .set_x(collision.centers[4].x() - collision.half_sizes[4].x());
        self.ball
            .set_y(collision.centers[4].y() - collision.half_sizes[4].y());
    }

    fn change_angle(&mut self) {
        let paddle_center = (self.paddle + PADDLE_WIDTH / 2) as f32;
        let angle: f32 = (((self.ball.x() + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-((PADDLE_WIDTH / 2) as f32))
            .min((PADDLE_WIDTH / 2) as f32)
            / (PADDLE_WIDTH / 2) as f32)
            .abs()
            * 70.0;
        let magnitude = f32::sqrt(self.ball_vel.x().powi(2) + self.ball_vel.y().powi(2));
        self.ball_vel.set_x(
            angle.to_radians().sin() * magnitude * if self.ball_vel.x() < 0.0 { -1.0 } else { 1.0 },
        );
        self.ball_vel.set_y(
            angle.to_radians().cos() * magnitude * if self.ball_vel.y() < 0.0 { -1.0 } else { 1.0 },
        );
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points) {
            resources.items.insert(PoolID::Points, 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    let value = resources.get_value_by_itemtype(item_type) as u8;
                    match item_type {
                        PoolID::Points => self.score = value,
                    }
                }
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
            self.ball.x(),
            self.ball.y(),
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
