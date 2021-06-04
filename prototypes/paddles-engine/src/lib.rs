#![allow(unused)]
#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    control::{InputType, KeyboardControl, MacroquadInputWrapper},
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
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::Logic;
pub use types::{
    ActionID, Ball, BallID, CollisionEnt, Paddle, PaddleID, RsrcPool, Score, ScoreID, Wall, WallID,
};
pub use types::{ColEvent, CtrlEvent, RsrcEvent};

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

pub enum EntID {
    Wall(WallID),
    Ball(BallID),
    Paddle(PaddleID),
    Score(ScoreID),
}

pub enum Ent {
    Wall(Wall),
    Ball(Ball),
    Paddle(Paddle),
    Score(Score),
}

#[derive(Default)]
pub struct State {
    remove_queue: Vec<EntID>,
    add_queue: Vec<Ent>,
    // bring back vecs of ids, things should have persistent identity
    num_paddles: usize,
    num_balls: usize,
    num_walls: usize,
    num_scores: usize,
}

impl State {
    pub fn get_col_idx(&self, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle(paddle) => paddle.idx(),
            CollisionEnt::Wall(wall) => wall.idx() + self.num_paddles,
            CollisionEnt::Ball(ball) => ball.idx() + self.num_paddles + self.num_walls,
        }
    }

    // WARNING: the logic for adding and removing things is EXTREMELY WRONG, i will eventually figure it out but not today
    pub fn queue_remove(&mut self, ent: EntID) {
        self.remove_queue.push(ent);
    }
    pub fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

pub struct Events {
    pub control: PredicateFn<CtrlEvent>,
    pub collision: PredicateFn<ColEvent>,
    pub resources: PredicateFn<RsrcEvent>,
    pub physics: PredicateFn<PhysicsEvent>,

    paddle_synth: PaddleSynth,
    ball_synth: BallSynth,
    wall_synth: WallSynth,
    score_synth: ScoreSynth,
}

struct PaddleSynth {
    ctrl: Option<Synthesis<Paddle>>,
    col: Option<Synthesis<Paddle>>,
}

struct BallSynth {
    col: Option<Synthesis<Ball>>,
    phys: Option<Synthesis<Ball>>,
}

struct WallSynth {
    col: Option<Synthesis<Wall>>,
}

struct ScoreSynth {
    rsrc: Option<Synthesis<Score>>,
}

impl Events {
    fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            resources: Vec::new(),
            physics: Vec::new(),

            paddle_synth: PaddleSynth {
                col: None,
                ctrl: None,
            },
            ball_synth: BallSynth {
                col: Some(Box::new(|ball: Ball| ball)),
                phys: Some(Box::new(|ball: Ball| ball)),
            },
            wall_synth: WallSynth { col: None },
            score_synth: ScoreSynth { rsrc: None },
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
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &CtrlEvent)>,
    ) {
        let key_event = CtrlEvent {
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
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &ColEvent)>,
    ) {
        let col_event =
            ColEvent::ByIndex(self.state.get_col_idx(col1), self.state.get_col_idx(col2));
        self.events.collision.push((col_event, on_collide));
    }

    pub fn add_rsrc_predicate(
        &mut self,
        pool: RsrcPool,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Fn(&mut State, &mut Logics, &RsrcEvent)>,
    ) {
        let rsrc_event = RsrcEvent {
            pool,
            event_type: rsrc_event,
        };
        self.events.resources.push((rsrc_event, on_rsrc_event));
    }

    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        let id = PaddleID::new(self.state.num_paddles);
        self.logics
            .consume_paddle(id, self.state.get_col_idx(CollisionEnt::Paddle(id)), paddle);
        self.state.num_paddles += 1;
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.num_balls);
        self.logics
            .consume_ball(id, self.state.get_col_idx(CollisionEnt::Ball(id)), ball);
        self.state.num_balls += 1;
        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.num_walls);
        self.logics
            .consume_wall(id, self.state.get_col_idx(CollisionEnt::Wall(id)), wall);
        self.state.num_walls += 1;
        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.num_scores);
        self.logics.consume_score(id, score);
        self.state.num_scores += 1;
        id
    }

    fn remove_paddle(&mut self, paddle: PaddleID) {
        self.state.num_paddles -= 1;
        self.logics.control.mapping.remove(paddle.idx());
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(CollisionEnt::Paddle(paddle)),
            ));
    }

    fn remove_wall(&mut self, wall: WallID) {
        self.state.num_walls -= 1;
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(CollisionEnt::Wall(wall)),
            ));
    }

    fn remove_ball(&mut self, ball: BallID) {
        self.state.num_balls -= 1;
        self.logics
            .physics
            .handle_predicate(&PhysicsReaction::RemoveBody(ball.idx()));
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(CollisionEnt::Ball(ball)),
            ));
    }

    fn remove_score(&mut self, score: ScoreID) {
        self.state.num_scores -= 1;
        let rsrc = RsrcPool::Score(score);
        self.logics.resources.items.remove(&rsrc);
    }
}

pub async fn run(mut game: Game) {
    use std::time::*;
    let mut available_time = 0.0;
    let mut since = Instant::now();
    const DT: f32 = 1.0 / 60.0;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);
        available_time += since.elapsed().as_secs_f32();
        since = Instant::now();

        // framerate
        while available_time >= DT {
            available_time -= DT;

            let add_queue = std::mem::take(&mut game.state.add_queue);
            for ent in add_queue {
                match ent {
                    Ent::Wall(wall) => {
                        game.add_wall(wall);
                    }
                    Ent::Ball(ball) => {
                        game.add_ball(ball);
                    }
                    Ent::Paddle(paddle) => {
                        game.add_paddle(paddle);
                    }
                    Ent::Score(score) => {
                        game.add_score(score);
                    }
                };
            }

            control(&mut game);
            physics(&mut game);
            collision(&mut game);
            resources(&mut game);

            let remove_queue = std::mem::take(&mut game.state.remove_queue);
            for ent in remove_queue {
                match ent {
                    EntID::Wall(wall) => {
                        game.remove_wall(wall);
                    }
                    EntID::Ball(ball) => {
                        game.remove_ball(ball);
                    }
                    EntID::Paddle(paddle) => {
                        game.remove_paddle(paddle);
                    }
                    EntID::Score(score) => {
                        game.remove_score(score);
                    }
                };
            }
        }

        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.paddle_ctrl_synthesis();

    game.logics.control.update(&());

    for (predicate, reaction) in game.events.control.iter() {
        if game.logics.control.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn physics(game: &mut Game) {
    game.ball_phys_synthesis();

    game.logics.physics.update();

    for (predicate, reaction) in game.events.physics.iter() {
        if game.logics.physics.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn collision(game: &mut Game) {
    game.paddle_col_synthesis();
    game.ball_col_synthesis();
    game.wall_synthesis();

    game.logics.collision.update();

    for (predicate, reaction) in game.events.collision.iter() {
        if game.logics.collision.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn resources(game: &mut Game) {
    game.score_synthesis();

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
