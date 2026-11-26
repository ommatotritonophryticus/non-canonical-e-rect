//! The long texts the menus can show: how to play, in two languages, and the
//! credits.
//!
//! Written as paragraphs and wrapped here rather than by each renderer, so the
//! core knows how many lines there are and can scroll them itself. The wrap is
//! by characters, at a width chosen so the widest line still fits a 480-pixel
//! PSP screen at its smallest readable size; a desktop has more room than that
//! at the same proportions.
//!
//! A paragraph starting `# ` is a heading, one starting `- ` is an item whose
//! wrapped lines are indented under it, and an empty one is a blank line.
//!
//! The help covers the same ground as `tools/dist-readme/game.txt`, the README
//! every archive carries. A change to how the game plays belongs in both.

use alloc::string::String;
use alloc::vec::Vec;

/// Characters on one line, at most.
pub const WRAP: usize = 44;

/// Which language the how-to-play screen is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    pub const EVERY: [Lang; 2] = [Lang::En, Lang::Ru];

    /// Written in the language itself, the way a language picker names it.
    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "ENGLISH",
            Lang::Ru => "РУССКИЙ",
        }
    }
}

/// One line of a wrapped text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    /// Drawn brighter: the start of a section.
    pub heading: bool,
}

/// How to play, wrapped.
pub fn help(lang: Lang) -> Vec<Line> {
    wrap(match lang {
        Lang::En => HELP_EN,
        Lang::Ru => HELP_RU,
    })
}

/// Who made it and under what terms, wrapped. `version` is printed under the
/// title.
pub fn credits(version: &str) -> Vec<Line> {
    let mut out = wrap(&[&alloc::format!("# E-RECT {version}")]);
    out.extend(wrap(CREDITS));
    out
}

/// The name the credits give the author.
pub const AUTHOR: &str = "ommatotritonophryticus";

const CREDITS: &[&str] = &[
    "",
    "# MADE BY",
    AUTHOR,
    "Game, music and sound effects.",
    "",
    "# LICENCE",
    "E-Rect is free software under the GNU General Public License, version 3: \
     you may share it and change it under its terms. The full text is in \
     LICENSE, next to the game.",
    "",
    "The music and sound effects are original work by the author, under the \
     same licence.",
    "",
    "# FONT",
    "Tiny5 by the Tiny5 Project Authors, under the SIL Open Font License 1.1 - \
     see LICENSE-Tiny5.txt.",
    "",
    "# MADE WITH",
    "Rust; macroquad and miniquad on the desktop and in the browser, and the \
     psp crate on the PSP. The libraries are under MIT, Apache-2.0 and similar \
     licences.",
];

