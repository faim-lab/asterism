#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::QueuedResources,
};
use macroquad::prelude::*;

mod syntheses;
mod tables;
mod types;
use syntheses::*;
use tables::*;

// reexports
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::{Logic, QueryTable};
pub use types::*;

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

#[derive(PartialEq, Eq)]
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
    paddles: Vec<PaddleID>,
    walls: Vec<WallID>,
    balls: Vec<BallID>,
    scores: Vec<ScoreID>,
    paddle_id_max: usize,
    ball_id_max: usize,
    wall_id_max: usize,
    score_id_max: usize,
}

impl State {
    pub fn get_col_idx(&self, i: usize, col: CollisionEnt) -> usize {
        match col {
            CollisionEnt::Paddle => i,
            CollisionEnt::Wall => i + self.paddles.len(),
            CollisionEnt::Ball => i + self.paddles.len() + self.walls.len(),
        }
    }

    // i hope this logic is correct...
    pub fn get_id(&self, idx: usize) -> EntID {
        let mut idx = idx as isize;
        if idx - (self.paddles.len() as isize) < 0 {
            let paddle = self.paddles[idx as usize];
            return EntID::Paddle(paddle);
        }
        idx -= self.paddles.len() as isize;
        if idx - (self.walls.len() as isize) < 0 {
            let wall = self.walls[idx as usize];
            return EntID::Wall(wall);
        }
        idx -= self.walls.len() as isize;
        let ball = self.balls[idx as usize];
        EntID::Ball(ball)
    }

    pub fn queue_remove(&mut self, ent: EntID) {
        self.remove_queue.push(ent);
    }
    pub fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

pub struct Events {
    pub control: Vec<Predicate<CtrlEvent>>,
    pub collision: Vec<Predicate<ColEvent>>,
    pub resources: Vec<Predicate<RsrcEvent>>,
    pub physics: Vec<Predicate<PhysEvent>>,

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

    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        let id = PaddleID::new(self.state.paddle_id_max);
        self.logics.consume_paddle(
            id,
            self.state
                .get_col_idx(self.state.paddles.len(), CollisionEnt::Paddle),
            paddle,
        );
        self.state.paddle_id_max += 1;
        self.state.paddles.push(id);
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.ball_id_max);
        self.logics.consume_ball(
            self.state
                .get_col_idx(self.state.balls.len(), CollisionEnt::Ball),
            ball,
        );
        self.state.ball_id_max += 1;
        self.state.balls.push(id);

        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.wall_id_max);
        self.logics.consume_wall(
            self.state
                .get_col_idx(self.state.walls.len(), CollisionEnt::Wall),
            wall,
        );
        self.state.wall_id_max += 1;
        self.state.walls.push(id);

        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.score_id_max);
        self.logics.consume_score(id, score);
        self.state.score_id_max += 1;
        self.state.scores.push(id);
        id
    }

    fn remove_paddle(&mut self, paddle: PaddleID) {
        let ent_i = self
            .state
            .paddles
            .iter()
            .position(|pid| *pid == paddle)
            .unwrap();
        self.logics.control.mapping.remove(ent_i);
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Paddle),
            ));

        self.state.paddles.remove(ent_i);
    }

    fn remove_wall(&mut self, wall: WallID) {
        let ent_i = self
            .state
            .walls
            .iter()
            .position(|wid| *wid == wall)
            .unwrap();
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Wall),
            ));

        self.state.walls.remove(ent_i);
    }

    fn remove_ball(&mut self, ball: BallID) {
        let ent_i = self
            .state
            .balls
            .iter()
            .position(|bid| *bid == ball)
            .unwrap();
        self.logics
            .physics
            .handle_predicate(&PhysicsReaction::RemoveBody(ent_i));
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Ball),
            ));

        self.state.balls.remove(ent_i);
    }

    fn remove_score(&mut self, score: ScoreID) {
        let ent_i = self
            .state
            .scores
            .iter()
            .position(|sid| *sid == score)
            .unwrap();
        let rsrc = RsrcPool::Score(score);
        self.logics.resources.items.remove(&rsrc);

        self.state.scores.remove(ent_i);
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

    for Predicate {
        predicate,
        reaction,
    } in game.events.control.iter()
    {
        let predicate =
            Box::new(|event: &<CtrlEvent as PaddlesEvent>::AsterEvent| event == predicate);
        for event in game.logics.control.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, &event);
        }
    }
}

fn physics(game: &mut Game) {
    game.ball_phys_synthesis();

    game.logics.physics.update();

    for Predicate {
        predicate,
        reaction,
    } in game.events.physics.iter()
    {
        let square = |x| x * x;

        let predicate = Box::new(
            |(_, ident_data): &<PhysEvent as PaddlesEvent>::AsterEvent| {
                ident_data
                    .vel
                    .length_squared()
                    .partial_cmp(&square(predicate.vel_threshold))
                    .unwrap()
                    == predicate.vel_op
                    && ident_data
                        .acc
                        .length_squared()
                        .partial_cmp(&square(predicate.acc_threshold))
                        .unwrap()
                        == predicate.acc_op
            },
        );

        for event in game.logics.physics.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, &event);
        }
    }
}

fn collision(game: &mut Game) {
    game.paddle_col_synthesis();
    game.ball_col_synthesis();
    game.wall_synthesis();

    game.logics.collision.update();

    for Predicate {
        predicate,
        reaction,
    } in game.events.collision.iter()
    {
        let predicate = Box::new(
            |(i, j): &<ColEvent as PaddlesEvent>::AsterEvent| match predicate {
                ColEvent::ByType(ty_i, ty_j) => {
                    game.logics.collision.metadata[*i].id == *ty_i
                        && game.logics.collision.metadata[*j].id == *ty_j
                }
                ColEvent::ByIdx(ev_i, ev_j) => ev_i == i && ev_j == j,
            },
        );

        for event in game.logics.collision.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, &event);
        }
    }
}

fn resources(game: &mut Game) {
    game.score_synthesis();

    game.logics.resources.update();

    for Predicate {
        predicate,
        reaction,
    } in game.events.resources.iter()
    {
        let predicate = Box::new(|event: &<RsrcEvent as PaddlesEvent>::AsterEvent| {
            predicate.success == (event.event_type == ResourceEventType::PoolUpdated)
        });

        for event in game.logics.resources.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, &event);
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
