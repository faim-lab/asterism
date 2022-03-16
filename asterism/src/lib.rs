//! # Asterism
//!
//! An asterism is a pattern people can see in stars, and while there is a fixed set of true constellations we can come up with as many asterisms as we like.
//!
//! Asterism is a project in operationalizing operational logics to the extent that they can be composed to form game engines. This means that instead of a monolithic `update()` function that combines different logics and extremely concrete instantiations of abstract processes, the game loop arbitrates its rules by configuring and calling out to a variety of logics.
//!
//! The descriptions of logics in the module documentation are lightly modified from Prof Osborn's dissertation.
//!
//! Requires at least Rust 1.51---if this doesn't compile, update your rustc.

#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

// logics
// pub mod collision;
// pub mod control;
// pub mod entity_state;
// pub mod graph;
// pub mod linking;
pub mod physics;
// pub mod resources;

// putting the logics together/operational integrations &c
pub mod animation;
pub mod tables;
pub use tables::OutputTable;

/// An operational logic
// (old trait bound) + OutputTable<<Self as Logic>::Event>
// and also OutputTable<(<Self as Logic>::Ident, <Self as Logic>::IdentData)>
pub trait Logic {
    /// the reactions that this logic can act on
    type Reaction: Reaction;

    /// processes the reaction if a predicate condition is met
    fn handle_predicate(&mut self, reaction: &Self::Reaction);
}

pub trait Event: Clone {}
pub trait Reaction {}
