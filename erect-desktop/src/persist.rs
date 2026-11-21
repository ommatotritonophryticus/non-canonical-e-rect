//! Loading and saving [`Settings`].
//!
//! The core deliberately has no file access, so this is the whole of the
//! platform's persistence story. Two stores, one format: a JSON file in a
//! per-OS config directory on a desktop, and the same JSON under one
//! localStorage key in a browser. Only the two functions at the very bottom of
//! each half differ.

use erect_core::settings::PlayerConfig;
use erect_core::settings::Settings;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// The stored shape, kept separate from the core type so the save format can
/// change without the simulation caring.
#[derive(Serialize, Deserialize)]
struct StoredSettings {
    players: Vec<StoredPlayer>,
    #[serde(default)]
    record_solo: i64,
    #[serde(default)]
    record_duo: i64,
    #[serde(default = "full_volume")]
    music_volume: u32,
    #[serde(default = "full_volume")]
    sfx_volume: u32,
    /// Only there to keep the seed moving; see `Settings::launches`.
    #[serde(default)]
    launches: u64,
    /// One entry per weapon, in `AttackKind::EVERY` order. Written flat rather
    /// than as objects: nine objects per mode would be most of the room a
    /// browser gives this document, and three parallel lists say the same thing
    /// in a fifth of the space.
    #[serde(default)]
    campaign_cleared: Vec<bool>,
    #[serde(default)]
    campaign_wave: Vec<i64>,
    #[serde(default)]
    campaign_score: Vec<i64>,
    /// The same three lists again for the tutorials. A separate set rather than
    /// a wider one because the two modes are recorded separately, and a save
    /// that predates tutorials must still read as "none attempted" rather than
    /// as a misaligned campaign table.
    #[serde(default)]
    tutorial_cleared: Vec<bool>,
    #[serde(default)]
    tutorial_wave: Vec<i64>,
    #[serde(default)]
    tutorial_score: Vec<i64>,
    /// And again for the head starts, for the same reason: a save from before
    /// them must read as "none attempted".
    #[serde(default)]
    head_start_cleared: Vec<bool>,
    #[serde(default)]
    head_start_wave: Vec<i64>,
    #[serde(default)]
    head_start_score: Vec<i64>,
    /// Which head starts are open, in the same order. Absent in a save from
    /// before they were locked, which reads as "none" - a cleared campaign
    /// opens its head start regardless, so nothing earned is lost.
    #[serde(default)]
    head_start_unlocked: Vec<bool>,
}

/// Missing from an older save file means the setting did not exist yet, and
/// full volume is what the player was hearing at the time.
fn full_volume() -> u32 {
    100
}

#[derive(Serialize, Deserialize)]
struct StoredPlayer {
    scheme: usize,
    color_index: usize,
}

#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("", "", "erect")?;
    Some(dirs.config_dir().join("settings.json"))
}

/// Text to settings. Shared, so the browser and the desktop can never drift
/// into reading the same file differently.
fn parse(text: &str) -> Settings {
    let Ok(stored) = serde_json::from_str::<StoredSettings>(text) else {
        return Settings::default();
    };
    let mut settings = Settings::default();
    for (i, player) in stored.players.iter().take(settings.players.len()).enumerate() {
        settings.players[i] = PlayerConfig {
            scheme: player.scheme,
            color_index: player.color_index,
        };
    }
    settings.record_solo = stored.record_solo;
    settings.record_duo = stored.record_duo;
    settings.music_volume = stored.music_volume;
    settings.sfx_volume = stored.sfx_volume;
    settings.launches = stored.launches;
    // Short lists are what a save from before campaigns existed looks like, and
    // a longer one is a save from a version that knew more weapons than this
    // build does. Both read as "nothing recorded" for whatever is missing.
    for (i, slot) in settings.campaign.iter_mut().enumerate() {
        slot.cleared = stored.campaign_cleared.get(i).copied().unwrap_or(false);
        slot.best_wave = stored.campaign_wave.get(i).copied().unwrap_or(0);
        slot.best_score = stored.campaign_score.get(i).copied().unwrap_or(0);
    }
    for (i, slot) in settings.tutorial.iter_mut().enumerate() {
        slot.cleared = stored.tutorial_cleared.get(i).copied().unwrap_or(false);
        slot.best_wave = stored.tutorial_wave.get(i).copied().unwrap_or(0);
        slot.best_score = stored.tutorial_score.get(i).copied().unwrap_or(0);
    }
    for (i, open) in settings.head_start_unlocked.iter_mut().enumerate() {
        *open = stored.head_start_unlocked.get(i).copied().unwrap_or(false);
    }
    for (i, slot) in settings.head_start.iter_mut().enumerate() {
        slot.cleared = stored.head_start_cleared.get(i).copied().unwrap_or(false);
        slot.best_wave = stored.head_start_wave.get(i).copied().unwrap_or(0);
        slot.best_score = stored.head_start_score.get(i).copied().unwrap_or(0);
    }
    settings.dirty = false;
    // Game::new sanitizes this against the real scheme count.
    settings
}

