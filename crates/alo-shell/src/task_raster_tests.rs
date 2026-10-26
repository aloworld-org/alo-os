//! That a task window's furniture rasterises, that its shapes are where the
//! layout puts them, and that a window too small is refused rather than drawn
//! wrong.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported"
)]

use alo_access::TurnedOn;
use alo_appearance::{Accent, Appearance, TimeOfDay};
use alo_strings::Direction;
use smithay::utils::Rectangle;

use super::{NoTaskPicture, TaskPicture, furniture};
use crate::desktop_look::DesktopLook;

/// The ordinary look.
fn a_look() -> DesktopLook {
    let mut appearance = Appearance::shipped();
    appearance.set_accent(Accent::Indigo);
    DesktopLook::of(
        &appearance,
        &TurnedOn::nothing(),
        TimeOfDay::checked(10, 0).expect("ten in the morning"),
        Direction::LeftToRight,
        &crate::desktop_testing::a_display(),
    )
}

/// The two shared files a fixture names.
fn two_files() -> Vec<String> {
    vec!["Launch brief.pdf".to_owned(), "Product notes.md".to_owned()]
}

/// Everything a task window says, as the design's own example words.
///
/// **Example content, and the component description says so twice.** None of
/// it is simulated state: these are the words a caller would have said, handed
/// in exactly as `the_docks_words` hands the Dock's.
fn the_words(files: &[String]) -> crate::task_text::TheWordsInATaskWindow<'_> {
    crate::task_text::TheWordsInATaskWindow {
        title: "Prepare the launch presentation",
        kind: "alo task \u{b7} Work",
        status: "alo is working",
        model: "SmolLM2 1.7B",
        destination: "On this computer",
        invitation: "Reply to alo or change the direction\u{2026}",
        scope_note: "Permissions and model choices apply to this task.",
        files,
        how_many_files: "2 files shared with this task",
    }
}

/// What was said, as the design's frame draws it.
fn what_was_said() -> Vec<crate::task_text::WhatWasSaid<'static>> {
    vec![
        crate::task_text::WhatWasSaid {
            whose: alo_tasks::WhoseBlock::ThePersons,
            words: "Make a six-slide launch presentation from these two files.",
            // Nothing fills these yet: a proposal is drawn as words
            // and its two answers are not under it.
            answers: None,
        },
        crate::task_text::WhatWasSaid {
            whose: alo_tasks::WhoseBlock::Alos,
            words: "I\u{2019}ll read the two files, shape the story, and build a draft.",
            // Nothing fills these yet: a proposal is drawn as words
            // and its two answers are not under it.
            answers: None,
        },
        crate::task_text::WhatWasSaid {
            whose: alo_tasks::WhoseBlock::AnAccount,
            words: "Reading your documents",
            // Nothing fills these yet: a proposal is drawn as words
            // and its two answers are not under it.
            answers: None,
        },
    ]
}

/// A window at the size the design was drawn at, with two files shared.
fn as_drawn() -> TaskPicture {
    let files = two_files();
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    furniture(
        Rectangle::new((120, 80).into(), (1040, 800).into()),
        a_look(),
        2,
        &the_words(&files),
        &said,
        &mut fonts,
    )
    .expect("the size the design was drawn at")
}

/// The four bytes at this point of the panel's raster.
fn pixel(of: &TaskPicture, x: i32, y: i32) -> [u8; 4] {
    let washed = &of.panel;
    let width = washed.area.size.w;
    let at = ((y as usize) * (width as usize) + (x as usize)) * 4;
    washed
        .pixels
        .get(at..at + 4)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .expect("four bytes at a point inside the raster")
}

/// Where a point in the window's own coordinates lands in the raster, which is
/// offset by the room the shadow needs.
fn in_the_raster(of: &TaskPicture, x: i32, y: i32) -> (i32, i32) {
    (
        of.at.loc.x + x - of.panel.area.loc.x,
        of.at.loc.y + y - of.panel.area.loc.y,
    )
}

