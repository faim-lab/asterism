/// State machine with links represented by a directed graph with an adjacency matrix.
///
/// Uses a condition table to check if an edge is traversable. If `graph.conditions[node_idx] == true`, then the edge from `graph.nodes[current_node]` to `graph.nodes[node_idx]` is traversable.
pub struct StateMachine<NodeID: Copy> {
    /// list of nodes in the graph. Possibly unnecessary, I can't decide if I want to remove this or not
    pub nodes: Vec<NodeID>,
    /// adjacency matrix
    pub edges: Vec<Vec<bool>>,
    /// index of the current node we're on
    pub current_node: usize,
    /// condition tables for the status of links in the current node in the graph
    pub conditions: Vec<bool>,
}

impl<NodeID: Copy> StateMachine<NodeID> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            current_node: 0,
            conditions: Vec::new(),
        }
    }

    /// set current node, reset condition table
    pub fn set_current_node(&mut self, node: usize) {
        self.current_node = node;
        self.conditions.fill(false);
    }

    /// set current node, reset condition table
    pub fn get_current_node(&self) -> NodeID {
        self.nodes[self.current_node]
    }

    pub fn add_node(&mut self, node: NodeID) {
        self.nodes.push(node);
        self.resize_matrix();
    }

    /// add multiple nodes at once to avoid resizing the adjacency matrix multiple times
    pub fn add_nodes(&mut self, nodes: &[NodeID]) {
        for node in nodes.iter() {
            self.nodes.push(*node);
        }
        self.resize_matrix();
    }

    /// resize adjacency matrix to the current number of nodes in the graph
    fn resize_matrix(&mut self) {
        let nodes = self.nodes.len();
        for row in self.edges.iter_mut() {
            row.resize_with(nodes, || false);
        }
        self.edges.resize_with(nodes, || vec![false; nodes]);
        self.conditions.resize_with(nodes, || false);
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges[from][to] = true;
    }

    pub fn edge_exists(&mut self, from: usize, to: usize) -> bool {
        self.edges[from][to]
    }

    // "but heap allocation costs---" shhhhh
    pub fn get_edges(&mut self, node: usize) -> Vec<usize> {
        self.edges[node]
            .iter()
            .enumerate()
            .filter_map(|(i, linked)| if *linked { Some(i) } else { None })
            .collect::<Vec<_>>()
    }
}
