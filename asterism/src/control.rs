//! # Control logics
//!
//! Control logics communicate that different entities are controlled by different inputs at
//! different times. They map button inputs, AI intentions, network socket messages, etc onto
//! high-level game actions.
//!
//! We're currently trying to consider analog as well as digital inputs, but we haven't implemented
//! controller support, so some of these fields don't really make sense yet.

use std::collections::BTreeSet;

use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

/// Information for a key/button press.
trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

/// Generic keyboard control.
pub struct KeyboardControl<ID, KeyCode>
where
    ID: Copy + Eq + Ord,
    KeyCode: Copy,
{
    pub mapping: Vec<Vec<Action<ID, KeyCode>>>,
    pub values: Vec<Vec<Values>>,
}

impl<ID: Copy + Eq + Ord, KeyCode: Copy> Default for KeyboardControl<ID, KeyCode> {
    fn default() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<ID: Copy + Eq + Ord, KeyCode: Copy> KeyboardControl<ID, KeyCode> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks and updates what inputs are being pressed every frame.
    pub fn update<InputHelper>(
        &mut self,
        event_wrapper: &mut impl InputWrapper<KeyCode, InputHelper>,
        events: &InputHelper,
    ) {
        event_wrapper.update(
            // "please stop trying to do things with iterators cynthia"
            // absolutely not. i have to use my hard-won cs54 knowledge for _something_
            // anyway this is probably expensive or something. the way we log input schemes is so nightmarish :smiling_face_with_tear_emoji:
            &self
                .mapping
                .iter()
                .flat_map(|actions| {
                    actions
                        .iter()
                        .map(|Action { key_input, .. }| key_input.keycode)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            events,
        );
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
                        if *is_valid {
                            if event_wrapper.is_held(&key_input.keycode, events) {
                                if !event_wrapper.is_pressed(&key_input.keycode, events) {
                                    *changed_by = 0.0;
                                } else {
                                    *changed_by = 1.0;
                                }
                            } else if event_wrapper.is_released(&key_input.keycode, events) {
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
    }

    /// Returns the [Values] for the first action in the mapping with the given ID.
    pub fn get_action(&self, id: ID) -> Option<Values> {
        for (i, ..) in self.mapping.iter().enumerate() {
            if let Some(values) = self.get_action_in_set(i, id) {
                return Some(values);
            }
        }
        None
    }

    /// Returns the [Values] for the action with the given ID in the given set of mappings.
    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<Values> {
        if let Some(i) = self.mapping[action_set].iter().position(|act| act.id == id) {
            return Some(self.values[action_set][i]);
        }
        None
    }

    /// Adds a single keymap to the logic.
    pub fn add_key_map(&mut self, locus_idx: usize, keycode: KeyCode, id: ID) {
        if locus_idx >= self.mapping.len() {
            self.mapping.resize_with(locus_idx + 1, Default::default);
            self.values.resize_with(locus_idx + 1, Default::default);
        }
        self.mapping[locus_idx].push(Action {
            id,
            key_input: KeyInput { keycode },
            is_valid: false,
            input_type: InputType::Digital,
        });
        self.values[locus_idx].push(Values {
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

pub trait InputWrapper<KeyCode, InputHelper> {
    fn update(&mut self, keys: &[KeyCode], events: &InputHelper);
    fn is_pressed(&self, key: &KeyCode, events: &InputHelper) -> bool;
    fn is_released(&self, key: &KeyCode, events: &InputHelper) -> bool;
    fn is_held(&self, key: &KeyCode, events: &InputHelper) -> bool;
}

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode as MqKeyCode};
pub struct MacroquadInputWrapper {
    this_frame_inputs: Vec<MqKeyCode>,
    last_frame_inputs: Vec<MqKeyCode>,
}

impl MacroquadInputWrapper {
    pub fn new() -> Self {
        Self {
            this_frame_inputs: Vec::new(),
            last_frame_inputs: Vec::new(),
        }
    }
}

impl InputWrapper<MqKeyCode, ()> for MacroquadInputWrapper {
    fn update(&mut self, keys: &[MqKeyCode], _events: &()) {
        self.last_frame_inputs = std::mem::take(&mut self.this_frame_inputs);
        for key in keys.iter() {
            if is_key_pressed(*key) {
                self.this_frame_inputs.push(*key);
            }
        }
    }

    fn is_pressed(&self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_pressed(*key)
    }

    fn is_released(&self, key: &MqKeyCode, _events: &()) -> bool {
        let was_pressed = self
            .last_frame_inputs
            .iter()
            .position(|key_pressed| key == key_pressed)
            .is_some();
        let is_not_pressed = self
            .this_frame_inputs
            .iter()
            .position(|key_pressed| key == key_pressed)
            .is_none();
        was_pressed && is_not_pressed
    }

    fn is_held(&self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_down(*key)
    }
}

pub struct WinitInputWrapper {
    this_frame_keys: BTreeSet<VirtualKeyCode>,
    last_frame_keys: BTreeSet<VirtualKeyCode>,
}

impl InputWrapper<VirtualKeyCode, WinitInputHelper> for WinitInputWrapper {
    fn update(&mut self, keys: &[VirtualKeyCode], events: &WinitInputHelper) {
        self.last_frame_keys = std::mem::take(&mut self.this_frame_keys);
        for key in keys.iter() {
            if events.key_held(*key) {
                self.this_frame_keys.insert(*key);
            }
        }
    }

    fn is_held(&self, key: &VirtualKeyCode, events: &WinitInputHelper) -> bool {
        events.key_held(*key)
    }

    fn is_pressed(&self, key: &VirtualKeyCode, _events: &WinitInputHelper) -> bool {
        self.this_frame_keys.contains(key) && !self.last_frame_keys.contains(key)
    }

    fn is_released(&self, key: &VirtualKeyCode, events: &WinitInputHelper) -> bool {
        events.key_released(*key)
    }
}

#[cfg(feature = "bevy-engine")]
use bevy_input::{keyboard::KeyCode as BevyKeyCode, Input as BevyInput};

#[cfg(feature = "bevy-engine")]
pub struct BevyInputWrapper;

#[cfg(feature = "bevy-engine")]
impl InputWrapper<BevyKeyCode, BevyInput<BevyKeyCode>> for BevyInputWrapper {
    fn update(&mut self, _keys: &[BevyKeyCode], _events: &BevyInput<BevyKeyCode>) {}

    fn is_held(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.pressed(*key)
    }

    fn is_pressed(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_pressed(*key)
    }

    fn is_released(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_released(*key)
    }
}
