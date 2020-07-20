use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;
use std::collections::BTreeMap;

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

// #[derive(Default)]
pub struct Action {
    pub key_input: KeyInput,
    pub is_valid: bool,
    pub input_type: InputType,
    pub changed_by: f32,
    pub value: f32,
}

pub struct WinitKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<BTreeMap<ID, Action>>,
    last_frame_inputs: Vec<VirtualKeyCode>,
    this_frame_inputs: Vec<VirtualKeyCode>,
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

impl<ID: Copy + Eq + Ord> WinitKeyboardControl<ID> {
    pub fn new() -> Self {
        Self {
            mapping: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    pub fn update(&mut self, events: &WinitInputHelper) {
        for map in self.mapping.iter_mut() {
            for (.., action) in map.iter_mut() {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    mut changed_by,
                    ..
                } = action;
                match input_type {
                    InputType::Digital => {
                        if events.key_held(key_input.keycode) {
                            self.this_frame_inputs.push(key_input.keycode);
                            if *is_valid {
                                if let Some(..) = self.last_frame_inputs.iter().position(|vkc| *vkc == key_input.keycode) {
                                    changed_by = 0.0;
                                } else {
                                    changed_by = 1.0;
                                }
                            }
                        } else {
                            if *is_valid {
                                if let Some(..) = self.last_frame_inputs.iter().position(|vkc| *vkc == key_input.keycode) {
                                    changed_by = -1.0;
                                } else {
                                    changed_by = 0.0;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                action.value = (action.value + changed_by).max(Input::min(key_input)).min(Input::max(key_input));
                action.changed_by = changed_by;
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
        }
        self.mapping[locus_idx].insert(
            id,
            Action {
                key_input: KeyInput {
                    keycode: keycode
                },
                is_valid: false,
                input_type: InputType::Digital,
                changed_by: 0.0,
                value: 0.0
            }
        );
    }

    pub fn get_action(&self, id: ID) -> Option<(f32, f32)> {
        for (i, map) in self.mapping.iter().enumerate() {
            if map.contains_key(&id) {
                return self.get_action_in_set(i, id);
            }
        }
        None
    }

    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<(f32, f32)> {
        if let Some(action) = self.mapping[action_set].get(&id) {
            return Some((action.value, action.changed_by));
        }
        None
    }
}
