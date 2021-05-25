use crate::{Game, Logics};
use asterism::control::{Action, InputType};
use asterism::Reaction;

use macroquad::{input::KeyCode, math::Vec2};

pub type Reactions = Vec<Box<dyn Reaction>>;

/* generates identifier structs (i got tired of typing all of them out). ex:

id_impl_new!([derive(PartialOrd, Ord)] ScoreID) expands out to

#[derive(PartialOrd, Ord)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScoreID(usize);
impl ScoreID {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }
    pub fn idx(&self) -> usize {
        self.0
    }
} */

macro_rules! id_impl_new {
    ($([$($derive:meta)*] $id_type:ident),*) => {
        $(
            $(#[$derive])*
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct $id_type(usize);

            impl $id_type {
                pub fn new(idx: usize) -> Self {
                    Self(idx)
                }

                pub fn idx(&self) -> usize {
                    self.0
                }
            }
        )*
    };
}

id_impl_new!([] PaddleID, [] WallID, [] BallID, [derive(PartialOrd, Ord)] ScoreID, [] ActionID);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Paddle(PaddleID),
    Wall(WallID),
    Ball(BallID),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RsrcPool {
    Score(ScoreID),
}

#[derive(Default)]
pub struct Paddle {
    pos: Vec2,
    size: Vec2,
    controls: Vec<Action<usize, KeyCode>>,
}

impl Paddle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn add_control_map(&mut self, keycode: KeyCode) -> ActionID {
        let num_controls = self.controls.len();
        self.controls
            .push(Action::new(num_controls, keycode, InputType::Digital));
        ActionID(num_controls)
    }
}

#[derive(Default)]
pub struct Ball {
    pos: Vec2,
    size: Vec2,
    vel: Vec2,
}

impl Ball {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_vel(&mut self, vel: Vec2) {
        self.vel = vel;
    }
}

#[derive(Default)]
pub struct Wall {
    pos: Vec2,
    size: Vec2,
}

impl Wall {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }
}

#[derive(Default)]
pub struct Score {
    value: u16,
}

impl Score {
    const MIN: u16 = 0;
    const MAX: u16 = u16::MAX;
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

impl Logics {
    pub fn consume_paddle(&mut self, id: PaddleID, paddle: Paddle) {
        self.collision.add_entity_as_xywh(
            paddle.pos,
            paddle.size,
            Vec2::ZERO,
            true,
            false,
            CollisionEnt::Paddle(id),
        );
        self.control.mapping.push(paddle.controls);
    }

    pub fn consume_wall(&mut self, id: WallID, wall: Wall) {
        self.collision.add_entity_as_xywh(
            wall.pos,
            wall.size,
            Vec2::ZERO,
            true,
            false,
            CollisionEnt::Wall(id),
        );
    }

    pub fn consume_ball(&mut self, id: BallID, ball: Ball) {
        self.physics
            .add_physics_entity(ball.pos, ball.vel, Vec2::ZERO);
        self.collision.add_entity_as_xywh(
            ball.pos,
            ball.size,
            Vec2::ZERO,
            true,
            false,
            CollisionEnt::Ball(id),
        );
    }

    pub fn consume_score(&mut self, id: ScoreID, score: Score) {
        self.resources
            .items
            .insert(RsrcPool::Score(id), (score.value, Score::MIN, Score::MAX));
    }
}
