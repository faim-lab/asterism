use crate::types::*;
use crate::{Logics, Predicate, State};
// use std::collections::HashMap;

#[allow(unused)]
pub type ReactionFn<Event> = Box<dyn Fn(&mut State, &mut Logics, &Event)>;

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
    // pub reactions: HashMap<QueryID, ReactionFn>,
    // pub stages: Stages,
}

// pub struct Stages {
//     pub control: Vec<QueryID>,
//     pub collision: Vec<QueryID>,
//     pub physics: Vec<QueryID>,
//     pub resources: Vec<QueryID>,
// }

impl Events {
    pub fn new() -> Self {
        Self {
            queries_max_id: 0,
            control: Vec::new(),
            collision: Vec::new(),
            resources: Vec::new(),
            resource_ident: Vec::new(),
            physics: Vec::new(),
            // reactions: HashMap::new(),

            // stages: Stages {
            //     control: Vec::new(),
            //     collision: Vec::new(),
            //     physics: Vec::new(),
            //     resources: Vec::new(),
            // },
        }
    }
}
