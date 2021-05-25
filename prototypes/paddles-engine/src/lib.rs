#![allow(unused)]
#![allow(clippy::new_without_default)]
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
pub use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction, Transaction};

pub use asterism::Logic;

mod types;
use types::*;
pub use types::{
    Ball, BallID, CollisionEnt, Paddle, PaddleID, RsrcPool, Score, ScoreID, Wall, WallID,
};

pub struct Logics {
    pub collision: AabbCollision<CollisionEnt>,
    pub physics: PointPhysics,
    pub resources: QueuedResources<RsrcPool, u16>,
    pub control: KeyboardControl<usize, MacroquadInputWrapper>,
}

#[derive(Default)]
struct Events {
    control: Vec<(ControlEvent<usize>, Reactions)>,
    collision: Vec<(CollisionEvent<CollisionEnt>, Reactions)>,
    resources: Vec<(ResourceEvent<RsrcPool>, Reactions)>,
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

    fn handle_reaction(&mut self, reaction: &dyn Reaction) {
        unimplemented!();
        // if let Some(reaction) =
        //     (&reaction as &dyn std::any::Any).downcast_ref::<ResourceReaction<RsrcPool, u16>>()
        // {
        //     self.resources.handle_predicate(reaction);
        // }
    }
}

pub struct Game {
    pub state: State,
    pub logics: Logics,
    events: Events,
}

#[derive(Default)]
pub struct State {
    pub paddles: Vec<PaddleID>,
    pub balls: Vec<BallID>,
    pub walls: Vec<WallID>,
    pub scores: Vec<ScoreID>,
}

impl State {
    fn get_col_idx(&self, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle(paddle) => paddle.idx(),
            CollisionEnt::Wall(wall) => wall.idx() + self.paddles.len(),
            CollisionEnt::Ball(ball) => ball.idx() + self.paddles.len() + self.walls.len(),
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            logics: Logics::new(),
            events: Events::default(),
        }
    }

    pub fn add_ctrl_predicate(
        &mut self,
        paddle: PaddleID,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Reaction>,
    ) {
        let key_event = ControlEvent {
            event_type: key_event,
            action_id: action.idx(),
            set: paddle.idx(),
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

    pub fn add_collision_predicate(
        &mut self,
        col1: CollisionEnt,
        col2: CollisionEnt,
        on_collide: Box<dyn Reaction>,
    ) {
        let col_event = CollisionEvent(col1, col2);
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

    pub fn add_rsrc_predicate(
        &mut self,
        score: ScoreID,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Reaction>,
    ) {
        let rsrc_event = ResourceEvent {
            pool: RsrcPool::Score(score),
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
        let id = PaddleID::new(self.state.paddles.len());
        self.logics.consume_paddle(id, paddle);
        self.state.paddles.push(id);
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.balls.len());
        self.logics.consume_ball(id, ball);
        self.state.balls.push(id);
        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.walls.len());
        self.logics.consume_wall(id, wall);
        self.state.walls.push(id);
        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.scores.len());
        self.logics.consume_score(id, score);
        self.state.scores.push(id);
        id
    }
}

pub async fn run(mut game: Game) {
    loop {
        game.logics.control.update(&());
        for (predicate, reactions) in game.events.control.iter() {
            if game.logics.control.check_predicate(predicate) {
                for reaction in reactions.iter() {
                    game.logics.handle_reaction(reaction.as_ref());
                }
            }
        }
    }
    game.logics.physics.update();
    game.logics.collision.update();
    game.logics.resources.update();

    // bad default draw fn
    clear_background(BLUE);

    for (center, hs) in game
        .logics
        .collision
        .centers
        .iter()
        .zip(game.logics.collision.half_sizes.iter())
    {
        let pos = *center - *hs;
        let size = *hs * 2.0;
        draw_rectangle(pos.x, pos.y, size.x, size.y, WHITE);
    }

    next_frame().await;
}
