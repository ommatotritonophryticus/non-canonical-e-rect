//! The choice between waves.
//!
//! Three options stand in the lull as things to be hit. That is deliberate:
//! the player already knows how to attack, so nothing new has to be taught and
//! no button has to be borrowed from the fight. Walking up and swinging is the
//! whole interface.
//!
//! Both players share one offer and either can take it. The trigger is the team
//! total, which is also what drives the chase speed - so upgrades and difficulty
//! run off the same clock.

use alloc::string::{String, ToString};

use crate::attack::{AttackKind, MAX_LEVEL};
use crate::boon::{Boon, Boons};
use crate::config::RULE_OFFER_WAVE;
use crate::entities::Rng;
use crate::geom::{Body, Viewport};
use crate::waves::WaveRule;

/// Team score between offers.
pub const OFFER_SCORE_STEP: i64 = 4000;

/// How many stand at once.
pub const OFFER_CHOICES: usize = 3;

/// Ticks the options stand there inert before a hit counts as taking one.
///
/// A wave ends while the player is still swinging at what is left of it, and
/// the options appear inside that swing. Without a pause the choice is made by
/// whichever one happened to be nearest when the last enemy died - which is not
/// a choice at all.
pub const OFFER_ARM_TICKS: i64 = 90;

/// Where they stand, as a percentage of the view. Spread wide enough that
/// reaching one is a decision rather than an accident of where you stopped.
///
/// Three is what a rolled offer always has; a scripted one can have fewer, and
/// then they spread to fill the same width rather than huddling at the left.
/// One stands in the middle, where the player is.
fn spread(count: usize) -> &'static [f32] {
    match count {
        0 => &[],
        1 => &[50.0],
        2 => &[33.0, 67.0],
        _ => &[25.0, 50.0, 75.0],
    }
}

/// What one of the three standing options would give.
///
/// Weapons and boons share the pool rather than alternating. A boon is the
/// safer pick whenever one is on the table - it is kept, and it costs no level
/// - but there are only three of them and a taken one never returns, so the
/// pool drains back to weapons on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferItem {
    Attack {
        kind: AttackKind,
        /// What the player would end up carrying - a fresh 1, or one more of
        /// what they already have.
        level: u8,
    },
    Boon(Boon),
    /// Terms for the wave about to start, rather than anything the player
    /// keeps. Only appears from [`RULE_OFFER_WAVE`], and only for the one wave
    /// it is taken in: the next lull asks again.
    Rule(WaveRule),
}

#[derive(Clone, Copy, Debug)]
pub struct OfferChoice {
    pub item: OfferItem,
    pub body: Body,
}

impl OfferChoice {
    /// True when this is another level of the weapon already being held.
    pub fn is_upgrade(&self, current: AttackKind) -> bool {
        matches!(self.item, OfferItem::Attack { kind, .. } if kind == current)
    }

    /// What the option says above its head.
    ///
    /// The level is printed whenever it is not the first one - on the weapon
    /// already carried, that is the step up being offered; on a different one,
    /// it is how much of the climb comes with it. Only a plain level 1 goes
    /// unmarked, where the number would read as noise.
    pub fn label(&self, current: AttackKind) -> String {
        match self.item {
            OfferItem::Attack { kind, level } if kind == current || level > 1 => {
                let mut text = kind.label().to_string();
                text.push(' ');
                text.push((b'0' + level) as char);
                text
            }
            OfferItem::Attack { kind, .. } => kind.label().to_string(),
            OfferItem::Boon(boon) => boon.label().to_string(),
            // A modifier with no label is the plain wave, which is a real
            // choice here: it is the option to be left alone.
            OfferItem::Rule(rule) => rule.label().unwrap_or("PLAIN").to_string(),
        }
    }

    /// The line under the name: what this option is for.
    ///
    /// Empty for a boon, whose own name already says it - `DOUBLE JUMP` needs
    /// no gloss, and a second line under every option would be three lines of
    /// text across the field rather than three words.
    pub fn role(&self) -> &'static str {
        match self.item {
            // The last level is where a weapon stops being more of itself and
            // starts doing something else. Worth saying so: it is the only step
            // that is not a number.
            OfferItem::Attack { level, .. } if level >= MAX_LEVEL => "MASTER",
            OfferItem::Attack { kind, .. } => kind.role(),
            OfferItem::Boon(_) => "",
            // Said on every one of them, because the thing a player has to
            // know about a modifier is that it is not kept.
            OfferItem::Rule(_) => "THIS WAVE",
        }
    }
}

