//! Live label selection, placement, dismissal and ordinary input isolation.
use super::{
    mapped,
    routing::{CLOSE, DOWN, UP, route},
};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    PaintedWindowControls, WindowControlLabelError, WindowControlLabelSelection as Selection,
    WindowControlLabelTarget, WindowControlLabels,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn target(
    f: &Fixture,
    root: &WlSurface,
    selection: Selection,
    viewport: (i32, i32),
    origin: (i32, i32),
    size: (i32, i32),
) -> std::result::Result<Option<WindowControlLabelTarget>, WindowControlLabelError> {
    let root = root.clone();
    f.backend(move |s| {
        s.window_control_label_target(
            Some(PaintedWindowControls {
                surface: &root,
                viewport,
                origin,
            }),
            selection,
            size,
        )
    })
}

#[test]
fn window_control_labels_live_disabled_focus_and_hover_prepare_without_input_effects() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    let mut labels = WindowControlLabels::new()?;
    let strings = Strings::of(shortcut_words()?);
    app.sync();
    let sizes = app.events.sizes.clone();
    for selection in [
        Selection::Pointer(40.0, 5.0),
        Selection::Focus(Action::MaximiseWindow),
    ] {
        let chosen = target(&f, &root, selection, (320, 180), (3, 4), (240, 80))?
            .ok_or("missing disabled name")?;
        assert!(!chosen.control.enabled());
        assert_eq!(chosen.control.action(), Action::MaximiseWindow);
        assert_eq!(chosen.geometry.origin, (39, 40));
        let label = labels.prepare(
            &chosen.control,
            &strings,
            chosen.geometry,
            Scheme::Light,
            TextScale::percent(100).map_err(|_| "invalid fixture scale")?,
        )?;
        assert_eq!(
            label.said().text(),
            Action::MaximiseWindow.said(&strings).text()
        );
        assert!(!label.clipped());
        assert!(f.key(30, KeyState::Pressed)?);
        assert!(f.key(30, KeyState::Released)?);
    }
    app.sync();
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.keyboard.keys.len(), 4);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert!(app.events.pointer.buttons.is_empty());
    Ok(())
}

#[test]
fn window_control_labels_live_edges_small_outputs_and_invalid_geometry() -> Result {
    let f = Fixture::new();
    let _app = mapped(&f);
    let root = f.root();
    for (viewport, origin, size, expected_origin, expected_size) in [
        ((320, 180), (280, 140), (100, 60), (220, 76), (100, 60)),
        ((120, 48), (-10, -10), (240, 80), (0, 0), (120, 48)),
        ((9, 9), (0, 0), (240, 80), (0, 0), (9, 9)),
        ((120, 100), (3, 4), (40, 80), (3, 20), (40, 80)),
    ] {
        let chosen = target(
            &f,
            &root,
            Selection::Focus(Action::MinimiseWindow),
            viewport,
            origin,
            size,
        )?
        .ok_or("edge label missing")?;
        assert_eq!(chosen.geometry.origin, expected_origin);
        assert_eq!(chosen.geometry.size, expected_size);
    }
    for (viewport, size) in [
        ((8, 9), (100, 40)),
        ((120, 48), (8, 40)),
        ((120, 48), (2049, 40)),
        ((0, 48), (100, 40)),
    ] {
        assert!(matches!(
            target(
                &f,
                &root,
                Selection::Focus(Action::MinimiseWindow),
                viewport,
                (0, 0),
                size
            ),
            Err(WindowControlLabelError::Geometry)
        ));
    }
    for selection in [
        Selection::Pointer(f64::NAN, 5.0),
        Selection::Pointer(35.0, 5.0),
        Selection::Pointer(120.0, 5.0),
        Selection::Dismissed,
    ] {
        assert!(target(&f, &root, selection, (120, 48), (3, 4), (100, 40))?.is_none());
    }
    assert!(
        target(
            &f,
            &root,
            Selection::Focus(Action::CloseWindow),
            (50, 48),
            (3, 4),
            (40, 40)
        )?
        .is_none()
    );
    assert!(
        f.backend(|s| s.window_control_label_target(
            None,
            Selection::Focus(Action::CloseWindow),
            (100, 40)
        ))?
        .is_none()
    );
    Ok(())
}

#[test]
fn window_control_labels_live_press_cancel_client_grab_and_hidden_target_dismiss() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let selected = || {
        target(
            &f,
            &root,
            Selection::Pointer(CLOSE.0, CLOSE.1),
            (120, 48),
            (3, 4),
            (100, 40),
        )
    };
    assert!(selected()?.is_some());
    route(&f, Some(&root), CLOSE, DOWN)?;
    assert!(selected()?.is_none());
    f.backend(|s| s.cancel_window_control());
    assert!(selected()?.is_none());
    route(&f, Some(&root), CLOSE, UP)?;
    assert!(selected()?.is_some());
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(selected()?.is_none());
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    let hidden = root.clone();
    f.backend(move |s| s.set_window_minimized(&hidden, true))?;
    assert!(selected()?.is_none());
    let hidden = root.clone();
    f.backend(move |s| s.set_window_minimized(&hidden, false))?;
    assert!(selected()?.is_some());
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(selected()?.is_none());
    let foreign = Fixture::new();
    let _foreign_app = mapped(&foreign);
    assert!(
        target(
            &f,
            &foreign.root(),
            Selection::Pointer(CLOSE.0, CLOSE.1),
            (120, 48),
            (3, 4),
            (100, 40)
        )?
        .is_none()
    );
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
