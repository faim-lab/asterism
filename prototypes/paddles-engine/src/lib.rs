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

pub use asterism::collision::{CollisionEvent, CollisionReaction};
pub use asterism::control::{ControlEvent, ControlEventType, ControlReaction};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction};
pub use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction};

#[derive(Clone, Copy)]
pub struct PaddleID(usize);
#[derive(Clone, Copy)]
pub struct WallID(usize);
#[derive(Clone, Copy)]
pub struct BallID(usize);
#[derive(Clone, Copy)]
pub struct ScoreID(usize);

pub enum CollisionEnt {
    Paddle(PaddleID),
    Wall(WallID),
    Ball(BallID),
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionID(usize);

type Reactions = Vec<Box<dyn Reaction>>;

struct Logics {
    collision: AabbCollision<usize>,
    physics: PointPhysics,
    // i'm sorry but if you play pong up to 255 points you're a sad person
    resources: QueuedResources<usize, u8>,
    control: KeyboardControl<usize, MacroquadInputWrapper>,
}

#[derive(Default)]
struct Events {
    control: Vec<(ControlEvent<usize>, Reactions)>,
    collision: Vec<(CollisionEvent<usize>, Reactions)>,
    resources: Vec<(ResourceEvent<usize>, Reactions)>,
    // physics: similar except idk what a physics event is
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

#[derive(Default)]
pub struct Game {
    paddles: Vec<Paddle>,
    balls: Vec<Ball>,
    walls: Vec<Wall>,
    scores: Vec<Score>,
    events: Events,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_ctrl_event(
        &mut self,
        paddle: PaddleID,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Reaction>,
    ) {
        let key_event = ControlEvent {
            event_type: key_event,
            action_id: action.0,
            set: paddle.0,
        };
        if let Some(i) = self
            .events
            .control
            .iter()
            .position(|(event, _)| event == &key_event)
        {
            self.events.control[i].1.push(on_key_event);
        } else {
            self.events.control.push((key_event, vec![on_key_event]));
        }
    }

    pub fn add_collision_react(
        &mut self,
        col1: CollisionEnt,
        col2: CollisionEnt,
        on_collide: Box<dyn Reaction>,
    ) {
        let id1 = self.get_col_idx(col1);
        let id2 = self.get_col_idx(col2);
        let col_event = CollisionEvent(id1, id2);
        if let Some(i) = self
            .events
            .collision
            .iter()
            .position(|(event, _)| *event == col_event)
        {
            self.events.collision[i].1.push(on_collide);
        } else {
            self.events.collision.push((col_event, vec![on_collide]));
        }
    }

    pub fn add_rsrc_event(
        &mut self,
        score: ScoreID,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Reaction>,
    ) {
        let rsrc_event = ResourceEvent {
            pool: score.0,
            event_type: rsrc_event,
        };
        if let Some(i) = self
            .events
            .resources
            .iter()
            .position(|(event, _)| event == &rsrc_event)
        {
            self.events.resources[i].1.push(on_rsrc_event);
        } else {
            self.events
                .resources
                .push((rsrc_event, vec![on_rsrc_event]));
        }
    }

    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        self.paddles.push(paddle);
        PaddleID(self.paddles.len() - 1)
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        self.balls.push(ball);
        BallID(self.balls.len() - 1)
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        self.walls.push(wall);
        WallID(self.walls.len() - 1)
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        self.scores.push(score);
        ScoreID(self.scores.len() - 1)
    }

    fn get_col_idx(&self, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle(paddle) => paddle.0,
            CollisionEnt::Wall(wall) => wall.0 + self.paddles.len(),
            CollisionEnt::Ball(ball) => ball.0 + self.paddles.len() + self.walls.len(),
        }
    }
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

    pub fn set_vel(&mut self, size: Vec2) {
        self.size = size;
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
    on_increase: Reactions,
}

impl Score {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

pub async fn run(game: Game) {
    let mut logics = Logics::new();

    let paddles_len = game.paddles.len();
    for (i, paddle) in game.paddles.into_iter().enumerate() {
        logics
            .collision
            .add_entity_as_xywh(paddle.pos, paddle.size, Vec2::ZERO, true, false, i);
        logics.control.mapping.push(paddle.controls);
    }

    let walls_len = game.walls.len();
    for (i, wall) in game.walls.into_iter().enumerate() {
        let i = i + paddles_len;
        logics
            .collision
            .add_entity_as_xywh(wall.pos, wall.size, Vec2::ZERO, true, false, i);
    }

    for (i, ball) in game.balls.into_iter().enumerate() {
        let i = i + paddles_len + walls_len;
        logics
            .physics
            .add_physics_entity(ball.pos, ball.vel, Vec2::ZERO);
        logics
            .collision
            .add_entity_as_xywh(ball.pos, ball.size, Vec2::ZERO, true, false, i);
    }

    let events = game.events;
    loop {
        logics.control.update(&());
        logics.physics.update();
        logics.collision.update();
        logics.resources.update();
        // draw. i think i need a game state for this one right now, at least, so i guess we still have to do projection? :(
        // :( :( :( :( :(
        // or maybe there's a reduced game state that just has a bunch of unit/identifier types saying what goes where but holds no data (except for rendering data????????), and those identifier types then act as a relay for logics to get data from each other
        next_frame().await;
    }
}
