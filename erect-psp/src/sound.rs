//! PSP audio: load the pack off the Memory Stick, mix on a dedicated thread.
//!
//! `sceAudioOutputBlocking` sleeps until the hardware has taken the buffer,
//! which is exactly what an audio thread wants and exactly what the vsync-locked
//! render loop must not do - a 1024-sample block is 23 ms against a 16.7 ms
//! frame, so mixing on the main thread would drag the game down to 43 fps.
//!
//! The two threads share state through atomics rather than a lock: the whole
//! music state packs into one word, and one-shot cues are just counters.

use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};

use erect_audio::packs;
use erect_audio::{GainEngine, Mixer, Samples, SfxSpec};
use erect_core::audio::{AudioEvent, MusicState, Situation};
use psp::sys::{self, AudioFormat, IoOpenFlags, ThreadAttributes};

/// Multiple of 64, as the hardware requires. 1024 frames is 23 ms of audio.
const BLOCK: usize = 1024;
/// Where the packs are looked for, in order. The first is relative to the
/// EBOOT's own folder - the psp crate changes into it before `psp_main` - which
/// is the only spelling that works on a PSP Go, whose internal storage is
/// `ef0:` rather than `ms0:`. The absolute one is kept for anything that starts
/// the EBOOT without telling it where it lives.
const PACK_ROOTS: [&[u8]; 2] = [b"pack/", b"ms0:/PSP/GAME/ERECT/pack/"];
/// What the game itself must still be able to allocate once the music is in.
/// Entities, per-frame vectors and HUD strings all come out of the same
/// partition, and running it dry does not fail - the allocator spins forever.
const HEADROOM: usize = 2 * 1024 * 1024;
/// The audio thread's stack. 8 KB used to be enough on PPSSPP, which does not
/// care what a thread overruns; real hardware does.
const AUDIO_STACK: i32 = 64 * 1024;

/// `situation | zombies | flyers | boss`, packed so a single atomic carries it.
static MUSIC_STATE: AtomicU32 = AtomicU32::new(0);
/// One counter per cue; the audio thread takes and clears them.
static CUE_HIT: AtomicU32 = AtomicU32::new(0);
static CUE_JUMP: AtomicU32 = AtomicU32::new(0);
static CUE_SLAM: AtomicU32 = AtomicU32::new(0);
/// `music | sfx << 8`, both 0..=100.
static VOLUMES: AtomicU32 = AtomicU32::new(100 | (100 << 8));
/// Which pack the mixer should be playing. The core re-rolls on arriving at the
/// menu and on starting a run; both are already loaded, so the swap is free.
static WANT_PACK: AtomicU32 = AtomicU32::new(0);
/// Free bytes left after the packs were loaded, for the title screen to show.
static FREE_AFTER_LOAD: AtomicU32 = AtomicU32::new(0);
/// Why there is no sound, if there is none. Silence used to be indistinguishable
/// from a missing pack, a refused channel or a dead thread, which left nothing
/// to go on when a build came up quiet on someone else's machine.
static STATUS: AtomicU32 = AtomicU32::new(SoundStatus::Starting as u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundStatus {
    Starting = 0,
    Playing = 1,
    /// A file under `pack/` beside the EBOOT could not be opened or read.
    NoPack = 2,
    /// `sceAudioChReserve` refused; every channel was already taken.
    NoChannel = 3,
    /// The audio thread could not be created.
    NoThread = 4,
    /// Not even one pack fits in what memory is left.
    NoMemory = 5,
}

pub fn status() -> SoundStatus {
    match STATUS.load(Ordering::Relaxed) {
        1 => SoundStatus::Playing,
        2 => SoundStatus::NoPack,
        3 => SoundStatus::NoChannel,
        4 => SoundStatus::NoThread,
        5 => SoundStatus::NoMemory,
        _ => SoundStatus::Starting,
    }
}

impl SoundStatus {
    /// Short enough for the title screen, specific enough to act on.
    pub fn message(self) -> Option<&'static str> {
        match self {
            SoundStatus::Starting | SoundStatus::Playing => None,
            SoundStatus::NoPack => Some("NO SOUND: PACK NOT FOUND"),
            SoundStatus::NoChannel => Some("NO SOUND: NO FREE AUDIO CHANNEL"),
            SoundStatus::NoThread => Some("NO SOUND: AUDIO THREAD FAILED"),
            SoundStatus::NoMemory => Some("NO SOUND: NOT ENOUGH MEMORY"),
        }
    }
}

