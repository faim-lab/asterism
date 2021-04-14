#![deny(clippy::all)]
#![forbid(unsafe_code)]
use asterism::{
    collision::AabbCollision,
    control::{KeyboardControl, MacroQuadKeyboardControl},
    entity_state::FlatEntityState,
    physics::PointPhysics,
    resources::{PoolInfo, QueuedResources, Transaction},
};
use json::*;
use macroquad::prelude::*;
use serde;
use serde::Deserialize;
use serde_json;
use std::fs::File;

const WIDTH: u32 = 255;
const HEIGHT: u32 = 255;
const PADDLE_OFF_Y: u32 = 36;
const PADDLE_HEIGHT: u32 = 36;
const PADDLE_WIDTH: u32 = 36;
const BALL_SIZE: u32 = 36;
const BALL_NUM: u8 = 4;
const WALL_DEPTH: u32 = 8;
const GOAL_WIDTH: u32 = 72;

const BASE_COLOR: Color = Color::new(0.86, 1., 0.86, 1.);
const FENCE_COLOR: Color = Color::new(0.94, 0.9, 0.54, 1.);

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveRight(Player),
    MoveLeft(Player),
    MoveUp(Player),
    MoveDown(Player),
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum StateID {
    PlayerRunning1,
    PlayerRunning2,
    PlayerResting,
    BallRunning1,
    BallRunning2,
    BallResting,
    Scored,
    //^^makes non visible, might want to remove but then need to find way to not mess up displays
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveLeft(Player::P1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum CollisionID {
    Paddle(Player),
    Ball(usize),
    InertWall,
    Goal(Player),
}

#[derive(Clone, Copy)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Paddle(Player::P1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player),
}
impl PoolInfo for PoolID {
    fn max_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MAX as f64,
        }
    }
    fn min_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MIN as f64,
        }
    }
}

struct SpriteSheet {
    image: Texture2D,
    data: Vec<Sprite>,
}

impl SpriteSheet {
    async fn new(image_file: &str, data_file: Vec<Sprite>) -> Self {
        Self {
            image: load_texture(image_file).await,
            data: data_file,
        }
    }

    fn create_param(&self, index: usize) -> DrawTextureParams {
        let mut texture = DrawTextureParams::default();
        texture.dest_size = Some(Vec2::new(
            self.data[index].source_size.w as f32,
            self.data[index].source_size.h as f32,
        ));
        texture.source = Some(Rect::new(
            self.data[index].frame.x as f32,
            self.data[index].frame.y as f32,
            self.data[index].frame.w as f32,
            self.data[index].frame.h as f32,
        ));

        return texture;
    }
}

#[derive(Debug, Deserialize)]
struct Rectangle {
    x: u64,
    y: u64,
    w: u64,
    h: u64,
}

#[derive(Debug, Deserialize)]
struct Size {
    w: u64,
    h: u64,
}

#[derive(Debug, Deserialize)]
struct Sprite {
    name: String,
    frame: Rectangle,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: Rectangle,
    source_size: Size,
}

struct Animation {
    sheet: SpriteSheet,
    frames_drawn: u8,
}

