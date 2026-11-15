//! **The pointer, walked**: what a person would notice about alo's pointer, on
//! the desktop, over three applications alo did not make, and as Settings
//! changes it.
//!
//! The owner's hard rule for the cursor is that it must work in applications
//! alo did not make (`docs/features.md`, *The alo Cursor*: *one cursor, drawn by
//! alo, in every application*). So the walk opens them — GNOME Text Editor,
//! VSCodium and Firefox, all three installed by every fresh machine
//! (`crates/alo-software/shipped.toml`) — and looks at the pointer over each.
//!
//! # What it checks, in order
//!
//! 1. **The screen capture shows the pointer at all.** alo draws its pointer
//!    into the frame itself (`alo_shell::scene_drawing`), not on a hardware
//!    cursor plane, so qemu's `screendump` should have it — and this is
//!    measured on every run rather than assumed, because every other check
//!    here passes or fails on nothing if it is not.
//! 2. **On the desktop it is the kite**, navy, at the size every pointer is —
//!    not a default arrow and not the familiar one.
//! 3. **Over Text Editor and over VSCodium it is still alo's**, in whatever
//!    shape the application asked for.
//! 4. **The person makes it large and white in Settings**, and over Settings
//!    it is the large white kite at once; **the familiar arrow** chosen there
//!    switches the shape, and the kite chosen again switches it back.
//! 5. **Firefox, opened after the change**, shows the large white kite over a
//!    page, alo's **text cursor** over a text field and alo's **hand** over a
//!    link. Firefox is opened after the change on purpose: an application
//!    that draws its own pointer reads which one when it starts
//!    (`alo_desktop::telling_applications_about_the_pointer`), so this is the
//!    road that proves the choice reached an application alo did not make.
//!
//! # The page Firefox is shown
//!
//! [`the_page`], typed into the address bar: a magenta text field, a cyan
//! band, and a lime link, filling the window. The colours are how the walk
//! finds each part on the screen (`crate::a_place_on_the_screen`); nothing in
//! it is a picture of a pointer, so nothing on it can be mistaken for one.
//!
//! # What it does not see
//!
//! - **An application already running when the look changed** is not checked
//!   for the new look: by `alo_desktop`'s own account it keeps the pointer it
//!   started with unless it asks alo by name, and that is a different promise.
//! - **Black**, and the largest size, are not chosen here: the matching is the
//!   same for every look (`the_pointer_seen_tests.rs` holds all eighteen), and
//!   each choice walked is minutes of a runner.
//! - **The flow between shapes** (*arrow into text cursor into hand, short and
//!   stable*) is a starred promise not yet built; a screenshot a second could
//!   not see it anyway.

use std::time::Duration;

use alo_appearance::{PointerColour, PointerShape, PointerSize};
use alo_cursors::CursorIcon;

use crate::a_place_on_the_screen::Target;
use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Pointing, Walk};
use crate::the_pointer_seen::Look;

/// The text field's colour on [`the_page`].
pub const MAGENTA: [u8; 3] = [255, 0, 255];

/// The plain band's colour on [`the_page`].
pub const CYAN: [u8; 3] = [0, 255, 255];

/// The link's colour on [`the_page`].
pub const LIME: [u8; 3] = [0, 255, 0];

/// The page Firefox is shown: a text field, a plain band and a link, each a
/// colour of its own, written from [`MAGENTA`], [`CYAN`] and [`LIME`] so the
/// page and the walk that looks for it cannot disagree. A `data:` address, so
/// nothing is fetched and nothing is written to the machine; and no `#` or
/// `%`, which an address reads as something else.
#[must_use]
pub fn the_page() -> String {
    let rgb = |[red, green, blue]: [u8; 3]| format!("rgb({red},{green},{blue})");
    let (plain, field, link) = (rgb(CYAN), rgb(MAGENTA), rgb(LIME));
    [
        "data:text/html,".to_owned(),
        format!("<body style=margin:0;background:{plain}>"),
        format!(
            "<textarea style=display:block;width:100vw;height:35vh;border:0;margin:0;resize:none;background:{field}></textarea>"
        ),
        "<div style=height:10vh></div>".to_owned(),
        format!("<a href=x style=display:block;height:35vh;background:{link}></a></body>"),
    ]
    .concat()
}