const HELP_EN: &[&str] = &[
    "Waves of rectangles come at you from both sides of a field that runs on \
     forever. Clear a wave, take a breath in the lull, and the next one starts \
     after a countdown.",
    "",
    "# CONTROLS",
    "- LEFT, RIGHT: walk.",
    "- JUMP: off the ground.",
    "- ATTACK: swing your weapon. Three swings in quick succession make a \
     combo, and the third reaches twice as far.",
    "- DASH: a short burst that shoves what it passes through; nothing can \
     touch you while it lasts. It costs a third of your energy bar.",
    "- SLAM: in the air, drop fast and raise the wall on landing if you hold a \
     charge. The wall hits everything it reaches.",
    "Every one of these can be moved to other buttons in SETTINGS, CONTROLS.",
    "",
    "# ENERGY",
    "Hitting and killing fills the energy bar, and a full bar becomes a wall \
     charge, counted at the top of the screen. The more hurt you are, the \
     faster it fills - three times as fast on the brink.",
    "",
    "# UPGRADES",
    "Every 4000 points three boxes stand in the lull. Hit one to take it; a \
     swing that reaches several takes the nearest in front of you. Starting \
     the next wave early turns the offer down.",
    "From wave 26 there is an offer after every wave, and two of its boxes are \
     terms for the coming wave only.",
    "",
    "# WEAPONS",
    "One at a time, each up to level 3. The third level, MASTER, changes what \
     the weapon does. A different weapon arrives one level below the one you \
     carry.",
    "- BASIC: the third hit of a combo leaves a blast that hurts everything \
     around it.",
    "- HAMMER: short and slow; one heavy blow per enemy that throws it far.",
    "- PIERCE: does not throw, so the enemy keeps taking the swing. While an \
     enemy is in its reach the game slows down - to a quarter of its speed \
     point-blank.",
    "- LUNGE: reaches out through the swing and carries what it catches on its \
     tip.",
    "- SINGLE: one sure hit per enemy per swing.",
    "- SWEEP: a low strip out of both sides at once.",
    "- HEAVY: shorter, but covers you top to bottom.",
    "- SHOT: throws a square; hold attack to keep firing.",
    "- TRAP: places boxes that hurt whatever walks into them.",
    "",
    "# BOONS",
    "Kept for the rest of the run. A second level is offered once you hold the \
     first.",
    "- DOUBLE JUMP: one jump in the air. Level 2: it also comes back when you \
     are hit or caught in a blast.",
    "- FREE DASH: no cooldown. Level 2: a fifth of the energy.",
    "- SHIELD: takes one hit, back a second later. Level 2: that hit goes off \
     as a blast.",
    "- BLACK WALL, GREY WALL: the wall drags every enemy in, or throws them \
     all away.",
    "- AUTO ATTACK: hold attack to keep swinging.",
    "",
    "# MODES",
    "- 1 PLAYER, 2 PLAYERS: the endless run. It ends when you do.",
    "- WEAPON RUNS: one weapon, from start to end. TUTORIAL is three waves that \
     raise it a level each. CAMPAIGN offers only that weapon and the boons, and \
     is won one wave after the weapon reaches MASTER. HEAD START is a campaign \
     from wave 7 with 5000 points and the weapon in hand, unlocked by taking \
     that weapon in its campaign.",
    "",
    "# WAVES",
    "Every fifth wave brings bosses; the one on wave 10 flies and blinks away \
     when hit. A wave can carry modifiers, announced in the countdown: who \
     turns up (GROUND ONLY, FLYERS ONLY, HORDE, JUMPERS) and how it plays \
     (HOLD THE LINE, GROUNDED, NO WALL, BLIND, NO FOOTING).",
    "Blasts never hurt you, they only throw you up. A shooter's white sight \
     turns red just before it fires - step out of the line.",
];

