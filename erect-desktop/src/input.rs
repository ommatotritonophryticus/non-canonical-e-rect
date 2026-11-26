//! Keyboard and gamepad, collapsed into a per-frame snapshot of intents.
//!
//! The browser version was event-driven (keydown/keyup). macroquad and gilrs are
//! both polled, so instead of dispatching events we build one `InputFrame` per
//! rendered frame: `held` for movement, `pressed` for one-shot actions.
//!
//! What plays the game goes through the player's bindings (see
//! `erect_core::binds`): every key and pad button the game will bind has a
//! number here, the index into [`KEY_NAMES`] or [`PAD_NAMES`], and a binding is
//! a list of those. The menus and the pause button do not: they stay on fixed
//! keys, so a layout rebound into uselessness can always be walked back.

use erect_core::binds::{button, Action, Bindings, NONE};
use erect_core::config::MAX_PLAYERS;
// Only the physical-pad path reads the deadzone, and a browser has no pads.
#[cfg(not(target_arch = "wasm32"))]
use erect_core::config::GAMEPAD_DEADZONE;
use erect_core::input::{InputFrame, MenuIntent, PlayerIntent};
use erect_core::settings::SchemeInfo;
#[cfg(not(target_arch = "wasm32"))]
use gilrs::{Axis, Button, Gilrs};
use macroquad::prelude::*;

/// Every key a binding may use, by the number the binding stores, with the
/// name the controls screen shows for it. Escape is missing on purpose - it is
/// pause, and pause cannot be moved - and so are the function keys, which the
/// frontend keeps for itself (F2 toggles the touch layout).
pub const KEY_NAMES: [&str; 62] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
    "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "0", "1", "2", "3", "4", "5", "6", "7",
    "8", "9", "SPACE", "ENTER", "TAB", "BKSP", "LSHIFT", "RSHIFT", "LCTRL", "RCTRL",
    "LALT", "RALT", "UP", "DOWN", "LEFT", "RIGHT", ",", ".", "/", ";", "'", "[", "]", "-",
    "=", "\\", "`", "DEL",
];

/// The keys [`KEY_NAMES`] names, in the same order.
const KEY_CODES: [KeyCode; 62] = [
    KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E, KeyCode::F, KeyCode::G,
    KeyCode::H, KeyCode::I, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::M, KeyCode::N,
    KeyCode::O, KeyCode::P, KeyCode::Q, KeyCode::R, KeyCode::S, KeyCode::T, KeyCode::U,
    KeyCode::V, KeyCode::W, KeyCode::X, KeyCode::Y, KeyCode::Z, KeyCode::Key0, KeyCode::Key1,
    KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6, KeyCode::Key7,
    KeyCode::Key8, KeyCode::Key9, KeyCode::Space, KeyCode::Enter, KeyCode::Tab,
    KeyCode::Backspace, KeyCode::LeftShift, KeyCode::RightShift, KeyCode::LeftControl,
    KeyCode::RightControl, KeyCode::LeftAlt, KeyCode::RightAlt, KeyCode::Up, KeyCode::Down,
    KeyCode::Left, KeyCode::Right, KeyCode::Comma, KeyCode::Period, KeyCode::Slash,
    KeyCode::Semicolon, KeyCode::Apostrophe, KeyCode::LeftBracket, KeyCode::RightBracket,
    KeyCode::Minus, KeyCode::Equal, KeyCode::Backslash, KeyCode::GraveAccent,
    KeyCode::Delete,
];

/// Every pad input a binding may use, named as on an Xbox-layout pad. The
/// stick's four directions count as buttons, so walking can be bound to the
/// stick, the d-pad, both, or neither. Start is pause and is not here.
pub const PAD_NAMES: [&str; 19] = [
    "A", "B", "X", "Y", "LB", "RB", "LT", "RT", "SELECT", "L3", "R3", "D-UP", "D-DOWN",
    "D-LEFT", "D-RIGHT", "STICK UP", "STICK DOWN", "STICK LEFT", "STICK RIGHT",
];

/// What each entry of [`PAD_NAMES`] reads on the pad.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy)]
enum PadInput {
    Button(Button),
    /// The left stick past the deadzone, along one axis, one way.
    Stick { axis: Axis, positive: bool },
}

#[cfg(not(target_arch = "wasm32"))]
const PAD_INPUTS: [PadInput; 19] = [
    PadInput::Button(Button::South),
    PadInput::Button(Button::East),
    PadInput::Button(Button::West),
    PadInput::Button(Button::North),
    PadInput::Button(Button::LeftTrigger),
    PadInput::Button(Button::RightTrigger),
    PadInput::Button(Button::LeftTrigger2),
    PadInput::Button(Button::RightTrigger2),
    PadInput::Button(Button::Select),
    PadInput::Button(Button::LeftThumb),
    PadInput::Button(Button::RightThumb),
    PadInput::Button(Button::DPadUp),
    PadInput::Button(Button::DPadDown),
    PadInput::Button(Button::DPadLeft),
    PadInput::Button(Button::DPadRight),
    // gilrs reports stick up as positive Y, unlike the browser's axis.
    PadInput::Stick { axis: Axis::LeftStickY, positive: true },
    PadInput::Stick { axis: Axis::LeftStickY, positive: false },
    PadInput::Stick { axis: Axis::LeftStickX, positive: false },
    PadInput::Stick { axis: Axis::LeftStickX, positive: true },
];

