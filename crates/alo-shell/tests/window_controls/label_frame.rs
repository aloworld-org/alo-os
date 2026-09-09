//! Fresh label transactions and opaque input exclusion with actual clients.
use super::{mapped, routing::CLOSE};
use crate::Fixture;
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    Cursor, FrameTarget, RenderError, WindowControlFrame, WindowControlLabelFrame,
    WindowControlLabels, WindowControlPointerEvent as Event, WindowControlRoute as Route,
    WindowControlScene,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use smithay::{
    backend::input::{Axis, ButtonState, KeyState},
    input::pointer::AxisFrame,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Target {
    fail: bool,
    label: Option<String>,
    calls: usize,
}
impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (320, 180).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
    fn submit_controls(
        &mut self,
        roots: &[WlSurface],
        _: &[alo_shell::Popup],
        _: &Cursor,
        scene: Option<WindowControlScene<'_>>,
    ) -> std::result::Result<Vec<WlSurface>, RenderError> {
        self.calls += 1;
        self.label = scene.and_then(|scene| {
            scene.label.map(|label| {
                assert!(!label.clipped());
                assert_eq!(label.bounds().loc.y, 40);
                label.said().text().to_owned()
            })
        });
        if self.fail {
            return Err(RenderError::Submission("fixture label refusal".into()));
        }
        Ok(roots.to_vec())
    }
}

fn draw(
    f: &Fixture,
    root: &WlSurface,
    position: Option<(f64, f64)>,
    size: (i32, i32),
    fail: bool,
) -> Result<Option<String>> {
    draw_text(f, root, position, size, fail, None)
}

fn draw_text(
    f: &Fixture,
    root: &WlSurface,
    position: Option<(f64, f64)>,
    size: (i32, i32),
    fail: bool,
    text: Option<String>,
) -> Result<Option<String>> {
    let root = root.clone();
    Ok(f.backend(move |s| -> std::result::Result<_, String> {
        let mut labels = WindowControlLabels::new().map_err(|e| e.to_string())?;
        let vocabulary = shortcut_words().map_err(|e| e.to_string())?;
        let mut strings = Strings::of(vocabulary.clone());
        if let Some(text) = text {
            let language = alo_strings::Language::written("de").map_err(|e| e.to_string())?;
            let translation = vocabulary
                .check(
                    alo_strings::Translation::into_language(language.clone())
                        .says(Action::CloseWindow.word().key(), &text),
                )
                .map_err(|e| e.to_string())?;
            strings.speaks(translation).map_err(|e| e.to_string())?;
            strings.prefers(&[language]);
        }
        let mut target = Target {
            fail,
            ..Default::default()
        };
        let result = s.render_labeled_window_controls(
            &mut target,
            Some(WindowControlFrame {
                surface: &root,
                origin: (3, 4),
                position,
                scheme: Scheme::Light,
            }),
            Some(WindowControlLabelFrame {
                labels: &mut labels,
                strings: &strings,
                size,
                scale: TextScale::percent(100).map_err(|_| "scale")?,
            }),
            91,
        );
        if matches!(
            result,
            Err(RenderError::ControlScene | RenderError::ControlLabel(_))
        ) {
            assert_eq!(target.calls, 0);
        }
        result.map_err(|e| e.to_string())?;
        assert_eq!(target.calls, 1);
        Ok(target.label)
    })?)
}

