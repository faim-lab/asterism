pub struct JumperEntityState<ID: Copy + Eq> {
    pub maps: Vec<StateMap<ID>>,
    pub conditions: Vec<Vec<bool>>,
    pub states: Vec<usize>
}

// 1. create condition table.
// 2. update condition table in project.
// 3. use condition table to change state.
// 4. ???
// 5. profit
impl<ID: Copy + Eq> JumperEntityState<ID> {
    pub fn new() -> Self {
        Self {
            // one map per entity
            maps: Vec::new(),
            conditions: Vec::new(),
            states: Vec::new()
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
    }

    pub fn get_id_for_entity(&self, ent: usize) -> ID {
        self.maps[ent].states[self.states[ent]].id
    }
}

pub struct StateMap<ID> {
    pub states: Vec<State<ID>>,
}

pub struct State<ID> {
    pub id: ID,
    pub edges: Vec<usize>
}


