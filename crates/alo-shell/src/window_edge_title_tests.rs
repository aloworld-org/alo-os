//! A short title, a long one and a non-Latin one — the three the owner named.
//!
//! These shape with the real font system, so what they measure is what the
//! edge would draw rather than an arithmetic stand-in.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// The design's `Label/Medium`: Manrope SemiBold 13, line height 18.
///
/// All three are the design's now. Until 2026-10-09 the typeface was not —
/// this crate bundled only Inter — and these measurements were taken in a
/// face the design does not use.
fn label_medium() -> Metrics {
    Metrics::new(13.0, 18.0)
}

/// A font system with both bundled faces, as the shell builds one.
///
/// **Manrope first and Inter behind it.** Manrope is what alo's own surfaces
/// are drawn in; Inter stays loaded so a script Manrope does not cover still
/// renders. Dropping it would narrow the languages this machine can draw,
/// which is a worse outcome than two files in a folder.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
}

/// **The title is actually drawn in Manrope**, not merely asked for.
///
/// Shaping a string in the face alo asks for and in a face it does not must
/// give different widths. If the request silently fell back, these would be
/// equal and every measurement in this file would be Inter's wearing
/// Manrope's name.
#[test]
fn the_face_alo_asks_for_is_the_face_that_shapes() {
    use cosmic_text::{Attrs, Buffer, Family, Shaping};
    let mut fonts = fonts();
    let mut width = |attrs: &Attrs| {
        let mut buffer = Buffer::new(&mut fonts, label_medium());
        buffer.set_text(&mut fonts, "Launch strategy", attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut fonts, false);
        buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max)
    };
    let manrope = width(&Attrs::new().family(Family::Name("Manrope")));
    let inter = width(&Attrs::new().family(Family::Name("Inter")));
    assert!(manrope > 0.0 && inter > 0.0, "{manrope} {inter}");
    assert!(
        (manrope - inter).abs() > 0.5,
        "Manrope and Inter shaped the same string to the same width \
         ({manrope} vs {inter}) - the requested face is probably not loaded"
    );
}

/// **A combining accent is never cut from its letter.**
///
/// The owner: *do not split accented letters, emoji sequences or other
/// combined characters.* Here each `e` carries a combining acute, so every
/// character boundary inside a cluster is a place the old code could cut and
/// this one may not.
#[test]
fn a_combining_accent_is_never_cut_from_its_letter() {
    let mut fonts = fonts();
    // "é" as e + U+0301, twenty times over.
    let title: String = "e\u{301}".repeat(20);
    let fitted = fitted(&mut fonts, &title, 40, label_medium());
    assert!(fitted.cut, "{fitted:?}");
    let kept = fitted.shown.trim_end_matches(AN_ELLIPSIS);
    assert!(
        !kept.ends_with('\u{301}') || kept.chars().count().is_multiple_of(2),
        "a combining mark was left without its letter: {kept:?}"
    );
    // Every cluster kept is a whole one: an even number of code points.
    assert!(
        kept.chars().count().is_multiple_of(2),
        "cut inside a cluster: {kept:?}"
    );
}

/// **An emoji built from several code points is never split.**
///
/// A family sequence is five code points joined by zero-width joiners and is
/// one grapheme. Cutting inside it renders as several people rather than one
/// family, which is a thing the person never typed.
#[test]
fn an_emoji_sequence_is_never_split() {
    let mut fonts = fonts();
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let title = family.repeat(8);
    let fitted = fitted(&mut fonts, &title, 60, label_medium());
    let kept = fitted.shown.trim_end_matches(AN_ELLIPSIS);
    assert!(
        !kept.ends_with('\u{200D}'),
        "cut on a zero-width joiner, which splits the sequence: {kept:?}"
    );
    assert!(
        kept.len().is_multiple_of(family.len()),
        "cut inside an emoji sequence: {kept:?}"
    );
}

/// **A short title is drawn whole and nothing is cut.**
#[test]
fn a_short_title_is_left_alone() {
    let mut fonts = fonts();
    let fitted = fitted(&mut fonts, "Calculator", 400, label_medium());
    assert_eq!(fitted.shown, "Calculator");
    assert_eq!(fitted.whole, "Calculator");
    assert!(!fitted.cut);
    assert!(!fitted.shown.contains(AN_ELLIPSIS));
}

