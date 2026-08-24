//! The written runs: one weapon, learned and then taken to mastery.
//!
//! An ordinary run rolls its offers out of the whole roster and never finishes.
//! A written run narrows the roster to one weapon and gains an end, so the same
//! simulation becomes something that can be *completed* - and completed once
//! per weapon, which is the whole of both modes.
//!
//! There are two of them, and they narrow it by different amounts.
//!
//! A **tutorial** is three waves and writes down every step: the weapon is in
//! hand before the first wave and each lull raises it a level, so wave one is
//! fought at level one and wave three at three. Nothing is chosen and nothing
//! is rolled, which is what makes it a lesson rather than a run.
//!
//! A **campaign** rolls. Its pool is the chosen weapon's next level and every
//! boon not yet held - the whole game minus the roster - and it ends one wave
//! after the weapon reaches its last level. That makes its length the player's
//! own doing: taking the weapon whenever it appears is the short, thin run,
//! and taking the boons first is the long, strong one.
//!
//! Everything else is the ordinary game. The waves, their modifiers, the
//! rolled heavies, the bosses, the crowd limits: all of it as it always was.

use alloc::vec::Vec;

use crate::attack::{AttackKind, MAX_LEVEL};
use crate::boon::Boon;
use crate::offer::OfferItem;

/// The wave that, cleared, finishes a tutorial.
///
/// Three, because a tutorial is not a run: one wave per level of the weapon,
/// and then it has nothing left to show. A fourth would only be more of the
/// same, and the campaign is where more of the same belongs.
pub const TUTORIAL_LAST_WAVE: i64 = 3;

/// Which of the two written runs this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeKind {
    /// Three waves, one weapon, a level a wave. Where a weapon is met.
    Tutorial,
    /// The weapon to its last level, and every boon along the way. Where it is
    /// spent.
    Campaign,
}

impl ModeKind {
    /// Both, in the order the screen flips through them: a tutorial is what
    /// comes before a campaign, so it is what the screen opens on.
    pub const EVERY: [ModeKind; 2] = [ModeKind::Tutorial, ModeKind::Campaign];

    pub fn label(self) -> &'static str {
        match self {
            ModeKind::Tutorial => "TUTORIAL",
            ModeKind::Campaign => "CAMPAIGN",
        }
    }

    /// The line under the heading, which is the whole of what the mode
    /// promises.
    pub fn tagline(self) -> &'static str {
        match self {
            ModeKind::Tutorial => "THREE WAVES, THREE LEVELS",
            ModeKind::Campaign => "ONE WEAPON, TO MASTERY",
        }
    }

    /// Which column of the record table this mode is filed in. A stored
    /// format, like [`crate::attack::AttackKind::index`]: never renumber it.
    pub fn index(self) -> usize {
        match self {
            ModeKind::Tutorial => 0,
            ModeKind::Campaign => 1,
        }
    }
}

/// One offer, as a tutorial writes it: what stands in the lull.
///
/// A list rather than a single item because a step with nothing in it is how a
/// script says "this lull is quiet", and because a step could hold a choice.
pub type Step = Vec<OfferItem>;

/// What a tutorial for `weapon` starts the player holding.
///
/// The weapon itself, at level one, before the first wave rather than after
/// it - a tutorial that spent its first wave on the plain swing would be
/// teaching the thing it is not about. `Basic` has no weapon to be given and
/// starts as every run starts.
pub fn tutorial_start(weapon: AttackKind) -> Option<AttackKind> {
    (weapon != AttackKind::Basic).then_some(weapon)
}

/// What a tutorial for `weapon` hands out, in order.
///
/// Two steps, because three waves have two lulls between them, and each one is
/// the next level of the weapon already in hand. Level one is not here: it was
/// handed over before the run began, by [`tutorial_start`].
///
/// `Basic` has no levels to climb, so it spends both steps on the two boons the
/// plain swing has to work with instead.
pub fn tutorial_script(weapon: AttackKind) -> Vec<Step> {
    if weapon == AttackKind::Basic {
        alloc::vec![
            alloc::vec![OfferItem::Boon(Boon::DoubleJump)],
            alloc::vec![OfferItem::Boon(Boon::Shield)],
        ]
    } else {
        let mut out = Vec::new();
        for level in 2..=MAX_LEVEL {
            out.push(alloc::vec![OfferItem::Attack { kind: weapon, level }]);
        }
        out
    }
}

/// How many offers a tutorial for `weapon` has to give.
pub fn tutorial_steps(weapon: AttackKind) -> usize {
    tutorial_script(weapon).len()
}
