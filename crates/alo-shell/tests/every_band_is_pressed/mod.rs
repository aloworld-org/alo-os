//! **An application's band is pressed by the road every band is**, over a
//! real Wayland client: a screen reader finds each control alo's band has on
//! it and can press it, finds none it does not have, and Alt+F4 asks this
//! window to close and no other.
//!
//! The rule over alo's own windows is `src/a_band_is_pressed_tests.rs`; this
//! is the same rule where the window is an application's. Until 2026-11-21 a
//! reader was told an application's band controls and could press none of
//! them (`Pressed` had no way to name one).
//!
//! Two kinds of application, by what each draws itself
//! (`window_edge_who_draws`): one alo knows nothing about, which gets Window
//! actions alone, and Firefox, measured drawing Close and no Minimise, which
//! gets Window actions and Minimise. **Not covered here: an application that
//! leaves its header to the shell** (Dolphin and Qt), because this test's
//! client does not bind `xdg_decoration`. Its band is pressed by the same
//! function with a longer list of controls; the walk's band section presses
//! Dolphin's three on a booted machine.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use alo_access::{Surface, TurnedOn};
use alo_shell::{ABand, OnTheEdge, Pressed, WhatAReaderPressCameTo};
use alo_shortcuts::{Action, Shortcuts};

use super::{Application, Fixture};

/// A mapped window, with `app_id` if it declares one.
fn mapped(fixture: &Fixture, app_id: Option<&str>) -> Application {
    let mut app = Application::new(fixture);
    if let Some(app_id) = app_id {
        app.toplevel.set_app_id(app_id.to_owned());
    }
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Every press a reader is offered on this window's band.
fn what_a_reader_can_press_on(fixture: &Fixture, band: &ABand) -> Vec<OnTheEdge> {
    let band = band.clone();
    fixture.backend(move |server| {
        server
            .read_aloud_with_the_frames_open(
                &alo_strings::Strings::of(alo_strings::Vocabulary::empty()),
                &Surface::ALL,
                &TurnedOn::nothing(),
            )
            .what_can_be_pressed()
            .into_iter()
            .filter_map(|pressed| match pressed {
                Pressed::OnABand { band: on, control } if on == band => Some(control),
                _ => None,
            })
            .collect()
    })
}

/// A reader pressed `control` on `band`.
fn a_reader_presses(fixture: &Fixture, band: &ABand, control: OnTheEdge) -> WhatAReaderPressCameTo {
    let band = band.clone();
    fixture.backend(move |server| server.a_reader_pressed(Pressed::OnABand { band, control }))
}

/// **Each kind: a reader is offered exactly the controls the band has**, and
/// each one pressed does something, while a control the band does not have
/// does nothing.
#[test]
fn a_reader_presses_every_control_on_every_kind_of_application_window() {
    for (app_id, has) in [
        (None, vec![OnTheEdge::WindowActions]),
        (
            Some("org.mozilla.firefox"),
            vec![OnTheEdge::WindowActions, OnTheEdge::Minimise],
        ),
    ] {
        let f = Fixture::keyboard();
        let _app = mapped(&f, app_id);
        let band = ABand::App(f.root());
        assert_eq!(
            what_a_reader_can_press_on(&f, &band),
            has,
            "{app_id:?}: a reader is not offered the controls its band has"
        );
        for control in [
            OnTheEdge::WindowActions,
            OnTheEdge::Minimise,
            OnTheEdge::Close,
        ] {
            let came_to = a_reader_presses(&f, &band, control);
            if has.contains(&control) {
                assert_eq!(
                    came_to,
                    WhatAReaderPressCameTo::ABandDidIt,
                    "{app_id:?}: a reader's {control:?} did nothing"
                );
            } else {
                assert_eq!(
                    came_to,
                    WhatAReaderPressCameTo::NothingOnABand,
                    "{app_id:?}: a reader pressed a {control:?} its band does not have"
                );
            }
        }
    }
}

/// **A reader's press on a window that has gone does nothing**: a tree is a
/// moment old by the time it is pressed.
#[test]
fn a_readers_press_on_a_band_that_has_gone_does_nothing() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, None);
    let band = ABand::App(f.root());
    app.toplevel.destroy();
    app.xdg.destroy();
    app.surface.destroy();
    app.sync();
    f.wait_for((0, 0));
    assert_eq!(
        a_reader_presses(&f, &band, OnTheEdge::WindowActions),
        WhatAReaderPressCameTo::NothingOnABand
    );
}

/// **Alt+F4 asks the window with the keyboard to close, and no other**, on
/// an application that draws its own Close and so has none on alo's band.
#[test]
fn alt_f4_closes_an_application_whose_band_has_no_close() {
    let f = Fixture::keyboard();
    let mut firefox = mapped(&f, Some("org.mozilla.firefox"));
    let root = f.root();
    let mut other = mapped(&f, None);
    assert!(f.focus_surface(root).is_ok());
    let close = Shortcuts::shipped()
        .chord_for(Action::CloseWindow)
        .expect("the release ships Alt+F4");
    let carried = f.backend(move |server| {
        server.the_shortcuts_are(Shortcuts::shipped());
        server.dispatch_window_shortcut(&Shortcuts::shipped(), close)
    });
    assert!(
        matches!(carried, Ok(Some(Action::CloseWindow))),
        "Alt+F4 was not carried out: {carried:?}"
    );
    firefox.sync();
    other.sync();
    assert_eq!(
        firefox.events.close_requests, 1,
        "it was not asked to close"
    );
    assert_eq!(
        other.events.close_requests, 0,
        "the window behind was asked"
    );
}
