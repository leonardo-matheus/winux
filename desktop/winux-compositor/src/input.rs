//! Input handling for the Winux compositor
//!
//! Handles keyboard, mouse/pointer, and touch input events.

use crate::config::InputConfig;
use crate::state::WinuxState;
use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
        TouchEvent,
    },
    input::{
        keyboard::{FilterResult, KeyboardHandle, XkbConfig},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerHandle},
    },
    utils::SERIAL_COUNTER,
};
use std::collections::HashSet;

/// Input handler for processing input events
#[derive(Debug)]
pub struct InputHandler {
    /// Keyboard configuration
    keyboard_config: KeyboardConfig,
    /// Pointer configuration
    pointer_config: PointerConfig,
    /// Touchpad configuration
    touchpad_config: TouchpadConfig,
    /// Currently pressed keys
    pressed_keys: HashSet<u32>,
    /// Current pointer position
    pointer_position: (f64, f64),
    /// Modifier state
    modifiers: Modifiers,
}

/// Keyboard configuration state
#[derive(Debug, Clone)]
struct KeyboardConfig {
    layout: String,
    variant: String,
    options: String,
    repeat_delay: u32,
    repeat_rate: u32,
}

/// Pointer configuration state
#[derive(Debug, Clone)]
struct PointerConfig {
    accel_speed: f64,
    natural_scroll: bool,
    left_handed: bool,
}

/// Touchpad configuration state
#[derive(Debug, Clone)]
struct TouchpadConfig {
    tap_to_click: bool,
    natural_scroll: bool,
    disable_while_typing: bool,
    accel_speed: f64,
}

/// Keyboard modifier state
#[derive(Debug, Default, Clone, Copy)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new(config: &InputConfig) -> Self {
        Self {
            keyboard_config: KeyboardConfig {
                layout: config.keyboard.layout.clone(),
                variant: config.keyboard.variant.clone(),
                options: config.keyboard.options.clone(),
                repeat_delay: config.keyboard.repeat_delay,
                repeat_rate: config.keyboard.repeat_rate,
            },
            pointer_config: PointerConfig {
                accel_speed: config.pointer.accel_speed,
                natural_scroll: config.pointer.natural_scroll,
                left_handed: config.pointer.left_handed,
            },
            touchpad_config: TouchpadConfig {
                tap_to_click: config.touchpad.tap_to_click,
                natural_scroll: config.touchpad.natural_scroll,
                disable_while_typing: config.touchpad.disable_while_typing,
                accel_speed: config.touchpad.accel_speed,
            },
            pressed_keys: HashSet::new(),
            pointer_position: (0.0, 0.0),
            modifiers: Modifiers::default(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &InputConfig) {
        self.keyboard_config = KeyboardConfig {
            layout: config.keyboard.layout.clone(),
            variant: config.keyboard.variant.clone(),
            options: config.keyboard.options.clone(),
            repeat_delay: config.keyboard.repeat_delay,
            repeat_rate: config.keyboard.repeat_rate,
        };
        self.pointer_config = PointerConfig {
            accel_speed: config.pointer.accel_speed,
            natural_scroll: config.pointer.natural_scroll,
            left_handed: config.pointer.left_handed,
        };
        self.touchpad_config = TouchpadConfig {
            tap_to_click: config.touchpad.tap_to_click,
            natural_scroll: config.touchpad.natural_scroll,
            disable_while_typing: config.touchpad.disable_while_typing,
            accel_speed: config.touchpad.accel_speed,
        };
    }

    /// Get XKB configuration for keyboard setup
    pub fn xkb_config(&self) -> XkbConfig<'_> {
        XkbConfig {
            layout: &self.keyboard_config.layout,
            variant: &self.keyboard_config.variant,
            options: Some(self.keyboard_config.options.clone()),
            ..Default::default()
        }
    }

    /// Get current pointer position
    pub fn pointer_position(&self) -> (f64, f64) {
        self.pointer_position
    }

    /// Get current modifier state
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, keycode: u32) -> bool {
        self.pressed_keys.contains(&keycode)
    }

    /// Apply pointer acceleration
    fn apply_pointer_accel(&self, delta: (f64, f64)) -> (f64, f64) {
        let factor = 1.0 + self.pointer_config.accel_speed;
        (delta.0 * factor, delta.1 * factor)
    }

    /// Apply natural scroll if enabled
    fn apply_natural_scroll(&self, delta: f64, is_touchpad: bool) -> f64 {
        let natural = if is_touchpad {
            self.touchpad_config.natural_scroll
        } else {
            self.pointer_config.natural_scroll
        };

        if natural {
            -delta
        } else {
            delta
        }
    }
}