#[test]
fn expanded_label_frame_owns_new_area_and_refuses_exhausted_space_without_callbacks() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    let placed = root.clone();
    f.backend(move |s| s.place_window(&placed, (0, 40)))?;
    let covered = (1.0, 45.0);
    assert_eq!(route(&f, covered, Event::Motion)?, Route::Client(true));
    let words = "Dieses Fenster schließen";
    assert_eq!(
        draw_text(&f, &root, Some(CLOSE), (9, 9), false, Some(words.into()))?.as_deref(),
        Some(words)
    );
    assert_eq!(route(&f, covered, Event::Motion)?, Route::Consumed);
    assert!(!f.backend(move |s| s.route_presented_window_control_axis(
        covered,
        AxisFrame::new(3).value(Axis::Vertical, 10.0)
    ))?);
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    assert!(
        draw_text(
            &f,
            &root,
            Some(CLOSE),
            (9, 9),
            false,
            Some("long ".repeat(100))
        )
        .is_err()
    );
    assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(app.events.pointer.axes.is_empty());
    assert_eq!(route(&f, covered, Event::Motion)?, Route::Client(true));
    assert!(draw(&f, &root, Some(CLOSE), (9, 9), true).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert!(draw(&f, &root, Some(CLOSE), (9, 9), false)?.is_some());
    assert_eq!(
        route(&f, covered, Event::Button(0x111, ButtonState::Pressed))?,
        Route::Consumed
    );
    assert_eq!(draw(&f, &root, Some(CLOSE), (9, 9), false)?, None);
    assert_eq!(
        route(&f, covered, Event::Button(0x111, ButtonState::Released))?,
        Route::Consumed
    );
    app.sync();
    assert_eq!(app.events.frames, [91]);
    assert!(app.events.pointer.buttons.is_empty());
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

fn route(f: &Fixture, position: (f64, f64), event: Event) -> Result<Route> {
    Ok(f.backend(move |s| s.route_presented_window_control_pointer(position, event, 2))?)
}

#[test]
fn label_frame_fresh_focus_replacement_dismissal_and_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    let close = draw(&f, &root, Some(CLOSE), (160, 40), false)?.ok_or("missing close")?;
    app.sync();
    assert_eq!(app.events.frames, [91]);
    assert!(f.backend(|s| s.focus_window_control(Some(Action::MinimiseWindow))));
    let minimize = draw(&f, &root, None, (160, 40), false)?.ok_or("missing focus label")?;
    assert_ne!(close, minimize);
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    let mut other = mapped(&f);
    let replacement = f
        .backend(move |s| s.mapped_surfaces().find(|r| **r != root).cloned())
        .ok_or("missing replacement")?;
    assert_eq!(draw(&f, &replacement, None, (160, 40), false)?, None);
    assert!(draw(&f, &replacement, Some(CLOSE), (160, 40), false)?.is_some());
    route(&f, (2.0, 2.0), Event::Motion)?;
    assert_eq!(
        draw(&f, &replacement, Some((2.0, 2.0)), (160, 40), false)?,
        None
    );
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(app.events.close_requests + other.events.close_requests, 0);
    Ok(())
}