impl Animation {
    fn new(sprite_sheet: SpriteSheet) -> Self {
        Self {
            sheet: sprite_sheet,
            frames_drawn: 0,
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "clowder".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

struct Logics {
    control: MacroQuadKeyboardControl<ActionID>,
    entity_state: FlatEntityState<StateID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = MacroQuadKeyboardControl::new();
                control.add_key_map(0, KeyCode::J, ActionID::MoveRight(Player::P1));
                control.add_key_map(0, KeyCode::L, ActionID::MoveLeft(Player::P1));
                control.add_key_map(0, KeyCode::I, ActionID::MoveUp(Player::P1));
                control.add_key_map(0, KeyCode::K, ActionID::MoveDown(Player::P1));
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.add_entity_as_xywh(
                    0.0,
                    (HEIGHT - WALL_DEPTH) as f32, //bottom wall
                    WIDTH as f32,
                    WALL_DEPTH as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::InertWall,
                );

                collision.add_entity_as_xywh(
                    0.0, //left wall
                    0.0,
                    WALL_DEPTH as f32,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::InertWall,
                );
                collision.add_entity_as_xywh(
                    0.0, //top 1
                    0.0,
                    ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
                    WALL_DEPTH as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::InertWall,
                );
                collision.add_entity_as_xywh(
                    ((WIDTH / 2) + (GOAL_WIDTH / 2)) as f32, //top 2
                    WALL_DEPTH as f32,
                    ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
                    0.0,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::InertWall,
                );
                collision.add_entity_as_xywh(
                    0.0, //goal
                    0.0,
                    WIDTH as f32,
                    0.0,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::Goal(Player::P1),
                );
                collision.add_entity_as_xywh(
                    (WIDTH - WALL_DEPTH) as f32, //right wall
                    0.0,
                    WALL_DEPTH as f32,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::InertWall,
                );
                collision
            },

            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points(Player::P1), 0.0);
                resources
            },
            entity_state: {
                let mut entity_state = FlatEntityState::new();

                for _i in 0..BALL_NUM {
                    entity_state.add_state_map(
                        2,
                        vec![
                            (StateID::BallRunning1, vec![1, 2, 3]),
                            (StateID::BallRunning2, vec![0, 2, 3]),
                            (StateID::BallResting, vec![1]),
                            (StateID::Scored, vec![0]),
                        ],
                    );
                }

                entity_state.add_state_map(
                    2,
                    vec![
                        (StateID::PlayerRunning1, vec![1, 2]),
                        (StateID::PlayerRunning2, vec![0]),
                        (StateID::PlayerResting, vec![1]),
                    ],
                );
                entity_state
            },
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
enum Player {
    P1,
}

struct World {
    paddles: (Vec2, Vec2),
    balls: Vec<Ball>,
    ball_err: Vec2,
    score: (u8, u8),
    time: f64,
    interval: f64,
}

#[macroquad::main(window_conf)]
async fn main() {
    let file = File::open("src/clowder_sprite.json").unwrap();
    let sprite_info: Vec<Sprite> =
        serde_json::from_reader(file).expect("error while reading or parsing");
    let sprites = SpriteSheet::new("src/clowder_sprite.png", sprite_info).await;

    let mut animation = Animation::new(sprites);
    let mut world = World::new();
    let mut logics = Logics::new();

    for _i in 0..BALL_NUM {
        world.balls.push(Ball::new(Vec2::new(
            rand::gen_range(BALL_SIZE as f32, (WIDTH - BALL_SIZE) as f32),
            rand::gen_range((BALL_SIZE * 2) as f32, (HEIGHT - BALL_SIZE) as f32),
        )));
    }

    loop {
        if let Ok(cont) = world.update(&mut logics) {
            if !cont {
                break;
            }
        }
        world.draw(&mut logics.entity_state, &mut animation);
        //world.draw(&mut sprites);
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (
                Vec2::new((WIDTH / 2 - PADDLE_WIDTH / 2) as f32, PADDLE_OFF_Y as f32),
                Vec2::new(
                    (WIDTH / 2 - PADDLE_WIDTH / 2) as f32,
                    (HEIGHT - PADDLE_OFF_Y - PADDLE_HEIGHT) as f32,
                ),
            ),
            balls: Vec::new(),
            ball_err: Vec2::new(0.0, 0.0),
            score: (0, 0),
            time: 0.0,
            interval: 1.0,
        }
    }

    fn update(&mut self, logics: &mut Logics) -> Result<bool> {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &mut logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        self.project_entity_state(&mut logics.entity_state, &logics.collision);
        logics.entity_state.update();
        self.unproject_entity_state(&logics.entity_state);

        if logics.control.values[0][4].value != 0.0 {
            return Ok(false);
        }

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Goal(_player), CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::Goal(_player)) => {
                    if i <= self.balls.len() {
                        // self.balls.remove(i);
                        logics
                            .resources
                            .transactions
                            .push(vec![(PoolID::Points(Player::P1), Transaction::Change(1.0))]);
                    }
                }

