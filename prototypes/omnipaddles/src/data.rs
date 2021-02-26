#![allow(dead_code)]
use crate::ids::{CollisionID, PoolID};
use asterism::{resources::QueuedResources, GameState, Logic};
use std::collections::BTreeMap;

pub struct Data<State: GameState> {
    pub collision_interactions:
        BTreeMap<CollisionEvent<CollisionID>, Box<dyn Reaction<State, QueuedResources<PoolID>>>>,
}

impl<State: GameState> Data<State> {
    pub fn new(
        interact: Vec<(
            CollisionEvent<CollisionID>,
            Box<dyn Reaction<State, QueuedResources<PoolID>>>,
        )>,
    ) -> Self {
        Self {
            collision_interactions: {
                let mut interactions = BTreeMap::new();
                for (event, reaction) in interact {
                    interactions.insert(event, reaction);
                }
                interactions
            },
        }
    }

    pub fn add_collision_interaction(
        &mut self,
        event: CollisionEvent<CollisionID>,
        reaction: Box<dyn Reaction<State, QueuedResources<PoolID>>>,
    ) {
        self.collision_interactions.insert(event, reaction);
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct CollisionEvent<ID>(ID, ID);

impl<ID> CollisionEvent<ID> {
    pub fn new(id1: ID, id2: ID) -> Self {
        Self(id1, id2)
    }
}

pub trait Reaction<State: GameState, ReactingLogic: Logic> {
    /// changes the state or the logic
    fn react(&self, state: &mut State, logic: &mut ReactingLogic);
}

pub trait ReactionMetadata {}