/// Settings to text. The other half of [`parse`].
fn encode(settings: &Settings) -> Option<String> {
    let stored = StoredSettings {
        players: settings
            .players
            .iter()
            .map(|p| StoredPlayer {
                scheme: p.scheme,
                color_index: p.color_index,
            })
            .collect(),
        record_solo: settings.record_solo,
        record_duo: settings.record_duo,
        music_volume: settings.music_volume,
        sfx_volume: settings.sfx_volume,
        launches: settings.launches,
        campaign_cleared: settings.campaign.iter().map(|c| c.cleared).collect(),
        campaign_wave: settings.campaign.iter().map(|c| c.best_wave).collect(),
        campaign_score: settings.campaign.iter().map(|c| c.best_score).collect(),
        tutorial_cleared: settings.tutorial.iter().map(|c| c.cleared).collect(),
        tutorial_wave: settings.tutorial.iter().map(|c| c.best_wave).collect(),
        tutorial_score: settings.tutorial.iter().map(|c| c.best_score).collect(),
        head_start_cleared: settings.head_start.iter().map(|c| c.cleared).collect(),
        head_start_wave: settings.head_start.iter().map(|c| c.best_wave).collect(),
        head_start_score: settings.head_start.iter().map(|c| c.best_score).collect(),
        head_start_unlocked: settings.head_start_unlocked.to_vec(),
    };
    serde_json::to_string(&stored).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Settings {
    match path().and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(text) => parse(&text),
        None => Settings::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(settings: &Settings) {
    let Some(path) = path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Some(text) = encode(settings) {
        // Losing settings is not worth interrupting play over.
        let _ = std::fs::write(path, text);
    }
}

/* ---------------- browser ---------------- */

// Written by `web/erect_web.js`, which registers them as a miniquad plugin and
// keeps the whole settings document under one localStorage key.
//
// Strings cross as a pointer and a length into the wasm heap; there is no
// allocator on the JS side of this boundary and nothing is retained past the
// call. `read` is handed a buffer to fill and answers how much it used, so the
// Rust side owns every byte involved.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn erect_storage_write(ptr: *const u8, len: u32);
    fn erect_storage_read(ptr: *mut u8, cap: u32) -> u32;
}

/// Room for the settings document.
///
/// Ours to choose, not the browser's: the buffer is allocated on this side and
/// localStorage itself holds megabytes. A kilobyte was enough until the third
/// written mode, whose three lists took a full document past it; four leaves
/// room for several more before this has to move again, and the tests below
/// check the fullest possible document against it.
#[cfg(any(target_arch = "wasm32", test))]
const STORAGE_CAP: usize = 4096;

#[cfg(target_arch = "wasm32")]
pub fn load() -> Settings {
    let mut buf = alloc_buf();
    // A browser can refuse storage outright - private windows do - and it can
    // hold something older or hand-edited. Every one of those means "no
    // settings yet", which is what a first run looks like anyway.
    let len = unsafe { erect_storage_read(buf.as_mut_ptr(), STORAGE_CAP as u32) } as usize;
    if len == 0 || len > STORAGE_CAP {
        return Settings::default();
    }
    buf.truncate(len);
    match core::str::from_utf8(&buf) {
        Ok(text) => parse(text),
        Err(_) => Settings::default(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save(settings: &Settings) {
    if let Some(text) = encode(settings) {
        unsafe { erect_storage_write(text.as_ptr(), text.len() as u32) };
    }
}

#[cfg(target_arch = "wasm32")]
fn alloc_buf() -> Vec<u8> {
    vec![0u8; STORAGE_CAP]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact document a browser was found holding, byte for byte.
    const STORED: &str = r#"{"players":[{"scheme":0,"color_index":0},{"scheme":1,"color_index":1}],"record_solo":0,"record_duo":0,"music_volume":100,"sfx_volume":100,"launches":1}"#;

    #[test]
    fn what_is_written_is_what_is_read_back() {
        let mut settings = Settings::default();
        settings.record_solo = 4321;
        settings.music_volume = 40;
        settings.launches = 7;
        let text = encode(&settings).expect("settings should encode");
        let back = parse(&text);
        assert_eq!(back.record_solo, 4321);
        assert_eq!(back.music_volume, 40);
        assert_eq!(back.launches, 7);
        assert!(!back.dirty, "a freshly loaded document has nothing to save");
    }

    #[test]
    fn a_document_from_a_real_browser_parses() {
        let settings = parse(STORED);
        assert_eq!(settings.launches, 1, "the launch counter should survive");
        assert_eq!(settings.music_volume, 100);
    }

    #[test]
    fn nonsense_reads_as_a_first_run_rather_than_failing() {
        assert_eq!(parse("").launches, 0);
        assert_eq!(parse("{").launches, 0);
        assert_eq!(parse("[1,2,3]").launches, 0);
    }

    #[test]
    fn a_document_with_every_mode_recorded_still_fits() {
        // Nine weapons times three modes, every one cleared, every number at
        // its widest. The browser side refuses to write past its buffer, so a
        // document that outgrew it would silently stop loading - and it would
        // do so only for the players who had got furthest.
        let mut settings = Settings::default();
        settings.record_solo = i64::MAX;
        settings.record_duo = i64::MAX;
        settings.launches = u64::MAX;
        for slot in settings
            .campaign
            .iter_mut()
            .chain(settings.tutorial.iter_mut())
            .chain(settings.head_start.iter_mut())
        {
            slot.cleared = true;
            slot.best_wave = 999;
            slot.best_score = i64::MAX;
        }
        settings.head_start_unlocked = [true; erect_core::attack::AttackKind::EVERY.len()];
        let text = encode(&settings).expect("settings should encode");
        assert!(text.len() < STORAGE_CAP, "the document is {} bytes", text.len());
        let back = parse(&text);
        assert_eq!(back.campaign, settings.campaign, "and it must survive the trip");
        assert_eq!(back.tutorial, settings.tutorial, "every table of it");
        assert_eq!(back.head_start, settings.head_start, "every table of it");
        assert_eq!(back.head_start_unlocked, settings.head_start_unlocked, "and the unlocks");
    }

    #[test]
    fn a_save_from_before_written_runs_still_loads() {
        // Nine lists that are simply absent. Every weapon reads as untouched in
        // every mode, which is what it was.
        let old = r#"{"players":[{"scheme":0,"color_index":0},{"scheme":1,"color_index":1}],"record_solo":700,"record_duo":0,"music_volume":100,"sfx_volume":100,"launches":9}"#;
        let settings = parse(old);
        assert_eq!(settings.record_solo, 700, "the old fields still read");
        assert!(settings.campaign.iter().all(|c| !c.cleared && c.best_wave == 0));
        assert!(settings.tutorial.iter().all(|c| !c.cleared && c.best_wave == 0));
        assert!(settings.head_start.iter().all(|c| !c.cleared && c.best_wave == 0));
        assert!(settings.head_start_unlocked.iter().all(|open| !open));
    }

    #[test]
    fn the_document_fits_the_buffer_the_browser_is_given() {
        // The browser side refuses to write past the buffer, so a document that
        // outgrew it would silently stop loading.
        let mut settings = Settings::default();
        settings.record_solo = i64::MAX;
        settings.record_duo = i64::MAX;
        settings.launches = u64::MAX;
        let text = encode(&settings).expect("settings should encode");
        assert!(text.len() < STORAGE_CAP, "the document is {} bytes", text.len());
    }
}
