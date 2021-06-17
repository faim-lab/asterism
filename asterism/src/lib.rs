#![allow(clippy::new_without_default)]
//! # Asterism
//!
//! An asterism is a pattern people can see in stars, and while there is a fixed set of true constellations we can come up with as many asterisms as we like.
//!
//! Asterism is a project in operationalizing operational logics to the extent that they can be composed to form game engines. This means that instead of a monolithic `update()` function that combines different logics and extremely concrete instantiations of abstract processes, the game loop arbitrates its rules by configuring and calling out to a variety of logics.
//!
//! The descriptions of logics in the modules are lightly modified from Prof Osborn's dissertation.
//!
//! Requires at least Rust 1.51---if this doesn't compile, update your rustc.

pub mod collision;
pub mod control;
pub mod entity_state;
pub mod graph;
pub mod linking;
pub mod physics;
pub mod resources;

// pub trait Logic: QueryTable<(<Self as Logic>::Ident, <Self as Logic>::IdentData)>
pub trait Logic {
    type Event: Event;
    type Reaction: Reaction;

    /// a single unit/entity within the logic
    type Ident: Clone + Copy;
    type IdentData;

    /// checks if a predicate is occuring
    fn check_predicate(&self, event: &Self::Event) -> bool;

    /// processes the reaction if a predicate condition is met
    fn handle_predicate(&mut self, reaction: &Self::Reaction);

    /// exposes the data associated with a particular ""entity"" of the logic. NOTE that modifying the data returned here does NOT change the logic's data!!!
    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData;

    /// updates the data of a unit of the logic
    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData);
}

pub trait Event {
    type EventType: EventType;
    fn get_type(&self) -> &Self::EventType;
}

pub trait EventType {}

pub trait Reaction {}

/// Builds a query table over each "unit" of a logic.
///
/// kind of weird! Couldn't figure out how to get an iterator working. I don't like the reallocations but I don't think it's worse than what I'm doing in the engines with building syntheses every frame.
pub trait QueryTable<QueryOver> {
    fn predicate(&self, predicate: impl Fn(&QueryOver) -> bool) -> Vec<QueryOver>;
}