/// **A window at the design's size rasterises, and its raster is larger than
/// the window** — by the room the shadow needs on every side.
#[test]
fn a_window_rasterises_with_room_for_its_shadow() {
    let drawn = as_drawn();

    assert!(
        drawn.panel.area.size.w > drawn.at.size.w,
        "the raster is no wider than the window, so there is no room for a shadow"
    );
    assert!(drawn.panel.area.size.h > drawn.at.size.h);
    assert_eq!(
        drawn.panel.pixels.len(),
        (drawn.panel.area.size.w * drawn.panel.area.size.h * 4) as usize,
        "the raster's bytes do not match its area"
    );
}

/// **The middle of an empty window is its own surface**, opaque and the
/// palette's.
///
/// **Given no conversation on purpose.** This sampled the middle of a window
/// that had three things said in it and passed until bubbles were drawn as real
/// shapes — at which point the middle was a bubble, correctly. The fixture is
/// now a window with nothing said, so the point really is empty surface.
#[test]
fn the_middle_of_an_empty_window_is_its_surface() {
    let files = two_files();
    let mut fonts = cosmic_text::FontSystem::new();
    let drawn = furniture(
        Rectangle::new((120, 80).into(), (1040, 800).into()),
        a_look(),
        0,
        &the_words(&files),
        &[],
        &mut fonts,
    )
    .expect("the design's size");

    let (x, y) = in_the_raster(&drawn, drawn.at.size.w / 2, 300);
    let [r, g, b, a] = pixel(&drawn, x, y);

    assert_eq!(a, 255, "the middle of a window is opaque");
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    assert_eq!(
        [r, g, b],
        palette.surface,
        "the middle is not the surface colour"
    );
}

/// **The reply field is outlined, not filled.**
///
/// The design gives it `bg/surface` with a `border/default` hairline, and the
/// panel behind it is already that colour — so what makes it a field is its
/// edge.
///
/// **This test asserted the opposite until 2026-10-13**, when a picture of a
/// real draw showed what a cool fill costs: the field read as a grey slab and
/// the three composer controls, which are `bg/cool` themselves, vanished into
/// it. The test had encoded the bug, which is why changing the code broke it —
/// a test can be wrong, and the way to tell is to look at the thing.
#[test]
fn the_reply_field_is_outlined_rather_than_filled() {
    let drawn = as_drawn();
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");

    // Well inside the field, which must be the window's own surface.
    let inside_it = rows.composer.from_the_top
        + alo_tasks::measures::ABOVE_THE_FIELD
        + alo_tasks::measures::THE_FIELD_IS_TALL / 2;
    let (x, y) = in_the_raster(
        &drawn,
        drawn.at.size.w / 2,
        i32::try_from(inside_it).expect("a small number"),
    );
    let [r, g, b, _] = pixel(&drawn, x, y);

    assert_eq!(
        [r, g, b],
        palette.surface,
        "the reply field is filled rather than outlined"
    );
    assert_ne!(
        [r, g, b],
        palette.cool,
        "the field is cool, which is what hid the controls that sit on it"
    );
}

/// **The one button is drawn in the ink colour**, which is the inversion that
/// marks it as the thing a person presses to act.
#[test]
fn the_one_button_is_drawn_in_the_ink_colour() {
    let drawn = as_drawn();
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");
    let controls = rows.controls();

    let middle = alo_tasks::measures::A_CONTROL_IS / 2;
    let (x, y) = in_the_raster(
        &drawn,
        i32::try_from(controls.the_one_button + middle).expect("a small number"),
        i32::try_from(controls.from_the_top + middle).expect("a small number"),
    );
    let [r, g, b, _] = pixel(&drawn, x, y);

    assert_eq!([r, g, b], palette.the_one_button);
    assert_ne!(
        [r, g, b],
        palette.cool,
        "the one button is the same colour as the controls beside it"
    );
}

