//! Standing upgrades that are not weapons.
//!
//! An attack is exclusive - carrying one means not carrying the rest, and
//! swapping costs whatever level the old one had reached. A boon is nothing
//! like that: it is kept, it costs nothing to take, and it never comes up
//! again. The two sit in the same offer anyway, which means a boon on the table
//! is always the safe pick. What keeps that from swallowing the weapon roster
//! is that the pool drains: a handful of boons exist, and once they are held
//! the offer has nothing left to give but weapons.
//!
//! Three of them have a second level, offered only once the first is held. A
//! second level keeps everything the first did and adds one thing: the double
//! jump comes back when the player is hit or caught in a blast, the dash costs
//! a fifth of the energy, and a hit the shield takes goes off as a blast.

use crate::color::Rgb;
use crate::config::TICKS_PER_SEC;

/// Ticks between one absorbed hit and the next.
///
/// A second is long against a single enemy and short against a crowd, which is
/// the right way round: it caps how fast a swarm can chew through the player
/// without doing much for someone taking one hit at a time.
pub const SHIELD_COOLDOWN_TICKS: i64 = TICKS_PER_SEC as i64;

/// How many times cheaper the dash is at [`Boon::DashFree2`].
pub const DASH_FREE_2_DISCOUNT: i64 = 5;

/// Ticks the attack rests between two swings [`Boon::AutoAttack`] makes on its
/// own. A swing ends, four ticks pass with nothing out, and the next one starts.
pub const AUTO_ATTACK_GAP_TICKS: i64 = 4;

/// What the wall does to everything on the field when it goes up, beyond
/// standing there.
///
/// The two are one setting rather than two switches, because a wall cannot pull
/// and push at once. Taking one replaces the other, and because the offer never
/// shows what is already held, the swap is exactly what gets offered next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WallMod {
    #[default]
    Plain,
    /// Black. Drags every enemy toward the player who raised it.
    Pull,
    /// Grey. Throws every enemy away from them.
    Push,
}

impl WallMod {
    /// What the wall is drawn in. A flat slab and nothing else: no outline, so
    /// the colour is the whole of the signal.
    pub fn color(self) -> Rgb {
        match self {
            WallMod::Plain => Rgb::new(255.0, 255.0, 255.0),
            WallMod::Pull => Rgb::new(8.0, 8.0, 8.0),
            WallMod::Push => Rgb::new(140.0, 140.0, 140.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boon {
    /// One jump in the air, back after the next landing.
    DoubleJump,
    /// The air jump also comes back on being hit or caught in a blast.
    DoubleJump2,
    /// The dash keeps only its energy price.
    DashFree,
    /// And that price is a fifth of what it was: [`DASH_FREE_2_DISCOUNT`].
    DashFree2,
    /// Absorbs one touch, then waits [`SHIELD_COOLDOWN_TICKS`].
    Shield,
    /// The touch it absorbs goes off as a blast that hurts what is near.
    Shield2,
    /// The wall turns black and gathers the field in.
    WallPull,
    /// The wall turns grey and clears the field out.
    WallPush,
    /// Holding attack keeps swinging, the way the thrown attack keeps firing:
    /// a fresh swing [`AUTO_ATTACK_GAP_TICKS`] after the last one ends.
    AutoAttack,
}

impl Boon {
    pub const ALL: [Boon; 9] = [
        Boon::DoubleJump,
        Boon::DoubleJump2,
        Boon::DashFree,
        Boon::DashFree2,
        Boon::Shield,
        Boon::Shield2,
        Boon::WallPull,
        Boon::WallPush,
        Boon::AutoAttack,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Boon::DoubleJump => "DOUBLE JUMP",
            Boon::DoubleJump2 => "DOUBLE JUMP 2",
            Boon::DashFree => "FREE DASH",
            Boon::DashFree2 => "FREE DASH 2",
            Boon::Shield => "SHIELD",
            Boon::Shield2 => "SHIELD 2",
            Boon::WallPull => "BLACK WALL",
            Boon::WallPush => "GREY WALL",
            Boon::AutoAttack => "AUTO ATTACK",
        }
    }

    /// The line under a boon in the offer. Empty for a first level, whose name
    /// already says what it does; a second level's name says only that it is
    /// more, so this says what the more is.
    pub fn role(self) -> &'static str {
        match self {
            Boon::DoubleJump2 => "REFILL",
            Boon::DashFree2 => "1/5 COST",
            Boon::Shield2 => "BLAST",
            _ => "",
        }
    }

    /// The boon that has to be held before this one is offered, if any.
    pub fn requires(self) -> Option<Boon> {
        match self {
            Boon::DoubleJump2 => Some(Boon::DoubleJump),
            Boon::DashFree2 => Some(Boon::DashFree),
            Boon::Shield2 => Some(Boon::Shield),
            _ => None,
        }
    }
}

/// Which boons a run is carrying. Held per player, but taken together: the
/// offer is shared, so what one takes everyone gets.
///
/// A second level is its own flag beside the first rather than a number in
/// place of it. It is only ever offered on top of the first, so the pair reads
/// as a level, and every check of the first level stays a check of one flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boons {
    pub double_jump: bool,
    pub double_jump_2: bool,
    pub dash_free: bool,
    pub dash_free_2: bool,
    pub shield: bool,
    pub shield_2: bool,
    pub wall: WallMod,
    pub auto_attack: bool,
}

impl Boons {
    /// Every boon at both levels and the first modified wall: what a run
    /// carrying everything looks like.
    pub const ALL_HELD: Boons = Boons {
        double_jump: true,
        double_jump_2: true,
        dash_free: true,
        dash_free_2: true,
        shield: true,
        shield_2: true,
        wall: WallMod::Pull,
        auto_attack: true,
    };

