//! # Control logics
//!
//! Control logics communicate that different entities are controlled by different inputs at different times. They map button inputs, AI intentions, network socket messages, etc onto high-level game actions.
//!
//! We're currently trying to consider analog as well as digital inputs, but we haven't implemented controller support, so some of these fields don't really make sense yet.

use std::collections::BTreeSet;

use crate::{Event, Logic, LogicType, Reaction};

/// Information for a key/button press.
trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

/// A keyboard control logic.
///
/// A Wrapper is a helper struct that helps keep track of information that libraries may not but we do want, such as [BevyInputWrapper], [MacroquadInputWrapper], or [WinitInputWrapper].
pub struct KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    /// Input mappings from actions to keypresses. Each outer Vec is a set of inputs, ex. one player gets the first set of mappings, another gets a second set of mappings, an AI player gets the third.
    pub mapping: Vec<Vec<Action<ID, Wrapper::KeyCode>>>,
    /// The values for each keypress in the sets described above.
    pub values: Vec<Vec<Values>>,
    /// An input wrapper
    input_wrapper: Wrapper,
}

impl<ID, Wrapper> Default for KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    fn default() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            input_wrapper: Wrapper::new(),
        }
    }
}

impl<ID, Wrapper> Logic for KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    type Event = ControlEvent<ID>;
    type Reaction = ControlReaction<ID, Wrapper::KeyCode>;

    /// use KeyboardControl::update() instead???? maybe
    fn update(&mut self) {
        unimplemented!();
    }

    // eventually should do actual error handling for this, probably
    fn react(&mut self, reaction_type: Self::Reaction) {
        match reaction_type {
            ControlReaction::AddKeyToSet(idx, id, keycode, input_type) => {
                self.mapping[idx].push(Action::new(id, keycode, input_type))
            }
            ControlReaction::RemoveKeyFromSet(set_idx, id) => {
                if let Some(action_idx) = self.mapping[set_idx].iter().position(|act| act.id == id)
                {
                    self.mapping[set_idx].remove(action_idx);
                }
            }
            ControlReaction::SetKeyValid(set_idx, id) => {
                if let Some(action_idx) = self.mapping[set_idx].iter().position(|act| act.id == id)
                {
                    self.mapping[set_idx][action_idx].is_valid = true;
                }
            }
            ControlReaction::SetKeyInvalid(set_idx, id) => {
                if let Some(action_idx) = self.mapping[set_idx].iter().position(|act| act.id == id)
                {
                    self.mapping[set_idx][action_idx].is_valid = false;
                }
            }
        }
    }
}