const fn k(name: &str) -> u16 {
    button(&KEY_NAMES, name)
}

const fn p(name: &str) -> u16 {
    button(&PAD_NAMES, name)
}

/// One button for an action and nothing else.
const fn one(b: u16) -> [u16; 3] {
    [b, NONE, NONE]
}

/// The layout every pad starts with: what the pads always did.
const PAD_DEFAULTS: Bindings = Bindings::new([
    [p("D-LEFT"), p("STICK LEFT"), NONE],
    [p("D-RIGHT"), p("STICK RIGHT"), NONE],
    one(p("A")),
    one(p("X")),
    [p("LB"), p("RB"), NONE],
    // Slam is B or down, whichever the player reaches for.
    [p("B"), p("D-DOWN"), p("STICK DOWN")],
]);

/// What this platform offers. Index order is what gets stored in settings.
pub static SCHEMES: [SchemeInfo; 4] = [
    SchemeInfo {
        label: "W A S D + SPACE",
        is_gamepad: false,
        pad_index: 0,
        buttons: &KEY_NAMES,
        defaults: Bindings::new([
            one(k("A")),
            one(k("D")),
            one(k("W")),
            one(k("SPACE")),
            one(k("LSHIFT")),
            one(k("S")),
        ]),
    },
    SchemeInfo {
        label: "ARROWS + ENTER",
        is_gamepad: false,
        pad_index: 0,
        buttons: &KEY_NAMES,
        // The second keyboard player gets the other Shift, which is the only
        // key on that side of the board that pairs with the arrows the way
        // LeftShift does with WASD.
        defaults: Bindings::new([
            one(k("LEFT")),
            one(k("RIGHT")),
            one(k("UP")),
            one(k("ENTER")),
            one(k("RSHIFT")),
            one(k("DOWN")),
        ]),
    },
    SchemeInfo {
        label: "GAMEPAD 1",
        is_gamepad: true,
        pad_index: 0,
        buttons: &PAD_NAMES,
        defaults: PAD_DEFAULTS,
    },
    SchemeInfo {
        label: "GAMEPAD 2",
        is_gamepad: true,
        pad_index: 1,
        buttons: &PAD_NAMES,
        defaults: PAD_DEFAULTS,
    },
];

/// Which [`PAD_NAMES`] entries the menus read, whatever the bindings say.
const PAD_MENU_UP: [u16; 2] = [p("D-UP"), p("STICK UP")];
const PAD_MENU_DOWN: [u16; 2] = [p("D-DOWN"), p("STICK DOWN")];
const PAD_MENU_LEFT: [u16; 2] = [p("D-LEFT"), p("STICK LEFT")];
const PAD_MENU_RIGHT: [u16; 2] = [p("D-RIGHT"), p("STICK RIGHT")];
const PAD_MENU_CONFIRM: [u16; 2] = [p("A"), p("X")];
const PAD_MENU_BACK: [u16; 1] = [p("B")];

fn key_down(b: u16) -> bool {
    KEY_CODES.get(b as usize).is_some_and(|k| is_key_down(*k))
}

fn key_pressed(b: u16) -> bool {
    KEY_CODES.get(b as usize).is_some_and(|k| is_key_pressed(*k))
}

/// Level state of one gamepad, so edges can be derived frame to frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PadState {
    /// By [`PAD_NAMES`] index.
    held: [bool; PAD_NAMES.len()],
    /// Start, apart from the rest: it is pause, and never bindable.
    start: bool,
}

pub struct InputReader {
    /// Absent in a browser: there is no gamepad subsystem to ask, and the pad
    /// on screen is what serves that role there.
    #[cfg(not(target_arch = "wasm32"))]
    gilrs: Option<Gilrs>,
    prev: [PadState; MAX_PLAYERS],
    cur: [PadState; MAX_PLAYERS],
    connected: [bool; MAX_PLAYERS],
}

impl InputReader {
    pub fn new() -> Self {
        Self {
            // A missing gamepad subsystem must not stop the game from running.
            #[cfg(not(target_arch = "wasm32"))]
            gilrs: Gilrs::new().ok(),
            prev: [PadState::default(); MAX_PLAYERS],
            cur: [PadState::default(); MAX_PLAYERS],
            connected: [false; MAX_PLAYERS],
        }
    }

