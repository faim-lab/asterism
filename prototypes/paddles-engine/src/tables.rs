use crate::types::*;
use crate::{Game, Logics, State};

pub struct Predicate<Ev: PaddlesEvent> {
    pub predicate: Ev,
    pub reaction: Box<dyn Fn(&mut State, &mut Logics, &<Ev as PaddlesEvent>::AsterEvent)>,
}

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        predicate: CtrlEvent,
        on_key_event: Box<
            dyn Fn(&mut State, &mut Logics, &<CtrlEvent as PaddlesEvent>::AsterEvent),
        >,
    ) {
        self.events.control.push(Predicate {
            predicate,
            reaction: on_key_event,
        });
    }

    pub fn add_collision_predicate(
        &mut self,
        predicate: ColEvent,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &<ColEvent as PaddlesEvent>::AsterEvent)>,
    ) {
        self.events.collision.push(Predicate {
            predicate,
            reaction: on_collide,
        });
    }

    pub fn add_rsrc_predicate(
        &mut self,
        predicate: RsrcEvent,
        on_rsrc_event: Box<
            dyn Fn(&mut State, &mut Logics, &<RsrcEvent as PaddlesEvent>::AsterEvent),
        >,
    ) {
        self.events.resources.push(Predicate {
            predicate,
            reaction: on_rsrc_event,
        });
    }

    pub fn add_rsrc_ident_predicate(
        &mut self,
        predicate: RsrcIdent,
        on_rsrc_event: Box<
            dyn Fn(&mut State, &mut Logics, &<RsrcIdent as PaddlesEvent>::AsterEvent),
        >,
    ) {
        self.events.resource_ident.push(Predicate {
            predicate,
            reaction: on_rsrc_event,
        });
    }
}
