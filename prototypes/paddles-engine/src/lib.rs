// i keep trying to write an ecs. help

#![allow(unused)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    collision::AabbCollision,
    control::{Action, InputType, KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::QueuedResources,
    Event, Reaction,
};
use macroquad::prelude::*;

pub trait FromActionID: Into<usize> {}
pub trait FromCollisionID: Into<usize> {}
pub trait FromPoolID: Into<usize> {}

#[derive(Default)]
pub struct Game {
    paddles: Vec<Paddle>,
    balls: Vec<Ball>,
    walls: Vec<Wall>,
    scores: Vec<Score>,
}

struct Logics {
    collision: AabbCollision<usize>,
    physics: PointPhysics,
    resources: QueuedResources<usize, u8>,
    control: KeyboardControl<usize, MacroquadInputWrapper>,
}

impl Logics {
    fn new() -> Self {
        Self {
            collision: AabbCollision::new(),
            physics: PointPhysics::new(),
            resources: QueuedResources::new(),
            control: KeyboardControl::new(),
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&mut self) {
        let mut logics = Logics::new();
        loop {
            // update
            // draw
            next_frame().await;
        }
    }

    // fns for builder pattern
    pub fn with_paddle(mut self, paddle: Paddle) -> Self {
        self.paddles.push(paddle);
        self
    }

    pub fn with_ball(mut self, ball: Ball) -> Self {
        self.balls.push(ball);
        self
    }

    pub fn with_wall(mut self, wall: Wall) -> Self {
        self.walls.push(wall);
        self
    }
}

#[derive(Default)]
pub struct Paddle {
    pos: Vec2,
    size: Vec2,
    controls: Vec<Action<usize, KeyCode>>,
    on_key_event: Vec<(usize, Box<dyn Reaction>)>,
    on_collide: Vec<Box<dyn Reaction>>,
}

impl Paddle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_control_map(mut self, action: impl FromActionID, keycode: KeyCode) -> Self {
        self.controls
            .push(Action::new(action.into(), keycode, InputType::Digital));
        self
    }

    pub fn with_collision_react(mut self, on_collide: Box<dyn Reaction>) -> Self {
        self.on_collide.push(on_collide);
        self
    }
}

#[derive(Default)]
pub struct Ball {
    pos: Vec2,
    size: Vec2,
    vel: Vec2,
    on_collide: Vec<Box<dyn Reaction>>,
}

impl Ball {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_vel(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_collision_react(mut self, on_collide: Box<dyn Reaction>) -> Self {
        self.on_collide.push(on_collide);
        self
    }
}

#[derive(Default)]
pub struct Wall {
    pos: Vec2,
    size: Vec2,
    on_collide: Vec<Box<dyn Reaction>>,
}

impl Wall {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_collision_react(mut self, on_collide: Box<dyn Reaction>) -> Self {
        self.on_collide.push(on_collide);
        self
    }
}

#[derive(Default)]
struct Score {
    value: u16,
    on_increase: Vec<Box<dyn Reaction>>,
}

impl Score {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(mut self, value: u16) -> Self {
        self.value = value;
        self
    }

    pub fn with_increase_react(mut self, on_increase: Box<dyn Reaction>) -> Self {
        self.on_increase.push(on_increase);
        self
    }
}
