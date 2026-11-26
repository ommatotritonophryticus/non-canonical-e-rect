//! Which buttons play the game, per control scheme.
//!
//! Only the six things done in play can be moved. The pause button and the
//! menus keep fixed buttons on every platform, so however badly a layout is
//! rebound, the way back into the menu to fix it always works.
//!
//! A button is a `u16` the core never interprets: it is an index into the
//! frontend's own table of the buttons that platform has, which is also where
//! the names shown on the controls screen come from (see
//! [`crate::settings::SchemeInfo::buttons`]). A keyboard, a gamepad and a PSP
//! each number theirs differently, and none of that has to reach the core.
//!
//! Every action holds up to [`SLOTS`] buttons, and nothing stops one button
//! from standing on several actions - both kinds of duplicate are allowed on
//! purpose. The shipped layouts already use the first kind (a PSP jumps on
//! Cross or L), and the second is someone's choice to make, not a mistake to
//! refuse.

/// Buttons each action can hold at once.
pub const SLOTS: usize = 3;

/// No button in this slot.
pub const NONE: u16 = u16::MAX;

/// What a button can be bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Left,
    Right,
    Jump,
    Attack,
    Dash,
    Slam,
}

impl Action {
    /// Every action, in the order the controls screen lists them and the save
    /// formats store them. Never reorder: it is a stored format.
    pub const EVERY: [Action; 6] = [
        Action::Left,
        Action::Right,
        Action::Jump,
        Action::Attack,
        Action::Dash,
        Action::Slam,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Action::Left => "LEFT",
            Action::Right => "RIGHT",
            Action::Jump => "JUMP",
            Action::Attack => "ATTACK",
            Action::Dash => "DASH",
            Action::Slam => "SLAM",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Action::Left => 0,
            Action::Right => 1,
            Action::Jump => 2,
            Action::Attack => 3,
            Action::Dash => 4,
            Action::Slam => 5,
        }
    }
}

/// The buttons bound to every action, for one control scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bindings {
    /// By [`Action::index`], then by slot. [`NONE`] in an empty slot.
    pub buttons: [[u16; SLOTS]; Action::EVERY.len()],
}

impl Default for Bindings {
    /// Nothing bound at all.
    fn default() -> Self {
        Self { buttons: [[NONE; SLOTS]; Action::EVERY.len()] }
    }
}

impl Bindings {
    /// A layout written out action by action, in [`Action::EVERY`] order -
    /// what a frontend's default layouts are made with. `const`, so they can
    /// sit in the same static table as the rest of a scheme.
    pub const fn new(buttons: [[u16; SLOTS]; 6]) -> Self {
        Self { buttons }
    }

    pub fn get(&self, action: Action, slot: usize) -> u16 {
        self.buttons[action.index()][slot.min(SLOTS - 1)]
    }

    pub fn set(&mut self, action: Action, slot: usize, button: u16) {
        self.buttons[action.index()][slot.min(SLOTS - 1)] = button;
    }

    /// True when any button bound to `action` passes `test` - the frontend's
    /// "is it held" or "was it just pressed", whichever it is asking.
    pub fn any(&self, action: Action, test: impl Fn(u16) -> bool) -> bool {
        self.buttons[action.index()].iter().any(|&b| b != NONE && test(b))
    }

    /// Empties every slot holding a button the platform does not have - a save
    /// written by a build with a longer table, or edited by hand.
    pub fn sanitize(&mut self, button_count: usize) {
        for slots in self.buttons.iter_mut() {
            for b in slots.iter_mut() {
                if *b != NONE && *b as usize >= button_count {
                    *b = NONE;
                }
            }
        }
    }
}

/// The index of `name` in a frontend's table of button names, for writing
/// default layouts by name rather than by position. `const`, so a default
/// layout can be a static.
///
/// Panics - at compile time, when used in a const - on a name the table does
/// not have, which is what a typo in a default layout should do.
pub const fn button(names: &[&str], name: &str) -> u16 {
    let mut i = 0;
    while i < names.len() {
        if str_eq(names[i], name) {
            return i as u16;
        }
        i += 1;
    }
    panic!("no such button in the table")
}

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}
