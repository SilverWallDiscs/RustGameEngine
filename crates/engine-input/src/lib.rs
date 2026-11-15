#![forbid(unsafe_code)]
//! Platform input collection and action mapping using winit.
//!
//! # Examples
//!
//! ~~~rust
//! use engine_input::{ActionMap, InputState};
//! use winit::keyboard::KeyCode;
//! let mut actions = ActionMap::default();
//! actions.bind_key("jump", KeyCode::Space);
//! assert!(!actions.action_down("jump", &InputState::default()));
//! ~~~

use std::collections::{HashMap, HashSet};
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

/// Keyboard and mouse state accumulated from window events.
///
/// Edge-triggered states and motion deltas are cleared by end_frame.
///
/// # Examples
///
/// ~~~rust
/// use engine_input::InputState;
/// use winit::keyboard::KeyCode;
/// assert!(!InputState::default().is_key_down(KeyCode::Space));
/// ~~~
#[derive(Debug, Default)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    mouse_down: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    cursor: Option<(f64, f64)>,
    cursor_delta: (f64, f64),
    scroll: (f32, f32),
    focused: bool,
}

impl InputState {
    /// Applies one winit event to the current input snapshot.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::event::WindowEvent;
    /// let mut input = InputState::default();
    /// input.process_window_event(&WindowEvent::Focused(true));
    /// assert!(input.is_focused());
    /// ~~~
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(focused) => {
                self.focused = *focused;
                if !focused { self.release_all(); }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            if self.keys_down.insert(key) { self.keys_pressed.insert(key); }
                        }
                        ElementState::Released => {
                            if self.keys_down.remove(&key) { self.keys_released.insert(key); }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if self.mouse_down.insert(*button) { self.mouse_pressed.insert(*button); }
                }
                ElementState::Released => {
                    if self.mouse_down.remove(button) { self.mouse_released.insert(*button); }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let next = (position.x, position.y);
                if let Some(previous) = self.cursor {
                    self.cursor_delta.0 += next.0 - previous.0;
                    self.cursor_delta.1 += next.1 - previous.1;
                }
                self.cursor = Some(next);
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.scroll.0 += *x;
                    self.scroll.1 += *y;
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    self.scroll.0 += pos.x as f32;
                    self.scroll.1 += pos.y as f32;
                }
            },
            _ => {}
        }
    }

    /// Returns true while the given physical key is held.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::keyboard::KeyCode;
    /// assert!(!InputState::default().is_key_down(KeyCode::KeyW));
    /// ~~~
    pub fn is_key_down(&self, key: KeyCode) -> bool { self.keys_down.contains(&key) }

    /// Returns true only on the frame a key becomes pressed.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::keyboard::KeyCode;
    /// assert!(!InputState::default().was_key_pressed(KeyCode::Space));
    /// ~~~
    pub fn was_key_pressed(&self, key: KeyCode) -> bool { self.keys_pressed.contains(&key) }

    /// Returns true only on the frame a key becomes released.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::keyboard::KeyCode;
    /// assert!(!InputState::default().was_key_released(KeyCode::Space));
    /// ~~~
    pub fn was_key_released(&self, key: KeyCode) -> bool { self.keys_released.contains(&key) }

    /// Returns true while a mouse button is held.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::event::MouseButton;
    /// assert!(!InputState::default().is_mouse_button_down(MouseButton::Left));
    /// ~~~
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_down.contains(&button)
    }

    /// Returns true on the frame a mouse button becomes pressed.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::event::MouseButton;
    /// assert!(!InputState::default().was_mouse_button_pressed(MouseButton::Left));
    /// ~~~
    pub fn was_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    /// Returns true on the frame a mouse button becomes released.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// use winit::event::MouseButton;
    /// assert!(!InputState::default().was_mouse_button_released(MouseButton::Left));
    /// ~~~
    pub fn was_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }

    /// Returns the latest cursor position in physical pixels.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// assert_eq!(InputState::default().cursor_position(), None);
    /// ~~~
    pub fn cursor_position(&self) -> Option<(f64, f64)> { self.cursor }

    /// Returns accumulated cursor movement since the last end_frame call.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// assert_eq!(InputState::default().cursor_delta(), (0.0, 0.0));
    /// ~~~
    pub fn cursor_delta(&self) -> (f64, f64) { self.cursor_delta }

    /// Returns horizontal and vertical wheel movement accumulated this frame.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// assert_eq!(InputState::default().scroll_delta(), (0.0, 0.0));
    /// ~~~
    pub fn scroll_delta(&self) -> (f32, f32) { self.scroll }

    /// Returns whether the window currently has focus.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// assert!(!InputState::default().is_focused());
    /// ~~~
    pub fn is_focused(&self) -> bool { self.focused }

    /// Clears one-frame transitions and motion while preserving held keys/buttons.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::InputState;
    /// let mut input = InputState::default();
    /// input.end_frame();
    /// assert_eq!(input.cursor_delta(), (0.0, 0.0));
    /// ~~~
    pub fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.cursor_delta = (0.0, 0.0);
        self.scroll = (0.0, 0.0);
    }

    fn release_all(&mut self) {
        self.keys_released.extend(self.keys_down.drain());
        self.mouse_released.extend(self.mouse_down.drain());
    }
}

