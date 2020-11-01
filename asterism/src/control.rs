use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;
use bevy_input::{keyboard::KeyCode as BevyKeyCode, Input as BevyInput};
use macroquad::prelude::{KeyCode as MqKeyCode, is_key_down};

pub trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

pub trait KeyboardControl<ID: Copy + Eq + Ord, KeyCode, InputHandler> {
    fn new() -> Self;
    fn update(&mut self, events: &InputHandler);

    fn mapping(&self) -> &Vec<Vec<Action<ID, KeyCode>>>;
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, KeyCode>>>;
    fn values(&self) -> &Vec<Vec<Values>>;
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>>;

    fn get_action(&self, id: ID) -> Option<(f32, f32)> {
        for (i, ..) in self.mapping().iter().enumerate() {
            return self.get_action_in_set(i, id);
        }
        None
    }

    fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<(f32, f32)> {
        if let Some(i) = self.mapping()[action_set].iter().position(|act| act.id == id) {
            return Some((self.values()[action_set][i].value, self.values()[action_set][i].changed_by));
        }
        None
    }

    fn add_key_map(&mut self, locus_idx: usize, keycode: KeyCode, id: ID) {
        if locus_idx >= self.mapping().len() {
            self.mapping_mut().resize_with(locus_idx + 1, Default::default);
            self.values_mut().resize_with(locus_idx + 1, Default::default);
        }
        self.mapping_mut()[locus_idx].push(
            Action {
                id,
                key_input: KeyInput { keycode },
                is_valid: false,
                input_type: InputType::Digital,
            });
        self.values_mut()[locus_idx].push(
            Values {
                value: 0.0,
                changed_by: 0.0,
            });
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyInput<KeyCode> {
    keycode: KeyCode,
}

impl<KeyCode> Input for KeyInput<KeyCode> {
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

pub struct Action<ID, KeyCode> {
    pub id: ID,
    pub key_input: KeyInput<KeyCode>,
    pub is_valid: bool,
    pub input_type: InputType,
}

pub struct WinitKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID, VirtualKeyCode>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<VirtualKeyCode>,
    this_frame_inputs: Vec<VirtualKeyCode>,
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

pub struct BevyKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID, BevyKeyCode>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<BevyKeyCode>,
    this_frame_inputs: Vec<BevyKeyCode>,
}

pub struct MacroQuadKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID, MqKeyCode>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<MqKeyCode>,
    this_frame_inputs: Vec<MqKeyCode>,
}

impl<ID: Copy + Eq + Ord> KeyboardControl<ID, VirtualKeyCode, WinitInputHelper> for WinitKeyboardControl<ID> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    fn mapping(&self) -> &Vec<Vec<Action<ID, VirtualKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, VirtualKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> { &self.values }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> { &mut self.values }

    fn update(&mut self, events: &WinitInputHelper) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {key_input, input_type, is_valid, ..} = action;
                let Values {value, changed_by} = &mut values;
                match input_type {
                    _ => {
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
}


impl<ID: Copy + Eq + Ord> KeyboardControl<ID, BevyKeyCode, BevyInput<BevyKeyCode>> for BevyKeyboardControl<ID> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    fn mapping(&self) -> &Vec<Vec<Action<ID, BevyKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, BevyKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> { &self.values }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> { &mut self.values }

    fn update(&mut self, events: &BevyInput<BevyKeyCode>) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {key_input, input_type, is_valid, ..} = action;
                let Values {value, changed_by} = &mut values;
                match input_type {
                    _ => {
                        if events.pressed(key_input.keycode) {
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
}

impl<ID: Copy + Eq + Ord> KeyboardControl<ID, MqKeyCode, ()> for MacroQuadKeyboardControl<ID> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new()
        }
    }

    fn mapping(&self) -> &Vec<Vec<Action<ID, MqKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, MqKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> { &self.values }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> { &mut self.values }

    fn update(&mut self, _events: &()) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {key_input, input_type, is_valid, ..} = action;
                let Values {value, changed_by} = &mut values;
                match input_type {
                    _ => {
                        if is_key_down(key_input.keycode) {
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

}