/// **A chip is drawn for each shared file, and none when nothing is shared.**
///
/// Asserted by sampling the same point with two files and with none, which
/// catches a chip drawn unconditionally — a row of empty grounds on a task
/// nobody attached anything to.
#[test]
fn a_chip_is_drawn_per_shared_file_and_none_without() {
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");
    let down = rows.shared_files.from_the_top + alo_tasks::measures::THE_SHARED_FILES_ARE_TALL / 2;
    let across = alo_tasks::measures::BESIDE_A_ROW + alo_tasks::measures::A_CHIP_IS_WIDE / 2;

    let sample = |how_many: usize| {
        let files = two_files();
        let said = what_was_said();
        let mut fonts = cosmic_text::FontSystem::new();
        let drawn = furniture(
            Rectangle::new((120, 80).into(), (1040, 800).into()),
            a_look(),
            how_many,
            &the_words(&files),
            &said,
            &mut fonts,
        )
        .expect("the design's size");
        let (x, y) = in_the_raster(
            &drawn,
            i32::try_from(across).expect("a small number"),
            i32::try_from(down).expect("a small number"),
        );
        let [r, g, b, _] = pixel(&drawn, x, y);
        [r, g, b]
    };

    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    assert_eq!(sample(2), palette.cool, "no chip was drawn for two files");
    assert_eq!(
        sample(0),
        palette.surface,
        "a chip was drawn on a task with nothing shared"
    );
}

/// **A window too small is refused, and the refusal says what it would need.**
#[test]
fn a_window_too_small_is_refused_rather_than_drawn_wrong() {
    let files = two_files();
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    let refused = furniture(
        Rectangle::new((0, 0).into(), (200, 100).into()),
        a_look(),
        0,
        &the_words(&files),
        &said,
        &mut fonts,
    )
    .expect_err("a 200 by 100 window was rasterised");

    assert!(
        matches!(refused, NoTaskPicture::TooSmall(_)),
        "{refused:?} is not a refusal about size"
    );
}

/// **More chips than fit are not drawn past the row's edge.**
///
/// Twenty files on a window that holds five: the row stops rather than running
/// chips off the side.
#[test]
fn chips_stop_at_the_rows_edge_rather_than_running_off_it() {
    let files = two_files();
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    let many = furniture(
        Rectangle::new((0, 0).into(), (1040, 800).into()),
        a_look(),
        20,
        &the_words(&files),
        &said,
        &mut fonts,
    )
    .expect("the design's size");

    // The last pixel column inside the window must still be the surface: no
    // chip reached it.
    let (x, y) = in_the_raster(
        &many,
        many.at.size.w - 2,
        i32::try_from(
            alo_tasks::layout::TheRows::of(1040, 800)
                .expect("the design's size")
                .shared_files
                .from_the_top
                + alo_tasks::measures::THE_SHARED_FILES_ARE_TALL / 2,
        )
        .expect("a small number"),
    );
    let [r, g, b, _] = pixel(&many, x, y);
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");

    assert_eq!(
        [r, g, b],
        palette.surface,
        "a chip was drawn against the window's own edge"
    );
}

