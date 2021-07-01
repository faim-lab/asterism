#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::QueuedResources,
};
use macroquad::prelude::*;

mod entities;
mod events;
mod predicates;
mod syntheses;
// mod tables;
mod types;
use events::*;
use predicates::*;
use syntheses::*;
// use tables::*;

// reexports
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::tables::*;
pub use asterism::Compare;
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

pub struct Game {
    pub state: State,
    pub logics: Logics,
    pub(crate) events: Events,
    pub(crate) table: ConditionTables<QueryID>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            logics: Logics::new(),
            events: Events::new(),
            table: ConditionTables::new(),
        }
    }
}

pub async fn run(mut game: Game) {
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);

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

        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.paddle_ctrl_synthesis();

    game.logics.control.update(&());

    for Predicate { id, predicate } in game.events.control.iter() {
        let pred_fn =
            Box::new(|event: &<CtrlEvent as PaddlesEvent>::AsterEvent| event == predicate);
        game.table
            .update_query(*id, game.logics.control.check_predicate(pred_fn));
    }

    for condition in game.events.stages.control.iter() {
        let reaction = game.events.reactions.get(condition).unwrap();
        game.table.check_condition(*condition);
        let Condition { compose, output } = game.table.get_condition(*condition);
        for _ in output.iter().filter(|ans| **ans) {
            reaction(&mut game.state, &mut game.logics, compose);
        }
    }
}

fn physics(game: &mut Game) {
    game.ball_phys_synthesis();

    game.logics.physics.update();

    for Predicate { predicate, id } in game.events.physics.iter() {
        let square = |x| x * x;

        let pred_fn = Box::new(
            |(_, ident_data): &<PhysIdent as PaddlesEvent>::AsterEvent| {
                predicate.vel_op.cmp(
                    ident_data.vel.length_squared(),
                    square(predicate.vel_threshold),
                ) && predicate.vel_op.cmp(
                    ident_data.acc.length_squared(),
                    square(predicate.acc_threshold),
                )
            },
        );

        let answers = game.logics.physics.check_predicate(pred_fn);
        game.table.update_query(*id, answers);
    }

    for condition in game.events.stages.physics.iter() {
        let reaction = game.events.reactions.get(condition).unwrap();
        game.table.check_condition(*condition);
        let Condition { compose, output } = game.table.get_condition(*condition);
        for _ in output.iter().filter(|ans| **ans) {
            reaction(&mut game.state, &mut game.logics, compose);
        }
    }
}

fn collision(game: &mut Game) {
    game.paddle_col_synthesis();
    game.ball_col_synthesis();
    game.wall_synthesis();

    game.logics.collision.update();

    for Predicate { predicate, id } in game.events.collision.iter() {
        let pred_fn = Box::new(
            |(i, j): &<ColEvent as PaddlesEvent>::AsterEvent| match predicate {
                ColEvent::ByType(ty_i, ty_j) => {
                    game.logics.collision.metadata[*i].id == *ty_i
                        && game.logics.collision.metadata[*j].id == *ty_j
                }
                ColEvent::ByIdx(ev_i, ev_j) => ev_i == i && ev_j == j,
            },
        );
        let answers = game.logics.collision.check_predicate(pred_fn);
        game.table.update_query(*id, answers);
    }

    for condition in game.events.stages.collision.iter() {
        let reaction = game.events.reactions.get(condition).unwrap();
        game.table.check_condition(*condition);
        let Condition { compose, output } = game.table.get_condition(*condition);
        for _ in output.iter().filter(|ans| **ans) {
            reaction(&mut game.state, &mut game.logics, compose);
        }
    }
}

fn resources(game: &mut Game) {
    game.score_synthesis();

    game.logics.resources.update();

    for Predicate { predicate, id } in game.events.resources.iter() {
        let predicate = Box::new(|event: &<RsrcEvent as PaddlesEvent>::AsterEvent| {
            predicate.success == (event.event_type == ResourceEventType::PoolUpdated)
        });

        let answers = game.logics.resources.check_predicate(predicate);
        game.table.update_query(*id, answers);
    }

    for Predicate { predicate, id } in game.events.resource_ident.iter() {
        let predicate = Box::new(|(id, vals): &<RsrcIdent as PaddlesEvent>::AsterEvent| {
            predicate.op.cmp(vals.0, predicate.threshold)
                && if let Some(pool) = predicate.pool {
                    pool == *id
                } else {
                    true
                }
        });

        let answers = game.logics.resources.check_predicate(predicate);
        game.table.update_query(*id, answers);
    }

    for condition in game.events.stages.resources.iter() {
        let reaction = game.events.reactions.get(condition).unwrap();
        game.table.check_condition(*condition);
        let Condition { compose, output } = game.table.get_condition(*condition);
        for _ in output.iter().filter(|ans| **ans) {
            reaction(&mut game.state, &mut game.logics, compose);
        }
    }
}

pub fn draw(game: &Game) {
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
