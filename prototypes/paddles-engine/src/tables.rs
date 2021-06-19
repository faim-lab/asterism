use crate::types::*;
use crate::{Game, Logics, State};

pub struct PredicateFn<Event: asterism::Event> {
    pub predicate: Box<dyn Fn(&Event) -> bool>,
    pub reaction: Box<dyn Fn(&mut State, &mut Logics, &Event)>,
}

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        predicate: Box<dyn Fn(&CtrlEvent) -> bool>,
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &CtrlEvent)>,
    ) {
        self.events.control.push(PredicateFn {
            predicate,
            reaction: on_key_event,
        });
    }

    pub fn add_collision_predicate(
        &mut self,
        predicate: Box<dyn Fn(&ColEvent) -> bool>,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &ColEvent)>,
    ) {
        self.events.collision.push(PredicateFn {
            predicate,
            reaction: on_collide,
        });
    }

    pub fn add_rsrc_predicate(
        &mut self,
        predicate: Box<dyn Fn(&RsrcEvent) -> bool>,
        on_rsrc_event: Box<dyn Fn(&mut State, &mut Logics, &RsrcEvent)>,
    ) {
        self.events.resources.push(PredicateFn {
            predicate,
            reaction: on_rsrc_event,
        });
    }
}
