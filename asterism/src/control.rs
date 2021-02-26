//! # Control logics
//!
//! Control logics communicate that different entities are controlled by different inputs at
//! different times. They map button inputs, AI intentions, network socket messages, etc onto
//! high-level game actions.
//!
//! We're currently trying to consider analog as well as digital inputs, but we haven't implemented
//! controller support, so some of these fields don't really make sense yet.

use macroquad::prelude::{is_key_down, KeyCode as MqKeyCode};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

/// Information for a key/button press.
trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

/// Trait for generic keyboard control.
pub trait KeyboardControl<ID, KeyCode, InputHandler>: Default
where
    ID: Copy + Eq + Ord,
{
    fn new() -> Self {
        Self::default()
    }

    /// Checks and updates what inputs are being pressed every frame.
    fn update(&mut self, events: &InputHandler);

    fn mapping(&self) -> &Vec<Vec<Action<ID, KeyCode>>>;
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, KeyCode>>>;
    fn values(&self) -> &Vec<Vec<Values>>;
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>>;

    /// Returns the [Values] for the first action in the mapping with the given ID.
    fn get_action(&self, id: ID) -> Option<Values> {
        for (i, ..) in self.mapping().iter().enumerate() {
            if let Some(values) = self.get_action_in_set(i, id) {
                return Some(values);
            }
        }
        None
    }

    /// Returns the [Values] for the action with the given ID in the given set of mappings.
    fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<Values> {
        if let Some(i) = self.mapping()[action_set]
            .iter()
            .position(|act| act.id == id)
        {
            return Some(self.values()[action_set][i]);
        }
        None
    }

    /// Adds a single keymap to the logic.
    fn add_key_map(&mut self, locus_idx: usize, keycode: KeyCode, id: ID) {
        if locus_idx >= self.mapping().len() {
            self.mapping_mut()
                .resize_with(locus_idx + 1, Default::default);
            self.values_mut()
                .resize_with(locus_idx + 1, Default::default);
        }
        self.mapping_mut()[locus_idx].push(Action {
            id,
            key_input: KeyInput { keycode },
            is_valid: false,
            input_type: InputType::Digital,
        });
        self.values_mut()[locus_idx].push(Values {
            value: 0.0,
            changed_by: 0.0,
        });
    }
}

/// A keyboard input.
pub struct KeyInput<KeyCode> {
    /// The keycode that the input is tracking.
    keycode: KeyCode,
}

impl<KeyCode> Input for KeyInput<KeyCode> {
    /// Minimum value for a keypress is 0.0.
    fn min(&self) -> f32 {
        0.0
    }
    /// Maximum value for a keypress is 1.0.
    fn max(&self) -> f32 {
        1.0
    }
}

pub enum InputType {
    Analog,
    Digital,
}

#[derive(Copy, Clone)]
pub struct Values {
    /// How much the value of the input was changed last frame.
    pub changed_by: f32,
    /// What the value of the input is now.
    pub value: f32,
}

pub struct Action<ID, KeyCode> {
    pub id: ID,
    /// The input's keycode and min/max.
    pub key_input: KeyInput<KeyCode>,
    /// If the input is valid that frame, i.e. should be able to be pressed.
    pub is_valid: bool,
    /// If the input is digital or analog.
    pub input_type: InputType,
}

/// Implementation of keyboard control for `winit_input_helper`.
pub struct WinitKeyboardControl<ID: Copy + Eq + Ord> {
    /// Keymaps for each player.
    ///
    /// An index in the outer vec, i.e. `mapping[i]`, points to a keymap for a player. Indexing
    /// that vec will get you actual actions.
    pub mapping: Vec<Vec<Action<ID, VirtualKeyCode>>>,
    /// The values for each input in `mapping`.
    pub values: Vec<Vec<Values>>,
    // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len()
    last_frame_inputs: Vec<VirtualKeyCode>,
    this_frame_inputs: Vec<VirtualKeyCode>,
}

/// Implementation of keyboard control for Macroquad's input handler. See [WinitKeyboardControl] for
/// documentation of fields.
pub struct MacroQuadKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID, MqKeyCode>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<MqKeyCode>,
    this_frame_inputs: Vec<MqKeyCode>,
}

impl<ID> KeyboardControl<ID, VirtualKeyCode, WinitInputHelper> for WinitKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn mapping(&self) -> &Vec<Vec<Action<ID, VirtualKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, VirtualKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> {
        &self.values
    }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> {
        &mut self.values
    }

    fn update(&mut self, events: &WinitInputHelper) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values { value, changed_by } = &mut values;
                match input_type {
                    InputType::Digital => {
                        if events.key_held(key_input.keycode) {
                            self.this_frame_inputs.push(key_input.keycode);
                            if *is_valid {
                                if let Some(..) = self
                                    .last_frame_inputs
                                    .iter()
                                    .position(|vkc| *vkc == key_input.keycode)
                                {
                                    *changed_by = 0.0;
                                } else {
                                    *changed_by = 1.0;
                                }
                            }
                        } else if *is_valid {
                            if let Some(..) = self
                                .last_frame_inputs
                                .iter()
                                .position(|vkc| *vkc == key_input.keycode)
                            {
                                *changed_by = -1.0;
                            } else {
                                *changed_by = 0.0;
                            }
                        }
                    }
                    InputType::Analog => todo!(),
                }
                *value = (*value + *changed_by)
                    .max(key_input.min())
                    .min(key_input.max());
            }
        }
        self.last_frame_inputs.clear();
        for input in self.this_frame_inputs.iter() {
            self.last_frame_inputs.push(*input);
        }
        self.this_frame_inputs.clear();
    }
}