    fn poll_pads(&mut self) {
        self.prev = self.cur;
        self.cur = [PadState::default(); MAX_PLAYERS];
        self.connected = [false; MAX_PLAYERS];
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_gilrs();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_gilrs(&mut self) {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return;
        };
        // Drain the event queue so gilrs keeps its internal state current.
        while gilrs.next_event().is_some() {}

        for (slot, (_id, pad)) in gilrs.gamepads().enumerate() {
            if slot >= MAX_PLAYERS {
                break;
            }
            self.connected[slot] = true;
            let state = &mut self.cur[slot];
            for (held, input) in state.held.iter_mut().zip(PAD_INPUTS.iter()) {
                *held = match *input {
                    PadInput::Button(b) => pad.is_pressed(b),
                    PadInput::Stick { axis, positive } => {
                        let v = pad.value(axis);
                        if positive {
                            v > GAMEPAD_DEADZONE
                        } else {
                            v < -GAMEPAD_DEADZONE
                        }
                    }
                };
            }
            state.start = pad.is_pressed(Button::Start);
        }
    }

    fn pad_down(&self, slot: usize, b: u16) -> bool {
        self.cur[slot].held.get(b as usize).copied().unwrap_or(false)
    }

    fn pad_pressed(&self, slot: usize, b: u16) -> bool {
        let i = b as usize;
        i < PAD_NAMES.len() && self.cur[slot].held[i] && !self.prev[slot].held[i]
    }

    fn pad_any_pressed(&self, slot: usize, buttons: &[u16]) -> bool {
        buttons.iter().any(|&b| self.pad_pressed(slot, b))
    }

    /// The first bindable button pressed this frame on a pad, for the controls
    /// screen.
    fn pad_raw(&self, slot: usize) -> Option<u16> {
        (0..PAD_NAMES.len() as u16).find(|&b| self.pad_pressed(slot, b))
    }

    /// `controls` is each player's scheme and the layout it plays with, in
    /// player order - `Game::player_controls`.
    pub fn read(&mut self, controls: &[(usize, Bindings)]) -> InputFrame {
        self.poll_pads();

        let mut frame = InputFrame {
            pads_connected: self.connected,
            ..Default::default()
        };

        // Menus accept any keyboard and any pad, so a player can always navigate
        // regardless of what they picked in settings - or bound.
        frame.menu = MenuIntent {
            up: is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
            down: is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
            left: is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
            right: is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
            confirm: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space),
            back: is_key_pressed(KeyCode::Escape),
        };
        frame.pause = is_key_pressed(KeyCode::Escape);
        // Nothing a player reaches for, and the core only acts on it from the
        // title screen.
        frame.dev_menu = is_key_pressed(KeyCode::Key8);

        for slot in 0..MAX_PLAYERS {
            if !self.connected[slot] {
                continue;
            }
            frame.menu.up |= self.pad_any_pressed(slot, &PAD_MENU_UP);
            frame.menu.down |= self.pad_any_pressed(slot, &PAD_MENU_DOWN);
            frame.menu.left |= self.pad_any_pressed(slot, &PAD_MENU_LEFT);
            frame.menu.right |= self.pad_any_pressed(slot, &PAD_MENU_RIGHT);
            frame.menu.confirm |= self.pad_any_pressed(slot, &PAD_MENU_CONFIRM);
            frame.menu.back |= self.pad_any_pressed(slot, &PAD_MENU_BACK);
            frame.pause |= self.cur[slot].start && !self.prev[slot].start;
        }

        // What each scheme's device just had pressed, for the controls screen.
        let key_raw = (0..KEY_NAMES.len() as u16).find(|&b| key_pressed(b));
        for (raw, scheme) in frame.raw.iter_mut().zip(SCHEMES.iter()) {
            *raw = if scheme.is_gamepad {
                if scheme.pad_index < MAX_PLAYERS && self.connected[scheme.pad_index] {
                    self.pad_raw(scheme.pad_index)
                } else {
                    None
                }
            } else {
                key_raw
            };
        }

        for (player_index, (scheme_index, binds)) in controls.iter().enumerate().take(MAX_PLAYERS) {
            let scheme = &SCHEMES[(*scheme_index).min(SCHEMES.len() - 1)];
            frame.players[player_index] = if scheme.is_gamepad {
                let slot = scheme.pad_index;
                if slot >= MAX_PLAYERS || !self.connected[slot] {
                    PlayerIntent::default()
                } else {
                    intent(binds, |b| self.pad_down(slot, b), |b| self.pad_pressed(slot, b))
                }
            } else {
                intent(binds, key_down, key_pressed)
            };
        }

        frame
    }
}

/// One player's intents, read through their bindings: held for walking and
/// for a held attack, pressed for everything that happens once.
fn intent(
    binds: &Bindings,
    down: impl Fn(u16) -> bool,
    pressed: impl Fn(u16) -> bool,
) -> PlayerIntent {
    PlayerIntent {
        left: binds.any(Action::Left, &down),
        right: binds.any(Action::Right, &down),
        jump: binds.any(Action::Jump, &pressed),
        slam: binds.any(Action::Slam, &pressed),
        attack: binds.any(Action::Attack, &pressed),
        attack_held: binds.any(Action::Attack, &down),
        dash: binds.any(Action::Dash, &pressed),
    }
}
