//! # Linking logics
//!
//! Linking logics present the idea that some things, in some context, are related or connected to each other. They maintain, enumerate, and follow/activate directed connections between concepts.
//!
//! Linking logics are incredibly broad and have a wide range of uses. They're slightly confusing to visualize; see [StateMachine][asterism::graph::StateMachine] documentation for more information.
use crate::graph::*;
use crate::{Event, EventType, Logic, Reaction};

/// A generic linking logic.
pub struct GraphedLinking {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<usize>>,
    /// If the state machine has just traversed an edge or not
    pub just_traversed: Vec<bool>,
}

impl GraphedLinking {
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            just_traversed: Vec::new(),
        }
    }

    /// Updates the linking logic.
    ///
    /// Check the status of all the links from the current node in the condition table. If any of those links are `true`, i.e. that node can be moved to, move the current position.
    pub fn update(&mut self) {
        self.just_traversed.fill(false);
        for (graph, traversed) in self.graphs.iter_mut().zip(self.just_traversed.iter_mut()) {
            for i in graph.get_edges(graph.current_node) {
                if graph.conditions[i] {
                    graph.current_node = i;
                    *traversed = true;
                    break;
                }
            }
        }
    }

    /// Adds a map of nodes to the logic.
    ///
    /// `starting_pos` is where the node the graph traversal starts on. `edges` is a list of adjacency lists. All conditions are set to false.
    ///
    /// const generics <3
    pub fn add_graph<const NUM_NODES: usize>(
        &mut self,
        starting_pos: usize,
        edges: [&[usize]; NUM_NODES],
    ) {
        let mut graph = StateMachine::new();
        graph.add_nodes((0..NUM_NODES).collect::<Vec<_>>().as_slice());
        graph.current_node = starting_pos;
        for (from, node_edges) in edges.iter().enumerate() {
            for to in node_edges.iter() {
                graph.add_edge(from, *to);
            }
        }
        self.graphs.push(graph);
        self.just_traversed.push(false);
    }
}

pub struct LinkingEvent {
    pub graph: usize,
    pub node: usize,
    event_type: LinkingEventType,
}

pub enum LinkingEventType {
    Activated,
    Traversed,
}
impl EventType for LinkingEventType {}

impl Event for LinkingEvent {
    type EventType = LinkingEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub enum LinkingReaction {
    Activate(usize, usize),
    Traverse(usize, usize),
}

impl Reaction for LinkingReaction {}

impl Logic for GraphedLinking {
    type Event = LinkingEvent;
    type Reaction = LinkingReaction;

    /// index of graph
    type Ident = usize;
    /// current position in logic + condition table?
    type IdentData = (usize, Vec<bool>);

    fn check_predicate(&self, event: &Self::Event) -> bool {
        match event.get_type() {
            LinkingEventType::Activated => self.graphs[event.graph].conditions[event.node],
            LinkingEventType::Traversed => self.just_traversed[event.graph],
        }
    }

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            LinkingReaction::Activate(graph, node) => self.graphs[*graph].conditions[*node] = true,
            LinkingReaction::Traverse(graph, node) => {
                self.graphs[*graph].set_current_node(*node);
                self.just_traversed[*graph] = true;
            }
        }
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        let graph = &self.graphs[ident];
        (graph.current_node, graph.conditions.clone())
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        let graph = &mut self.graphs[ident];
        graph.current_node = data.0;
        assert_eq!(data.1.len(), graph.conditions.len());
        graph.conditions = data.1;
    }
}
