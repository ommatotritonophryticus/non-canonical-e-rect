//! Turns one confusing failure into an instruction.
//!
//! `cargo psp` run from the repository root builds the whole workspace for
//! `mipsel-sony-psp` - this crate included, which cannot work: it is macroquad,
//! cpal and gilrs, none of which know what a PSP is. What the developer sees
//! without this is `gilrs` panicking inside its own build script on a missing
//! `CARGO_CFG_TARGET_FAMILY`, which is true (the PSP target has no family) and
//! says nothing at all about the actual mistake.
//!
//! The PSP frontend is a separate workspace for exactly this reason and has to
//! be built from its own directory.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("psp") {
        panic!(
            "\n\n\
             erect-desktop cannot be built for the PSP.\n\n\
             This usually means `cargo psp` was run from the repository root,\n\
             where it tries to build every crate in the workspace for the PSP.\n\
             The PSP frontend is its own workspace - build it from there:\n\n    \
             cd erect-psp && cargo psp --release\n\n"
        );
    }
}
