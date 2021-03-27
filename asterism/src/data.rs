#![allow(clippy::all)]
use std::collections::BTreeMap;

use crate::collision::{CollisionEvent, CollisionReaction, Vec2};
use crate::control::{ControlEvent, ControlReaction};
use crate::physics::{PhysicsEvent, PhysicsReaction};
use crate::resources::{PoolInfo, ResourceEvent, ResourceReaction};
use crate::{Event, LogicType, Reaction};

pub struct Data<CollisionID, ActionID, PoolID, V2, KeyCode>
where
    CollisionID: Copy + Eq,
    ActionID: Copy + Eq + Ord,
    PoolID: PoolInfo,
    V2: Vec2,
    KeyCode: Copy + Eq,
    ActionID: Copy + Eq,
{
    pub events: Events<CollisionID, ActionID, PoolID>,
    pub reactions: Reactions<PoolID, V2, ActionID, KeyCode>,
    graph: BTreeMap<(LogicType, usize), Vec<Option<(LogicType, usize)>>>,
}

impl<CollisionID, ActionID, PoolID, V2, KeyCode> Data<CollisionID, ActionID, PoolID, V2, KeyCode>
where
    CollisionID: Copy + Eq,
    ActionID: Copy + Eq + Ord,
    PoolID: PoolInfo,
    V2: Vec2,
    ActionID: Copy + Eq,
    KeyCode: Copy + Eq,
{
    pub fn new() -> Self {
        Self {
            events: Events {
                physics: Vec::new(),
                collision: Vec::new(),
                resource: Vec::new(),
                control: Vec::new(),
            },
            reactions: Reactions {
                physics: Vec::new(),
                collision: Vec::new(),
                resource: Vec::new(),
                control: Vec::new(),
            },
            graph: BTreeMap::new(),
        }
    }

    pub fn add_interaction(
        &mut self,
        event: EventWrapper<CollisionID, ActionID, PoolID>,
        reaction: ReactionWrapper<PoolID, V2, ActionID, KeyCode>,
    ) {
        let ev_idx;
        let ev_logic;
        let rct_idx;
        let rct_logic;

        // these two matches suck, rip
        match event {
            EventWrapper::Physics(event) => {
                ev_logic = event.for_logic();
                if let Some(idx) = self
                    .events
                    .physics
                    .iter()
                    .position(|phys_event| *phys_event == event)
                {
                    ev_idx = idx;
                } else {
                    self.events.physics.push(event);
                    ev_idx = self.events.physics.len() - 1;
                }
            }
            EventWrapper::Collision(event) => {
                ev_logic = event.for_logic();
                if let Some(idx) = self
                    .events
                    .collision
                    .iter()
                    .position(|clln_event| *clln_event == event)
                {
                    ev_idx = idx;
                } else {
                    self.events.collision.push(event);
                    ev_idx = self.events.collision.len() - 1;
                }
            }
            EventWrapper::Resource(event) => {
                ev_logic = event.for_logic();
                if let Some(idx) = self
                    .events
                    .resource
                    .iter()
                    .position(|rsrc_event| *rsrc_event == event)
                {
                    ev_idx = idx;
                } else {
                    self.events.resource.push(event);
                    ev_idx = self.events.resource.len() - 1;
                }
            }
            EventWrapper::Control(event) => {
                ev_logic = event.for_logic();
                if let Some(idx) = self
                    .events
                    .control
                    .iter()
                    .position(|ctrl_event| *ctrl_event == event)
                {
                    ev_idx = idx;
                } else {
                    self.events.control.push(event);
                    ev_idx = self.events.control.len() - 1;
                }
            }
        }

        match reaction {
            ReactionWrapper::Physics(reaction) => {
                rct_logic = reaction.for_logic();
                if let Some(idx) = self
                    .reactions
                    .physics
                    .iter()
                    .position(|phys_reaction| *phys_reaction == reaction)
                {
                    rct_idx = idx;
                } else {
                    self.reactions.physics.push(reaction);
                    rct_idx = self.reactions.physics.len() - 1;
                }
            }
            ReactionWrapper::Collision(reaction) => {
                rct_logic = reaction.for_logic();
                if let Some(idx) = self
                    .reactions
                    .collision
                    .iter()
                    .position(|clln_reaction| *clln_reaction == reaction)
                {
                    rct_idx = idx;
                } else {
                    self.reactions.collision.push(reaction);
                    rct_idx = self.reactions.collision.len() - 1;
                }
            }
            ReactionWrapper::Resource(reaction) => {
                rct_logic = reaction.for_logic();
                if let Some(idx) = self
                    .reactions
                    .resource
                    .iter()
                    .position(|rsrc_reaction| *rsrc_reaction == reaction)
                {
                    rct_idx = idx;
                } else {
                    self.reactions.resource.push(reaction);
                    rct_idx = self.reactions.resource.len() - 1;
                }
            }
            ReactionWrapper::Control(reaction) => {
                rct_logic = reaction.for_logic();
                if let Some(idx) = self
                    .reactions
                    .control
                    .iter()
                    .position(|rsrc_reaction| *rsrc_reaction == reaction)
                {
                    rct_idx = idx;
                } else {
                    self.reactions.control.push(reaction);
                    rct_idx = self.reactions.resource.len() - 1;
                }
            }
            ReactionWrapper::GameState => {
                let interaction = self
                    .graph
                    .entry((ev_logic, ev_idx))
                    .or_insert_with(Vec::new);
                interaction.push(None);
                return;
            }
        }

        let interaction = self
            .graph
            .entry((ev_logic, ev_idx))
            .or_insert_with(Vec::new);
        interaction.push(Some((rct_logic, rct_idx)));
    }

    pub fn get_reaction(
        &self,
        event: EventWrapper<CollisionID, ActionID, PoolID>,
    ) -> Option<Vec<ReactionWrapper<PoolID, V2, ActionID, KeyCode>>> {
        let ev_logic;
        let ev_idx;

        match event {
            EventWrapper::Physics(event) => {
                if let Some(idx) = self
                    .events
                    .physics
                    .iter()
                    .position(|phys_event| *phys_event == event)
                {
                    ev_logic = event.for_logic();
                    ev_idx = idx;
                } else {
                    return None;
                }
            }
            EventWrapper::Collision(event) => {
                if let Some(idx) = self
                    .events
                    .collision
                    .iter()
                    .position(|clln_event| *clln_event == event)
                {
                    ev_logic = event.for_logic();
                    ev_idx = idx;
                } else {
                    return None;
                }
            }
            EventWrapper::Resource(event) => {
                if let Some(idx) = self
                    .events
                    .resource
                    .iter()
                    .position(|rsrc_event| *rsrc_event == event)
                {
                    ev_logic = event.for_logic();
                    ev_idx = idx;
                } else {
                    return None;
                }
            }
            EventWrapper::Control(event) => {
                if let Some(idx) = self
                    .events
                    .control
                    .iter()
                    .position(|ctrl_event| *ctrl_event == event)
                {
                    ev_logic = event.for_logic();
                    ev_idx = idx;
                } else {
                    return None;
                }
            }
        }

        if let Some(reactions) = self.graph.get(&(ev_logic, ev_idx)) {
            Some(
                reactions
                    .iter()
                    .map(|reaction| {
                        if let Some((rct_logic, rct_idx)) = reaction {
                            match rct_logic {
                                LogicType::Physics => {
                                    ReactionWrapper::Physics(self.reactions.physics[*rct_idx])
                                }
                                LogicType::Resource => {
                                    ReactionWrapper::Resource(self.reactions.resource[*rct_idx])
                                }
                                LogicType::Collision => {
                                    ReactionWrapper::Collision(self.reactions.collision[*rct_idx])
                                }
                                LogicType::Control => {
                                    ReactionWrapper::Control(self.reactions.control[*rct_idx])
                                }
                            }
                        } else {
                            ReactionWrapper::GameState
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum EventWrapper<CollisionID, ActionID, PoolID>
where
    ActionID: Copy + Eq + Ord,
    PoolID: PoolInfo,
{
    Physics(PhysicsEvent),
    Collision(CollisionEvent<CollisionID>),
    Resource(ResourceEvent<PoolID>),
    Control(ControlEvent<ActionID>),
}

#[derive(Debug)]
pub enum ReactionWrapper<PoolID, V2, ActionID, KeyCode>
where
    V2: Vec2,
    ActionID: Copy + Eq,
    KeyCode: Copy + Eq,
{
    Physics(PhysicsReaction<V2>),
    Collision(CollisionReaction),
    Resource(ResourceReaction<PoolID>),
    Control(ControlReaction<ActionID, KeyCode>),
    GameState,
}

pub struct Events<CollisionID, ActionID, PoolID>
where
    CollisionID: Copy + Eq,
    ActionID: Copy + Eq + Ord,
    PoolID: PoolInfo,
{
    pub control: Vec<ControlEvent<ActionID>>,
    pub physics: Vec<PhysicsEvent>,
    pub collision: Vec<CollisionEvent<CollisionID>>,
    pub resource: Vec<ResourceEvent<PoolID>>,
}

pub struct Reactions<PoolID, V2, ActionID, KeyCode>
where
    PoolID: PoolInfo,
    V2: Vec2,
    ActionID: Copy + Eq,
    KeyCode: Copy + Eq,
{
    pub physics: Vec<PhysicsReaction<V2>>,
    pub collision: Vec<CollisionReaction>,
    pub resource: Vec<ResourceReaction<PoolID>>,
    pub control: Vec<ControlReaction<ActionID, KeyCode>>,
}
