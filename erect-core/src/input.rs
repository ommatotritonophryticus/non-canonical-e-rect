//! What the player is asking for this tick, as plain data.
//!
//! Frontends fill this in from whatever they have - macroquad key polling and
//! gilrs on a desktop, sceCtrl on a PSP. The core never touches a device.

use crate::config::MAX_PLAYERS;

/// Held state for movement, edge-triggered for one-shot actions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerIntent {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub slam: bool,
    pub attack: bool,
    /// The attack button held down, as opposed to the edge above.
    ///
    /// Only the thrown attack reads it, and only to keep firing. Every other
    /// kind wants exactly one swing per press: a held button that kept swinging
    /// would make the combo impossible to break out of.
    pub attack_held: bool,
    pub dash: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuIntent {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub back: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputFrame {
    pub players: [PlayerIntent; MAX_PLAYERS],
    pub menu: MenuIntent,
    pub pause: bool,
    /// Opens the developer menu from the title screen. A deliberately awkward
    /// chord on every platform: nothing behind it should be reachable by a
    /// player who was not looking for it.
    pub dev_menu: bool,
    /// Drives the "(OFF)" marker on the settings screen.
    pub pads_connected: [bool; MAX_PLAYERS],
}

impl InputFrame {
    /// Folds a second source of intents into this one. Nothing is exclusive:
    /// a keyboard and a drawn pad, or this frame and presses an earlier frame
    /// raised that no tick has taken yet, simply add up.
    pub fn merge(&mut self, from: &InputFrame) {
        for (dst, src) in self.players.iter_mut().zip(from.players.iter()) {
            dst.left |= src.left;
            dst.right |= src.right;
            dst.jump |= src.jump;
            dst.slam |= src.slam;
            dst.attack |= src.attack;
            dst.attack_held |= src.attack_held;
            dst.dash |= src.dash;
        }
        self.menu.up |= from.menu.up;
        self.menu.down |= from.menu.down;
        self.menu.left |= from.menu.left;
        self.menu.right |= from.menu.right;
        self.menu.confirm |= from.menu.confirm;
        self.menu.back |= from.menu.back;
        self.pause |= from.pause;
        self.dev_menu |= from.dev_menu;
    }

    /// The one-shot half of a frame, with the held half dropped.
    ///
    /// For a rendered frame that ran no tick - which a slowed world does more
    /// often than not. Held state is re-read every frame and must not be
    /// carried; edges are the opposite - they happen once and have to survive
    /// until a tick can take them.
    pub fn edges_only(&self) -> InputFrame {
        let mut kept = *self;
        for p in kept.players.iter_mut() {
            p.left = false;
            p.right = false;
            p.attack_held = false;
        }
        kept
    }

    /// One-shot intents must fire on exactly one simulation tick, even when a
    /// rendered frame ran several of them.
    pub fn clear_edges(&mut self) {
        for p in self.players.iter_mut() {
            p.jump = false;
            p.slam = false;
            p.attack = false;
            p.dash = false;
        }
        self.menu = MenuIntent::default();
        self.pause = false;
        self.dev_menu = false;
    }
}
