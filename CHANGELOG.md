# Changelog

What changed for players, release by release. The release workflow publishes
the section for a tag as that release's notes (`tools/release-notes.sh`), so a
section is written before the tag is pushed and headed with the version alone,
as `## 0.3` is for `v0.3`. Releases before 0.3 are on the repository's Releases
page.

## 0.3

### New

- **HEAD START**, a third weapon run beside the tutorial and the campaign: a
  campaign that starts on wave 7 with 5000 points and the weapon already in
  hand. Each weapon's head start is locked until that weapon has been taken in
  its campaign; the weapon-runs screen greys START out and says so.
- **Second levels for three boons**, offered once the first level is held:
  - DOUBLE JUMP 2 - the air jump comes back when you are hit or caught in a
    blast.
  - FREE DASH 2 - the dash costs a fifth of the energy.
  - SHIELD 2 - a hit the shield takes goes off as a blast that hurts what is
    near.
- **AUTO ATTACK** boon: holding attack keeps swinging with any weapon, resting
  four frames between swings. Not offered while carrying SHOT, which already
  does this.
- **HOW TO PLAY** and **CREDITS** on the title screen: how the game plays,
  scrolling, in English or Russian; and who made it, with its licences and
  version.
- **SETTINGS > CONTROLS**: every game action - left, right, jump, attack, dash,
  slam - can be moved to other keys or buttons, on every platform with a
  keyboard or a pad (not the on-screen touch pad). Each action takes up to
  three, one key may serve several actions, and each scheme can be put back as
  it ships. Pause and the menus keep fixed keys, so a layout can always be
  fixed from the menu.
- **PIERCE slows the game down** while an enemy is within its reach - in a
  straight line from full speed at the tip of the reach to a quarter of the
  speed with the enemy right against you. The slow motion is smooth: frames
  between two steps of the game are drawn part of the way between them.

### Changed

- **LUNGE carries what it hits** instead of knocking it away: the enemy is held
  just inside the tip as the lunge reaches out, keeps taking the swing, and
  walks back in once the swing is over. An enemy caught next to you now takes
  about twice the damage from one lunge.
- When one swing reaches more than one box in the between-wave offer, it takes
  the nearest one in the direction you face. Facing left with a long swing used
  to take the farther one.
- A plain-attack campaign runs until every boon is held, second levels and
  AUTO ATTACK included.
- The developer menu sets each boon to off, level 1 or level 2, and has a row
  for AUTO ATTACK.

### PSP

- Runs on real hardware; 0.2 started to a black screen on a PSP Go. Loading
  the soundtracks now shows a progress counter instead of a black screen, never
  asks for more memory than is free - running out used to hang the PSP - and
  plays with as many soundtracks as fit, or silently and saying why.
  Floating-point traps, the data cache and the audio thread's stack, all of
  which an emulator forgives, are set up the way real hardware needs.
- Music and settings are found beside the EBOOT, so the game also runs from a
  PSP Go's internal storage (`ef0:`) and from a renamed folder.
- L jumps and R dashes, as well as Cross and Triangle.
- The game is listed as "E-Rect" in the PSP's menu instead of "erect-psp".
- Which head starts are unlocked, and the controls, are saved. Settings files
  from 0.2 are read and carried over.

### Browser and Android

- The browser build starts full-window like a desktop and switches to the
  on-screen pad the first time the page is touched; a key press switches back.
  F2 toggles the pad by hand.
- The APK now carries its version (0.3.0), so Android recognises the next one
  as an update.

### Windows

- `erect.exe` built by the release workflow links the C runtime in, so it no
  longer needs the Visual C++ runtime installed.