#[derive(Debug, Default)]
struct AxisBinding {
    negative: HashSet<KeyCode>,
    positive: HashSet<KeyCode>,
}

/// Maps named game actions and digital axes to physical keys.
///
/// # Examples
///
/// ~~~rust
/// use engine_input::ActionMap;
/// use winit::keyboard::KeyCode;
/// let mut map = ActionMap::default();
/// map.bind_key("interact", KeyCode::KeyE);
/// map.bind_axis("move_x", KeyCode::KeyA, KeyCode::KeyD);
/// ~~~
#[derive(Debug, Default)]
pub struct ActionMap {
    actions: HashMap<String, HashSet<KeyCode>>,
    axes: HashMap<String, AxisBinding>,
}

impl ActionMap {
    /// Binds a key to a named action.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::ActionMap;
    /// use winit::keyboard::KeyCode;
    /// let mut map = ActionMap::default();
    /// map.bind_key("jump", KeyCode::Space);
    /// ~~~
    pub fn bind_key(&mut self, action: impl Into<String>, key: KeyCode) {
        self.actions.entry(action.into()).or_default().insert(key);
    }

    /// Binds negative and positive keys to a named axis.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::ActionMap;
    /// use winit::keyboard::KeyCode;
    /// let mut map = ActionMap::default();
    /// map.bind_axis("move_x", KeyCode::KeyA, KeyCode::KeyD);
    /// ~~~
    pub fn bind_axis(&mut self, axis: impl Into<String>, negative: KeyCode, positive: KeyCode) {
        let binding = self.axes.entry(axis.into()).or_default();
        binding.negative.insert(negative);
        binding.positive.insert(positive);
    }

    /// Returns true if any key bound to the action is held.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::{ActionMap, InputState};
    /// assert!(!ActionMap::default().action_down("jump", &InputState::default()));
    /// ~~~
    pub fn action_down(&self, action: &str, input: &InputState) -> bool {
        self.actions.get(action).is_some_and(|keys| keys.iter().any(|key| input.is_key_down(*key)))
    }

    /// Returns true on the frame a bound action key is pressed.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::{ActionMap, InputState};
    /// assert!(!ActionMap::default().action_pressed("jump", &InputState::default()));
    /// ~~~
    pub fn action_pressed(&self, action: &str, input: &InputState) -> bool {
        self.actions.get(action).is_some_and(|keys| keys.iter().any(|key| input.was_key_pressed(*key)))
    }

    /// Returns -1, 0, or 1 for a named digital axis.
    ///
    /// Opposite held directions cancel each other.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_input::{ActionMap, InputState};
    /// assert_eq!(ActionMap::default().axis_value("move_x", &InputState::default()), 0.0);
    /// ~~~
    pub fn axis_value(&self, axis: &str, input: &InputState) -> f32 {
        let Some(binding) = self.axes.get(axis) else { return 0.0; };
        let negative = binding.negative.iter().any(|key| input.is_key_down(*key));
        let positive = binding.positive.iter().any(|key| input.is_key_down(*key));
        match (negative, positive) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbound_actions_and_axes_are_neutral() {
        let input = InputState::default();
        let map = ActionMap::default();
        assert!(!map.action_down("missing", &input));
        assert!(!map.action_pressed("missing", &input));
        assert_eq!(map.axis_value("missing", &input), 0.0);
    }

    #[test]
    fn binding_an_action_does_not_trigger_without_input() {
        let input = InputState::default();
        let mut map = ActionMap::default();
        map.bind_key("jump", KeyCode::Space);
        assert!(!map.action_down("jump", &input));
    }
}