pub struct Offer {
    /// Only the first [`Self::count`] are standing. The array is fixed so the
    /// core stays free of an allocation per lull on a console with 32 MB.
    pub choices: [OfferChoice; OFFER_CHOICES],
    /// How many of them there actually are. A rolled offer always has three; a
    /// scripted one has as many as its step wrote down.
    pub count: usize,
    /// First tick on which a hit counts. A deadline rather than a countdown,
    /// for the same reason the dash uses one: the offer is created partway
    /// through a tick that has already run some of its work.
    pub live_from: i64,
}

impl Offer {
    /// True once hitting an option means taking it.
    pub fn armed(&self, timer: i64) -> bool {
        timer >= self.live_from
    }

    /// Stands exactly what it is given, in the order it is given.
    ///
    /// No pool, no draw, no filtering by what is already held: a script has
    /// already decided all of that. Anything past the third is dropped rather
    /// than queued - three is what the lull has room for.
    pub fn scripted(
        v: &Viewport,
        camera_x: f32,
        ground_y: f32,
        items: &[OfferItem],
        timer: i64,
    ) -> Self {
        let count = items.len().min(OFFER_CHOICES);
        let mut offer = Self::empty(timer);
        offer.count = count;
        for (i, item) in items.iter().take(count).enumerate() {
            offer.choices[i] = OfferChoice {
                item: *item,
                body: Self::stand(v, camera_x, ground_y, count, i),
            };
        }
        offer
    }

    /// Where the `i`th of `count` boxes stands.
    fn stand(v: &Viewport, camera_x: f32, ground_y: f32, count: usize, i: usize) -> Body {
        let w = v.wper(5.0);
        let h = v.hper(10.0);
        let at = spread(count).get(i).copied().unwrap_or(50.0);
        Body::new(camera_x + v.wper(at) - w / 2.0, ground_y - h, w, h)
    }

    fn empty(timer: i64) -> Self {
        Self {
            choices: [OfferChoice {
                item: OfferItem::Attack { kind: AttackKind::Basic, level: 1 },
                body: Body::default(),
            }; OFFER_CHOICES],
            count: 0,
            live_from: timer + OFFER_ARM_TICKS,
        }
    }

    /// Draws three distinct options for a player carrying `current` at `level`
    /// and holding `held`.
    ///
    /// The pool is every weapon they are not holding, plus one more level of
    /// the one they are - so a run can deepen a weapon instead of only swapping
    /// it, and a maxed-out weapon quietly stops being offered - plus every boon
    /// not already taken.
    #[allow(clippy::too_many_arguments)]
    pub fn roll(
        v: &Viewport,
        camera_x: f32,
        ground_y: f32,
        current: AttackKind,
        level: u8,
        held: Boons,
        wave: i64,
        rolled: WaveRule,
        timer: i64,
        rng: &mut Rng,
    ) -> Self {
        let (pool, n) = build_pool(current, level, held, None, wave);
        if wave < RULE_OFFER_WAVE {
            return Self::draw(v, camera_x, ground_y, pool, n, timer, rng);
        }

        // Late on, two of the three boxes stop being upgrades. One still is -
        // the run should never stop offering anything at all - and the other
        // two are the terms the coming wave will be fought under.
        let mut offer = Self::empty(timer);
        let mut count = 0;
        let picked = Self::draw(v, camera_x, ground_y, pool, n, timer, rng);
        if picked.count > 0 {
            offer.choices[0] = picked.choices[0];
            count += 1;
        }

        // Whatever the wave already rolled is left out: taking it would be a
        // box that changes nothing, and two of the three boxes here are the
        // only choice the player has left.
        let mut rules = [OfferItem::Boon(Boon::DoubleJump); POOL_MAX];
        let mut rn = 0;
        for rule in WaveRule::EVERY {
            if rule != rolled {
                rules[rn] = OfferItem::Rule(rule);
                rn += 1;
            }
        }
        for i in 0..(OFFER_CHOICES - count).min(rn) {
            let j = i + (rng.range(0, (rn - i - 1) as i32).max(0) as usize);
            rules.swap(i, j);
            offer.choices[count + i] = OfferChoice {
                item: rules[i],
                body: Body::default(),
            };
        }
        offer.count = (count + (OFFER_CHOICES - count).min(rn)).min(OFFER_CHOICES);
        // Placed only once the count is settled, or they would stand at the
        // spacing of an offer that never existed.
        for i in 0..offer.count {
            offer.choices[i].body = Self::stand(v, camera_x, ground_y, offer.count, i);
        }
        offer
    }