fn pack_state(s: &MusicState) -> u32 {
    let situation = match s.situation {
        Situation::Calm => 0u32,
        Situation::Combat => 1,
        Situation::Paused => 2,
    };
    situation | ((s.zombies.min(255) as u32) << 2) | ((s.flyers.min(255) as u32) << 10)
        | ((s.boss as u32) << 18)
}

fn unpack_state(v: u32) -> MusicState {
    MusicState {
        situation: match v & 3 {
            0 => Situation::Calm,
            1 => Situation::Combat,
            _ => Situation::Paused,
        },
        zombies: ((v >> 2) & 0xff) as usize,
        flyers: ((v >> 10) & 0xff) as usize,
        boss: (v >> 18) & 1 == 1,
    }
}

/// Called from the game loop each frame.
pub fn set_state(state: &MusicState) {
    MUSIC_STATE.store(pack_state(state), Ordering::Relaxed);
}

/// Called from the game loop; `roll` is whatever the core last drew.
pub fn set_roll(roll: u64) {
    let picked = packs::choose(roll);
    let idx = packs::PACKS.iter().position(|p| p.dir == picked.dir).unwrap_or(0);
    WANT_PACK.store(idx as u32, Ordering::Relaxed);
}

/// Bytes free once both packs were in memory. Zero before loading finishes.
pub fn free_after_load() -> u32 {
    FREE_AFTER_LOAD.load(Ordering::Relaxed)
}

pub fn set_volumes(music: u32, sfx: u32) {
    VOLUMES.store((music.min(100)) | (sfx.min(100) << 8), Ordering::Relaxed);
}

