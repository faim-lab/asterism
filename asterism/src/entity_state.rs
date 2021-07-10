//! # Entity-state Logics
//!
//! Entity-state logics communicate that game entities act in different ways or have different capabilities at different times, in ways that are intrinsic to each such entity. They govern the finite, discrete states of a set of game characters or other entities, update states when necessary, and condition the operators of other logics on entities' discrete states.

use crate::graph::StateMachine;
use crate::{tables::OutputTable, Event, EventType, Logic, Reaction};

/// An entity-state logic for flat entity state machines.
pub struct FlatEntityState<ID: Copy + Eq> {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<ID>>,
    pub just_traversed: Vec<bool>,
}

impl<ID: Copy + Eq> FlatEntityState<ID> {
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            just_traversed: Vec::new(),
        }
    }

    /// Updates the entity-state logic.
    ///
    /// Check the status of all the links from the current node in the condition table. If any of those links are `true`, i.e. that node can be moved to, move the current position.
    pub fn update(&mut self) {
        self.just_traversed.fill(false);
        for (graph, traversed) in self.graphs.iter_mut().zip(self.just_traversed.iter_mut()) {
            for i in graph.graph.get_edges(graph.current_node) {
                if graph.conditions[i] {
                    graph.current_node = i;
                    *traversed = true;
                    break;
                }
            }
        }
    }

    /// Gets the current state of the entity by its index
    pub fn get_id_for_entity(&self, ent: <Self as Logic>::Ident) -> ID {
        self.graphs[ent].get_current_node()
    }

    /// Adds a map of nodes to the logic.
    ///
    /// `starting_pos` is where the node the graph traversal starts on. `edges` is a list of adjacency lists. All conditions are set to false.
    pub fn add_graph<const NUM_NODES: usize>(
        &mut self,
        starting_pos: usize,
        edges: [(ID, &[ID]); NUM_NODES],
    ) {
        let mut graph = StateMachine::new();
        let (ids, edges): (Vec<_>, Vec<_>) = edges.iter().cloned().unzip();
        graph.add_nodes(ids.as_slice());
        graph.current_node = starting_pos;
        for (from, node_edges) in edges.iter().enumerate() {
            for to in node_edges.iter() {
                graph
                    .graph
                    .add_edge(from, ids.iter().position(|id| to == id).unwrap());
            }
        }
        self.graphs.push(graph);
        self.just_traversed.push(false);
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

pub struct EntityEvent {
    pub graph: usize,
    pub node: usize,
    event_type: EntityEventType,
}

pub enum EntityEventType {
    Activated,
    Traversed,
}
impl EventType for EntityEventType {}

impl Event for EntityEvent {
    type EventType = EntityEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub enum EntityReaction {
    Activate(usize, usize),
    Traverse(usize, usize),
}

impl Reaction for EntityReaction {}

impl<ID: Copy + Eq> Logic for FlatEntityState<ID> {
    type Event = EntityEvent;
    type Reaction = EntityReaction;

    /// index of graph
    type Ident = usize;
    /// current position in logic
    type IdentData = ID;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            EntityReaction::Activate(graph, node) => self.graphs[*graph].conditions[*node] = true,
            EntityReaction::Traverse(graph, node) => {
                self.graphs[*graph].set_current_node(*node);
                self.just_traversed[*graph] = true;
            }
        }
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        self.graphs[ident].get_current_node()
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        let graph = &mut self.graphs[ident];
        let node = graph.graph.nodes.iter().position(|id| *id == data);
        if let Some(idx) = node {
            graph.current_node = idx;
        }
    }
}

type QueryIdent<ID> = (
    <FlatEntityState<ID> as Logic>::Ident,
    <FlatEntityState<ID> as Logic>::IdentData,
);
impl<ID: Copy + Eq> OutputTable<QueryIdent<ID>> for FlatEntityState<ID> {
    fn get_table(&self) -> Vec<QueryIdent<ID>> {
        (0..self.graphs.len())
            .map(|idx| (idx, self.get_synthesis(idx)))
            .collect()
    }
}

type QueryEvent<ID> = <FlatEntityState<ID> as Logic>::Event;

impl<ID: Copy + Eq> OutputTable<QueryEvent<ID>> for FlatEntityState<ID> {
    fn get_table(&self) -> Vec<QueryEvent<ID>> {
        let mut events = Vec::new();
        for (i, (graph, traversed)) in self
            .graphs
            .iter()
            .zip(self.just_traversed.iter())
            .enumerate()
        {
            if *traversed {
                let event = EntityEvent {
                    graph: i,
                    node: graph.current_node,
                    event_type: EntityEventType::Traversed,
                };
                events.push(event);
            }
            for (node, activated) in graph.conditions.iter().enumerate() {
                if *activated && node != graph.current_node {
                    let event = EntityEvent {
                        graph: i,
                        node,
                        event_type: EntityEventType::Activated,
                    };
                    events.push(event);
                }
            }
        }
        events
    }
}
