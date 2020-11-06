pub struct FlatEntityState<ID: Copy + Eq> {
    pub maps: Vec<StateMap<ID>>,
    pub conditions: Vec<Vec<bool>>,
    pub states: Vec<usize>,
}

impl<ID: Copy + Eq> FlatEntityState<ID> {
    pub fn new() -> Self {
        Self {
            // one map per entity
            maps: Vec::new(),
            conditions: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // update states
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

    pub fn get_id_for_entity(&self, ent: usize) -> ID {
        self.maps[ent].states[self.states[ent]].id
    }

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

pub struct StateMap<ID> {
    pub states: Vec<State<ID>>,
}

impl<ID: Copy + Eq> Default for StateMap<ID> {
    fn default() -> Self {
        Self { states: Vec::new() }
    }
}

pub struct State<ID> {
    pub id: ID,
    pub edges: Vec<usize>,
}