/// The colour of *Continue* on Firefox's first-start dialog, read off the
/// picture run 43658977070 kept (`38-the-pointer-page.png`) at four places on
/// the button, each the same. Firefox's own, so a Firefox that changes it is a
/// walk that says *not on the screen* rather than one that clicks elsewhere.
pub const FIREFOXS_CONTINUE: [u8; 3] = [118, 78, 221];

/// The look the person chooses in Settings in this walk.
pub const LARGE_AND_WHITE: Look = Look {
    shape: PointerShape::Kite,
    colour: PointerColour::White,
    size: PointerSize::Large,
};

/// The same with the familiar arrow.
pub const LARGE_WHITE_AND_FAMILIAR: Look = Look {
    shape: PointerShape::Classic,
    ..LARGE_AND_WHITE
};

/// Rows on Settings' Accessibility page, counted from the first, in the order
/// `alo_shell::settings_window` lists them: the screen reader, the text size,
/// then every size, every arrow and every colour **in the order of
/// `alo_appearance`'s own `ALL` lists**, which are what that page iterates.
/// So the walk counts from the same lists the page is built from, rather than
/// from a number somebody counted once.
mod rows {
    use alo_appearance::{PointerColour, PointerShape, PointerSize};

    /// The rows before the pointer's: the screen reader and the text size.
    const BEFORE_THE_POINTER: usize = 2;

    /// The row of one size.
    pub fn size(size: PointerSize) -> usize {
        BEFORE_THE_POINTER + at(&PointerSize::ALL, size)
    }

    /// The row of one arrow.
    pub fn shape(shape: PointerShape) -> usize {
        BEFORE_THE_POINTER + PointerSize::ALL.len() + at(&PointerShape::ALL, shape)
    }

    /// The row of one colour.
    pub fn colour(colour: PointerColour) -> usize {
        BEFORE_THE_POINTER
            + PointerSize::ALL.len()
            + PointerShape::ALL.len()
            + at(&PointerColour::ALL, colour)
    }

    /// Where `one` is in `all`.
    fn at<T: PartialEq>(all: &[T], one: T) -> usize {
        all.iter().position(|it| *it == one).unwrap_or(0)
    }
}

/// How long an application is given to open, from the Bar's Enter: a first
/// start, under KVM.
const AN_APPLICATION_OPENS: Duration = Duration::from_secs(45);

/// Then the pointer: on the desktop, over Text Editor and VSCodium, through
/// Settings, and over Firefox. It starts where the first walk leaves the
/// machine — alo's terminal open and maximised (`the_first_walk`'s
/// terminal section) — so it closes the terminal first, with Alt+F4, to reach
/// the desktop.
#[must_use]
pub fn then_the_pointer(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let walk = walk
        .press(Chord::with("alt", "f4"))
        .look("the-desktop-for-the-pointer", look)
        .pointer(Pointing::SeenAtAll {
            check: "the screen capture shows the pointer".to_owned(),
        })
        .pointer(point_at(
            "the pointer goes to the middle of the desktop",
            Target::Middle,
        ))
        .pointer(is_drawn_as(
            "on the desktop the pointer is alo's navy kite",
            Some(CursorIcon::Default),
        ));
    let walk = over_an_application(walk, "Text Editor", "text-editor");
    let walk = over_an_application(walk, "VSCodium", "vscodium");
    let walk = through_settings(walk);
    over_firefox(walk)
}

/// Open an application from the Bar by its name, point at its window, and
/// check the pointer there is alo's; then close it.
fn over_an_application(walk: Walk, name: &str, picture: &str) -> Walk {
    let look = Settling::UNDER_KVM;
    walk.pointer(Pointing::Remember)
        .press(Chord::with("meta_l", "spc"))
        .type_text(name)
        .press(Chord::key("ret"))
        .wait(AN_APPLICATION_OPENS)
        .look(picture, look.at_most(Duration::from_secs(120)))
        .pointer(point_at(
            &format!("the pointer goes onto {name}"),
            Target::WhatChanged,
        ))
        .pointer(is_drawn_as(
            &format!("over {name} the pointer is still alo's"),
            None,
        ))
        .press(Chord::with("alt", "f4"))
        .look(&format!("{picture}-closed"), look)
}

