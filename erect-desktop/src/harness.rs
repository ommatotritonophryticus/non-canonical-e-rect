//! Runs the game unattended and saves what it drew.
//!
//! Verifying that something *renders* is the one thing the test suite cannot
//! do: it exercises the core, and a marker that never reaches the screen looks
//! identical from there. Driving a browser by synthetic clicks was the other
//! option and it lost most of them; a native build with a fixed seed, a fixed
//! frame count and a file on disk is repeatable.
//!
//! Off unless `ERECT_HARNESS` is set, so an ordinary run never notices.

use erect_core::dev::DevSetup;
use erect_core::game::Game;
use erect_core::input::InputFrame;
use macroquad::prelude::*;

pub struct Harness {
    /// Frames to draw before saving. Enough to let the wave spawn.
    frames: u32,
    shots: Vec<(u32, String)>,
    /// Saved on the first frame a rolled heavy is being announced. A fixed
    /// frame number cannot catch that: when it arrives depends on how fast the
    /// wave spent its budget.
    on_elite: Option<String>,
    /// Frames to wait after a heavy is announced before saving.
    ///
    /// Enemies are placed just outside the view and walk in, so the frame a
    /// heavy is announced on is the one frame it is guaranteed *not* to be
    /// visible in. Three attempts at photographing one went that way.
    on_elite_delay: u32,
    on_elite_at: Option<u32>,
    /// Saves the frame the between-wave offer is standing, which is the only
    /// time its three options are on screen.
    on_offer: Option<String>,
    /// Saves the frame a blind wave lets its enemies show, and the frame after
    /// it does not. Frame numbers cannot be used for this: they are not ticks.
    on_glimpse: Option<String>,
    after_glimpse: Option<String>,
    /// Frame the "hidden" companion shot is due on. Taken a few frames later
    /// than the glimpse rather than the very next one: reading the framebuffer
    /// gives back the frame that was last presented, which is one behind.
    after_glimpse_at: Option<u32>,
    /// Not before this frame. A blind wave shows its enemies on tick zero, when
    /// there are none yet to show.
    glimpse_after: u32,
    on_offer_delay: u32,
    on_offer_at: Option<u32>,
    /// Keeps the players standing. Nothing drives them here, so an ordinary run
    /// ends within a wave or two - long before the thing under test arrives.
    keep_alive: bool,
    /// Swings on a fixed rhythm and turns to face whatever is nearest.
    ///
    /// Not a good player - a working one. A wave only spawns as fast as the
    /// field is cleared, so a run that never kills anything stalls at the crowd
    /// limit and the later half of the wave, heavy included, never arrives.
    fight: bool,
    /// Raises a wall every so often, so one can be looked at.
    wall: bool,
    /// Presses attack once, on the first frame, and then nothing.
    ///
    /// `fight` swings every twelve ticks, which for a placed box means it is
    /// always a fresh one. Watching a box age needs exactly one press.
    once: bool,
}

