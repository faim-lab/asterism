//! # Linking logics
//!
//! Linking logics present the idea that some things, in some context, are connected to each other. They maintain, enumerate, and follow/activate directed connections between concepts.
//!
//! Linking logics are incredibly broad and have a wide range of uses.
use crate::graph::StateMachine;
use crate::{tables::OutputTable, Event, EventType, Logic, Reaction};

/// A generic linking logic. See [StateMachine][asterism::graph::StateMachine] documentation for more information.
///
/// I think this is the exact same code as FlatEntityState actually. The difference might make become more clear when rendering?
pub struct GraphedLinking<NodeID: Copy + Eq> {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<NodeID>>,
    /// If the state machine has just traversed an edge or not
    pub just_traversed: Vec<Option<usize>>,
}

impl<NodeID: Copy + Eq> GraphedLinking<NodeID> {
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
        self.just_traversed.fill(None);
        for (graph, traversed) in self.graphs.iter_mut().zip(self.just_traversed.iter_mut()) {
            for i in graph.graph.get_edges(graph.current_node) {
                if graph.conditions[i] {
                    *traversed = Some(graph.current_node);
                    graph.current_node = i;
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
        edges: [(NodeID, &[NodeID]); NUM_NODES],
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
        self.just_traversed.push(None);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LinkingEvent {
    pub graph: usize,
    pub node: usize,
    pub event_type: LinkingEventType,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LinkingEventType {
    Activated,
    Traversed(usize), // last node (which edge)
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
    // AddNode(usize),
    // AddEdge(usize, (usize, usize))
    // RemoveNode(usize),
    // RemoveEdge(usize, (usize, usize)),
}

impl Reaction for LinkingReaction {}

impl<NodeID: Copy + Eq> Logic for GraphedLinking<NodeID> {
    type Event = LinkingEvent;
    type Reaction = LinkingReaction;

    /// index of graph
    type Ident = usize;
    /// list of graph nodes and edges
    type IdentData = NodeID;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            LinkingReaction::Activate(graph, node) => self.graphs[*graph].conditions[*node] = true,
            LinkingReaction::Traverse(graph, node) => {
                self.just_traversed[*graph] = Some(self.graphs[*graph].current_node);
                self.graphs[*graph].set_current_node(*node);
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
    <GraphedLinking<ID> as Logic>::Ident,
    <GraphedLinking<ID> as Logic>::IdentData,
);

impl<ID: Copy + Eq> OutputTable<QueryIdent<ID>> for GraphedLinking<ID> {
    fn get_table(&self) -> Vec<QueryIdent<ID>> {
        (0..self.graphs.len())
            .map(|idx| (idx, self.get_synthesis(idx)))
            .collect()
    }
}

type QueryEvent<ID> = <GraphedLinking<ID> as Logic>::Event;

impl<ID: Copy + Eq> OutputTable<QueryEvent<ID>> for GraphedLinking<ID> {
    fn get_table(&self) -> Vec<QueryEvent<ID>> {
        let mut events = Vec::new();
        for (i, (graph, traversed)) in self
            .graphs
            .iter()
            .zip(self.just_traversed.iter())
            .enumerate()
        {
            if let Some(last_node) = traversed {
                let event = LinkingEvent {
                    graph: i,
                    node: graph.current_node,
                    event_type: LinkingEventType::Traversed(*last_node),
                };
                events.push(event);
            }
            for (node, activated) in graph.conditions.iter().enumerate() {
                if *activated && node != graph.current_node {
                    let event = LinkingEvent {
                        graph: i,
                        node,
                        event_type: LinkingEventType::Activated,
                    };
                    events.push(event);
                }
            }
        }
        events
    }
}