/// Process input events from a backend
pub fn process_input_event<B: InputBackend>(state: &mut WinuxState, event: InputEvent<B>) {
    match event {
        InputEvent::Keyboard { event } => handle_keyboard_event(state, event),
        InputEvent::PointerMotion { event } => handle_pointer_motion(state, event),
        InputEvent::PointerMotionAbsolute { event } => handle_pointer_motion_absolute(state, event),
        InputEvent::PointerButton { event } => handle_pointer_button(state, event),
        InputEvent::PointerAxis { event } => handle_pointer_axis(state, event),
        InputEvent::TouchDown { event } => handle_touch_down(state, event),
        InputEvent::TouchMotion { event } => handle_touch_motion(state, event),
        InputEvent::TouchUp { event } => handle_touch_up(state, event),
        InputEvent::TouchCancel { event } => handle_touch_cancel(state, event),
        InputEvent::TouchFrame { event } => handle_touch_frame(state, event),
        _ => {
            tracing::trace!("Unhandled input event");
        }
    }
}

/// Handle keyboard events
fn handle_keyboard_event<B: InputBackend>(state: &mut WinuxState, event: B::KeyboardKeyEvent) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = Event::time_msec(&event);
    let keycode = event.key_code();
    let key_state = event.state();

    // Track pressed keys
    match key_state {
        KeyState::Pressed => {
            state.input_handler.pressed_keys.insert(keycode);
        }
        KeyState::Released => {
            state.input_handler.pressed_keys.remove(&keycode);
        }
    }

    // Get keyboard handle and process
    if let Some(keyboard) = state.seat.get_keyboard() {
        // Check for compositor keybindings first
        let action = keyboard.input::<KeyAction, _>(
            state,
            keycode,
            key_state,
            serial,
            time,
            |state, modifiers, keysym| {
                // Update modifier state
                state.input_handler.modifiers = Modifiers {
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    shift: modifiers.shift,
                    logo: modifiers.logo,
                };

                // Check for compositor keybindings
                if key_state == KeyState::Pressed {
                    if let Some(action) = check_keybinding(modifiers, keysym.modified_sym()) {
                        return FilterResult::Intercept(action);
                    }
                }

                FilterResult::Forward
            },
        );

        // Handle compositor actions
        if let Some(action) = action {
            handle_key_action(state, action);
        }
    }
}

/// Handle pointer motion events
fn handle_pointer_motion<B: InputBackend>(state: &mut WinuxState, event: B::PointerMotionEvent) {
    let delta = state.input_handler.apply_pointer_accel((event.delta_x(), event.delta_y()));

    let (x, y) = state.input_handler.pointer_position;
    let new_position = (x + delta.0, y + delta.1);

    // Clamp to output bounds (simplified - should check actual outputs)
    let clamped = (new_position.0.max(0.0), new_position.1.max(0.0));
    state.input_handler.pointer_position = clamped;

    if let Some(pointer) = state.pointer() {
        let serial = SERIAL_COUNTER.next_serial();
        let under = state
            .space
            .element_under(smithay::utils::Point::from(clamped))
            .map(|(w, loc)| (w.wl_surface().unwrap().clone(), loc));

        pointer.motion(
            state,
            under,
            &MotionEvent {
                location: clamped.into(),
                serial,
                time: Event::time_msec(&event),
            },
        );
    }
}

/// Handle absolute pointer motion events
fn handle_pointer_motion_absolute<B: InputBackend>(
    state: &mut WinuxState,
    event: B::PointerMotionAbsoluteEvent,
) {
    // Get output size (simplified - should use actual output)
    let output_size = (1920.0, 1080.0);
    let position = (
        event.x_transformed(output_size.0 as u32) as f64,
        event.y_transformed(output_size.1 as u32) as f64,
    );

    state.input_handler.pointer_position = position;

    if let Some(pointer) = state.pointer() {
        let serial = SERIAL_COUNTER.next_serial();
        let under = state
            .space
            .element_under(smithay::utils::Point::from(position))
            .map(|(w, loc)| (w.wl_surface().unwrap().clone(), loc));

        pointer.motion(
            state,
            under,
            &MotionEvent {
                location: position.into(),
                serial,
                time: Event::time_msec(&event),
            },
        );
    }
}

