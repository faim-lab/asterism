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
mod types;
use events::*;
use predicates::*;

// reexports
pub use asterism::collision::{AabbColData, AabbCollision, CollisionReaction};
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::tables::*;
pub use asterism::Compare;
pub use asterism::{Logic, OutputTable};
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

    pub fn paddles(&self) -> &[PaddleID] {
        &self.paddles
    }
    pub fn walls(&self) -> &[WallID] {
        &self.walls
    }
    pub fn balls(&self) -> &[BallID] {
        &self.balls
    }
    pub fn scores(&self) -> &[ScoreID] {
        &self.scores
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
    pub(crate) tables: ConditionTables<QueryType>,
}

impl Game {
    pub fn new() -> Self {
        let mut tables = ConditionTables::new();

        // collision
        tables.add_query::<AColEvent>(QueryType::ColEvent, None);
        tables.add_query::<AColIdent>(QueryType::ColIdent, None);

        // phys
        tables.add_query::<APhysIdent>(QueryType::PhysIdent, None);
        tables.add_query::<PhysEvent>(QueryType::PhysEvent, None);

        // rsrc
        tables.add_query::<ARsrcEvent>(QueryType::RsrcEvent, None);
        tables.add_query::<ARsrcIdent>(QueryType::RsrcIdent, None);

        // ctrl
        tables.add_query::<CtrlEvent>(QueryType::CtrlEvent, None);
        tables.add_query::<CtrlIdent>(QueryType::CtrlIdent, None);

        // ball collision idents
        // ball physics idents are just physics idents
        tables.add_query::<AColIdent>(
            QueryType::BallCol,
            Some(Compose::Filter(QueryType::ColIdent)),
        );

        Self {
            state: State::default(),
            logics: Logics::new(),
            events: Events::new(),
            tables,
        }
    }
}

// macro to make matching entities to statements less verbose
macro_rules! match_ent {
    (
        $match_to:expr,
        $wall:ident: $wall_block:block,
        $ball:ident: $ball_block:block,
        $paddle:ident: $paddle_block:block,
        $score:ident: $score_block:block
    ) => {
        match $match_to {
            Ent::Wall($wall) => $wall_block,
            Ent::Ball($ball) => $ball_block,
            Ent::Paddle($paddle) => $paddle_block,
            Ent::Score($score) => $score_block,
        }
    };
    (
        $match_to:expr,
        only $ent:ident: $ent_block:block
    ) => {
        match $match_to {
            EntID::Wall($ent) => $ent_block,
            EntID::Ball($ent) => $ent_block,
            EntID::Paddle($ent) => $ent_block,
            EntID::Score($ent) => $ent_block,
        }
    };
}

// macro to make matching entity ids to statements less verbose
macro_rules! match_ent_id {
    (
        $match_to:expr,
        $wall:ident: $wall_block:block,
        $ball:ident: $ball_block:block,
        $paddle:ident: $paddle_block:block,
        $score:ident: $score_block:block
    ) => {
        match $match_to {
            EntID::Wall($wall) => $wall_block,
            EntID::Ball($ball) => $ball_block,
            EntID::Paddle($paddle) => $paddle_block,
            EntID::Score($score) => $score_block,
        }
    };
    (
        $match_to:expr,
        only $ent:ident: $ent_block:block
    ) => {
        match $match_to {
            EntID::Wall($ent) => $ent_block,
            EntID::Ball($ent) => $ent_block,
            EntID::Paddle($ent) => $ent_block,
            EntID::Score($ent) => $ent_block,
        }
    };
}

pub async fn run(mut game: Game) {
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);

        control(&mut game);
        physics(&mut game);
        collision(&mut game);
        resources(&mut game);

        // remove
        game.state.remove_queue.sort_by(|a, b| {
            let a = match_ent_id!(a, only ent: { ent.idx() } );
            let b = match_ent_id!(b, only ent: { ent.idx() });
            a.cmp(&b)
        });
        let remove_queue = std::mem::take(&mut game.state.remove_queue);
        for ent in remove_queue {
            match_ent_id!(
                ent,
                wall: { game.remove_wall(wall); },
                ball: { game.remove_ball(ball); },
                paddle: { game.remove_paddle(paddle); },
                score: { game.remove_score(score); }
            );
        }

        // add
        let add_queue = std::mem::take(&mut game.state.add_queue);
        for ent in add_queue {
            match_ent!(
                ent,
                wall: { game.add_wall(wall); },
                ball: { game.add_ball(ball); },
                paddle: { game.add_paddle(paddle); },
                score: { game.add_score(score); }
            );
        }

        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.logics.control.update(&());

    game.tables
        .update_single::<CtrlEvent>(QueryType::CtrlEvent, game.logics.control.get_table())
        .unwrap();
    game.tables
        .update_single::<CtrlIdent>(QueryType::CtrlIdent, game.logics.control.get_table())
        .unwrap();

    for Predicate {
        id,
        predicate,
        reaction,
    } in game.events.control.iter()
    {
        let predicate = Box::new(|event: &CtrlEvent| event == predicate);
        let ans = game
            .tables
            .update_filter(QueryType::User(*id), predicate)
            .unwrap();
        for event in ans.iter() {
            reaction(&mut game.state, &mut game.logics, event);
        }
    }
}

