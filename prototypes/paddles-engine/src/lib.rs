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

mod types;
use types::*;
mod syntheses;
use syntheses::*;

// reexports
pub use asterism::collision::{CollisionEvent, CollisionReaction};
pub use asterism::control::{ControlEvent, ControlEventType, ControlReaction};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction};
pub use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction, Transaction};
pub use asterism::Logic;
pub use types::{
    ActionID, Ball, BallID, CollisionEnt, Paddle, PaddleID, RsrcPool, Score, ScoreID, Wall, WallID,
};

pub struct Logics {
    pub collision: AabbCollision<CollisionEnt>,
    pub physics: PointPhysics,
    pub resources: QueuedResources<RsrcPool, u16>,
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
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
pub struct State {
    pub paddles: Vec<PaddleID>,
    pub balls: Vec<BallID>,
    pub walls: Vec<WallID>,
    pub scores: Vec<ScoreID>,
}

impl State {
    pub fn get_col_idx(&self, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle(paddle) => paddle.idx(),
            CollisionEnt::Wall(wall) => wall.idx() + self.paddles.len(),
            CollisionEnt::Ball(ball) => ball.idx() + self.paddles.len() + self.walls.len(),
        }
    }
}

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

pub struct Events {
    pub control: PredicateFn<ControlEvent<ActionID>>,
    pub collision: PredicateFn<CollisionEvent<CollisionEnt>>,
    pub resources: PredicateFn<ResourceEvent<RsrcPool>>,
    pub physics: PredicateFn<PhysicsEvent>,

    pub paddle_synth: Synthesis<Paddle>,
    pub ball_synth: Synthesis<Ball>,
    pub wall_synth: Synthesis<Wall>,
    pub score_synth: Synthesis<Score>,
}

impl Events {
    fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            resources: Vec::new(),
            physics: Vec::new(),

            paddle_synth: Box::new(|paddle| {}),
            ball_synth: Box::new(|ball| {}),
            wall_synth: Box::new(|wall| {}),
            score_synth: Box::new(|score| {}),
        }
    }
}

pub struct Game {
    pub state: State,
    pub logics: Logics,
    pub(crate) events: Events,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            logics: Logics::new(),
            events: Events::new(),
        }
    }

    pub fn add_ctrl_predicate(
        &mut self,
        paddle: PaddleID,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &ControlEvent<ActionID>)>,
    ) {
        let key_event = ControlEvent {
            event_type: key_event,
            action_id: action,
            set: paddle.idx(),
        };
        self.events.control.push((key_event, on_key_event));
    }

    pub fn add_collision_predicate(
        &mut self,
        col1: CollisionEnt,
        col2: CollisionEnt,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &CollisionEvent<CollisionEnt>)>,
    ) {
        let col_event = CollisionEvent(col1, col2);
        self.events.collision.push((col_event, on_collide));
    }

    pub fn add_rsrc_predicate(
        &mut self,
        pool: RsrcPool,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Fn(&mut State, &mut Logics, &ResourceEvent<RsrcPool>)>,
    ) {
        let rsrc_event = ResourceEvent {
            pool,
            event_type: rsrc_event,
        };
        self.events.resources.push((rsrc_event, on_rsrc_event));
    }

    pub fn add_paddle(&mut self, paddle: Paddle) {
        let id = PaddleID::new(self.state.paddles.len());
        self.logics
            .consume_paddle(id, self.state.get_col_idx(CollisionEnt::Paddle(id)), paddle);
        self.state.paddles.push(id);
    }

    pub fn add_ball(&mut self, ball: Ball) {
        let id = BallID::new(self.state.balls.len());
        self.logics
            .consume_ball(id, self.state.get_col_idx(CollisionEnt::Ball(id)), ball);
        self.state.balls.push(id);
    }

    pub fn add_wall(&mut self, wall: Wall) {
        let id = WallID::new(self.state.walls.len());
        self.logics
            .consume_wall(id, self.state.get_col_idx(CollisionEnt::Wall(id)), wall);
        self.state.walls.push(id);
    }

    pub fn add_score(&mut self, score: Score) {
        let id = ScoreID::new(self.state.scores.len());
        self.logics.consume_score(id, score);
        self.state.scores.push(id);
    }
}

pub async fn run(mut game: Game) {
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        // order is probably weird??
        control(&mut game);
        physics(&mut game);
        collision(&mut game);
        resources(&mut game);
        game.paddle_synthesis();
        game.wall_synthesis();
        game.ball_synthesis();
        game.score_synthesis();

        draw(&game);
        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.logics.control.update(&());

    for (predicate, reaction) in game.events.control.iter() {
        if game.logics.control.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn physics(game: &mut Game) {
    game.logics.physics.update();

    for (predicate, reaction) in game.events.physics.iter() {
        if game.logics.physics.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }

    // update collision positions, hard-coded structural synthesis
    for (i, ball) in game.state.balls.iter().enumerate() {
        let col_idx = game.state.get_col_idx(CollisionEnt::Ball(*ball));
        game.logics.collision.centers[col_idx] =
            game.logics.physics.positions[i] + game.logics.collision.half_sizes[col_idx];
    }
}

fn collision(game: &mut Game) {
    game.logics.collision.update();

    for (predicate, reaction) in game.events.collision.iter() {
        if game.logics.collision.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }

    // update physics positions
    // honestly this updating/passing on makes me think of the functional programming style of continuously modifying one state
    // kind of like projection/unprojection but instead you modify other logics' information directly...
    for (i, ball) in game.state.balls.iter().enumerate() {
        let col_idx = game.state.get_col_idx(CollisionEnt::Ball(*ball));
        game.logics.physics.positions[i] =
            game.logics.collision.centers[col_idx] - game.logics.collision.half_sizes[col_idx];
    }
}

fn resources(game: &mut Game) {
    game.logics.resources.update();
    for (predicate, reaction) in game.events.resources.iter() {
        if game.logics.resources.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn draw(game: &Game) {
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
}