/// Handle pointer button events
fn handle_pointer_button<B: InputBackend>(state: &mut WinuxState, event: B::PointerButtonEvent) {
    if let Some(pointer) = state.pointer() {
        let serial = SERIAL_COUNTER.next_serial();

        // Map button for left-handed mode
        let button = if state.input_handler.pointer_config.left_handed {
            match event.button_code() {
                0x110 => 0x111, // BTN_LEFT -> BTN_RIGHT
                0x111 => 0x110, // BTN_RIGHT -> BTN_LEFT
                b => b,
            }
        } else {
            event.button_code()
        };

        let button_state = match event.state() {
            ButtonState::Pressed => smithay::backend::input::ButtonState::Pressed,
            ButtonState::Released => smithay::backend::input::ButtonState::Released,
        };

        pointer.button(
            state,
            &ButtonEvent {
                button,
                state: button_state,
                serial,
                time: Event::time_msec(&event),
            },
        );

        // Focus window on click
        if event.state() == ButtonState::Pressed {
            let position = state.input_handler.pointer_position;
            if let Some((window, _)) = state
                .space
                .element_under(smithay::utils::Point::from(position))
            {
                if let Some(surface) = window.wl_surface() {
                    state.set_focus(Some(surface.clone()));
                }
            }
        }
    }
}

/// Handle pointer axis (scroll) events
fn handle_pointer_axis<B: InputBackend>(state: &mut WinuxState, event: B::PointerAxisEvent) {
    if let Some(pointer) = state.pointer() {
        let source = event.source();
        let is_touchpad = matches!(source, AxisSource::Finger);

        let mut frame = AxisFrame::new(Event::time_msec(&event)).source(source);

        // Handle horizontal axis
        if let Some(amount) = event.amount(Axis::Horizontal) {
            let scroll = state.input_handler.apply_natural_scroll(amount, is_touchpad);
            frame = frame.value(Axis::Horizontal, scroll);

            if let Some(discrete) = event.amount_v120(Axis::Horizontal) {
                let discrete = state.input_handler.apply_natural_scroll(discrete as f64, is_touchpad);
                frame = frame.v120(Axis::Horizontal, discrete as i32);
            }
        }

        // Handle vertical axis
        if let Some(amount) = event.amount(Axis::Vertical) {
            let scroll = state.input_handler.apply_natural_scroll(amount, is_touchpad);
            frame = frame.value(Axis::Vertical, scroll);

            if let Some(discrete) = event.amount_v120(Axis::Vertical) {
                let discrete = state.input_handler.apply_natural_scroll(discrete as f64, is_touchpad);
                frame = frame.v120(Axis::Vertical, discrete as i32);
            }
        }

        pointer.axis(state, frame);
    }
}

/// Handle touch down events
fn handle_touch_down<B: InputBackend>(_state: &mut WinuxState, event: B::TouchDownEvent) {
    tracing::trace!("Touch down: slot={}", event.slot().unwrap_or_default());
    // TODO: Implement touch handling
}

/// Handle touch motion events
fn handle_touch_motion<B: InputBackend>(_state: &mut WinuxState, event: B::TouchMotionEvent) {
    tracing::trace!("Touch motion: slot={}", event.slot().unwrap_or_default());
    // TODO: Implement touch handling
}

/// Handle touch up events
fn handle_touch_up<B: InputBackend>(_state: &mut WinuxState, event: B::TouchUpEvent) {
    tracing::trace!("Touch up: slot={}", event.slot().unwrap_or_default());
    // TODO: Implement touch handling
}

/// Handle touch cancel events
fn handle_touch_cancel<B: InputBackend>(_state: &mut WinuxState, _event: B::TouchCancelEvent) {
    tracing::trace!("Touch cancel");
    // TODO: Implement touch handling
}

