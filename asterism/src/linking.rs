//! # Linking logics
//!
//! Linking logics present the idea that some things, in some context, are connected to each other. They maintain, enumerate, and follow/activate directed connections between concepts.
//!
//! Linking logics are incredibly broad and have a wide range of uses.
use crate::graph::StateMachine;
use crate::{Event, EventType, Logic, QueryTable, Reaction};

/// A generic linking logic. See [StateMachine][asterism::graph::StateMachine] documentation for more information.
///
/// I think this is the exact same code as FlatEntityState actually. The difference might make become more clear when rendering?
pub struct GraphedLinking<NodeID: Copy + Eq> {
    /// A vec of state machines
    pub graphs: Vec<StateMachine<NodeID>>,
    /// If the state machine has just traversed an edge or not
    pub just_traversed: Vec<bool>,
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
        edges: [(NodeID, &[NodeID]); NUM_NODES],
    ) {
        let mut graph = StateMachine::new();
        let (ids, edges): (Vec<_>, Vec<_>) = edges.iter().cloned().unzip();
        graph.add_nodes(ids.as_slice());
        graph.current_node = starting_pos;
        for (from, node_edges) in edges.iter().enumerate() {
            for to in node_edges.iter() {
                graph.add_edge(from, ids.iter().position(|id| to == id).unwrap());
            }
        }
        self.graphs.push(graph);
        self.just_traversed.push(false);
    }
}

pub struct LinkingEvent {
    pub graph: usize,
    pub node: usize,
    pub event_type: LinkingEventType,
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
    type IdentData = (Vec<NodeID>, Vec<Vec<bool>>);

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
        (graph.nodes.clone(), graph.edges.clone())
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        let graph = &mut self.graphs[ident];
        assert_eq!(data.0.len(), graph.nodes.len());
        assert_eq!(data.1.len(), graph.nodes.len());
        if !data.1.is_empty() {
            assert_eq!(data.1[0].len(), graph.nodes.len());
        }
        graph.nodes = data.0;
        graph.edges = data.1;
    }
}

type QueryOver<ID> = (
    <GraphedLinking<ID> as Logic>::Ident,
    <GraphedLinking<ID> as Logic>::IdentData,
);
impl<ID: Copy + Eq> QueryTable<QueryOver<ID>> for GraphedLinking<ID> {
    fn check_predicate(&self, predicate: impl Fn(&QueryOver<ID>) -> bool) -> Vec<QueryOver<ID>> {
        (0..self.graphs.len())
            .filter_map(|i| {
                let query_over = (i, self.get_synthesis(i));
                predicate(&query_over).then(|| query_over)
            })
            .collect()
    }
}

type QueryEvent<ID> = <GraphedLinking<ID> as Logic>::Event;

impl<ID: Copy + Eq> QueryTable<QueryEvent<ID>> for GraphedLinking<ID> {
    fn check_predicate(&self, predicate: impl Fn(&QueryEvent<ID>) -> bool) -> Vec<QueryEvent<ID>> {
        let mut events = Vec::new();
        for (i, (graph, traversed)) in self
            .graphs
            .iter()
            .zip(self.just_traversed.iter())
            .enumerate()
        {
            if *traversed {
                let event = LinkingEvent {
                    graph: i,
                    node: graph.current_node,
                    event_type: LinkingEventType::Traversed,
                };
                if predicate(&event) {
                    events.push(event);
                }
            }
            for (node, activated) in graph.conditions.iter().enumerate() {
                if *activated && node != graph.current_node {
                    let event = LinkingEvent {
                        graph: i,
                        node,
                        event_type: LinkingEventType::Activated,
                    };
                    if predicate(&event) {
                        events.push(event);
                    }
                }
            }
        }
        events
    }
}
