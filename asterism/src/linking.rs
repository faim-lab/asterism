pub struct Linking {
    pub maps: Vec<NodeMap>,
    pub conditions: Vec<Vec<bool>>,
    pub positions: Vec<usize>
}

impl Linking {
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
            conditions: Vec::new(),
            positions: Vec::new()
        }
    }

    pub fn update(&mut self) {
        // update nodes
        for (i, node_idx) in self.positions.iter_mut().enumerate() {
            for link in &self.maps[i].nodes[*node_idx].links {
                if self.conditions[i][*link] {
                    *node_idx = *link;
                }
            }
        }
        for map_conditions in self.conditions.iter_mut() {
            for val in map_conditions.iter_mut() {
                *val = false;
            }
        }
    }

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
                }
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
    pub links: Vec<usize>
}