    pub fn has(&self, boon: Boon) -> bool {
        match boon {
            Boon::DoubleJump => self.double_jump,
            Boon::DoubleJump2 => self.double_jump_2,
            Boon::DashFree => self.dash_free,
            Boon::DashFree2 => self.dash_free_2,
            Boon::Shield => self.shield,
            Boon::Shield2 => self.shield_2,
            Boon::WallPull => self.wall == WallMod::Pull,
            Boon::WallPush => self.wall == WallMod::Push,
            Boon::AutoAttack => self.auto_attack,
        }
    }

    /// True when `boon` could be offered: not held, and whatever it builds on
    /// is.
    pub fn can_take(&self, boon: Boon) -> bool {
        !self.has(boon) && boon.requires().is_none_or(|first| self.has(first))
    }

    /// How far along one of the two-level boons this is: 0, 1 or 2. Named by
    /// its first level.
    pub fn level(&self, first: Boon) -> u8 {
        let (one, two) = match first {
            Boon::DoubleJump | Boon::DoubleJump2 => (self.double_jump, self.double_jump_2),
            Boon::DashFree | Boon::DashFree2 => (self.dash_free, self.dash_free_2),
            Boon::Shield | Boon::Shield2 => (self.shield, self.shield_2),
            Boon::WallPull | Boon::WallPush => return (self.wall != WallMod::Plain) as u8,
            Boon::AutoAttack => return self.auto_attack as u8,
        };
        one as u8 + (one && two) as u8
    }

    /// Sets one of the two-level boons to `level`, both flags at once, so
    /// nothing can be left holding a second level without its first.
    pub fn set_level(&mut self, first: Boon, level: u8) {
        let (one, two) = (level >= 1, level >= 2);
        match first {
            Boon::DoubleJump | Boon::DoubleJump2 => {
                self.double_jump = one;
                self.double_jump_2 = two;
            }
            Boon::DashFree | Boon::DashFree2 => {
                self.dash_free = one;
                self.dash_free_2 = two;
            }
            Boon::Shield | Boon::Shield2 => {
                self.shield = one;
                self.shield_2 = two;
            }
            Boon::WallPull | Boon::WallPush | Boon::AutoAttack => {}
        }
    }

    /// True when this run is carrying everything a run can carry at once.
    ///
    /// Not a fold over [`Boon::ALL`]: the two walls are one setting, so a run
    /// holding a wall is holding all the wall it can, and asking for both would
    /// be asking for something that cannot exist.
    pub fn full(&self) -> bool {
        self.double_jump
            && self.double_jump_2
            && self.dash_free
            && self.dash_free_2
            && self.shield
            && self.shield_2
            && self.wall != WallMod::Plain
            && self.auto_attack
    }

    pub fn take(&mut self, boon: Boon) {
        match boon {
            Boon::DoubleJump => self.double_jump = true,
            Boon::DoubleJump2 => self.double_jump_2 = true,
            Boon::DashFree => self.dash_free = true,
            Boon::DashFree2 => self.dash_free_2 = true,
            Boon::Shield => self.shield = true,
            Boon::Shield2 => self.shield_2 = true,
            // Assignment, not accumulation: the other one stops being true by
            // the same stroke, which is what makes the pair a choice.
            Boon::WallPull => self.wall = WallMod::Pull,
            Boon::WallPush => self.wall = WallMod::Push,
            Boon::AutoAttack => self.auto_attack = true,
        }
    }
}
