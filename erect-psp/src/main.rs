//! E-Rect for the PSP.
//!
//! The simulation is `erect-core`, byte for byte the same code the desktop
//! build runs. This crate only supplies the platform: sceGu drawing, sceCtrl
//! input, and Memory Stick saves.
//!
//! One tick per vblank. The PSP refreshes at ~60 Hz, which is exactly the rate
//! the core's constants are tuned for, so no accumulator is needed here.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

extern crate alloc;

#[cfg(feature = "screenshot")]
mod capture;
mod font;
mod gfx;
mod input;
mod persist;
mod render;
mod sound;

use erect_core::game::Game;
use erect_core::geom::Viewport;
use psp::sys;

psp::module!("erect", 1, 0);

/// Turns off floating-point traps for the calling thread.
///
/// On hardware the Allegrex FPU can trap on a division by zero, an overflow or
/// a NaN, and a trap kills the thread on the spot; PPSSPP does not model FPU
/// exceptions at all, so code that is clean there proves nothing about this.
/// pspsdk ships `pspSdkDisableFPUExceptions` for exactly that reason, and this
/// is what it does: keep the condition bit, clear the rest of FCR31 - every
/// enable, every cause, round-to-nearest. The game expects IEEE behaviour,
/// infinities and all, the same as on every other platform it runs on.
pub fn quiet_fpu() {
    unsafe {
        core::arch::asm!(
            "cfc1 $8, $31",
            "lui $9, 0x80",
            "and $8, $8, $9",
            "ctc1 $8, $31",
            out("$8") _,
            out("$9") _,
            options(nomem, nostack),
        );
    }
}

fn psp_main() {
    quiet_fpu();
    psp::enable_home_button();

    unsafe {
        gfx::init();
        // Something on screen before anything that can take time or fail, so
        // a black screen means the GE itself, not a slow Memory Stick.
        render::render_loading(0, sound::files_to_load());
    }

    let viewport = Viewport::new(gfx::SCREEN_W as f32, gfx::SCREEN_H as f32);
    let mut settings = persist::load();
    let seed = new_seed(&mut settings);
    let mut game = Game::new(
        viewport,
        seed,
        settings,
        &input::SCHEMES,
        // One controller, so the core hides two-player mode by itself.
        1,
    );

    // Silence is the fallback: a missing pack must not stop the game.
    let total = sound::files_to_load();
    let has_sound =
        unsafe { sound::start(seed, &mut |done| render::render_loading(done, total)) };

    let mut pad = unsafe { input::Input::new() };
    #[cfg(feature = "screenshot")]
    let mut frame_no: u32 = 0;

    loop {
        let frame = unsafe { pad.read() };
        game.tick(&frame);

        if has_sound {
            sound::set_roll(game.audio_roll);
            sound::set_volumes(game.settings.music_volume, game.settings.sfx_volume);
            let state = game.music_state();
            sound::set_state(&state);
            for event in game.audio.drain() {
                sound::fire(event);
            }
        } else {
            game.audio.clear();
        }

        // The core flags changed settings; writing them is the platform's job.
        if game.settings.dirty {
            persist::save(&game.settings);
            game.settings.dirty = false;
        }

        if game.quit_requested {
            unsafe {
                gfx::term();
                sys::sceKernelExitGame();
            }
        }

        // Capture the title screen, then a live gameplay frame, and stop.
        #[cfg(feature = "screenshot")]
        {
            frame_no += 1;
            if frame_no == 30 {
                capture::dump(b"ms0:/erect_title.raw\0");
                game.start_run(1);
            } else if frame_no == 160 {
                capture::dump(b"ms0:/erect_play.raw\0");
                capture::note(
                    b"ms0:/erect_state.txt\0",
                    &alloc::format!(
                        "state={:?} wave={} kind={:?} zombies={} flyers={} score={}\n",
                        game.state,
                        game.wave,
                        game.waves.kind,
                        game.zombies.len(),
                        game.flyers.len(),
                        game.players.first().map(|p| p.score).unwrap_or(-1),
                    ),
                );
                unsafe { sys::sceKernelExitGame() };
            }
        }

        unsafe {
            gfx::begin_frame(gfx::pack(game.background.to_rgb()));
            render::render(&game);
            gfx::end_frame();
        }
    }
}

/// A seed that actually differs between launches.
///
/// `sceKernelGetSystemTimeLow` is emulated uptime, and the game reaches it at
/// the same point every time: PPSSPP handed back 500506 on run after run, which
/// made every wave, every special, every skyline and the pack choice identical
/// from one launch to the next. The real-time clock moves; the launch counter
/// kept in the save file moves even if the clock does not.
fn new_seed(settings: &mut erect_core::settings::Settings) -> u64 {
    let mut tick: u64 = 0;
    unsafe {
        sys::sceRtcGetCurrentTick(&mut tick);
    }
    settings.launches = settings.launches.wrapping_add(1);
    settings.dirty = true;

    let uptime = unsafe { sys::sceKernelGetSystemTimeLow() } as u64;
    tick ^ uptime.rotate_left(21) ^ settings.launches.rotate_left(42)
}