/// Make the pointer large and white in Settings, and see it there; then the
/// familiar arrow, and the kite again.
fn through_settings(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let down = Chord::key("down");
    let up = Chord::key("up");
    let space = Chord::key("spc");
    let large = rows::size(PointerSize::Large);
    let white = rows::colour(PointerColour::White);
    let classic = rows::shape(PointerShape::Classic);
    let kite = rows::shape(PointerShape::Kite);
    walk.pointer(Pointing::Remember)
        .press(Chord::with("meta_l", "i"))
        .look("settings-for-the-pointer", look)
        .press(Chord::key("end"))
        .press(Chord::key("tab"))
        .press_times(&down, large)
        .press(space.clone())
        .look("the-pointer-made-large", look)
        .press_times(&down, white - large)
        .press(space.clone())
        .look("the-pointer-made-white", look)
        .pointer(Pointing::ThePersonChose(LARGE_AND_WHITE))
        .pointer(point_at(
            "the pointer goes onto Settings",
            Target::WhatChanged,
        ))
        .pointer(is_drawn_as(
            "over Settings the pointer is the large white kite",
            Some(CursorIcon::Default),
        ))
        .press_times(&up, white - classic)
        .press(space.clone())
        .look("the-familiar-arrow-chosen", look)
        .pointer(Pointing::ThePersonChose(LARGE_WHITE_AND_FAMILIAR))
        .pointer(is_drawn_as(
            "the familiar arrow, once chosen, is the pointer",
            Some(CursorIcon::Default),
        ))
        .press_times(&up, classic - kite)
        .press(space)
        .look("the-kite-chosen-again", look)
        .pointer(Pointing::ThePersonChose(LARGE_AND_WHITE))
        .pointer(is_drawn_as(
            "the kite, chosen again, is the pointer",
            Some(CursorIcon::Default),
        ))
        .press(Chord::key("esc"))
        .look("settings-closed-on-a-large-white-pointer", look)
}

/// Open Firefox after the change, show it [`the_page`], and check the pointer
/// over its plain band, its text field and its link.
fn over_firefox(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    walk.press(Chord::with("meta_l", "spc"))
        .type_text("Firefox")
        .press(Chord::key("ret"))
        .wait(AN_APPLICATION_OPENS)
        .look("firefox", look.at_most(Duration::from_secs(120)))
        // **Firefox's first start asks the person to continue**, in a dialog
        // over everything (run 43654461272, `38-the-pointer-page.png`), and
        // the keys sent to it on run 43658977070 — Escape, Enter and the whole
        // address — changed nothing. So the walk does what a person does:
        // points at *Continue* and clicks it.
        .pointer(point_at(
            "the pointer goes onto Firefox's Continue",
            Target::Colour(FIREFOXS_CONTINUE),
        ))
        .pointer(Pointing::Click)
        .look("firefox-continued", look.at_most(Duration::from_secs(60)))
        .press(Chord::with("ctrl", "l"))
        .type_text(&the_page())
        .press(Chord::key("ret"))
        .look("the-pointer-page", look.at_most(Duration::from_secs(60)))
        .pointer(point_at(
            "the pointer goes onto Firefox's page",
            Target::Colour(CYAN),
        ))
        .pointer(is_drawn_as(
            "over Firefox's page the pointer is the large white kite",
            Some(CursorIcon::Default),
        ))
        .pointer(point_at(
            "the pointer goes onto a text field in Firefox",
            Target::Colour(MAGENTA),
        ))
        .pointer(is_drawn_as(
            "over a text field in Firefox the pointer is alo's text cursor",
            Some(CursorIcon::Text),
        ))
        .pointer(point_at(
            "the pointer goes onto a link in Firefox",
            Target::Colour(LIME),
        ))
        .pointer(is_drawn_as(
            "over a link in Firefox the pointer is alo's hand",
            Some(CursorIcon::Pointer),
        ))
        .press(Chord::with("alt", "f4"))
        .look("firefox-closed", look)
}

/// A pointing step.
fn point_at(check: &str, at: Target) -> Pointing {
    Pointing::PointAt {
        check: check.to_owned(),
        at,
    }
}

/// A shape check.
fn is_drawn_as(check: &str, icon: Option<CursorIcon>) -> Pointing {
    Pointing::IsDrawnAs {
        check: check.to_owned(),
        icon,
    }
}

#[cfg(test)]
#[path = "the_pointer_walk_tests.rs"]
mod tests;