impl<ID, Wrapper> KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks and updates what inputs are being pressed every frame.
    pub fn update_input(&mut self, events: &Wrapper::InputHelper) {
        self.input_wrapper.clear();
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values { value, changed_by } = &mut values;
                // if not valid, reset and skip check. could cause problems if a key were pressed before it became valid then the key became valid while still being held. this is probably semi-reasonable, actually
                if !*is_valid {
                    *value = 0.0;
                    *changed_by = 0.0;
                    continue;
                }
                match input_type {
                    InputType::Digital => {
                        // NOTE: if update_held isn't called for every key in the mappings, it can completely break some of the input wrappers.
                        //
                        // This feels easily broken... but it feels less weird than filtering out and looping through all inputs beforehand to see if they're held, _then_ calling is_held again---which is just doing the same thing twice?
                        if self.input_wrapper.update_held(&key_input.keycode, events) {
                            if self.input_wrapper.is_pressed(&key_input.keycode, events) {
                                *changed_by = 1.0;
                            } else {
                                *changed_by = 0.0;
                            }
                        } else if self.input_wrapper.is_released(&key_input.keycode, events)
                        // see comment earlier about keypresses that are invalid. logic may not be correct though
                            && *value != 0.0
                        {
                            *changed_by = -1.0;
                        } else {
                            *changed_by = 0.0;
                        }
                    }
                    InputType::Analog => unimplemented!(),
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
    pub fn add_key_map(&mut self, locus_idx: usize, keycode: Wrapper::KeyCode, id: ID) {
        if locus_idx >= self.mapping.len() {
            self.mapping.resize_with(locus_idx + 1, Default::default);
            self.values.resize_with(locus_idx + 1, Default::default);
        }
        self.mapping[locus_idx].push(Action::new(id, keycode, InputType::Digital));
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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum InputType {
    Analog,
    Digital,
}

/// Information about the player's input related to one action.
#[derive(Copy, Clone)]
pub struct Values {
    /// How much the value of the input was changed last frame.
    pub changed_by: f32,
    /// What the value of the input is now.
    pub value: f32,
}

/// Information for an action and the input it's attached to.
pub struct Action<ID, KeyCode> {
    pub id: ID,
    /// The input's keycode and min/max.
    pub key_input: KeyInput<KeyCode>,
    /// If the input is valid that frame, i.e. should be able to be pressed.
    pub is_valid: bool,
    /// If the input is digital or analog.
    pub input_type: InputType,
}

impl<ID, KeyCode> Action<ID, KeyCode> {
    pub fn new(id: ID, keycode: KeyCode, input_type: InputType) -> Self {
        Self {
            id,
            key_input: KeyInput { keycode },
            is_valid: true,
            input_type,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ControlReaction<ID: Copy + Eq, KeyCode: Copy + Eq> {
    AddKeyToSet(usize, ID, KeyCode, InputType),
    RemoveKeyFromSet(usize, ID),
    SetKeyValid(usize, ID),
    SetKeyInvalid(usize, ID),
}

impl<ID: Copy + Eq, KeyCode: Copy + Eq> Reaction for ControlReaction<ID, KeyCode> {
    fn for_logic(&self) -> LogicType {
        LogicType::Control
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ControlEvent<ID: Copy + Eq + Ord>(pub ID, pub ControlEventType);

#[derive(PartialEq, Eq, Debug)]
pub enum ControlEventType {
    KeyPressed,
    KeyReleased,
    KeyHeld,
}

impl<ID: Copy + Eq + Ord> Event for ControlEvent<ID> {
    fn for_logic(&self) -> LogicType {
        LogicType::Control
    }
}

/// A wrapper to help keep track of input information that preexisting input handlers may not offer, but that we need.
pub trait InputWrapper {
    type KeyCode: Copy + Eq;
    type InputHelper;
    fn new() -> Self;

    /// clears input information for this frame
    fn clear(&mut self);

    /// if the key is held or not. If keeping track of current inputs, also logs what keys are being pressed this frame.
    fn update_held(&mut self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;

    /// if the key has just been pressed or not
    fn is_pressed(&self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;

    /// if the key has just been released or not
    fn is_released(&self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;
}

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode as MqKeyCode};
/// Macroquad doesn't keep track of when keys are released, so track the keys pressed last and this frame.
pub struct MacroquadInputWrapper {
    this_frame_keys: Vec<MqKeyCode>,
    last_frame_keys: Vec<MqKeyCode>,
}

impl InputWrapper for MacroquadInputWrapper {
    type KeyCode = MqKeyCode;
    type InputHelper = ();
    fn new() -> Self {
        Self {
            this_frame_keys: Vec::new(),
            last_frame_keys: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.last_frame_keys = std::mem::take(&mut self.this_frame_keys);
    }

    fn update_held(&mut self, key: &MqKeyCode, _events: &()) -> bool {
        if is_key_down(*key) {
            self.this_frame_keys.push(*key);
            return true;
        }
        false
    }

    fn is_pressed(&self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_pressed(*key)
    }

    fn is_released(&self, key: &MqKeyCode, _events: &()) -> bool {
        let was_pressed = self
            .last_frame_keys
            .iter()
            .any(|key_pressed| key == key_pressed);
        let is_not_pressed = !self
            .this_frame_keys
            .iter()
            .any(|key_pressed| key == key_pressed);
        was_pressed && is_not_pressed
    }
}

use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

/// WinitInputHelper doesn't handle key repeat properly (may depend on system?), so track the keys pressed last and this frame.
pub struct WinitInputWrapper {
    this_frame_keys: BTreeSet<VirtualKeyCode>,
    last_frame_keys: BTreeSet<VirtualKeyCode>,
}

impl InputWrapper for WinitInputWrapper {
    type KeyCode = VirtualKeyCode;
    type InputHelper = WinitInputHelper;

    fn new() -> Self {
        Self {
            this_frame_keys: BTreeSet::new(),
            last_frame_keys: BTreeSet::new(),
        }
    }

    fn clear(&mut self) {
        self.last_frame_keys = std::mem::take(&mut self.this_frame_keys);
    }

    fn update_held(&mut self, key: &VirtualKeyCode, events: &WinitInputHelper) -> bool {
        if events.key_held(*key) {
            self.this_frame_keys.insert(*key);
            return true;
        }
        false
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
/// Bevy's input handler already correctly handles the information we need, so this is just a wrapper for their functions
pub struct BevyInputWrapper;

#[cfg(feature = "bevy-engine")]
impl InputWrapper for BevyInputWrapper {
    type KeyCode = BevyKeyCode;
    type InputHelper = BevyInput<BevyKeyCode>;

    fn new() -> Self {
        Self
    }

    fn clear(&mut self) {}

    fn update_held(&mut self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.pressed(*key)
    }

    fn is_pressed(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_pressed(*key)
    }

    fn is_released(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_released(*key)
    }
}