pub fn fire(event: AudioEvent) {
    let counter = match event {
        AudioEvent::Hit => &CUE_HIT,
        AudioEvent::Jump => &CUE_JUMP,
        AudioEvent::Slam => &CUE_SLAM,
    };
    counter.fetch_add(1, Ordering::Relaxed);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoadError {
    /// Not there, or not readable, under any root.
    Missing,
    /// There, but holding it would leave the game too little to run on.
    Memory,
}

/// Reads a mono 8-bit PCM WAV from `<root>/<pack>/<name>.wav`, trying each of
/// [`PACK_ROOTS`] in turn. Only the shape `build_pack.py` writes.
fn read_wav(pack_dir: &str, name: &str) -> Result<Vec<u8>, LoadError> {
    for root in PACK_ROOTS {
        let mut path = [0u8; 128];
        let mut n = 0;
        for part in [root, pack_dir.as_bytes(), b"/", name.as_bytes(), b".wav\0"] {
            path[n..n + part.len()].copy_from_slice(part);
            n += part.len();
        }
        let fd = unsafe { sys::sceIoOpen(path.as_ptr(), IoOpenFlags::RD_ONLY, 0o777) };
        if fd.0 < 0 {
            continue;
        }
        let got = unsafe { read_open_wav(fd) };
        unsafe { sys::sceIoClose(fd) };
        return got;
    }
    Err(LoadError::Missing)
}

unsafe fn read_open_wav(fd: sys::SceUid) -> Result<Vec<u8>, LoadError> {
    unsafe {
        let size = sys::sceIoLseek(fd, 0, sys::IoWhence::End);
        sys::sceIoLseek(fd, 0, sys::IoWhence::Set);
        if size <= 44 {
            return Err(LoadError::Missing);
        }
        let size = size as usize;
        // Asked for up front, and refused rather than attempted when it would
        // not leave enough behind: a failed allocation does not come back, it
        // hangs the PSP on a black screen.
        if sys::sceKernelMaxFreeMemSize() < size + HEADROOM {
            return Err(LoadError::Memory);
        }
        let mut raw = Vec::new();
        if raw.try_reserve_exact(size).is_err() {
            return Err(LoadError::Memory);
        }
        raw.resize(size, 0);
        // Read in a loop. `sceIoRead` is allowed to hand back less than asked
        // for, and these files are about a megabyte each; a single call that
        // came up short used to make the whole pack fail silently.
        let mut done = 0usize;
        while done < raw.len() {
            let got = sys::sceIoRead(
                fd,
                raw.as_mut_ptr().add(done) as *mut c_void,
                (raw.len() - done) as u32,
            );
            if got <= 0 {
                break;
            }
            done += got as usize;
        }
        if done < raw.len() {
            return Err(LoadError::Missing);
        }

        // Walk the RIFF chunks to find `data`; the header is not always 44
        // bytes. The samples are slid to the front of the buffer they were
        // read into rather than copied out: a copy would hold two megabytes
        // for a moment per file, and that moment is where memory runs out.
        let mut p = 12usize;
        while p + 8 <= raw.len() {
            let id = &raw[p..p + 4];
            let len = u32::from_le_bytes([raw[p + 4], raw[p + 5], raw[p + 6], raw[p + 7]])
                as usize;
            if id == b"data" {
                let end = (p + 8 + len).min(raw.len());
                raw.copy_within(p + 8..end, 0);
                raw.truncate(end - (p + 8));
                return Ok(raw);
            }
            p += 8 + len + (len & 1);
        }
        Err(LoadError::Missing)
    }
}

struct PackData {
    layers: Vec<Vec<u8>>,
    sfx: Vec<Vec<u8>>,
}

/// Every shipped pack, loaded once and never freed so the mixer can hold plain
/// references. Both are held because the soundtrack changes on reaching the menu
/// and on starting a run - re-reading six megabytes off the Memory Stick at
/// those moments would stall the game for seconds.
static mut PACKS_DATA: Option<Vec<PackData>> = None;

/// Files [`start`] reads, for a progress display.
pub fn files_to_load() -> u32 {
    packs::PACKS
        .iter()
        .map(|p| (packs::LAYER_IDS.len() + p.sfx.len()) as u32)
        .sum()
}

/// Loads the packs and starts the audio thread. Returns false when there is
/// nothing to play, in which case the game simply runs silent.
///
/// `progress` is called before every file with how many are done, so the
/// screen can say what is happening: on hardware this takes seconds, and it
/// used to take them on a black screen.
///
/// Packs past the first are dropped, not fatal, when memory runs short - the
/// game then rotates through fewer soundtracks.
///
/// # Safety
/// Call once, before the game loop.
pub unsafe fn start(seed: u64, progress: &mut dyn FnMut(u32)) -> bool {
    set_roll(seed);

    let mut all = Vec::new();
    let mut done = 0u32;
    'packs: for spec in packs::PACKS.iter() {
        let mut layers = Vec::new();
        let mut sfx = Vec::new();
        let names = packs::LAYER_IDS
            .iter()
            .map(|id| (true, *id))
            .chain(spec.sfx.iter().map(|(id, _, _, _)| (false, *id)));
        for (is_layer, id) in names {
            progress(done);
            done += 1;
            match read_wav(spec.dir, id) {
                Ok(v) if is_layer => layers.push(v),
                Ok(v) => sfx.push(v),
                Err(why) => {
                    if all.is_empty() {
                        let status = match why {
                            LoadError::Missing => SoundStatus::NoPack,
                            LoadError::Memory => SoundStatus::NoMemory,
                        };
                        STATUS.store(status as u32, Ordering::Relaxed);
                        return false;
                    }
                    // What this pack already read is dropped here, so the
                    // game gets it back.
                    break 'packs;
                }
            }
        }
        all.push(PackData { layers, sfx });
    }
    progress(done);
    unsafe {
        FREE_AFTER_LOAD.store(sys::sceKernelMaxFreeMemSize() as u32, Ordering::Relaxed);
        PACKS_DATA = Some(all);

        let thread = sys::sceKernelCreateThread(
            b"erect_audio\0".as_ptr(),
            audio_thread,
            0x12,
            AUDIO_STACK,
            ThreadAttributes::USER,
            core::ptr::null_mut(),
        );
        if thread.0 < 0 {
            STATUS.store(SoundStatus::NoThread as u32, Ordering::Relaxed);
            return false;
        }
        sys::sceKernelStartThread(thread, 0, core::ptr::null_mut());
    }
    true
}