/// **A long title is cut with an ellipsis and still fits.**
///
/// The fitting is checked by measuring what came back, not by trusting the
/// function that produced it.
#[test]
fn a_long_title_is_cut_and_what_is_left_fits() {
    let mut fonts = fonts();
    let long = "Launch strategy for the first release, every open question and who owns it";
    let room = 200;
    let fitted = fitted(&mut fonts, long, room, label_medium());
    assert!(fitted.cut, "{fitted:?}");
    assert!(fitted.shown.ends_with(AN_ELLIPSIS), "{}", fitted.shown);
    assert!(
        crate::painted_text::how_wide(&mut fonts, &fitted.shown, label_medium()) <= room,
        "what was kept is still too wide: {:?}",
        fitted.shown
    );
    assert!(
        fitted.shown.chars().count() < long.chars().count(),
        "nothing was actually removed"
    );
}

/// **The whole title survives however little is drawn.**
///
/// The owner: *preserve the full title for accessibility*. A reader must be
/// able to name a window a person cannot read the name of.
#[test]
fn the_whole_title_is_kept_for_whoever_says_it() {
    let mut fonts = fonts();
    let long = "A window whose name is far longer than the room its edge has for it";
    let fitted = fitted(&mut fonts, long, 80, label_medium());
    assert!(fitted.cut);
    assert_eq!(fitted.whole, long, "the whole title must be untouched");
    assert!(fitted.shown.chars().count() < long.chars().count());
}

/// **A non-Latin title is fitted too**, and is not mangled into nothing.
///
/// Arabic, which is right-to-left, and Japanese, which has no spaces to break
/// at. Shaping is `Shaping::Advanced`'s answer; what this holds is that the
/// fitting does not throw the text away or return something wider than the
/// room.
#[test]
fn a_non_latin_title_is_fitted_rather_than_mangled() {
    let mut fonts = fonts();
    for title in [
        "نافذة المحطة الطرفية مع عنوان طويل جدا",
        "とても長いウィンドウのタイトルがここにあります",
        "Überwachung größerer Vorgänge",
    ] {
        let room = 120;
        let fitted = fitted(&mut fonts, title, room, label_medium());
        assert_eq!(fitted.whole, title, "the whole title must be untouched");
        assert!(
            crate::painted_text::how_wide(&mut fonts, &fitted.shown, label_medium()) <= room,
            "{title:?} fitted to {:?}, which is still too wide",
            fitted.shown
        );
        if fitted.cut {
            assert!(
                fitted.shown.ends_with(AN_ELLIPSIS),
                "{title:?} was cut without an ellipsis: {:?}",
                fitted.shown
            );
        }
    }
}

/// **No room means nothing is drawn, and the title is still said.**
///
/// A window narrow enough that its edge has no space left after the grip and
/// the controls. Drawing a lone ellipsis there would be an affordance that
/// says nothing; the reader still gets the name.
#[test]
fn no_room_draws_nothing_and_still_says_the_name() {
    let mut fonts = fonts();
    let fitted = fitted(&mut fonts, "Terminal", 0, label_medium());
    assert_eq!(fitted.shown, "");
    assert_eq!(fitted.whole, "Terminal");
    assert!(fitted.cut, "something was not shown, so something was cut");
}

/// **An empty title is not a cut title.** A window that named itself nothing
/// has nothing withheld from a reader.
#[test]
fn an_empty_title_is_not_a_truncation() {
    let mut fonts = fonts();
    let fitted = fitted(&mut fonts, "", 0, label_medium());
    assert_eq!(fitted.shown, "");
    assert!(!fitted.cut);
}

/// **The title never reaches the controls**, at any window width.
///
/// The owner: *ensure the title never overlaps the controls.* The room the
/// title is fitted to is what the drag region leaves after the grip, and the
/// drag region already stops one gap short of the first control - so this
/// holds the two files to each other rather than re-deriving either.
#[test]
fn a_fitted_title_never_reaches_the_controls() {
    use crate::window_edge::{Decorations, THE_GRIP_IS, edge_of};
    use smithay::utils::{Point, Rectangle, Size};

    let mut fonts = fonts();
    let long = "Launch strategy for the first release, every open question and who owns it";
    for wide in [320, 600, 936, 1920] {
        let window = Rectangle::new(Point::from((0, 44)), Size::from((wide, 400)));
        let edge = edge_of(window, Decorations::TheShellDraws, true);
        let first = edge.controls.first().expect("three controls");

        // What the drag region leaves once the grip has taken its place.
        let room = edge.drag.size.w - THE_GRIP_IS.0;
        let fitted = fitted(&mut fonts, long, room, label_medium());
        let drawn = crate::painted_text::how_wide(&mut fonts, &fitted.shown, label_medium());

        let title_starts = edge.title.expect("a shell-drawn edge has a title").x;
        assert!(
            title_starts + drawn <= first.target.loc.x,
            "at {wide} the title ends at {} and the first control starts at {}",
            title_starts + drawn,
            first.target.loc.x
        );
    }
}
