use crate::CollisionID;
use std::collections::BTreeMap;

pub struct Data<'data> {
    collision_interactions: BTreeMap<CollisionEvents<CollisionID>, Vec<Box<dyn Reaction + 'data>>>,
}

impl<'data> Data<'data> {
    pub fn new(interact: Vec<(CollisionEvents<CollisionID>, Box<dyn Reaction + 'data>)>) -> Self {
        Self {
            collision_interactions: {
                let mut interactions: BTreeMap<_, Vec<Box<dyn Reaction + 'data>>> = BTreeMap::new();
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

    pub fn add_interaction(
        &mut self,
        event: CollisionEvents<CollisionID>,
        reaction: Box<dyn Reaction>,
    ) {
    }
}

/* pub struct GameLogic {
    logic: some sort of reference to the actual logic or enum??? that says which logic it is, // why must my code be SAFE why can i not simply WRITE IT and have it RUN
    events: ???, // oh maybe add these as fields to the logic itself, or have a logics trait that ????
                // i trash talk java all the time but it's So Much Easier to do _anything_ with generics there
    reactions: ???????,
} */

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
pub enum Logic {
    Control,
    Physics,
    Collision,
    Resource,
    EntityState,
    Linking,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum CollisionEvents<ID> {
    Collided(ID, ID),
}

pub trait Reaction {
    fn for_logic(&self) -> Logic;
}
