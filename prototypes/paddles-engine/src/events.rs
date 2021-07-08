use crate::types::*;
use crate::{Logics, Predicate, State, Synthesis};
use std::any::Any;
use std::collections::HashMap;

pub type ReactionFn = Box<dyn Fn(&mut State, &mut Logics, &dyn Any)>; // use Any for now. unsure how to make it more types-y

pub struct Events {
    pub queries_max_id: usize,

    // queries
    pub control: Vec<Predicate<CtrlEvent>>,
    // pub control_ident: Vec<Predicate<CtrlIdent>>,
    pub collision: Vec<Predicate<ColEvent>>,
    // pub collision_ident: Vec<Predicate<ColIdent>>,
    pub resources: Vec<Predicate<RsrcEvent>>,
    pub resource_ident: Vec<Predicate<RsrcIdent>>,
    pub physics: Vec<Predicate<PhysIdent>>,

    pub reactions: HashMap<QueryID, ReactionFn>,
    pub stages: Stages,

    // syntheses
    pub paddle_synth: PaddleSynth,
    pub ball_synth: BallSynth,
    pub wall_synth: WallSynth,
    pub score_synth: ScoreSynth,
}

pub struct Stages {
    pub control: Vec<QueryID>,
    pub collision: Vec<QueryID>,
    pub physics: Vec<QueryID>,
    pub resources: Vec<QueryID>,
}

pub struct PaddleSynth {
    pub ctrl: Option<Synthesis<Paddle>>,
    pub col: Option<Synthesis<Paddle>>,
}

pub struct BallSynth {
    pub col: Option<Synthesis<Ball>>,
    pub phys: Option<Synthesis<Ball>>,
}

pub struct WallSynth {
    pub col: Option<Synthesis<Wall>>,
}

pub struct ScoreSynth {
    pub rsrc: Option<Synthesis<Score>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            queries_max_id: 0,
            control: Vec::new(),
            collision: Vec::new(),
            resources: Vec::new(),
            resource_ident: Vec::new(),
            physics: Vec::new(),
            reactions: HashMap::new(),

            stages: Stages {
                control: Vec::new(),
                collision: Vec::new(),
                physics: Vec::new(),
                resources: Vec::new(),
            },

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