    /// The same draw with the roster taken out of it: `weapon` and no other.
    ///
    /// This is the whole mechanical difference between a campaign and an
    /// ordinary run. The pool is the chosen weapon's next step and every boon
    /// not yet held, so a campaign still has real choices in it - just never
    /// the choice to become a different run.
    ///
    /// Once the weapon is at its last level the pool is boons alone, and once
    /// those run down it can be a single box. That is why the count is derived
    /// rather than fixed at three.
    #[allow(clippy::too_many_arguments)]
    pub fn roll_campaign(
        v: &Viewport,
        camera_x: f32,
        ground_y: f32,
        weapon: AttackKind,
        current: AttackKind,
        level: u8,
        held: Boons,
        timer: i64,
        rng: &mut Rng,
    ) -> Self {
        // A campaign keeps its own shape all the way through, modifiers
        // included: its pool is the point of the mode.
        let (pool, n) = build_pool(current, level, held, Some(weapon), 1);
        Self::draw(v, camera_x, ground_y, pool, n, timer, rng)
    }

    /// Shuffles the front of `pool` and stands up to three of it.
    fn draw(
        v: &Viewport,
        camera_x: f32,
        ground_y: f32,
        mut pool: [OfferItem; POOL_MAX],
        n: usize,
        timer: i64,
        rng: &mut Rng,
    ) -> Self {
        // Partial shuffle: three draws is all that is needed, and it keeps the
        // generator's use proportional to what is taken rather than to the pool.
        for i in 0..OFFER_CHOICES.min(n) {
            let j = i + (rng.range(0, (n - i - 1) as i32).max(0) as usize);
            pool.swap(i, j);
        }

        // A count that is *derived* cannot drift out of step with the array the
        // way a hard-coded three can - and a campaign's pool really does run
        // short of three near its end.
        let count = OFFER_CHOICES.min(n);
        let mut offer = Self::empty(timer);
        offer.count = count;
        for i in 0..count {
            offer.choices[i] = OfferChoice {
                item: pool[i],
                body: Self::stand(v, camera_x, ground_y, count, i),
            };
        }
        offer
    }
}

/// At most the whole roster, the level-up and every boon.
const POOL_MAX: usize = AttackKind::ALL.len() + 1 + Boon::ALL.len();

/// Everything a player carrying `current` at `level` and holding `held` could
/// be shown, and how much of it there is.
///
/// `roster` is which weapons may appear. `None` is the endless run's answer:
/// all of them. `Some(weapon)` is a campaign's: that one and no other, either
/// as a fresh pick-up or as its next level.
fn build_pool(
    current: AttackKind,
    level: u8,
    held: Boons,
    roster: Option<AttackKind>,
    wave: i64,
) -> ([OfferItem; POOL_MAX], usize) {
    let mut pool = [OfferItem::Attack { kind: AttackKind::Basic, level: 1 }; POOL_MAX];
    let mut n = 0;
    match roster {
        Some(weapon) => {
            // `Basic` is not a weapon that can be picked up or levelled, so a
            // campaign for it is a campaign of boons.
            if weapon != AttackKind::Basic {
                // Always at level one: this is the weapon the campaign was
                // started for arriving for the first time, not a swap away
                // from something else, so there is nothing to discount from.
                if current != weapon {
                    pool[n] = OfferItem::Attack { kind: weapon, level: 1 };
                    n += 1;
                } else if level < MAX_LEVEL {
                    pool[n] = OfferItem::Attack { kind: weapon, level: level + 1 };
                    n += 1;
                }
            }
        }
        None => {
            if current != AttackKind::Basic && level < MAX_LEVEL {
                pool[n] = OfferItem::Attack { kind: current, level: level + 1 };
                n += 1;
            }
            // Another weapon arrives one level short of what is already being
            // carried, never below one. A swap therefore costs a single step
            // rather than the whole climb, which is what it used to cost: a
            // run that had reached the third level of anything could not look
            // at the roster again without throwing two levels away, so it
            // never did, and the second half of every run played itself.
            //
            // One short and not level: the weapon in hand should still be
            // worth having kept.
            //
            // From [`RULE_OFFER_WAVE`] the discount stops being a discount and
            // the whole roster comes at its top level. By then a swap has to
            // be a real option inside a single wave: there is no run left to
            // spend climbing back up, and the boxes beside it are handing out
            // terms rather than levels.
            let swap = if wave >= RULE_OFFER_WAVE {
                MAX_LEVEL
            } else {
                level.clamp(1, MAX_LEVEL).saturating_sub(1).max(1)
            };
            for kind in AttackKind::ALL {
                if kind != current {
                    pool[n] = OfferItem::Attack { kind, level: swap };
                    n += 1;
                }
            }
        }
    }
    for boon in Boon::ALL {
        if !held.has(boon) {
            pool[n] = OfferItem::Boon(boon);
            n += 1;
        }
    }
    (pool, n)
}
