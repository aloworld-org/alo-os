//! **Every slot on the Dock that has a window open carries the open mark**, and
//! a slot with nothing open carries none — whatever kind of slot it is: an
//! application drawn with its own icon, one drawn with its first letter, and
//! alo's own two entries, Settings and Applications.
//!
//! The rule is the design's: `348:23886` *Open window indicator*, a dot
//! centred across the slot and flush to its bottom
//! (`docs/design/the-docks-visual-specification.md`, *The running
//! indicators*). It is tested over every kind of slot at once because the one
//! test of it before this file drew first letters only, and every application
//! with an icon lost its mark without a test noticing.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the case it was on"
)]

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt as _;

use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::places::{APlace, WhatIsHere};
use alo_dock::{ADockEntry, ANativeAction, AppId, Dock, Edge, OnTheDock};
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use crate::Server;
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;

/// The kinds of slot a window can be open behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AKindOfSlot {
    /// An application that drew its own icon.
    WithItsOwnIcon,
    /// An application drawn with its first letter.
    WithItsFirstLetter,
    /// Settings or Applications: alo's own entries.
    AlosOwn,
}

/// Which kind of slot `place` is, or [`None`] for a control that stands for
/// no window at all.
///
/// **Every variant named, with no catch-all**, so a new kind of slot is a
/// compile error here rather than a slot this rule quietly skips.
fn the_kind_of(place: &APlace, artwork: &TheArtwork) -> Option<AKindOfSlot> {
    match place.what() {
        WhatIsHere::Application(app) if artwork.of(app).is_some() => {
            Some(AKindOfSlot::WithItsOwnIcon)
        }
        WhatIsHere::Application(_) => Some(AKindOfSlot::WithItsFirstLetter),
        WhatIsHere::ANativeAction(_) => Some(AKindOfSlot::AlosOwn),
        WhatIsHere::TheOverflow
        | WhatIsHere::TheDivider
        | WhatIsHere::TheComposer
        | WhatIsHere::ShowAll
        | WhatIsHere::History => None,
    }
}

/// The Dock's row for `place`: the same application, or the same action.
fn its_row<'a>(place: &APlace, rows: &'a [OnTheDock]) -> &'a OnTheDock {
    rows.iter()
        .find(|row| match place.what() {
            WhatIsHere::Application(app) => row.app() == Some(app),
            WhatIsHere::ANativeAction(action) => row.native() == Some(*action),
            _ => false,
        })
        .unwrap_or_else(|| panic!("{:?} has a row on the Dock", place.what()))
}

/// `place`'s slot on the bar, in output pixels.
fn its_slot(picture: &DockPicture, place: &APlace) -> Rectangle<i32, Physical> {
    let dock = &picture.standing;
    let icon = dock.scale.px(alo_dock::measures::ICON);
    let along = dock.scale.px(place.from_the_start());
    let (left, top) = crate::dock_raster::a_slot_at(dock.band, dock.layout.edge(), along, icon);
    Rectangle::new(
        (left, top).into(),
        (dock.scale.px(place.across()), icon).into(),
    )
}

/// The open marks drawn in `slot`: the design's dot, centred across the slot
/// and ending flush with its bottom edge.
fn marks_in(picture: &DockPicture, slot: Rectangle<i32, Physical>) -> usize {
    let scale = picture.standing.scale;
    let wide = scale.px(alo_dock::measures::AN_OPEN_MARK_IS_WIDE);
    let tall = scale.px(alo_dock::measures::A_RUNNING_MARK_IS_TALL);
    picture
        .solids
        .iter()
        .filter(|solid| {
            let area = solid.area;
            area.size.w == wide
                && area.size.h == tall
                && area.loc.x == slot.loc.x + (slot.size.w - wide) / 2
                && area.loc.y + tall == slot.loc.y + slot.size.h
        })
        .count()
}

/// **The rule itself**: each slot with a window open has exactly one mark, and
/// each slot with none has no mark. Answers which kinds of slot were open, so
/// a caller can show that every kind was asked.
fn every_open_slot_is_marked(
    picture: &DockPicture,
    rows: &[OnTheDock],
    artwork: &TheArtwork,
    case: &str,
) -> BTreeSet<AKindOfSlot> {
    let mut open = BTreeSet::new();
    for place in picture.standing.places.each() {
        let Some(kind) = the_kind_of(place, artwork) else {
            continue;
        };
        let is_open = its_row(place, rows).is_open();
        let marks = marks_in(picture, its_slot(picture, place));
        assert_eq!(
            marks,
            usize::from(is_open),
            "{case}: {:?} ({kind:?}) is {} and has {marks} open marks",
            place.what(),
            if is_open { "open" } else { "not open" },
        );
        if is_open {
            open.insert(kind);
        }
    }
    open
}

/// An application the Dock takes.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a plain identifier is an application")
}

/// A picture of one colour, the size the badges are drawn at.
fn a_picture() -> APicture {
    let side = alo_dock::measures::GLYPH;
    let pixels = [0x2a, 0x6f, 0x97, 0xff].repeat(side as usize * side as usize);
    APicture::of(side, side, pixels).expect("a whole picture")
}

