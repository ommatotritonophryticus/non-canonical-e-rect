# E-Rect

A wave-survival brawler drawn entirely from rectangles. One body of game logic
runs on a desktop (macOS, Windows, Linux), in a browser, on Android and on a
PlayStation Portable.

Waves come at you across an endless field; between them you pick upgrades -
nine weapons, each up to a third level that changes how it works, and a set of
boons kept for the run. Besides the endless run there are weapon runs: a
tutorial, a campaign and a head start for every weapon. How it all plays is in
`tools/dist-readme/game.txt`, which ships as part of the README in every
archive; what changed in each release is in `CHANGELOG.md`.

## Layout

| Crate | What it is |
| --- | --- |
| `erect-core` | Platform-free game logic: simulation, waves, menus, settings data. `no_std`. |
| `erect-audio` | Adaptive music mixing: layer gains, fades and a software mixer. `no_std`. |
| `erect-desktop` | Desktop and browser frontend — macroquad rendering, keyboard, gamepads, and an on-screen pad for touch screens. The Android app is this browser build in a WebView. |
| `erect-psp` | PSP frontend — sceGu rendering, sceCtrl input, settings saved beside the EBOOT. |

`erect-core` knows nothing about windows, GPUs, gamepads or filesystems. A
frontend supplies an `InputFrame` each tick, the list of control schemes the
platform offers, and loading/saving of settings; everything else is shared.

Supporting directories:

- `packs/` — runtime sound packs the game loads at startup, in both encodings
  (FLAC for the desktop, 8-bit PCM for the PSP).
- `audio/` — the source packs those are built from, plus a `source.json` each.
- `web/` — the page and the JavaScript side of the browser build.
- `android/` — the activity and manifest the APK wraps the browser build in.
- `tools/` — `release.sh`, `web.sh` and `android.sh` build the artefacts,
  `release-notes.sh` prints a release's changelog section, `build_pack.py`
  turns a source pack into a runtime pack, and `dist-readme/` holds the READMEs
  that ship inside the archives.

## Building

Shippable archives for every platform, into `dist/`:

```
tools/release.sh       # macOS, Windows, Linux x86_64 and aarch64, PSP
tools/web.sh serve     # the browser build, served to the local network
tools/android.sh       # the APK
```

`BUILDING.md` covers the prerequisites, the pinned toolchains, what is built
in a container rather than on the host, and how a release is cut. The rest of
this section is the short version, for working on the game rather than
releasing it.

Desktop:

```
cargo run --release -p erect-desktop
```

The binary is named `erect`. On Linux it needs the usual X11/GL libraries; on
Windows and macOS there are no system dependencies.

Tests — the logic crates carry their own suites and run on the host:

```
cargo test
```

PSP, from inside its own directory:

```
cd erect-psp
cargo psp --release
```

`erect-psp` is deliberately excluded from the workspace: it only builds for the
`mipsel-sony-psp` target, and including it would break a plain `cargo build` on
the host. It needs a nightly toolchain and `cargo-psp`.

The PSP build runs on real hardware: it is tested on a PSP Go (N1000), firmware
6.60 with custom firmware, and in PPSSPP 1.20.4. Other PSP models have not been
tried yet.

## Sound packs

`packs/` holds generated assets, committed so the game runs straight from a
clone. To rebuild one from its source:

```
cd tools
python3 build_pack.py --src ../audio/pack1 --out ../packs/pack1
```

The PSP layers are 8-bit PCM rather than ADPCM on purpose: this music is
chiptune, and a predictive codec destroys square waves. The reasoning, with
measurements, is at the top of `build_pack.py`.

## Licences

The game is GPL-3.0; see `LICENSE`. The music and sound effects, both the
sources in `audio/` and the packs built from them, are original work by the
project author and are covered by that same licence.

The font is **Tiny5** by the Tiny5 Project Authors, under the SIL Open Font
Licence 1.1 — see `LICENSE-Tiny5.txt`. It ships inside every build: embedded in
the desktop binary, and baked into a bitmap atlas for the PSP at build time, so
the notice travels with each release archive as well.

Third-party crates are all permissive — MIT, Apache-2.0, BSD, ISC, Zlib, or a
choice including one of those — with a single MPL-2.0 (`option-ext`, reached
through `directories`). Nothing in the dependency graph conflicts with GPL-3.0.