#[test]
fn label_frame_blocks_click_scroll_and_drains_buttons_after_failure_and_leave() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let placed = root.clone();
    f.backend(move |s| s.place_window(&placed, (76, 40)))?;
    let covered = (80.0, 45.0);
    route(&f, covered, Event::Motion)?;
    app.sync();
    let enters = app.events.pointer.enters.len();
    assert!(enters > 0);
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), false)?.is_some());
    // Motion and presses are in one event batch, before any removal frame.
    assert_eq!(route(&f, covered, Event::Motion)?, Route::Consumed);
    assert!(!f.backend(move |s| s.route_presented_window_control_axis(
        covered,
        AxisFrame::new(3).value(Axis::Vertical, 10.0)
    ))?);
    for code in [0x110, 0x111] {
        assert_eq!(
            route(&f, covered, Event::Button(code, ButtonState::Pressed))?,
            Route::Consumed
        );
    }
    assert!(!f.backend(|s| s.focus_window_control(Some(Action::CloseWindow))));
    assert_eq!(
        f.backend(|s| s.presented_window_controls(Some(CLOSE)))
            .ok_or("missing held strip")?
            .layout()
            .controls()
            .map(|control| control.feedback()),
        [alo_shell::WindowControlFeedback::Idle; 3]
    );
    assert_eq!(draw(&f, &root, Some(CLOSE), (160, 40), false)?, None);
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), true).is_err());
    f.backend(|s| s.pointer_leave())?;
    assert_eq!(
        route(
            &f,
            (f64::NAN, f64::NAN),
            Event::Button(0x110, ButtonState::Released)
        )?,
        Route::Consumed
    );
    // Exercise inactive parent-adapter drainage without a fresh position.
    f.backend(|s| {
        alo_shell::NestedControlInput::default().route(
            s,
            false,
            Some(alo_shell::NestedPointerEvent::Button {
                code: 0x111,
                state: ButtonState::Released,
                time: 4,
            }),
        )
    })?;
    app.sync();
    assert!(app.events.pointer.buttons.is_empty());
    assert!(app.events.pointer.axes.is_empty());
    assert_eq!(app.events.pointer.enters.len(), enters);
    route(&f, covered, Event::Motion)?;
    for state in [ButtonState::Pressed, ButtonState::Released] {
        assert_eq!(
            route(&f, covered, Event::Button(0x110, state))?,
            Route::Client(true)
        );
    }
    assert!(f.backend(move |s| s.route_presented_window_control_axis(
        covered,
        AxisFrame::new(5).value(Axis::Vertical, 10.0)
    ))?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.pointer.axes.len(), 1);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn label_frame_failed_submission_keeps_callbacks_pending_and_retires_exclusion() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let placed = root.clone();
    f.backend(move |s| s.place_window(&placed, (76, 40)))?;
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), false)?.is_some());
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), true).is_err());
    app.sync();
    assert!(app.events.frames.is_empty());
    assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    assert_eq!(route(&f, (80.0, 45.0), Event::Motion)?, Route::Client(true));
    for state in [ButtonState::Pressed, ButtonState::Released] {
        assert_eq!(
            route(&f, (80.0, 45.0), Event::Button(0x111, state))?,
            Route::Client(true)
        );
    }
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), false)?.is_some());
    app.sync();
    assert_eq!(app.events.frames, [91]);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    Ok(())
}

#[test]
fn label_frame_expands_clipping_refuses_geometry_and_preserves_existing_client_grab() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    assert!(draw(&f, &root, Some(CLOSE), (9, 9), false)?.is_some());
    for size in [(8, 9), (8, 40)] {
        assert!(draw(&f, &root, Some(CLOSE), size, false).is_err());
        assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    }
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), false)?.is_some());
    route(&f, (2.0, 2.0), Event::Motion)?;
    assert_eq!(
        route(&f, (2.0, 2.0), Event::Button(0x111, ButtonState::Pressed))?,
        Route::Client(true)
    );
    assert_eq!(route(&f, (80.0, 45.0), Event::Motion)?, Route::Client(true));
    assert_eq!(draw(&f, &root, Some(CLOSE), (160, 40), false)?, None);
    assert_eq!(
        route(
            &f,
            (80.0, 45.0),
            Event::Button(0x111, ButtonState::Released)
        )?,
        Route::Client(true)
    );
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn label_frame_missing_vocabulary_refuses_before_submission() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    assert!(draw(&f, &root, Some(CLOSE), (160, 40), false)?.is_some());
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    f.backend(move |s| -> std::result::Result<(), String> {
        let mut labels = WindowControlLabels::new().map_err(|e| e.to_string())?;
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let mut target = Target::default();
        assert!(matches!(
            s.render_labeled_window_controls(
                &mut target,
                Some(WindowControlFrame {
                    surface: &root,
                    origin: (3, 4),
                    position: Some(CLOSE),
                    scheme: Scheme::Dark
                }),
                Some(WindowControlLabelFrame {
                    strings: &strings,
                    labels: &mut labels,
                    size: (160, 40),
                    scale: TextScale::percent(100).map_err(|_| "scale")?
                }),
                92
            ),
            Err(RenderError::ControlLabel(
                alo_shell::WindowControlLabelError::Vocabulary
            ))
        ));
        assert_eq!(target.calls, 0);
        assert!(s.presented_window_controls(None).is_none());
        Ok(())
    })?;
    app.sync();
    assert!(app.events.frames.is_empty());
    Ok(())
}