impl Harness {
    /// `ERECT_HARNESS=wave=7,alive,fight,frames=600,out=/tmp/a.png`
    ///
    /// Flags may be bare or written `=1`; anything unrecognised is ignored, so
    /// a stale spelling is a missing feature rather than a failure to start.
    pub fn from_env(game: &mut Game) -> Option<Self> {
        let spec = std::env::var("ERECT_HARNESS").ok()?;
        let mut dev = DevSetup::default();
        let mut frames = 600;
        let mut shots = Vec::new();
        let mut on_elite = None;
        let mut on_elite_delay = 0u32;
        let mut on_offer = None;
        let mut on_glimpse = None;
        let mut glimpse_after = 0u32;
        let mut on_offer_delay = 0u32;
        let mut keep_alive = false;
        let mut fight = false;
        let mut boons = false;
        let mut screen: Option<String> = None;
        let mut once = false;
        let mut written: Option<(erect_core::campaign::ModeKind, erect_core::attack::AttackKind)> =
            None;
        let mut wall = erect_core::boon::WallMod::Plain;
        for part in spec.split(',') {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            match key {
                "wave" => dev.wave = value.parse().unwrap_or(1),
                "score" => dev.score = value.parse().unwrap_or(0),
                "rule" => {
                    dev.rule = match value {
                        "blind" => Some(erect_core::waves::WaveRule::Hidden),
                        _ => None,
                    }
                }
                // `written=tutorial:hammer` - a real written run rather than a
                // dev one, so the mode can be watched from the outside as it
                // is actually played.
                "written" => {
                    if let Some((mode, weapon)) = value.split_once(':') {
                        let kind = match mode {
                            "campaign" => erect_core::campaign::ModeKind::Campaign,
                            _ => erect_core::campaign::ModeKind::Tutorial,
                        };
                        let weapon = erect_core::attack::AttackKind::EVERY
                            .iter()
                            .copied()
                            .find(|k| k.label().eq_ignore_ascii_case(weapon))
                            .unwrap_or(erect_core::attack::AttackKind::Basic);
                        written = Some((kind, weapon));
                    }
                }
                "once" => once = value != "0",
                "level" => dev.attack_level = value.parse().unwrap_or(1),
                // `arm=trap` - what a dev run starts holding, by the label the
                // offer screen shows.
                "arm" => {
                    if let Some(kind) = erect_core::attack::AttackKind::EVERY
                        .iter()
                        .copied()
                        .find(|k| k.label().eq_ignore_ascii_case(value))
                    {
                        dev.attack = kind;
                    }
                }
                "frames" => frames = value.parse().unwrap_or(600),
                "out" => shots.push((frames, value.to_string())),
                // A second and later snapshot, so one run can show a thing
                // arriving and the same thing being fought.
                "at" => {
                    if let Some((at, path)) = value.split_once(':') {
                        shots.push((at.parse().unwrap_or(0), path.to_string()));
                    }
                }
                "on_elite" => match value.split_once(':') {
                    Some((delay, path)) => {
                        on_elite_delay = delay.parse().unwrap_or(0);
                        on_elite = Some(path.to_string());
                    }
                    None => on_elite = Some(value.to_string()),
                },
                "on_glimpse" => match value.split_once(':') {
                    Some((from, path)) => {
                        glimpse_after = from.parse().unwrap_or(0);
                        on_glimpse = Some(path.to_string());
                    }
                    None => on_glimpse = Some(value.to_string()),
                },
                "on_offer" => match value.split_once(':') {
                    Some((delay, path)) => {
                        on_offer_delay = delay.parse().unwrap_or(0);
                        on_offer = Some(path.to_string());
                    }
                    None => on_offer = Some(value.to_string()),
                },
                "alive" => keep_alive = true,
                "fight" => fight = true,
                "boons" => boons = true,
                // Show a menu instead of playing, so a layout can be looked at.
                "screen" => screen = Some(value.to_string()),
                "wall" => wall = match value {
                    "pull" => erect_core::boon::WallMod::Pull,
                    "push" => erect_core::boon::WallMod::Push,
                    _ => erect_core::boon::WallMod::Plain,
                },
                _ => {}
            }
        }
        game.dev = dev;
        match screen.as_deref() {
            Some("dev") => game.state = erect_core::game::State::DevMenu,
            Some("title") => game.state = erect_core::game::State::Title,
            // `campaigns` opens on the tutorial, `campaigns2` on the campaign:
            // the screen changes its heading with the mode, so both halves of
            // it have to be capturable.
            Some("campaigns") => game.state = erect_core::game::State::Campaigns,
            Some("campaigns2") => {
                game.state = erect_core::game::State::Campaigns;
                game.campaign_mode = 1;
            }
            _ => match written {
                Some((kind, weapon)) => {
                    game.start_written(kind, weapon, 1);
                    // `score=` still applies, so a campaign lull can be reached
                    // without playing the twenty minutes it would cost to earn
                    // the first one.
                    if let Some(p) = game.players.first_mut() {
                        p.score = dev.score;
                    }
                    // And `level=` seeds the weapon, so the end of a campaign
                    // can be photographed without earning every offer between
                    // here and there.
                    if dev.attack_level > 1 {
                        for p in game.players.iter_mut() {
                            p.attack = weapon;
                            p.attack_level = dev.attack_level;
                        }
                    }
                }
                None => game.start_dev_run(),
            },
        }
        shots.sort_by_key(|(at, _)| *at);
        if boons || wall != erect_core::boon::WallMod::Plain {
            for p in game.players.iter_mut() {
                p.boons = erect_core::boon::Boons {
                    double_jump: boons,
                    dash_free: boons,
                    shield: boons,
                    wall,
                };
                // A wall to actually raise, since nothing here earns one.
                p.super_charges = 9;
            }
        }
        Some(Self {
            frames,
            shots,
            on_elite,
            on_elite_delay,
            on_elite_at: None,
            on_offer,
            after_glimpse: on_glimpse.as_ref().map(|p| format!("{p}.dark.png")),
            after_glimpse_at: None,
            on_glimpse,
            glimpse_after,
            on_offer_delay,
            on_offer_at: None,
            keep_alive,
            fight,
            wall: wall != erect_core::boon::WallMod::Plain,
            once,
        })
    }

