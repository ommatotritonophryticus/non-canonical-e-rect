//! What a swing is made of.
//!
//! The player has one attack box and it has always meant one thing. The
//! upgrades offered between waves replace that meaning rather than adding to
//! it: each kind still has to do the attack's job - reach out, damage, throw
//! back, earn score - while trading one of its characteristics for another.
//!
//! Everything a kind changes is expressed here as a parameter of the same box,
//! so the simulation stays one code path and a new kind is a row in these
//! matches rather than a branch at every damage site.
//!
//! The box that sits *inside* the player at rest is deliberately not part of
//! this. It is the direction indicator and the dash cooldown readout, and it
//! looks the same whatever the player is swinging.

/// Damage one tick of an ordinary swing deals, before armour.
pub const BASE_DAMAGE: f32 = 64.0;

/// Levels an upgrade can reach. Level 1 is what the first pick gives.
pub const MAX_LEVEL: u8 = 3;

/// Which edge of the box stays put when its height changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightAnchor {
    /// Grows and shrinks downward, so the top stays where the eye expects it.
    Top,
    /// Grows and shrinks upward. A short box then hugs the ground, which is
    /// what lets low enemies still be hit while flyers pass over untouched.
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    /// The swing the game ships with.
    Basic,
    /// Half the reach, twice the swing, and one heavy blow per enemy instead
    /// of a stream of light ones. Throws twice as far.
    ///
    /// It has to land once rather than per tick: a slow swing that damaged
    /// every tick would keep the enemy in the box for all twelve of them and
    /// deal eight times what an ordinary swing does, which is not a hammer but
    /// a woodchipper.
    Hammer,
    /// Ordinary reach that does not throw at all. The enemy stays in the box
    /// for the whole swing instead of being knocked clear a third of the way
    /// through, which is where the extra damage comes from - and it also stays
    /// close enough to keep touching back.
    Piercing,
    /// Starts short and reaches further every tick of the swing. Distant
    /// targets are only caught at the end, so it wants the player standing
    /// still.
    Lunge,
    /// One guaranteed hit per enemy per swing, for what an ordinary swing does
    /// at its best rather than at its average. Trades the ceiling for a floor.
    SingleHit,
    /// A shallow strip out of both sides at once, sitting low. Sacrifices
    /// height for surrounding coverage: small enemies are still caught, flyers
    /// and jumpers pass above it.
    Thin,
    /// Shorter, but tall enough to cover the whole player. Buys the low targets
    /// an ordinary swing rides over.
    Tall,
    /// The swing is thrown instead of held: a square that travels, stops on
    /// what it hits and works for a moment longer. Throws what it reaches
    /// exactly as a swing does.
    ///
    /// Range without the risk of standing in reach, paid for in damage: the
    /// throw carries the target clear after one connecting tick, so a shot
    /// lands about a third of what a swing does.
    Bullet,
    /// The box is left standing where it was put and keeps working, throwing
    /// what it catches exactly as a swing does.
    ///
    /// It deals much less per tick than a swing in exchange for standing on its
    /// own. Its life is short enough that a thrown enemy does not come back
    /// down inside it - the arc is longer than the box lasts - so it holds
    /// ground without juggling anything.
    Frozen,
}

impl AttackKind {
    /// Every kind, in the order they should be offered.
    pub const ALL: [AttackKind; 8] = [
        AttackKind::Hammer,
        AttackKind::Piercing,
        AttackKind::Lunge,
        AttackKind::SingleHit,
        AttackKind::Thin,
        AttackKind::Tall,
        AttackKind::Bullet,
        AttackKind::Frozen,
    ];

    /// Every kind there is, the plain swing included.
    ///
    /// Separate from [`Self::ALL`], which is the *upgrade roster* and leaves
    /// the plain swing out because it is never offered. This one is the list of
    /// things a player can be carrying, and it is what a campaign is chosen
    /// from and what a saved result is filed under - so its order is a stored
    /// format and must not be shuffled.
    pub const EVERY: [AttackKind; 9] = [
        AttackKind::Basic,
        AttackKind::Hammer,
        AttackKind::Piercing,
        AttackKind::Lunge,
        AttackKind::SingleHit,
        AttackKind::Thin,
        AttackKind::Tall,
        AttackKind::Bullet,
        AttackKind::Frozen,
    ];

