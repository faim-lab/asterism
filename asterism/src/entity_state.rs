//! # Entity-state Logics
//!
//! Entity-state logics communicate that game entities act in different ways or have different
//! capabilities at different times, in ways that are intrinsic to each such entity. They govern
//! the finite, discrete states of a set of game characters or other entities, update states when
//! necessary, and condition the operators of other logics on entities' discrete states.

/// An entity-state logic for flat entity state machines.
pub struct FlatEntityState<ID: Copy + Eq> {
    /// A vec of graphs representing each state machine.
    pub maps: Vec<StateMap<ID>>,
    /// Condition tables for each map in `self.maps`. If `conditions[i][j]` is true,
    /// that means the node `j` in `maps[i]` can be moved to, i.e. `position[i]` can be set to
    /// `j`.
    pub conditions: Vec<Vec<bool>>,
    /// The current state the entity is in. `states[i]` is an index in `maps[i].states`.
    pub states: Vec<usize>,
    // invariants: maps, conditions, positions length are all equal. forall i, states[i] < conditions[i].len()
    // conditions[i].len() = maps[i].states.len()
}

impl<ID: Copy + Eq> FlatEntityState<ID> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the enitity-state logic.
    ///
    /// First, check the status of all the edges from the current state in the condition table. If
    /// any of those edges are `true`, i.e. that state can be moved to, update the current state
    /// to that one. Then, reset the condition table.
    pub fn update(&mut self) {
        for (i, state_idx) in self.states.iter_mut().enumerate() {
            for edge in &self.maps[i].states[*state_idx].edges {
                if self.conditions[i][*edge] {
                    *state_idx = *edge;
                }
            }
        }
        for map_conditions in self.conditions.iter_mut() {
            for val in map_conditions.iter_mut() {
                *val = false;
            }
        }
    }

    /// Gets the current state of the entity by its index in `maps` or `states`.
    pub fn get_id_for_entity(&self, ent: usize) -> ID {
        self.maps[ent].states[self.states[ent]].id
    }

    /// Adds a state machine to the logic.
    ///
    /// At each index i of `states`, the vec of indices represents the indices of states to which
    /// the entity can move to from state i.
    ///
    /// All conditions by default are set to false.
    pub fn add_state_map(&mut self, starting_state: usize, states: Vec<(ID, Vec<usize>)>) {
        let mut state_map = StateMap { states: Vec::new() };
        for (id, edges) in states.iter() {
            state_map.states.push(State {
                id: *id,
                edges: {
                    let mut self_edges: Vec<usize> = Vec::new();
                    for edge in edges.iter() {
                        self_edges.push(*edge);
                    }
                    self_edges
                },
            });
        }
        self.maps.push(state_map);
        self.conditions.push(vec![false; states.len()]);
        self.states.push(starting_state);
    }
}

/// A representation of a map of states.
pub struct StateMap<ID> {
    pub states: Vec<State<ID>>,
}

/// A state in a state machine.
pub struct State<ID> {
    pub id: ID,
    /// The edges to the states that the entity can move to from the current state.
    pub edges: Vec<usize>,
}

impl<ID: Copy + Eq> Default for FlatEntityState<ID> {
    fn default() -> Self {
        Self {
            maps: Vec::new(),
            conditions: Vec::new(),
            states: Vec::new(),
        }
    }
}