    /// True when the run is over and the process should stop.
    /// Fills in what a player would be doing.
    pub fn drive(&self, frame: &mut InputFrame, game: &Game, tick: u32) {
        // Jump, then slam: the wall needs the player off the floor.
        if self.wall {
            if let Some(me) = game.players.first() {
                frame.players[0].jump = me.grounded && tick % 90 == 0;
                frame.players[0].slam = !me.grounded && tick % 90 == 6;
            }
        }
        if self.once {
            frame.players[0].attack = tick == 1;
            return;
        }
        if !self.fight {
            return;
        }
        let Some(me) = game.players.first() else {
            return;
        };
        let nearest = game
            .zombies
            .iter()
            .map(|z| z.body.x)
            .chain(game.flyers.iter().map(|f| f.body.x))
            .min_by(|a, b| {
                (a - me.body.x)
                    .abs()
                    .partial_cmp(&(b - me.body.x).abs())
                    .unwrap()
            });
        let intent = &mut frame.players[0];
        if let Some(x) = nearest {
            let away = x - me.body.x;
            intent.right = away > 0.0;
            intent.left = away < 0.0;
        }
        // A swing every few ticks: held down, most kinds would only swing once.
        intent.attack = tick % 12 == 0;
        intent.attack_held = true;
    }

    /// Before the tick, so the players never reach a frame dead.
    pub fn before_tick(&self, game: &mut Game) {
        if self.wall {
            for p in game.players.iter_mut() {
                p.super_charges = p.super_charges.max(1);
            }
        }
        if self.keep_alive {
            for p in game.players.iter_mut() {
                p.hp = p.hpmax;
                p.dead = false;
            }
        }
    }

    pub fn after_frame(&mut self, frame: u32, game: &Game) -> bool {
        if self.on_glimpse.is_some() && frame >= self.glimpse_after && game.enemies_visible() {
            let path = self.on_glimpse.take().unwrap();
            get_screen_data().export_png(&path);
            println!("harness: frame {frame} (enemies showing) -> {path}");
            self.after_glimpse_at = Some(frame + 5);
        }
        if self.after_glimpse_at.is_some_and(|due| frame >= due) && !game.enemies_visible() {
            self.after_glimpse_at = None;
            if let Some(path) = self.after_glimpse.take() {
                get_screen_data().export_png(&path);
                println!("harness: frame {frame} (enemies hidden) -> {path}");
            }
        }
        if self.on_offer.is_some() {
            if self.on_offer_at.is_none() && game.offer.is_some() {
                self.on_offer_at = Some(frame + self.on_offer_delay);
            }
            if self.on_offer_at.is_some_and(|due| frame >= due) {
                let path = self.on_offer.take().unwrap();
                get_screen_data().export_png(&path);
                println!("harness: frame {frame} (offer standing) -> {path}");
            }
        }
        if self.on_elite.is_some() {
            if self.on_elite_at.is_none() && game.elite_notice().is_some() {
                self.on_elite_at = Some(frame + self.on_elite_delay);
            }
            if self.on_elite_at.is_some_and(|due| frame >= due) {
                let path = self.on_elite.take().unwrap();
                get_screen_data().export_png(&path);
                println!("harness: frame {frame} (heavy, +{} frames) -> {path}", self.on_elite_delay);
            }
        }
        while let Some((at, path)) = self.shots.first() {
            if frame < *at {
                break;
            }
            // Straight off the framebuffer, so it is exactly what a player
            // would be looking at - pad, letterboxing and all.
            get_screen_data().export_png(path);
            println!("harness: frame {frame} -> {path}");
            self.shots.remove(0);
        }
        // The frame cap ends the run whatever happened. An event that never
        // fires is a result to report, not a reason to sit there drawing.
        if frame < self.frames {
            return false;
        }
        if let Some(path) = self.on_elite.take() {
            println!("harness: no heavy was ever announced, {path} not written");
        }
        if let Some(path) = self.on_offer.take() {
            println!("harness: no offer ever stood, {path} not written");
        }
        true
    }
}