    /// Where this one sits in [`Self::EVERY`].
    pub fn index(self) -> usize {
        match self {
            AttackKind::Basic => 0,
            AttackKind::Hammer => 1,
            AttackKind::Piercing => 2,
            AttackKind::Lunge => 3,
            AttackKind::SingleHit => 4,
            AttackKind::Thin => 5,
            AttackKind::Tall => 6,
            AttackKind::Bullet => 7,
            AttackKind::Frozen => 8,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AttackKind::Basic => "BASIC",
            AttackKind::Hammer => "HAMMER",
            AttackKind::Piercing => "PIERCE",
            AttackKind::Lunge => "LUNGE",
            AttackKind::SingleHit => "SINGLE",
            AttackKind::Thin => "SWEEP",
            AttackKind::Tall => "HEAVY",
            AttackKind::Bullet => "SHOT",
            AttackKind::Frozen => "TRAP",
        }
    }

    /// What the weapon is for, in two words.
    ///
    /// Short on purpose: three of these stand side by side in the lull, each in
    /// a third of the view, and on a 480-wide panel a sentence would run into
    /// its neighbours. The name says which weapon; this says what it is for.
    pub fn role(self) -> &'static str {
        match self {
            AttackKind::Basic => "COMBO",
            AttackKind::Hammer => "KNOCKBACK",
            AttackKind::Piercing => "CONTACT",
            AttackKind::Lunge => "TIMING",
            AttackKind::SingleHit => "EXECUTE",
            AttackKind::Thin => "GROUND",
            AttackKind::Tall => "AIR",
            AttackKind::Bullet => "LINE",
            AttackKind::Frozen => "ZONE",
        }
    }

    /// How long a swing stays out - or, for a kind that puts something into
    /// the world instead of holding a box, how long until the next one.
    pub fn swing_ticks(self, base: i32) -> i32 {
        match self {
            AttackKind::Hammer => base * 2,
            // No wait at all: how many may stand at once is the only limit on
            // placing them, and that is what its level buys.
            AttackKind::Frozen => 0,
            _ => base,
        }
    }

    /// How many of this kind's things may be in the world at once. Only the
    /// detached kinds have any; it is what their level buys.
    pub fn max_instances(self, level: u8) -> usize {
        match self {
            // Capped one short of the level ladder on purpose. The third level
            // is where each of these stops being "one more of the same" and
            // starts behaving differently - a shot that carries on down the
            // line, a trap that turns back what walks in. Handing out a third
            // instance as well would make that step twice the size of the one
            // before it.
            AttackKind::Bullet | AttackKind::Frozen => level.clamp(1, 2) as usize,
            _ => 0,
        }
    }

    /// True at the level where a kind gains its own mechanic rather than
    /// another step of the same number.
    pub fn mastered(level: u8) -> bool {
        level >= MAX_LEVEL
    }

    /// Multiplies the horizontal reach the wave ramp worked out.
    ///
    /// `progress` is how far through the swing we are, 0 at the first tick and
    /// 1 at the last; only the lunge cares.
    /// `dashing` is whether the swing began inside the dash window; only the
    /// lunge at its last level reads it.
    pub fn length_scale(self, level: u8, progress: f32, dashing: bool) -> f32 {
        let mastered = Self::mastered(level);
        let level = level.clamp(1, MAX_LEVEL) as f32;
        match self {
            AttackKind::Basic | AttackKind::Piercing | AttackKind::SingleHit => 1.0,
            AttackKind::Hammer => 0.5,
            // 0.3 of the usual at the start, growing past it by the end.
            //
            // Mastered, a lunge that came out of a dash reaches further still.
            // The dash is already a commitment - no steering, one direction,
            // paid for out of the same bar the wall drinks from - so this pays
            // for a decision the player has already made rather than adding one.
            AttackKind::Lunge => {
                let end = 0.9 + 0.3 * level + if mastered && dashing { 0.7 } else { 0.0 };
                0.3 + end * progress
            }
            // Four fifths of an ordinary swing per side at level 1, rising
            // gently. It started at a quarter and played far too short for what
            // it gives up in height; the ladder is shallow because both sides
            // are swung at once, so every step is worth double.
            AttackKind::Thin => 0.6 + 0.2 * level,
            AttackKind::Tall => 0.7,
            // Neither of these uses the attached box for reach.
            AttackKind::Bullet => 0.0,
            AttackKind::Frozen => 0.5,
        }
    }

    /// Multiplies the box height, against the anchor below.
    pub fn height_scale(self, level: u8) -> f32 {
        let level = level.clamp(1, MAX_LEVEL) as f32;
        match self {
            // Low and shallow. Kept off 0.2: at that height whether a splitter
            // child or a dipping flyer is caught comes down to half a percent
            // of the screen, which is a pixel and a half on a PSP and reads as
            // random.
            AttackKind::Thin => 0.3,
            // Covers the whole player by level 2, and past it after that.
            AttackKind::Tall => 1.0 + 0.5 * level,
            _ => 1.0,
        }
    }

    pub fn height_anchor(self) -> HeightAnchor {
        match self {
            AttackKind::Thin => HeightAnchor::Bottom,
            _ => HeightAnchor::Top,
        }
    }

    /// True when the swing comes out of both sides at once.
    pub fn two_sided(self) -> bool {
        matches!(self, AttackKind::Thin)
    }

    /// Damage one connecting tick deals, before armour.
    pub fn damage(self, level: u8) -> f32 {
        let level = level.clamp(1, MAX_LEVEL) as f32;
        match self {
            // One blow per enemy, so this is the whole swing rather than a
            // tick of it. At level 1 it matches an ordinary swing's damage per
            // tick of swing time - the trade is half the reach and double the
            // throw, not raw output.
            AttackKind::Hammer => BASE_DAMAGE * (5.0 + level),
            // Everything an ordinary swing can manage at its best, guaranteed
            // once. An ordinary swing ranges from one connecting tick to six
            // depending on where the throw sends the enemy; this is the top of
            // that range with the luck taken out.
            AttackKind::SingleHit => BASE_DAMAGE * (3.0 + level),
            // Stands on its own without being aimed, so per-tick is small.
            AttackKind::Frozen => BASE_DAMAGE / 4.0,
            // Per connecting tick, like a swing - and like a swing, the throw
            // carries the target clear after the first, so a shot is worth
            // about a third of one in exchange for landing from out of reach.
            // The ticks it has left are not wasted: a stopped shot still
            // catches whatever walks into it.
            AttackKind::Bullet => BASE_DAMAGE,
            // Damage per tick, and it gets every tick of the swing because it
            // never throws the enemy clear.
            AttackKind::Piercing => BASE_DAMAGE * (0.75 + 0.25 * level),
            _ => BASE_DAMAGE,
        }
    }

    /// Multiplies how far sideways a connecting hit throws an enemy.
    ///
    /// This is the half that decides how much of a swing actually lands: it is
    /// what carries the enemy out through the side of the box, a third of the
    /// way through an ordinary swing.
    pub fn knockback_scale(self, level: u8) -> f32 {
        match self {
            AttackKind::Hammer => 2.0,
            // Without it the enemy stays and takes the whole swing - and stays
            // in touching range while doing it.
            AttackKind::Piercing => 0.0,
            // Mastered, the sweep stops being a sword that swings both ways and
            // becomes the thing that decides how much floor the player owns:
            // what it touches leaves on both sides at once.
            AttackKind::Thin if Self::mastered(level) => 2.0,
            _ => 1.0,
        }
    }

    /// Multiplies how far *up* a connecting hit throws an enemy.
    ///
    /// Kept apart from the sideways half because a bullet wants one and not the
    /// other: the lift reads as impact and clears the ground, while sideways
    /// travel would carry the target out of the square that is still working
    /// on it.
    pub fn lift_scale(self) -> f32 {
        match self {
            AttackKind::Hammer => 2.0,
            AttackKind::Piercing => 0.0,
            _ => 1.0,
        }
    }

    /// True when one swing may only hit a given enemy once.
    pub fn once_per_enemy(self) -> bool {
        matches!(self, AttackKind::Hammer | AttackKind::SingleHit)
    }

    /// True when the swing puts something into the world instead of holding a
    /// box on the player.
    pub fn detached(self) -> bool {
        matches!(self, AttackKind::Bullet | AttackKind::Frozen)
    }
}