/// One window of `app`'s, open on the canvas.
fn a_window_of(windows: &mut alo_dock::Windows, number: u64, app: AppId) {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("a patch");
    windows.opened(alo_dock::Window::of(
        alo_dock::WindowId::numbered(number),
        Some(app),
        "",
        patch,
        alo_dock::HowItSits::OnTheCanvas,
    ));
}

/// The bar, drawn as the shell draws it.
fn drawn(edge: Edge, scale: DisplayScale, rows: &[OnTheDock], artwork: &TheArtwork) -> DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let per_cent = i32::from(scale.per_cent());
    crate::dock_raster::picture(
        &dock,
        noon_look(&an_appearance(), Direction::LeftToRight),
        (1440 * per_cent / 100, 960 * per_cent / 100),
        rows,
        rows,
        (
            &crate::what_the_dock_has_open::NOTHING,
            crate::desktop_testing::no_words(),
        ),
        None,
        artwork,
        &mut crate::WindowControlLabels::new()
            .expect("the bundled faces")
            .fonts,
        scale,
    )
    .unwrap_or_else(|why| panic!("{edge:?} at {scale:?} would not draw: {why:?}"))
}

/// **An application with a window open is marked, whether it drew its own icon
/// or its first letter; one with none is not.** On every edge, at one to one
/// and at twice the pixels.
#[test]
fn every_application_with_a_window_open_is_marked_whatever_it_is_drawn_with() {
    let mut holding = alo_dock::Holding::nothing();
    for name in ["icon-open", "letter-open", "icon-shut", "letter-shut"] {
        holding.pin(app(name));
    }
    let mut windows = alo_dock::Windows::none();
    a_window_of(&mut windows, 1, app("icon-open"));
    a_window_of(&mut windows, 2, app("letter-open"));
    let rows = holding.showing(&windows);

    let mut artwork = TheArtwork::none();
    artwork.draws(app("icon-open"), a_picture());
    artwork.draws(app("icon-shut"), a_picture());

    for edge in Edge::EVERY {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let picture = drawn(edge, scale, &rows, &artwork);
            let open = every_open_slot_is_marked(
                &picture,
                &rows,
                &artwork,
                &format!("{edge:?} at {scale:?}"),
            );
            assert_eq!(
                open,
                BTreeSet::from([AKindOfSlot::WithItsOwnIcon, AKindOfSlot::WithItsFirstLetter]),
                "the premise: both kinds of application slot were open on {edge:?} at {scale:?}"
            );
        }
    }
}

/// A session whose Dock holds Settings and Applications, and whose Settings
/// has somewhere to read from, so pressing it opens it.
fn a_session_with_alos_own_entries(named: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(runtime.path(), named).expect("a display");
    let machine = crate::settings_testing::a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    let mut holding = alo_dock::Holding::nothing();
    holding.pin_entry(ADockEntry::Native(ANativeAction::Settings));
    holding.pin_entry(ADockEntry::Native(ANativeAction::Applications));
    server.what_the_dock_is_pinned_to = holding;
    (runtime, server)
}

/// **alo's own entries are marked while their window is open, and not
/// before.** Opened the way a person opens them, by pressing them, and read
/// back through the list every frame of the Dock is drawn from — so this is
/// the session and the bar together, not the bar given a row by hand.
#[test]
fn alos_own_entries_are_marked_while_their_window_is_open() {
    let (_runtime, mut server) = a_session_with_alos_own_entries("own-entries-marked");
    let artwork = TheArtwork::none();

    // The control: nothing open, so nothing marked.
    let rows = server.what_this_persons_dock_holds();
    let open = every_open_slot_is_marked(
        &drawn(Edge::Bottom, DisplayScale::ONE, &rows, &artwork),
        &rows,
        &artwork,
        "before either was opened",
    );
    assert!(open.is_empty(), "nothing was opened and {open:?} was open");

    for action in [ANativeAction::Settings, ANativeAction::Applications] {
        server.do_this_native_action(action);
    }
    assert!(server.settings.is_open(), "the premise: Settings opened");
    assert!(server.market.is_open(), "the premise: Applications opened");

    let rows = server.what_this_persons_dock_holds();
    for action in [ANativeAction::Settings, ANativeAction::Applications] {
        let row = rows
            .iter()
            .find(|row| row.native() == Some(action))
            .unwrap_or_else(|| panic!("{action:?} is on the Dock"));
        assert!(
            row.is_open(),
            "{action:?}'s window is open and its row on the Dock says it is not"
        );
    }
    for edge in Edge::EVERY {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let open = every_open_slot_is_marked(
                &drawn(edge, scale, &rows, &artwork),
                &rows,
                &artwork,
                &format!("both open, {edge:?} at {scale:?}"),
            );
            assert_eq!(
                open,
                BTreeSet::from([AKindOfSlot::AlosOwn]),
                "the premise: alo's own entries were open on {edge:?} at {scale:?}"
            );
        }
    }
}
