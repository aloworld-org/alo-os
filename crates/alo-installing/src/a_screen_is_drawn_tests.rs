//! That a screen's lines go where they belong, that nothing is drawn over
//! anything, and — the one that matters — that **bytes actually land**, read
//! back out of a file the real write path wrote.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{TheWordsOfAScreen, what_a_screen_shows};
use crate::alos_own_face::AlosOwnFace;
use crate::where_a_screen_goes::THE_DESIGN_WAS_DRAWN_AT;

fn face() -> AlosOwnFace<'static> {
    AlosOwnFace::carried().expect("the typeface compiled into this program")
}

/// The keyboard question, in the words `words.rs` actually declares — the
/// long introduction included, because that is the case the flow exists for.
fn the_keyboard_question() -> TheWordsOfAScreen {
    TheWordsOfAScreen {
        eyebrow: "BEFORE ANYTHING IS TYPED".to_owned(),
        title: "Which keyboard are you using?".to_owned(),
        introduction: "You will type a network password and the name of a disk. Both have to \
                       be exact, so the keyboard has to be right first"
            .to_owned(),
        cards: vec![
            (
                "Belgisch".to_owned(),
                "Press the keys shown to check: a z ; m".to_owned(),
            ),
            (
                "Something else".to_owned(),
                "Choose from every layout this computer knows".to_owned(),
            ),
        ],
        consequence: "If what you type below does not match, choose another layout".to_owned(),
    }
}

#[test]
fn every_sentence_asked_for_appears_somewhere_on_the_screen() {
    // A screen that silently dropped a sentence would look fine and say less
    // than it was asked to. The eyebrow, the question, the introduction, both
    // cards and the quiet line all have to be there.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let words = the_keyboard_question();
    let shows =
        what_a_screen_shows(across, down, None, face(), &words).expect("the reference lays out");
    let all: String = shows
        .lines
        .iter()
        .map(|line| line.says.clone())
        .collect::<Vec<_>>()
        .join(" ");

    for wanted in [
        "BEFORE ANYTHING IS TYPED",
        "Which keyboard are you using?",
        "Belgisch",
        "Something else",
        "choose another layout",
    ] {
        assert!(all.contains(wanted), "{wanted:?} is nowhere on the screen");
    }
    // And the introduction, which is wrapped, so it is checked by its ends.
    assert!(all.contains("You will type a network password"));
    assert!(all.contains("right first"));
}

#[test]
fn the_long_introduction_wraps_and_the_cards_move_below_where_it_ends() {
    // **The whole reason the block became a flow**, asserted through the
    // assembly rather than through the layout alone: the sentence is 120
    // characters, it does not fit on one line of a 600-wide block, and the
    // first card has to be below its last line.
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let words = the_keyboard_question();
    let shows = what_a_screen_shows(across, down, None, face(), &words).expect("lays out");

    let introduction: Vec<_> = shows
        .lines
        .iter()
        .filter(|line| {
            line.em == shows.places.body.tall && line.down < shows.places.first_card.down
        })
        .collect();
    assert!(
        introduction.len() >= 2,
        "the 120-character introduction came out on {} line(s)",
        introduction.len()
    );

    let lowest = introduction
        .iter()
        .map(|line| line.down + line.line_is_tall)
        .max()
        .expect("the introduction has lines");
    assert!(
        lowest <= shows.places.first_card.down,
        "the introduction ends at {lowest} and the first card starts at {}",
        shows.places.first_card.down
    );
}

#[test]
fn no_line_is_drawn_over_the_line_below_it() {
    // Swept over the sizes the environment lays out for. An overlap is the one
    // fault a person sees instantly, and the assembly is where it would now
    // come from rather than the layout.
    let words = the_keyboard_question();
    for (across, down) in [
        (640_u32, 480_u32),
        (1024, 768),
        (1280, 800),
        (1440, 960),
        (1920, 1080),
        (3840, 2160),
    ] {
        let Some(shows) = what_a_screen_shows(across, down, None, face(), &words) else {
            continue;
        };
        let mut sorted = shows.lines.clone();
        sorted.sort_by_key(|line| line.down);
        for pair in sorted.windows(2) {
            let above = &pair[0];
            let below = &pair[1];
            // Two lines may share a top — a card's name and nothing else does
            // — but a line must never reach past the top of the next one
            // down, unless they are the same line of a card's two.
            if below.down > above.down {
                assert!(
                    above.down + above.line_is_tall <= below.down + below.line_is_tall,
                    "at {across}x{down}, {:?} at {} overlaps {:?} at {}",
                    above.says,
                    above.down,
                    below.says,
                    below.down
                );
            }
        }
    }
}