impl<ID> Default for WinitKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn default() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new(),
        }
    }
}

/// Macroquad doesn't have a keyboard handler type, so use the unit type.
impl<ID> KeyboardControl<ID, MqKeyCode, ()> for MacroQuadKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn mapping(&self) -> &Vec<Vec<Action<ID, MqKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, MqKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> {
        &self.values
    }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> {
        &mut self.values
    }

    /// Macroquad doesn't have an input handler type, so pass in a reference to the unit type
    /// instead
    fn update(&mut self, _events: &()) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values { value, changed_by } = &mut values;
                match input_type {
                    InputType::Digital => {
                        if is_key_down(key_input.keycode) {
                            self.this_frame_inputs.push(key_input.keycode);
                            if *is_valid {
                                if let Some(..) = self
                                    .last_frame_inputs
                                    .iter()
                                    .position(|vkc| *vkc == key_input.keycode)
                                {
                                    *changed_by = 0.0;
                                } else {
                                    *changed_by = 1.0;
                                }
                            }
                        } else if *is_valid {
                            if let Some(..) = self
                                .last_frame_inputs
                                .iter()
                                .position(|vkc| *vkc == key_input.keycode)
                            {
                                *changed_by = -1.0;
                            } else {
                                *changed_by = 0.0;
                            }
                        }
                    }
                    InputType::Analog => todo!(),
                }
                *value = (*value + *changed_by)
                    .max(key_input.min())
                    .min(key_input.max());
            }
        }
        self.last_frame_inputs.clear();
        for input in self.this_frame_inputs.iter() {
            self.last_frame_inputs.push(*input);
        }
        self.this_frame_inputs.clear();
    }
}

impl<ID> Default for MacroQuadKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn default() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new(),
        }
    }
}

#[cfg(feature = "bevy-engine")]
use bevy_input::{keyboard::KeyCode as BevyKeyCode, Input as BevyInput};

/// Implementation of keyboard control for Bevy's input handler. See [WinitKeyboardControl] for
/// documentation of fields.
#[cfg(feature = "bevy-engine")]
pub struct BevyKeyboardControl<ID: Copy + Eq + Ord> {
    pub mapping: Vec<Vec<Action<ID, BevyKeyCode>>>,
    pub values: Vec<Vec<Values>>,
    last_frame_inputs: Vec<BevyKeyCode>,
    this_frame_inputs: Vec<BevyKeyCode>,
}

#[cfg(feature = "bevy-engine")]
impl<ID> KeyboardControl<ID, BevyKeyCode, BevyInput<BevyKeyCode>> for BevyKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn mapping(&self) -> &Vec<Vec<Action<ID, BevyKeyCode>>> {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Vec<Vec<Action<ID, BevyKeyCode>>> {
        &mut self.mapping
    }

    fn values(&self) -> &Vec<Vec<Values>> {
        &self.values
    }
    fn values_mut(&mut self) -> &mut Vec<Vec<Values>> {
        &mut self.values
    }

    fn update(&mut self, events: &BevyInput<BevyKeyCode>) {
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values { value, changed_by } = &mut values;
                match input_type {
                    InputType::Digital => {
                        if events.pressed(key_input.keycode) {
                            self.this_frame_inputs.push(key_input.keycode);
                            if *is_valid {
                                if let Some(..) = self
                                    .last_frame_inputs
                                    .iter()
                                    .position(|vkc| *vkc == key_input.keycode)
                                {
                                    *changed_by = 0.0;
                                } else {
                                    *changed_by = 1.0;
                                }
                            }
                        } else if *is_valid {
                            if let Some(..) = self
                                .last_frame_inputs
                                .iter()
                                .position(|vkc| *vkc == key_input.keycode)
                            {
                                *changed_by = -1.0;
                            } else {
                                *changed_by = 0.0;
                            }
                        }
                    }
                    InputType::Analog => todo!(),
                }
                *value = (*value + *changed_by)
                    .max(key_input.min())
                    .min(key_input.max());
            }
        }
        self.last_frame_inputs.clear();
        for input in self.this_frame_inputs.iter() {
            self.last_frame_inputs.push(*input);
        }
        self.this_frame_inputs.clear();
    }
}

#[cfg(feature = "bevy-engine")]
impl<ID> Default for BevyKeyboardControl<ID>
where
    ID: Copy + Eq + Ord,
{
    fn default() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            last_frame_inputs: Vec::new(),
            this_frame_inputs: Vec::new(),
        }
    }
}
