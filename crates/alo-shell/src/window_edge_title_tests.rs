//! A short title, a long one and a non-Latin one — the three the owner named.
//!
//! These shape with the real font system, so what they measure is what the
//! edge would draw rather than an arithmetic stand-in.

use super::*;

/// The design's `Label/Medium`: Manrope SemiBold 13, line height 18.
///
/// **The size and line height are the design's; the typeface is not yet.**
/// This crate bundles `fonts/Inter.ttf` and every caller asks for
/// `Family::SansSerif`, so nothing here draws Manrope. Recorded in
/// `docs/design/the-external-window-edge.md` as owed rather than papered over
/// — a measurement taken in one typeface and labelled another is worse than
/// one labelled honestly.
fn label_medium() -> Metrics {
    Metrics::new(13.0, 18.0)
}

/// A font system with the bundled face, as the shell builds one.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
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