unsafe extern "C" fn audio_thread(_args: usize, _argp: *mut c_void) -> i32 {
    // Every thread starts with its own FPU state, traps included.
    crate::quiet_fpu();
    let all = match unsafe { (*core::ptr::addr_of!(PACKS_DATA)).as_ref() } {
        Some(d) => d,
        None => return 0,
    };

    let music_of = |i: usize| -> Vec<Samples> {
        all[i].layers.iter().map(|v| Samples::U8Mono(v)).collect()
    };

    // Fewer packs may have fitted than ship; the roll is folded onto those.
    let wanted = || WANT_PACK.load(Ordering::Relaxed) as usize % all.len();
    let mut current = wanted();
    // Effects are the same three sounds in every pack, so a swap never has to
    // rebuild them.
    let sfx: Vec<SfxSpec> = packs::PACKS[0]
        .sfx
        .iter()
        .enumerate()
        .map(|(i, (id, max_voices, min_ms, gain_db))| SfxSpec {
            event: match *id {
                "hit" => AudioEvent::Hit,
                "jump" => AudioEvent::Jump,
                _ => AudioEvent::Slam,
            },
            samples: Samples::U8Mono(&all[0].sfx[i]),
            gain: erect_audio::gains::db_to_linear(*gain_db),
            max_voices: *max_voices,
            min_interval_frames: (*min_ms * packs::SAMPLE_RATE as f32 / 1000.0) as u64,
        })
        .collect();

    let gains = GainEngine::new(packs::layers(packs::PACKS[current].psp_gains_db));
    let mut mixer = Mixer::new(music_of(current), gains, sfx, packs::SAMPLE_RATE);

    let channel = unsafe {
        sys::sceAudioChReserve(-1, BLOCK as i32, AudioFormat::Stereo)
    };
    if channel < 0 {
        STATUS.store(SoundStatus::NoChannel as u32, Ordering::Relaxed);
        return 0;
    }
    STATUS.store(SoundStatus::Playing as u32, Ordering::Relaxed);

    let mut buf = vec![0i16; BLOCK * 2];
    let mut last_volumes = u32::MAX;
    loop {
        let want = wanted();
        if want != current {
            current = want;
            mixer.switch_music(
                music_of(current),
                GainEngine::new(packs::layers(packs::PACKS[current].psp_gains_db)),
            );
        }
        let volumes = VOLUMES.load(Ordering::Relaxed);
        if volumes != last_volumes {
            mixer.set_volumes(volumes & 0xff, (volumes >> 8) & 0xff);
            last_volumes = volumes;
        }
        let state = unpack_state(MUSIC_STATE.load(Ordering::Relaxed));
        let muted = state.situation == Situation::Paused;

        for (counter, event) in [
            (&CUE_HIT, AudioEvent::Hit),
            (&CUE_JUMP, AudioEvent::Jump),
            (&CUE_SLAM, AudioEvent::Slam),
        ] {
            let pending = counter.swap(0, Ordering::Relaxed).min(4);
            for _ in 0..pending {
                mixer.fire(event);
            }
        }

        mixer.render(&mut buf, &state, muted);
        unsafe {
            sys::sceAudioOutputBlocking(
                channel,
                sys::AUDIO_VOLUME_MAX as i32,
                buf.as_mut_ptr() as *mut c_void,
            );
        }
    }
}
