use crate::Logics;

use macroquad::{input::KeyCode, math::Vec2};

/// generates identifier structs (i got tired of typing all of them out). example: `id_impl_new!([derive(PartialOrd, Ord)] ScoreID)` expands out to
///
/// ```
/// #[derive(PartialOrd, Ord)]
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// pub struct ScoreID(usize);
/// impl ScoreID {
///     pub fn new(idx: usize) -> Self {
///         Self(idx)
///     }
///     pub fn idx(&self) -> usize {
///         self.0
///     }
/// }
/// ```
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

id_impl_new!([] PaddleID, [] WallID, [] BallID, [derive(PartialOrd, Ord, Debug)] ScoreID, [derive(PartialOrd, Ord, Debug)] ActionID, [derive(Hash, Debug)] UserQueryID);

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum QueryType {
    CtrlEvent,
    CtrlIdent,
    ColEvent,
    ColIdent,
    PhysEvent,
    PhysIdent,
    RsrcEvent,
    RsrcIdent,
    BallCol,
    User(UserQueryID),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Paddle,
    Wall,
    Ball,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum RsrcPool {
    Score(ScoreID),
}

#[derive(Default)]
pub struct Paddle {
    pub pos: Vec2,
    pub size: Vec2,
    pub controls: Vec<(ActionID, KeyCode, bool)>,
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

    pub fn add_control_map(&mut self, keycode: KeyCode, valid: bool) -> ActionID {
        let act_id = ActionID(self.controls.len());
        self.controls.push((act_id, keycode, valid));
        act_id
    }
}

#[derive(Default)]
pub struct Ball {
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
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
    pub pos: Vec2,
    pub size: Vec2,
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
    pub value: u16,
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
    pub fn consume_paddle(&mut self, id: PaddleID, col_idx: usize, paddle: Paddle) {
        let hs = paddle.size / 2.0;
        let center = paddle.pos + hs;
        self.collision.centers.insert(col_idx, center);
        self.collision.half_sizes.insert(col_idx, hs);
        self.collision.velocities.insert(col_idx, Vec2::ZERO);

        use asterism::collision::CollisionData;
        self.collision.metadata.insert(
            col_idx,
            CollisionData {
                solid: true,
                fixed: true,
                id: CollisionEnt::Paddle,
            },
        );

        for (act_id, keycode, valid) in paddle.controls {
            self.control.add_key_map(id.0, keycode, act_id, valid);
        }
    }

    pub fn consume_wall(&mut self, col_idx: usize, wall: Wall) {
        let hs = wall.size / 2.0;
        let center = wall.pos + hs;
        self.collision.centers.insert(col_idx, center);
        self.collision.half_sizes.insert(col_idx, hs);
        self.collision.velocities.insert(col_idx, Vec2::ZERO);

        use asterism::collision::CollisionData;
        self.collision.metadata.insert(
            col_idx,
            CollisionData {
                solid: true,
                fixed: true,
                id: CollisionEnt::Wall,
            },
        );
    }

    pub fn consume_ball(&mut self, col_idx: usize, ball: Ball) {
        self.physics
            .add_physics_entity(ball.pos, ball.vel, Vec2::ZERO);
        let hs = ball.size / 2.0;
        let center = ball.pos + hs;
        self.collision.centers.insert(col_idx, center);
        self.collision.half_sizes.insert(col_idx, hs);
        self.collision.velocities.insert(col_idx, Vec2::ZERO);

        use asterism::collision::CollisionData;
        self.collision.metadata.insert(
            col_idx,
            CollisionData {
                solid: true,
                fixed: false,
                id: CollisionEnt::Ball,
            },
        );
    }

    pub fn consume_score(&mut self, id: ScoreID, score: Score) {
        self.resources
            .items
            .insert(RsrcPool::Score(id), (score.value, Score::MIN, Score::MAX));
    }
}

use asterism::collision::CollisionEvent;
use asterism::control::ControlEvent;

pub type CtrlEvent = ControlEvent<ActionID>;
pub type CtrlIdent = (usize, Vec<asterism::control::Action<ActionID, KeyCode>>);
pub type ColEvent = CollisionEvent;
pub type ColIdent = (usize, asterism::collision::AabbColData<CollisionEnt>);
pub type RsrcIdent = (RsrcPool, (u16, u16, u16));
pub type RsrcEvent = asterism::resources::ResourceEvent<RsrcPool>;
pub type PhysIdent = (usize, asterism::physics::PointPhysData);
pub type PhysEvent = asterism::physics::PhysicsEvent;
