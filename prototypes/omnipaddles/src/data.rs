#![allow(dead_code)]
use crate::ids::CollisionID;
use asterism::{GameState, Logic, LogicType};
use std::collections::BTreeMap;

type ArbitraryReaction<GameState> = dyn Reaction<GameState, dyn Logic>;

pub struct Data<State: GameState> {
    collision_interactions:
        BTreeMap<CollisionEvent<CollisionID>, Vec<Box<ArbitraryReaction<State>>>>,
}

impl<State: GameState> Data<State> {
    pub fn new(
        interact: Vec<(CollisionEvent<CollisionID>, Box<ArbitraryReaction<State>>)>,
    ) -> Self {
        Self {
            collision_interactions: {
                let mut interactions: BTreeMap<_, Vec<_>> = BTreeMap::new();
                for (event, reaction) in interact {
                    if let Some(reactions) = interactions.get_mut(&event) {
                        reactions.push(reaction);
                    } else {
                        interactions.insert(event, {
                            let mut reactions = Vec::new();
                            reactions.push(reaction);
                            reactions
                        });
                    }
                }
                interactions
            },
        }
    }

    pub fn add_collision_interaction(
        &mut self,
        event: CollisionEvent<CollisionID>,
        reaction: Box<ArbitraryReaction<State>>,
    ) {
        if let Some(reactions) = self.collision_interactions.get_mut(&event) {
            reactions.push(reaction);
        } else {
            self.collision_interactions.insert(event, vec![reaction]);
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct CollisionEvent<ID>(ID, ID);

pub trait Reaction<State: GameState, ReactingLogic: Logic> {
    // possibly unnecessary
    fn for_logic(&self) -> LogicType;

    /// checks that the logic passed in matches with the logic this reaction is
    /// supposed to be for. possibly unnecessary
    ///
    /// also, it's only able to do a reaction for a single logic. might be cool if it took a
    /// vec (or slice??? &[&mut Logic] is such a cursed type) of logics instead. also,
    /// a bit weird if you only want to change the state and not a logic, or vice versa.
    ///
    /// not sure if passing in the state will cause borrow checker issues.
    fn react(&self, state: &mut State, logic: &mut ReactingLogic) {
        assert!(self.for_logic() == logic.logic_type());
        self.react_unchecked(state, logic);
    }

    /// changes the state or the logic
    fn react_unchecked(&self, state: &mut State, logic: &mut ReactingLogic);
}
