//! very shaky on the difference between predicate and structural synthesis but honestly the theoretical difference is also kind of vague so it's fine

use asterism::collision::{CollisionEvent, CollisionReaction};
use asterism::control::{ControlEvent, ControlEventType, ControlReaction};
use asterism::physics::{PhysicsEvent, PhysicsReaction};
use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction, Transaction};
use asterism::Logic;

use crate::types::*;
use crate::{Game, Logics, State};

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

#[derive(Default)]
pub struct Events {
    pub control: PredicateFn<ControlEvent<ActionID>>,
    pub collision: PredicateFn<CollisionEvent<CollisionEnt>>,
    pub resources: PredicateFn<ResourceEvent<RsrcPool>>,
    pub physics: PredicateFn<PhysicsEvent>,
}

impl Game {
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
}