/// **The words are drawn, and a window told nothing draws none of them.**
///
/// The test the rest of this file could not have caught: every assertion above
/// samples the panel's raster, and text is inked separately — so `inked` could
/// have been empty and all of them would still pass. A control run with every
/// string blank is what makes the positive count mean something.
#[test]
fn the_words_are_drawn_and_an_empty_window_draws_none() {
    let files = two_files();
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    let at = Rectangle::new((120, 80).into(), (1040, 800).into());

    let nothing = crate::task_text::TheWordsInATaskWindow {
        title: "",
        kind: "",
        status: "",
        model: "",
        destination: "",
        invitation: "",
        scope_note: "",
        files: &[],
        how_many_files: "",
    };
    let silent = furniture(at, a_look(), 0, &nothing, &[], &mut fonts).expect("the design's size");
    assert!(
        silent.inked.is_empty(),
        "a window with nothing to say drew {} pieces of text",
        silent.inked.len()
    );

    let spoken = furniture(at, a_look(), 2, &the_words(&files), &said, &mut fonts)
        .expect("the design's size");

    // **Derived, not a number.** Seven fixed strings — the title, the kind, the
    // status, the invitation, the model, the destination and the scope note —
    // plus a label per chip and one per thing said. Written as the sum because
    // the first version of this test said *eleven* from a miscount of the
    // seven, and a magic total cannot say which part of it moved.
    let fixed = 7;
    assert_eq!(
        spoken.inked.len(),
        fixed + files.len() + said.len(),
        "the window drew {} pieces and {fixed} + {} + {} were given to it",
        spoken.inked.len(),
        files.len(),
        said.len()
    );
    assert!(
        spoken
            .inked
            .iter()
            .all(|one| one.area.size.w > 0 && one.area.size.h > 0),
        "a piece of text was drawn with no area"
    );
}

/// **Every word is drawn inside the window.**
///
/// A long title must not write past the frame, and a block scrolled out of
/// view must not be drawn above the conversation's own top.
#[test]
fn no_word_is_drawn_outside_the_window() {
    let files = two_files();
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    let at = Rectangle::new((120, 80).into(), (1040, 800).into());

    let long = crate::task_text::TheWordsInATaskWindow {
        title: "Prepare the launch presentation and then also summarise every \
                single one of the attached documents in considerable detail",
        ..the_words(&files)
    };
    let drawn = furniture(at, a_look(), 2, &long, &said, &mut fonts).expect("the design's size");

    for one in &drawn.inked {
        assert!(
            one.area.loc.y >= at.loc.y,
            "text at y {} is above the window's top at {}",
            one.area.loc.y,
            at.loc.y
        );
        assert!(
            one.area.loc.y + one.area.size.h <= at.loc.y + at.size.h,
            "text reaching y {} is below the window's bottom at {}",
            one.area.loc.y + one.area.size.h,
            at.loc.y + at.size.h
        );
        assert!(
            one.area.loc.x >= at.loc.x,
            "text at x {} is left of the window",
            one.area.loc.x
        );
    }
}

/// **A person's request is drawn further right than alo's reply.**
///
/// The mark that tells whose words they are without relying on colour, checked
/// where it finally becomes pixels rather than only in the layout's own tests.
///
/// **Only the conversation is given to the window**, so the inked pieces *are*
/// the blocks. The first version of this test assumed they were the last three
/// of eleven and compared alo's reply against the activity block, because the
/// scope note is written after the conversation — an assertion about positions
/// in a list nobody promised an order for.
#[test]
fn a_persons_words_are_drawn_further_right_than_alos() {
    let said = what_was_said();
    let mut fonts = cosmic_text::FontSystem::new();
    let silent = crate::task_text::TheWordsInATaskWindow {
        title: "",
        kind: "",
        status: "",
        model: "",
        destination: "",
        invitation: "",
        scope_note: "",
        files: &[],
        how_many_files: "",
    };
    let drawn = furniture(
        Rectangle::new((0, 0).into(), (1040, 800).into()),
        a_look(),
        0,
        &silent,
        &said,
        &mut fonts,
    )
    .expect("the design's size");

    assert_eq!(
        drawn.inked.len(),
        said.len(),
        "with only the conversation given, the inked pieces must be the blocks"
    );
    let request = drawn.inked.first().expect("a request").area.loc.x;
    let reply = drawn.inked.get(1).expect("a reply").area.loc.x;

    assert!(
        request > reply,
        "a person's request was drawn at x {request} and alo's reply at x \
         {reply}, so the side no longer says whose words they are"
    );
}
