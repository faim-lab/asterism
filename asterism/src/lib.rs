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