                (CollisionID::InertWall, CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::InertWall) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));

                    if self.balls[i].vel.x != 0.0 {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x = sides_touched.x * -1.0;
                        } else if sides_touched.y != 0.0 {
                            self.balls[i].vel.x = sides_touched.y * -1.0;
                        }
                    }

                    if self.balls[i].vel.y == 0.0 {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.y = sides_touched.x * -1.0;
                        } else if sides_touched.y != 0.0 {
                            self.balls[i].vel.y = sides_touched.y * -1.0;
                        }
                    } else {
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                        }
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                        }
                    }
                }

                (CollisionID::Ball(i), CollisionID::Ball(j)) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));

                    if (self.balls[i].vel.x, self.balls[i].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[i].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else if (self.balls[j].vel.x, self.balls[j].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[j].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                            self.balls[j].vel.x *= -1.0;
                        }
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                            self.balls[j].vel.y *= -1.0;
                        }
                    }
                }

                (CollisionID::Ball(i), CollisionID::Paddle(player))
                | (CollisionID::Paddle(player), CollisionID::Ball(i)) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));

                    if (self.balls[i].vel.x, self.balls[i].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[i].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else {
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                        }
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                        }
                    }

                    self.change_angle(player, i);
                }
                _ => {}
            }
        }

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
                                }
                                println!(" scores! p1: {}", self.score.0);
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
        Ok(true)
    }

    fn project_control(&self, control: &mut MacroQuadKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[0][2].is_valid = true;
        control.mapping[0][3].is_valid = true;
        control.mapping[0][4].is_valid = true;
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.paddles.0.x = ((self.paddles.0.x - control.values[0][0].value as f32
            + control.values[0][1].value as f32) //confusing, incorporate ActionIds
            .max(0.0) as f32)
            .min((255 - PADDLE_WIDTH) as f32); //drive with data not code
        self.paddles.0.y = ((self.paddles.0.y as f32 - control.values[0][2].value as f32
            + control.values[0][3].value as f32)
            .max(0.0) as f32)
            .min((255 - PADDLE_HEIGHT) as f32);
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        for ball in self.balls.iter() {
            physics.add_physics_entity(ball.pos, ball.vel, Vec2::new(0.0, 0.0));
        }
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        for ((ball, pos), vel) in self
            .balls
            .iter_mut()
            .zip(physics.positions.iter())
            .zip(physics.velocities.iter())
        {
            ball.pos = *pos;
            ball.vel = *vel;
        }
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID, Vec2>,
        control: &mut MacroQuadKeyboardControl<ActionID>,
    ) {
        collision.centers.resize_with(6, Default::default);
        collision.half_sizes.resize_with(6, Default::default);
        collision.velocities.resize_with(6, Default::default);
        collision.metadata.resize_with(6, Default::default);

        collision.add_entity_as_xywh(
            self.paddles.0.x as f32,
            self.paddles.0.y as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(
                -control.values[0][0].value + control.values[0][1].value,
                -control.values[0][2].value + control.values[0][3].value,
            ),
            true,
            true,
            CollisionID::Paddle(Player::P1),
        );

        for (i, ball) in self.balls.iter().enumerate() {
            collision.add_entity_as_xywh(
                ball.pos.x as f32,
                ball.pos.y as f32,
                BALL_SIZE as f32,
                BALL_SIZE as f32,
                ball.vel,
                true,
                false,
                CollisionID::Ball(i),
            );
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        for (i, ball) in self.balls.iter_mut().enumerate() {
            ball.pos.x = (collision.centers[i + 7].x - collision.half_sizes[i + 7].x).trunc();
            ball.pos.y = (collision.centers[i + 7].y - collision.half_sizes[i + 7].y).trunc();
        }
    }

    fn change_angle(&mut self, player: Player, ball_index: usize) {
        let ball = &mut self.balls[ball_index];
        let Vec2 { x, y } = &mut ball.vel;
        let paddle_center = match player {
            Player::P1 => self.paddles.0.x + (PADDLE_WIDTH / 2) as f32,
        } as f32;
        let angle: f32 = ((ball.pos.x + (BALL_SIZE / 2) as f32 - paddle_center)
            .max(-(PADDLE_WIDTH as f32) / 2.0)
            .min(PADDLE_WIDTH as f32 / 2.0)
            / PADDLE_WIDTH as f32)
            .abs()
            * 80.0;
        let magnitude = f32::sqrt(*x * *x + *y * *y);
        *x = angle.to_radians().sin() * magnitude * if *x < 0.0 { -1.0 } else { 1.0 };
        *y = angle.to_radians().cos() * magnitude * if *y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_entity_state(
        &self,
        entity_state: &mut FlatEntityState<StateID>,
        collision: &AabbCollision<CollisionID, Vec2>,
    ) {
        if self.paddles.1.x != 0.0 || self.paddles.1.y != 0.0 {
            //running
            entity_state.conditions[BALL_NUM as usize][1] = true;
        } else {
            //resting
            entity_state.conditions[BALL_NUM as usize][0] = true;
        }

        for (i, ball) in self.balls.iter().enumerate() {
            if ball.vel.y != 0.0 || ball.vel.x != 0.0 {
                //running
                entity_state.conditions[i as usize][1] = true;
            } else {
                //resting
                entity_state.conditions[i as usize][0] = true;
            }
        }

        for contact in collision.contacts.iter() {
            match (
                collision.metadata[contact.i].id,
                collision.metadata[contact.j].id,
            ) {
                (CollisionID::Ball(i), CollisionID::Goal(_player)) => {
                    entity_state.conditions[i][2] = true;
                }

                _ => {}
            }
        }
    }

    fn unproject_entity_state(&mut self, entity_state: &FlatEntityState<StateID>) {
        for (i, state) in entity_state.states.iter().enumerate() {
            match entity_state.maps[i].states[*state].id {
                StateID::PlayerRunning1 | StateID::PlayerRunning2 | StateID::PlayerResting => {}

                StateID::BallResting
                | StateID::BallRunning1
                | StateID::BallRunning2
                | StateID::Scored => {}
            }
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points(Player::P1)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        let value = resources.get_value_by_itemtype(item_type).unwrap();
                        match item_type {
                            PoolID::Points(player) => match player {
                                Player::P1 => self.score.0 = value as u8,
                            },
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, state: &mut FlatEntityState<StateID>, animation: &mut Animation) {
        //arbitrary value that resets frames to prevent overflow

        if animation.frames_drawn >= 4 {
            animation.frames_drawn = 0;
        } else {
            animation.frames_drawn = animation.frames_drawn + 1;
        }

        clear_background(BASE_COLOR);

        if state.conditions[BALL_NUM as usize][0] || state.conditions[BALL_NUM as usize][2] {
            draw_texture_ex(
                animation.sheet.image,
                self.paddles.0.x as f32,
                self.paddles.0.y as f32,
                WHITE,
                animation.sheet.create_param(8),
            );

            if state.conditions[BALL_NUM as usize][0] && animation.frames_drawn == 0 {
                state.conditions[BALL_NUM as usize][1] = true;
            }
        } else {
            draw_texture_ex(
                animation.sheet.image,
                self.paddles.0.x as f32,
                self.paddles.0.y as f32,
                WHITE,
                animation.sheet.create_param(9),
            );

            if animation.frames_drawn == 0 {
                state.conditions[BALL_NUM as usize][1] = true;
            }
        }

        //top 1 (left)
        draw_rectangle(
            0.0,
            0.0,
            ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //top 2 (right)
        draw_rectangle(
            ((WIDTH / 2) + (GOAL_WIDTH / 2)) as f32,
            0.0,
            ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //left wall
        draw_rectangle(0.0, 0.0, WALL_DEPTH as f32, HEIGHT as f32, FENCE_COLOR);

        //right wall
        draw_rectangle(
            (WIDTH - WALL_DEPTH) as f32,
            0.0,
            WALL_DEPTH as f32,
            HEIGHT as f32,
            FENCE_COLOR,
        );

        //bottom wall
        draw_rectangle(
            0.0,
            (HEIGHT - WALL_DEPTH) as f32,
            WIDTH as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //balls
        for (i, ball) in self.balls.iter().enumerate() {
            if state.conditions[i][0] || state.conditions[i][2] {
                draw_texture_ex(
                    animation.sheet.image,
                    ball.pos.x as f32,
                    ball.pos.y as f32,
                    WHITE,
                    animation.sheet.create_param(i),
                );

                if state.conditions[i as usize][0] && animation.frames_drawn == 0 {
                    state.conditions[i as usize][1] = true;
                }
            } else if state.conditions[i][1] {
                draw_texture_ex(
                    animation.sheet.image,
                    ball.pos.x as f32,
                    ball.pos.y as f32,
                    WHITE,
                    animation.sheet.create_param(i + 1),
                );

                if animation.frames_drawn == 0 {
                    state.conditions[i as usize][0] = true;
                }
            } else {
            }
        }
    }
}

impl Ball {
    fn new(newpos: Vec2) -> Self {
        Self {
            pos: newpos,
            vel: Vec2::new(0.0, 0.0),
        }
    }
}