#[test]
fn every_line_is_inside_the_display() {
    let words = the_keyboard_question();
    for (across, down) in [(640_u32, 480_u32), (1280, 800), (1440, 960), (3840, 2160)] {
        let Some(shows) = what_a_screen_shows(across, down, None, face(), &words) else {
            continue;
        };
        for line in &shows.lines {
            assert!(
                line.down + line.line_is_tall <= down,
                "at {across}x{down}, {:?} reaches {} of {down}",
                line.says,
                line.down + line.line_is_tall
            );
            let wide = face().how_wide(&line.says, line.em);
            assert!(
                line.across + wide <= across,
                "at {across}x{down}, {:?} reaches {} across of {across}",
                line.says,
                line.across + wide
            );
        }
    }
}

#[test]
fn a_display_too_small_shows_no_screen_rather_than_a_broken_one() {
    let words = the_keyboard_question();
    assert!(what_a_screen_shows(320, 240, None, face(), &words).is_none());
}

#[test]
fn a_screen_with_nothing_on_it_has_no_lines() {
    let (across, down) = THE_DESIGN_WAS_DRAWN_AT;
    let nothing = TheWordsOfAScreen::default();
    let shows = what_a_screen_shows(across, down, None, face(), &nothing).expect("lays out");
    assert!(
        shows.lines.is_empty(),
        "a screen asked to say nothing drew {:?}",
        shows.lines
    );
}

/// **The end-to-end test, and it writes real bytes down the real road.**
///
/// `draw_a_screen` takes a `&File`, and `/dev/fb0` is a file — so a temporary
/// file stands in for the framebuffer and **nothing about the path under test
/// changes**: the same `fill`, the same coverage, the same
/// `write_all_at` at the same offsets. Then the bytes are read back and asked
/// whether a letter is actually there.
///
/// This is the difference between *the code ran* and *the screen has words on
/// it*, and it is as close to the second as a machine with no screen can get.
/// What it still cannot say is whether a real framebuffer accepts those bytes
/// in that order — that is owed to the certified machine's own boot.
#[cfg(unix)]
#[test]
fn a_screen_drawn_into_a_file_has_letters_in_it() {
    use crate::a_surface_to_draw_on::{Grey, TheSurface};
    use std::io::Read as _;

    let across = 1280_u32;
    let down = 800_u32;
    let stride = across * 4;
    let surface = TheSurface::from_what_the_kernel_says(
        &format!("{across},{down}\n"),
        &format!("{stride}\n"),
        "32\n",
    )
    .expect("a surface the shape a framebuffer is");

    let beneath = Grey::of(0x18);
    let ink = Grey::of(0xE8);
    let words = the_keyboard_question();
    let shows = what_a_screen_shows(across, down, None, face(), &words).expect("lays out");

    let at = std::env::temp_dir().join("alo-a-screen-drawn-into-a-file.fb");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&at)
        .expect("a file to stand in for the framebuffer");
    file.set_len(surface.altogether())
        .expect("a framebuffer-sized file");

    super::draw_a_screen(&file, surface, beneath, ink, face(), &shows)
        .expect("the screen is drawn");

    let mut wrote = Vec::new();
    std::fs::File::open(&at)
        .expect("the file back")
        .read_to_end(&mut wrote)
        .expect("its bytes");
    let _ = std::fs::remove_file(&at);

    assert_eq!(
        wrote.len() as u64,
        surface.altogether(),
        "the written file is not the size of the surface"
    );

    // Every byte is either the background or ink over it, and **at least one is
    // not the background** — which is the whole claim. A screen of pure
    // background is what every other test here would still pass against.
    let background = wrote.iter().filter(|&&byte| byte == 0x18).count();
    let inked = wrote.len() - background;
    assert!(
        inked > 2_000,
        "only {inked} bytes of {} differ from the background, so the screen is \
         effectively blank",
        wrote.len()
    );

    // And the ink is where the title is, not scattered: the row through the
    // middle of the question must have some, and a row above the whole block
    // must have none.
    let row_of = |down: u32| {
        let from = (down as usize) * (stride as usize);
        wrote[from..from + stride as usize].to_vec()
    };
    let through_the_title = shows.places.title.down + shows.places.title_type.tall / 2;
    assert!(
        row_of(through_the_title).iter().any(|&byte| byte != 0x18),
        "the row through the question has no ink in it"
    );
    assert!(
        row_of(2).iter().all(|&byte| byte == 0x18),
        "a row above the whole block has ink in it, so something is drawn \
         outside the block"
    );
}
