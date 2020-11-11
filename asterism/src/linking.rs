pub struct GraphedLinking {
    pub maps: Vec<NodeMap>,
    pub conditions: Vec<Vec<bool>>,
    pub positions: Vec<Option<usize>>,
    // invariants: maps, conditions, positions length are all equal. forall i, position[i] < conditions[i].len()
    // conditions[i].len() = maps.nodes.len()
}

impl GraphedLinking {
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
            conditions: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // update nodes
        for (i, node_idx) in self.positions.iter_mut().enumerate() {
            if let Some(idx) = node_idx.as_mut() {
                for link in &self.maps[i].nodes[*idx].links {
                    if self.conditions[i][*link] {
                        *idx = *link;
                    }
                }
            }
        }

        for map_conditions in self.conditions.iter_mut() {
            for val in map_conditions.iter_mut() {
                *val = false;
            }
        }
    }

    pub fn add_link_map(&mut self, starting_pos: Option<usize>, nodes: Vec<Vec<usize>>) {
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

pub struct NodeMap {
    pub nodes: Vec<Node>,
}

impl Default for NodeMap {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

pub struct Node {
    pub links: Vec<usize>,
}
