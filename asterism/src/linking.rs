//! # Linking logics
//!
//! Linking logics present the idea that some things, in some context, are related or connected to
//! each other. They maintain, enumerate, and follow/activate directed connections between
//! concepts.
//!
//! Linking logics are incredibly broad and have a wide range of uses. Linking logics are slightly
//! confusing to visualize because of all the nested vecs. I recommend looking at `yarn` and
//! `maze-minigame` in `/prototypes/` for examples.

/// A generic linking logic.
pub struct GraphedLinking {
    /// A vec of maps.
    pub maps: Vec<NodeMap>,
    /// Condition tables for each map in `self.maps`. If `conditions[i][j]` is true, that means the
    /// node `j` in `maps[i]` can be moved to, i.e. `position[i]` can be set to `j`.
    pub conditions: Vec<Vec<bool>>,
    /// The current node the map is on. `positions[i]` is an index in `maps[i].nodes`.
    pub positions: Vec<usize>,
}

impl GraphedLinking {
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
            conditions: Vec::new(),
            positions: Vec::new(),
        }
    }

    /// Updates the linking logic.
    ///
    /// First, check the status of all the links from the current node in the condition table. If
    /// any of those links are `true`, i.e. that node can be moved to, move the current position.
    /// Then, reset the condition table.
    pub fn update(&mut self) {
        for (i, idx) in self.positions.iter_mut().enumerate() {
            for link in &self.maps[i].nodes[*idx].links {
                if self.conditions[i][*link] {
                    *idx = *link;
                    break; // exit iteration after first match is found
                }
            }
        }

        for map_conditions in self.conditions.iter_mut() {
            for val in map_conditions.iter_mut() {
                *val = false;
            }
        }
    }

    /// Adds a map of nodes to the logic.
    ///
    /// `starting_pos` is where the node where the linking logic will start looking for links.
    ///
    /// At each index i of `nodes`, the vec of indices j_0, j_1, j_2, ... represents the indices of
    /// nodes to which node i can be linked to.
    ///
    /// All conditions by default are set to false.
    pub fn add_link_map(&mut self, starting_pos: usize, nodes: Vec<Vec<usize>>) {
        let mut node_map = NodeMap { nodes: Vec::new() };
        for nodes in nodes.iter() {
            node_map.nodes.push(Node {
                links: {
                    let mut self_nodes: Vec<usize> = Vec::new();
                    for node in nodes.iter() {
                        self_nodes.push(*node);
                    }
                    self_nodes
                },
            });
        }
        self.maps.push(node_map);
        self.conditions.push(vec![false; nodes.len()]);
        self.positions.push(starting_pos);
    }
}

/// A representation of a map of nodes.
pub struct NodeMap {
    /// A list of the nodes in the NodeMap.
    pub nodes: Vec<Node>,
}

/// A node of a map of links.
pub struct Node {
    /// List of the indices in [NodeMap] of the nodes that this node is linked to.
    pub links: Vec<usize>,
}
