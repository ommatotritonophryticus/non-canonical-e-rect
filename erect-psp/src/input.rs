//! sceCtrl, turned into the core's [`InputFrame`].
//!
//! The PSP has one controller, so there is exactly one scheme and only ever one
//! player.
//!
//! What plays the game goes through the player's bindings (see
//! `erect_core::binds`): every button the game will bind has a number here,
//! its index in [`BUTTON_NAMES`]. The menus and Start do not: they keep fixed
//! buttons, so a layout rebound into uselessness can always be walked back.

use erect_core::binds::{button, Action, Bindings, NONE};
use erect_core::config::GAMEPAD_DEADZONE;
use erect_core::input::{InputFrame, MenuIntent, PlayerIntent};
use erect_core::settings::SchemeInfo;
use psp::sys::{self, CtrlButtons, CtrlMode, SceCtrlData};

/// Every input a binding may use, with the name the controls screen shows.
/// The nub's four directions count as buttons, so walking can be bound to the
/// nub, the d-pad, both or neither. Start is pause and Home belongs to the
/// system, so neither is here.
pub const BUTTON_NAMES: [&str; 15] = [
    "CROSS", "CIRCLE", "SQUARE", "TRIANGLE", "L", "R", "UP", "DOWN", "LEFT", "RIGHT",
    "SELECT", "NUB UP", "NUB DOWN", "NUB LEFT", "NUB RIGHT",
];

/// What each entry of [`BUTTON_NAMES`] reads. The first eleven are buttons;
/// the nub directions follow and are worked out from the stick's position.
const BUTTON_BITS: [CtrlButtons; 11] = [
    CtrlButtons::CROSS,
    CtrlButtons::CIRCLE,
    CtrlButtons::SQUARE,
    CtrlButtons::TRIANGLE,
    CtrlButtons::LTRIGGER,
    CtrlButtons::RTRIGGER,
    CtrlButtons::UP,
    CtrlButtons::DOWN,
    CtrlButtons::LEFT,
    CtrlButtons::RIGHT,
    CtrlButtons::SELECT,
];

const fn b(name: &str) -> u16 {
    button(&BUTTON_NAMES, name)
}

/// One pad, so one scheme. This is also what makes the core hide 2-player mode.
///
/// The layout it ships with is the one the PSP always had: L doubles jump and
/// R doubles dash, so both can be had without taking a thumb off the d-pad or
/// the face buttons; slam is Circle or down, whichever the player reaches for.
pub static SCHEMES: [SchemeInfo; 1] = [SchemeInfo {
    label: "PSP PAD",
    is_gamepad: true,
    pad_index: 0,
    buttons: &BUTTON_NAMES,
    defaults: Bindings::new([
        [b("LEFT"), b("NUB LEFT"), NONE],
        [b("RIGHT"), b("NUB RIGHT"), NONE],
        [b("CROSS"), b("L"), NONE],
        [b("SQUARE"), NONE, NONE],
        [b("TRIANGLE"), b("R"), NONE],
        [b("CIRCLE"), b("DOWN"), NONE],
    ]),
}];

/// Held state of every bindable input, plus the ones the menus and pause read.
#[derive(Clone, Copy, Default, PartialEq)]
struct PadState {
    /// By [`BUTTON_NAMES`] index.
    held: [bool; BUTTON_NAMES.len()],
    start: bool,
}

impl PadState {
    fn get(&self, name: u16) -> bool {
        self.held.get(name as usize).copied().unwrap_or(false)
    }
}

pub struct Input {
    prev: PadState,
    cur: PadState,
}

impl Input {
    /// # Safety
    /// Initialises the controller driver; call once at startup.
    pub unsafe fn new() -> Self {
        unsafe {
            sys::sceCtrlSetSamplingCycle(0);
            sys::sceCtrlSetSamplingMode(CtrlMode::Analog);
        }
        Self {
            prev: PadState::default(),
            cur: PadState::default(),
        }
    }

    fn pressed(&self, name: u16) -> bool {
        self.cur.get(name) && !self.prev.get(name)
    }

    /// # Safety
    /// Reads the controller; call once per rendered frame.
    pub unsafe fn read(&mut self, binds: &Bindings) -> InputFrame {
        let mut pad = SceCtrlData::default();
        unsafe {
            sys::sceCtrlReadBufferPositive(&mut pad, 1);
        }

        // The analog nub reports 0..=255 with 128 at rest, on both axes, with
        // up as the low end.
        let nub_x = (pad.lx as f32 - 128.0) / 128.0;
        let nub_y = (pad.ly as f32 - 128.0) / 128.0;
        let buttons = pad.buttons;

        self.prev = self.cur;
        let mut held = [false; BUTTON_NAMES.len()];
        for (h, bit) in held.iter_mut().zip(BUTTON_BITS.iter()) {
            *h = buttons.contains(*bit);
        }
        held[b("NUB UP") as usize] = nub_y < -GAMEPAD_DEADZONE;
        held[b("NUB DOWN") as usize] = nub_y > GAMEPAD_DEADZONE;
        held[b("NUB LEFT") as usize] = nub_x < -GAMEPAD_DEADZONE;
        held[b("NUB RIGHT") as usize] = nub_x > GAMEPAD_DEADZONE;
        self.cur = PadState { held, start: buttons.contains(CtrlButtons::START) };

        let mut frame = InputFrame {
            pads_connected: [true, false],
            ..Default::default()
        };

        let down = |name: u16| self.cur.get(name);
        let pressed = |name: u16| self.pressed(name);
        frame.players[0] = PlayerIntent {
            left: binds.any(Action::Left, down),
            right: binds.any(Action::Right, down),
            jump: binds.any(Action::Jump, pressed),
            slam: binds.any(Action::Slam, pressed),
            attack: binds.any(Action::Attack, pressed),
            attack_held: binds.any(Action::Attack, down),
            dash: binds.any(Action::Dash, pressed),
        };

        // The menus keep their own buttons whatever the bindings say.
        frame.menu = MenuIntent {
            up: self.pressed(b("UP")),
            down: self.pressed(b("DOWN")),
            left: self.pressed(b("LEFT")) || self.pressed(b("NUB LEFT")),
            right: self.pressed(b("RIGHT")) || self.pressed(b("NUB RIGHT")),
            confirm: self.pressed(b("CROSS")) || self.pressed(b("SQUARE")),
            back: self.pressed(b("CIRCLE")),
        };
        frame.pause = self.cur.start && !self.prev.start;
        // Select held, then Down. A chord rather than a button, so it cannot be
        // hit by accident; the core only listens for it on the title screen.
        frame.dev_menu = self.cur.get(b("SELECT")) && self.pressed(b("DOWN"));
        // The first button just pressed, for the controls screen.
        frame.raw[0] = (0..BUTTON_NAMES.len() as u16).find(|&name| self.pressed(name));

        frame
    }
}