/// Handle touch frame events
fn handle_touch_frame<B: InputBackend>(_state: &mut WinuxState, _event: B::TouchFrameEvent) {
    tracing::trace!("Touch frame");
    // TODO: Implement touch handling
}

/// Actions that can be triggered by keybindings
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// Quit the compositor
    Quit,
    /// Switch to VT
    VtSwitch(i32),
    /// Close focused window
    CloseWindow,
    /// Toggle fullscreen
    ToggleFullscreen,
    /// Launch terminal
    LaunchTerminal,
    /// Launch application launcher
    LaunchLauncher,
    /// Reload configuration
    ReloadConfig,
    /// Screenshot
    Screenshot,
}

/// Check if a keybinding matches
fn check_keybinding(
    modifiers: &smithay::input::keyboard::ModifiersState,
    keysym: smithay::input::keyboard::Keysym,
) -> Option<KeyAction> {
    use smithay::input::keyboard::Keysym;

    // Logo + Shift + Q: Quit
    if modifiers.logo && modifiers.shift && keysym == Keysym::q {
        return Some(KeyAction::Quit);
    }

    // Logo + Q: Close window
    if modifiers.logo && !modifiers.shift && keysym == Keysym::q {
        return Some(KeyAction::CloseWindow);
    }

    // Logo + Enter: Launch terminal
    if modifiers.logo && keysym == Keysym::Return {
        return Some(KeyAction::LaunchTerminal);
    }

    // Logo + D: Launch launcher
    if modifiers.logo && keysym == Keysym::d {
        return Some(KeyAction::LaunchLauncher);
    }

    // Logo + F: Toggle fullscreen
    if modifiers.logo && keysym == Keysym::f {
        return Some(KeyAction::ToggleFullscreen);
    }

    // Logo + Shift + R: Reload config
    if modifiers.logo && modifiers.shift && keysym == Keysym::r {
        return Some(KeyAction::ReloadConfig);
    }

    // Print: Screenshot
    if keysym == Keysym::Print {
        return Some(KeyAction::Screenshot);
    }

    // Ctrl + Alt + F1-F12: VT switch
    if modifiers.ctrl && modifiers.alt {
        let vt = match keysym {
            Keysym::F1 => Some(1),
            Keysym::F2 => Some(2),
            Keysym::F3 => Some(3),
            Keysym::F4 => Some(4),
            Keysym::F5 => Some(5),
            Keysym::F6 => Some(6),
            Keysym::F7 => Some(7),
            Keysym::F8 => Some(8),
            Keysym::F9 => Some(9),
            Keysym::F10 => Some(10),
            Keysym::F11 => Some(11),
            Keysym::F12 => Some(12),
            _ => None,
        };
        if let Some(vt) = vt {
            return Some(KeyAction::VtSwitch(vt));
        }
    }

    None
}

/// Handle compositor key actions
fn handle_key_action(state: &mut WinuxState, action: KeyAction) {
    match action {
        KeyAction::Quit => {
            tracing::info!("Quit requested via keybinding");
            state.stop();
        }
        KeyAction::CloseWindow => {
            tracing::info!("Close window requested");
            // TODO: Close focused window
        }
        KeyAction::LaunchTerminal => {
            tracing::info!("Launch terminal requested");
            if let Err(e) = std::process::Command::new("winux-terminal").spawn() {
                tracing::error!("Failed to launch terminal: {}", e);
            }
        }
        KeyAction::LaunchLauncher => {
            tracing::info!("Launch launcher requested");
            if let Err(e) = std::process::Command::new("winux-shell").spawn() {
                tracing::error!("Failed to launch launcher: {}", e);
            }
        }
        KeyAction::ToggleFullscreen => {
            tracing::info!("Toggle fullscreen requested");
            // TODO: Toggle fullscreen for focused window
        }
        KeyAction::ReloadConfig => {
            tracing::info!("Reload config requested");
            if let Err(e) = state.reload_config() {
                tracing::error!("Failed to reload config: {}", e);
            }
        }
        KeyAction::VtSwitch(vt) => {
            tracing::info!("VT switch to {} requested", vt);
            // TODO: Implement VT switching
        }
        KeyAction::Screenshot => {
            tracing::info!("Screenshot requested");
            // TODO: Implement screenshot
        }
    }
}