fn physics(game: &mut Game) {
    game.logics.physics.update();

    game.tables
        .update_single::<PhysEvent>(QueryType::PhysEvent, game.logics.physics.get_table())
        .unwrap();

    let ans = game
        .tables
        .update_single::<APhysIdent>(QueryType::PhysIdent, game.logics.physics.get_table())
        .unwrap();

    // update physics positions to collision
    for (idx, data) in ans.iter() {
        let idx = game.state.get_col_idx(*idx, CollisionEnt::Ball);

        game.logics
            .collision
            .handle_predicate(&CollisionReaction::SetPos(idx, data.pos));
    }

    // user defined events
    for Predicate {
        predicate,
        id,
        reaction,
    } in game.events.physics.iter()
    {
        let square = |x| x * x;

        let pred_fn = Box::new(|(_, ident_data): &APhysIdent| {
            predicate.vel_op.cmp(
                ident_data.vel.length_squared(),
                square(predicate.vel_threshold),
            ) && predicate.vel_op.cmp(
                ident_data.acc.length_squared(),
                square(predicate.acc_threshold),
            )
        });

        let ans = game
            .tables
            .update_filter(QueryType::User(*id), pred_fn)
            .unwrap();
        for event in ans.iter() {
            reaction(&mut game.state, &mut game.logics, event);
        }
    }
}

fn collision(game: &mut Game) {
    game.logics.collision.update();

    game.tables
        .update_single::<AColEvent>(QueryType::ColEvent, game.logics.collision.get_table())
        .unwrap();
    game.tables
        .update_single::<AColIdent>(QueryType::ColIdent, game.logics.collision.get_table())
        .unwrap();

    // update collision positions to physics
    let paddles_len = game.state.paddles.len();
    let walls_len = game.state.walls.len();
    let ans = game
        .tables
        .update_filter(
            QueryType::BallCol,
            Box::new(|(idx, _): &AColIdent| *idx > paddles_len + walls_len),
        )
        .unwrap();
    for (idx, data) in ans.iter() {
        let idx = idx - paddles_len - walls_len;
        game.logics
            .physics
            .handle_predicate(&PhysicsReaction::SetPos(idx, data.center - data.half_size));
    }

    // user defined filters
    for Predicate {
        predicate,
        id,
        reaction,
    } in game.events.collision.iter()
    {
        let collision = &game.logics.collision;

        let pred_fn = Box::new(|(i, j): &AColEvent| match predicate {
            ColEvent::ByType(ty_i, ty_j) => {
                collision.metadata[*i].id == *ty_i && collision.metadata[*j].id == *ty_j
            }
            ColEvent::ByIdx(ev_i, ev_j) => ev_i == i && ev_j == j,
        });

        let ans = game
            .tables
            .update_filter(QueryType::User(*id), pred_fn)
            .unwrap();

        for event in ans.iter() {
            reaction(&mut game.state, &mut game.logics, event);
        }
    }
}

fn resources(game: &mut Game) {
    game.logics.resources.update();

    game.tables
        .update_single::<ARsrcEvent>(QueryType::RsrcEvent, game.logics.resources.get_table())
        .unwrap();
    game.tables
        .update_single::<ARsrcIdent>(QueryType::RsrcIdent, game.logics.resources.get_table())
        .unwrap();

    // user defined filters, idents
    for Predicate {
        predicate,
        id,
        reaction,
    } in game.events.resource_ident.iter()
    {
        let pred_fn = Box::new(|(id, vals): &ARsrcIdent| {
            predicate.op.cmp(vals.0, predicate.threshold)
                && if let Some(pool) = predicate.pool {
                    pool == *id
                } else {
                    true
                }
        });

        let ans = game
            .tables
            .update_filter(QueryType::User(*id), pred_fn)
            .unwrap();

        for event in ans.iter() {
            reaction(&mut game.state, &mut game.logics, event);
        }
    }

    // user defined filters, events
    for Predicate {
        predicate,
        id,
        reaction,
    } in game.events.resources.iter()
    {
        let pred_fn = Box::new(|event: &ARsrcEvent| {
            predicate.success == (event.event_type == ResourceEventType::PoolUpdated)
        });

        let ans = game
            .tables
            .update_filter(QueryType::User(*id), pred_fn)
            .unwrap();

        for event in ans.iter() {
            reaction(&mut game.state, &mut game.logics, event);
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