const HELP_RU: &[&str] = &[
    "Волны прямоугольников идут на вас с обеих сторон бесконечного поля. \
     Зачистите волну, передохните в паузе, и после отсчёта начнётся \
     следующая.",
    "",
    "# УПРАВЛЕНИЕ",
    "- LEFT, RIGHT: ходьба.",
    "- JUMP: прыжок.",
    "- ATTACK: удар оружием. Три удара подряд складываются в комбо, и третий \
     бьёт вдвое дальше.",
    "- DASH: короткий рывок, расталкивающий всех на пути; пока он длится, вас \
     нельзя задеть. Стоит треть шкалы энергии.",
    "- SLAM: в воздухе - быстро упасть и при приземлении поднять стену, если \
     есть заряд. Стена бьёт всех, кого достаёт.",
    "Любое из этих действий можно перенести на другие кнопки в SETTINGS, \
     CONTROLS.",
    "",
    "# ЭНЕРГИЯ",
    "Удары и убийства заполняют шкалу энергии, а полная шкала становится \
     зарядом стены - их число видно вверху экрана. Чем сильнее вы ранены, тем \
     быстрее она растёт: на грани смерти втрое быстрее.",
    "",
    "# УЛУЧШЕНИЯ",
    "Каждые 4000 очков в паузе между волнами встают три ящика. Ударьте по \
     одному, чтобы взять его; если удар достаёт несколько, берётся ближайший \
     перед вами. Начать следующую волну раньше - значит отказаться.",
    "С 26-й волны предложение есть после каждой волны, и два ящика из трёх - \
     условия только для следующей волны.",
    "",
    "# ОРУЖИЕ",
    "Одно за раз, каждое до 3-го уровня. Третий уровень, MASTER, меняет то, \
     как оружие работает. Новое оружие приходит на уровень ниже того, что у \
     вас в руках.",
    "- BASIC: третий удар комбо оставляет взрыв, ранящий всех вокруг.",
    "- HAMMER: короткий и медленный; один тяжёлый удар по каждому врагу, \
     отбрасывающий его далеко.",
    "- PIERCE: не отбрасывает, и враг продолжает получать удар. Пока враг в \
     зоне досягаемости, игра замедляется - вплотную до четверти скорости.",
    "- LUNGE: вытягивается за время удара и несёт пойманного врага на своём \
     конце.",
    "- SINGLE: один гарантированный удар по каждому врагу за взмах.",
    "- SWEEP: низкая полоса сразу в обе стороны.",
    "- HEAVY: короче, но закрывает вас сверху донизу.",
    "- SHOT: бросает квадрат; держите атаку, чтобы стрелять без остановки.",
    "- TRAP: ставит ящики, ранящие всех, кто в них зайдёт.",
    "",
    "# БОНУСЫ",
    "Остаются до конца забега. Второй уровень предлагается, когда первый уже \
     есть.",
    "- DOUBLE JUMP: прыжок в воздухе. Уровень 2: он возвращается и при \
     попадании по вам или во взрыве.",
    "- FREE DASH: рывок без перезарядки. Уровень 2: за пятую часть энергии.",
    "- SHIELD: принимает один удар и возвращается через секунду. Уровень 2: \
     этот удар взрывается.",
    "- BLACK WALL, GREY WALL: стена стягивает всех врагов к вам или \
     отбрасывает их прочь.",
    "- AUTO ATTACK: держите атаку, чтобы бить без остановки.",
    "",
    "# РЕЖИМЫ",
    "- 1 PLAYER, 2 PLAYERS: бесконечный забег, который длится, пока вы живы.",
    "- WEAPON RUNS: одно оружие от начала до конца. TUTORIAL - три волны, \
     каждая поднимает уровень оружия. CAMPAIGN предлагает только это оружие и \
     бонусы и выиграна через волну после того, как оружие стало MASTER. HEAD \
     START - кампания с 7-й волны, с 5000 очков и оружием в руках; \
     открывается, когда вы взяли это оружие в его кампании.",
    "",
    "# ВОЛНЫ",
    "Каждая пятая волна приводит боссов; босс 10-й волны летает и исчезает \
     при каждом попадании. У волны бывают модификаторы, их объявляют во время \
     отсчёта: кто придёт (GROUND ONLY, FLYERS ONLY, HORDE, JUMPERS) и как \
     играть (HOLD THE LINE, GROUNDED, NO WALL, BLIND, NO FOOTING).",
    "Взрывы никогда вас не ранят, только подбрасывают. Белый прицел стрелка \
     краснеет прямо перед выстрелом - уйдите с линии.",
];

/// Breaks paragraphs into lines of at most [`WRAP`] characters, at spaces.
fn wrap(paragraphs: &[&str]) -> Vec<Line> {
    let mut out = Vec::new();
    for p in paragraphs {
        if p.is_empty() {
            out.push(Line { text: String::new(), heading: false });
            continue;
        }
        let (body, heading, first, rest) = if let Some(h) = p.strip_prefix("# ") {
            (h, true, "", "")
        } else if let Some(item) = p.strip_prefix("- ") {
            (item, false, "- ", "  ")
        } else {
            (*p, false, "", "")
        };
        let mut line = String::from(first);
        let mut indent = first.chars().count();
        for word in body.split(' ').filter(|w| !w.is_empty()) {
            let used = line.chars().count();
            let fits = used == indent || used + 1 + word.chars().count() <= WRAP;
            if !fits {
                out.push(Line { text: core::mem::take(&mut line), heading });
                line.push_str(rest);
                indent = rest.chars().count();
            }
            if line.chars().count() > indent {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(Line { text: line, heading });
    }
    out
}
