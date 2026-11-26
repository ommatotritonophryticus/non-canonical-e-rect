//! Settings on the Memory Stick, via sceIo.
//!
//! A fixed-size binary record rather than JSON: no parser to carry, and a
//! corrupt or truncated file simply falls back to defaults.

use erect_core::attack::AttackKind;
use erect_core::binds::{Action, Bindings, SLOTS};
use erect_core::settings::Settings;
use psp::sys::{self, IoOpenFlags};

/// Beside the EBOOT, wherever that is: the psp crate changes into its folder
/// before `psp_main`. On a PSP Go that folder is on `ef0:`, internal storage,
/// and a fixed `ms0:` path finds nothing there.
const PATH: &[u8] = b"settings.dat\0";
/// For anything that starts the EBOOT without saying where it lives.
const FALLBACK_PATH: &[u8] = b"ms0:/PSP/GAME/ERECT/settings.dat\0";
const FALLBACK_DIR: &[u8] = b"ms0:/PSP/GAME/ERECT\0";
/// Bumped whenever the layout below changes, so a file is never read with the
/// wrong one.
const MAGIC: u32 = 0x4552_4305;
/// The layouts before this one, still read so that updating keeps records and
/// settings. 3 is the original record; 4 added one bit per weapon for which
/// head starts are open, by `AttackKind::index`.
const MAGIC_V3: u32 = 0x4552_4303;
const RECORD_LEN_V3: usize = 4 + 4 + 4 + 8 + 8 + 4 + 4 + 8;
const MAGIC_V4: u32 = 0x4552_4304;
const RECORD_LEN_V4: usize = RECORD_LEN_V3 + 4;
/// The current layout: version 4, then the pad's buttons if the player changed
/// them - a word that is 1 when they did, and every action's slots as `u16`s,
/// action by action.
const BINDINGS_LEN: usize = Action::EVERY.len() * SLOTS * 2;
const RECORD_LEN: usize = RECORD_LEN_V4 + 4 + BINDINGS_LEN;
// A weapon past the 32nd would have no bit to be saved in.
const _: () = assert!(AttackKind::EVERY.len() <= 32);

pub fn load() -> Settings {
    let mut settings = Settings::default();
    let mut buf = [0u8; RECORD_LEN];

    unsafe {
        let mut fd = sys::sceIoOpen(PATH.as_ptr(), IoOpenFlags::RD_ONLY, 0o777);
        if fd.0 < 0 {
            fd = sys::sceIoOpen(FALLBACK_PATH.as_ptr(), IoOpenFlags::RD_ONLY, 0o777);
        }
        if fd.0 < 0 {
            return settings;
        }
        let read = sys::sceIoRead(fd, buf.as_mut_ptr() as *mut _, RECORD_LEN as u32);
        sys::sceIoClose(fd);
        let len = match u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) {
            MAGIC => RECORD_LEN,
            MAGIC_V4 => RECORD_LEN_V4,
            MAGIC_V3 => RECORD_LEN_V3,
            _ => return settings,
        };
        if read < len as i32 {
            return settings;
        }
        if len >= RECORD_LEN_V4 {
            let mask = u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]);
            for (i, open) in settings.head_start_unlocked.iter_mut().enumerate() {
                *open = mask & (1 << i) != 0;
            }
        }
        if len >= RECORD_LEN && u32::from_le_bytes([buf[48], buf[49], buf[50], buf[51]]) == 1 {
            let mut binds = Bindings::default();
            let mut at = 52;
            for action in Action::EVERY {
                for slot in 0..SLOTS {
                    binds.set(action, slot, u16::from_le_bytes([buf[at], buf[at + 1]]));
                    at += 2;
                }
            }
            // Game::new drops any button this build does not have.
            settings.bindings[0] = Some(binds);
        }
    }

    settings.players[0].color_index =
        u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    settings.players[1].color_index =
        u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
    settings.record_solo = i64::from_le_bytes([
        buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19],
    ]);
    settings.record_duo = i64::from_le_bytes([
        buf[20], buf[21], buf[22], buf[23], buf[24], buf[25], buf[26], buf[27],
    ]);
    settings.music_volume = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
    settings.sfx_volume = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);
    settings.launches = u64::from_le_bytes([
        buf[36], buf[37], buf[38], buf[39], buf[40], buf[41], buf[42], buf[43],
    ]);
    settings.dirty = false;
    // Game::new sanitizes whatever came off the stick.
    settings
}

pub fn save(settings: &Settings) {
    let mut buf = [0u8; RECORD_LEN];
    buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    buf[4..8].copy_from_slice(&(settings.players[0].color_index as u32).to_le_bytes());
    buf[8..12].copy_from_slice(&(settings.players[1].color_index as u32).to_le_bytes());
    buf[12..20].copy_from_slice(&settings.record_solo.to_le_bytes());
    buf[20..28].copy_from_slice(&settings.record_duo.to_le_bytes());
    buf[28..32].copy_from_slice(&settings.music_volume.to_le_bytes());
    buf[32..36].copy_from_slice(&settings.sfx_volume.to_le_bytes());
    buf[36..44].copy_from_slice(&settings.launches.to_le_bytes());
    let mut mask = 0u32;
    for (i, open) in settings.head_start_unlocked.iter().enumerate() {
        if *open {
            mask |= 1 << i;
        }
    }
    buf[44..48].copy_from_slice(&mask.to_le_bytes());
    if let Some(binds) = settings.bindings[0] {
        buf[48..52].copy_from_slice(&1u32.to_le_bytes());
        let mut at = 52;
        for action in Action::EVERY {
            for slot in 0..SLOTS {
                buf[at..at + 2].copy_from_slice(&binds.get(action, slot).to_le_bytes());
                at += 2;
            }
        }
    }

    unsafe {
        let flags = || IoOpenFlags::CREAT | IoOpenFlags::TRUNC | IoOpenFlags::WR_ONLY;
        let mut fd = sys::sceIoOpen(PATH.as_ptr(), flags(), 0o777);
        if fd.0 < 0 {
            // The directory may not exist on a fresh stick; failure is fine.
            sys::sceIoMkdir(FALLBACK_DIR.as_ptr(), 0o777);
            fd = sys::sceIoOpen(FALLBACK_PATH.as_ptr(), flags(), 0o777);
        }
        if fd.0 >= 0 {
            sys::sceIoWrite(fd, buf.as_ptr() as *const _, RECORD_LEN);
            sys::sceIoClose(fd);
        }
    }
}
