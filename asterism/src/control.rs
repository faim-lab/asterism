use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

pub trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyInput {
    keycode: VirtualKeyCode,
}

impl Input for KeyInput {
    fn min(&self) -> f32 { 0.0 }
    fn max(&self) -> f32 { 1.0 }
}

pub enum InputType {
    Analog,
    Digital,
}

impl Default for InputType {
    fn default() -> Self { Self::Digital }
}

pub struct Values {
    pub changed_by: f32,
    pub value: f32,
}

// #[derive(Default)]
pub struct Action<ID> {
    pub id: ID,
    pub key_input: KeyInput,
    pub is_valid: bool,
    pub input_type: InputType,
}

pub struct WinitKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<VirtualKeyCode>,
    this_frame_inputs: Vec<VirtualKeyCode>,
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

impl<ID: Copy + Eq + Ord> WinitKeyboardControl<ID> {
    pub fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    pub fn update(&mut self, events: &WinitInputHelper) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values {
                    value,
                    changed_by
                } = &mut values;
                match input_type {
                    InputType::Digital => {
                        if events.key_held(key_input.keycode) {
                            self.this_frame_inputs.push(key_input.keycode);
                            if *is_valid {
                                if let Some(..) = self.last_frame_inputs.iter().position(|vkc| *vkc == key_input.keycode) {
                                    *changed_by = 0.0;
                                } else {
                                    *changed_by = 1.0;
                                }
                            }
                        } else {
                            if *is_valid {
                                if let Some(..) = self.last_frame_inputs.iter().position(|vkc| *vkc == key_input.keycode) {
                                    *changed_by = -1.0;
                                } else {
                                    *changed_by = 0.0;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                *value = (*value + *changed_by).max(Input::min(key_input)).min(Input::max(key_input));
            }
        }
        self.last_frame_inputs.clear();
        for input in self.this_frame_inputs.iter() {
            self.last_frame_inputs.push(*input);
        }
        self.this_frame_inputs.clear();
    }

    pub fn add_key_map(&mut self, locus_idx: usize, keycode: VirtualKeyCode, id: ID) {
        if locus_idx >= self.mapping.len() {
            self.mapping.resize_with(locus_idx + 1, Default::default);
            self.values.resize_with(locus_idx + 1, Default::default);
        }
        self.mapping[locus_idx].push(
            Action {
                id: id,
                key_input: KeyInput {
                    keycode: keycode
                },
                is_valid: false,
                input_type: InputType::Digital,
            });
        self.values[locus_idx].push(
            Values {
                value: 0.0,
                changed_by: 0.0,
            });
    }

    pub fn get_action(&self, id: ID) -> Option<(f32, f32)> {
        for (i, ..) in self.mapping.iter().enumerate() {
            return self.get_action_in_set(i, id);
        }
        None
    }

    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<(f32, f32)> {
        if let Some(i) = self.mapping[action_set].iter().position(|act| act.id == id) {
            return Some((self.values[action_set][i].value, self.values[action_set][i].changed_by));
        }
        None
    }
}
