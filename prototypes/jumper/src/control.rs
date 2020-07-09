#![allow(dead_code)]

use winit::event::{VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

pub trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

#[derive(Clone)]
pub enum KeyInput {
    Single(VirtualKeyCode),
    Pair(VirtualKeyCode, VirtualKeyCode)
}

impl Input for KeyInput {
    fn min(&self) -> f32 { 0.0 }

    fn max(&self) -> f32 { 1.0 }
}

#[derive(Clone)]
pub enum InputState {
    On, Off, Pressed, Released
}

pub enum ActionType {
    Instant(f32),
    Continuous(f32),
    Axis(f32, f32)
}

impl Default for ActionType {
    fn default() -> Self { Self::Instant(0.0) }
}

#[derive(Default)]
pub struct Action<ID: Copy + Eq> {
    pub id: ID,
    pub action_type: ActionType
}

pub struct InputMap<I: Input, ID: Copy + Eq> {
    pub inputs: Vec<(I, InputState)>,
    pub is_valid: Vec<bool>,
    pub actions: Vec<Action<ID>>
        // Invariants: inputs.len() == actions.actions.len()
}

pub struct WinitKeyboardControl<ID: Copy + Eq> {
    pub mapping: Vec<InputMap<KeyInput, ID>>,
    pub values: Vec<Vec<f32>>, // vector of values per mapping.
    last_frame_inputs: Vec<VirtualKeyCode>,
    this_frame_inputs: Vec<VirtualKeyCode>,
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

impl<ID: Copy + Eq> WinitKeyboardControl<ID> {
    pub fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    pub fn update(&mut self, events: &WinitInputHelper) {
        self.values.resize_with(self.mapping.len(), Default::default);
        for (map, vals) in self.mapping.iter().zip(self.values.iter_mut()) {
            vals.resize_with(map.inputs.len(), Default::default);
            for (action_map, value) in map.inputs.iter().zip(map.actions.iter()).zip(vals.iter_mut()) {
                let ((input, input_state), action) = action_map;
                let is_activated = |keycode: VirtualKeyCode| {
                    match input_state {
                        InputState::On | InputState::Pressed => events.key_held(keycode),
                        InputState::Off | InputState::Released => !events.key_held(keycode),
                    }
                };
                match (&action.action_type, input) {
                    (ActionType::Instant(val), KeyInput::Single(keycode)) |
                        (ActionType::Continuous(val), KeyInput::Single(keycode)) => {
                            if is_activated(*keycode) {
                                self.this_frame_inputs.push(*keycode);
                                if Self::action_occurs(*keycode, input_state, &self.last_frame_inputs) {
                                    *value = input.max() * val;
                                } else {
                                    *value = input.min();
                                }
                            }
                        }
                    (ActionType::Axis(axis_min, axis_max), KeyInput::Pair(keycode_min, keycode_max)) => {
                        *value = input.min();
                        if is_activated(*keycode_min) {
                            self.this_frame_inputs.push(*keycode_min);
                            if Self::action_occurs(*keycode_min, input_state, &self.last_frame_inputs) {
                                *value += input.max() * axis_min;
                            }
                        }
                        if is_activated(*keycode_max) {
                            self.this_frame_inputs.push(*keycode_max);
                            if Self::action_occurs(*keycode_max, input_state, &self.last_frame_inputs) {
                                *value += input.max() * axis_max;
                            }
                        }
                    }
                    (ActionType::Axis(axis_min, axis_max), KeyInput::Single(keycode)) => {
                        if is_activated(*keycode) {
                            self.this_frame_inputs.push(*keycode);
                            if Self::action_occurs(*keycode, input_state, &self.last_frame_inputs) {
                                *value = input.min() * axis_min;
                            }
                        } else {
                            *value = input.max() * axis_max;
                        }
                    }
                    _ => {}
                }
            }
        }
        self.last_frame_inputs.clear();
        for input in self.this_frame_inputs.iter() {
            self.last_frame_inputs.push(*input);
        }
        self.this_frame_inputs.clear();
    }

    fn action_occurs(keycode: VirtualKeyCode, input_state: &InputState, last_frame_inputs: &[VirtualKeyCode]) -> bool {
        match input_state {
            InputState::Pressed | InputState::Released => {
                for key in last_frame_inputs.iter() {
                    if *key == keycode {
                        return false;
                    }
                }
                true
            }
            _ => true
        }
    }

    pub fn get_action_by_index(&self, action_set: usize, idx: usize) -> f32 {
        self.values[action_set][idx]
    }

    // This gets the value of the first action whose `id` is `id`.
    pub fn get_action(&self, id: ID) -> Option<f32> {
        for (i, set) in self.mapping.iter().enumerate() {
            if let Some(j) = set.actions.iter().position(|act| act.id == id) {
                return Some(self.values[i][j]);
            }
        }
        None
    }

    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<f32> {
        if let Some(idx) = self.mapping[action_set].actions.iter().position(|act| act.id == id) {
            return Some(self.get_action_by_index(action_set, idx));
        }
        None
    }
}
