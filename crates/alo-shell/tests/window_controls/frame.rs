//! Real protocol clients at the submission/publication boundary.
use super::{
    mapped,
    routing::{CLOSE, DOWN, UP},
};
use crate::Fixture;
use alo_appearance::Scheme;
use alo_shell::{
    Cursor, FrameTarget, RenderError, WindowControlFrame, WindowControlRelease as Release,
    WindowControlRoute as Route, WindowControlScene,
};
use alo_shortcuts::Action;
use smithay::{
    backend::input::KeyState,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Target {
    fail: bool,
    omit: bool,
    calls: usize,
    enabled: Option<[bool; 3]>,
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
        self.enabled = scene.map(|scene| {
            assert!(scene.label.is_none());
            let [first, ..] = scene.layout.controls();
            assert_eq!(first.bounds().loc, (3, 4).into());
            scene.layout.controls().map(|c| c.enabled())
        });
        if self.fail {
            return Err(RenderError::Submission("fixture refusal".into()));
        }
        Ok(if self.omit {
            Vec::new()
        } else {
            roots.to_vec()
        })
    }
}

fn draw(
    f: &Fixture,
    root: &WlSurface,
    fail: bool,
    omit: bool,
) -> std::result::Result<(usize, Option<[bool; 3]>), RenderError> {
    let root = root.clone();
    f.backend(move |s| {
        let mut target = Target {
            fail,
            omit,
            ..Default::default()
        };
        let count = s.render_window_controls(
            &mut target,
            Some(WindowControlFrame {
                surface: &root,
                origin: (3, 4),
                position: Some(CLOSE),
                scheme: Scheme::Light,
            }),
            77,
        )?;
        assert_eq!(target.calls, 1);
        Ok((count, target.enabled))
    })
}

#[test]
fn control_frame_success_refresh_callbacks_and_normal_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    assert_eq!(
        draw(&f, &root, false, false)?,
        (1, Some([true, false, true]))
    );
    app.sync();
    assert_eq!(app.events.frames, [77]);
    // First output publication makes maximize available on the next fresh frame.
    assert_eq!(draw(&f, &root, false, false)?, (1, Some([true; 3])));
    assert!(f.backend(|s| s.focus_window_control(Some(Action::CloseWindow))));
    draw(&f, &root, false, false)?;
    assert!(
        f.backend(|s| s.presented_window_control_label(None, (100, 40)))?
            .is_some()
    );
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    assert_eq!(
        f.backend(|s| s.route_presented_window_control_pointer(CLOSE, DOWN, 1))?,
        Route::Consumed
    );
    draw(&f, &root, false, false)?;
    assert_eq!(
        f.backend(|s| s.route_presented_window_control_pointer(CLOSE, UP, 2))?,
        Route::Released(Release::Executed(Action::CloseWindow))
    );
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(app.events.pointer.buttons.is_empty());
    Ok(())
}

#[test]
fn control_frame_failures_retire_and_preserve_callbacks_and_owned_release() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    for case in 0..5 {
        draw(&f, &root, false, false)?;
        assert_eq!(
            f.backend(|s| s.route_presented_window_control_pointer(CLOSE, DOWN, 1))?,
            Route::Consumed
        );
        let before = app.events.frames.len();
        app.surface.frame(&app.queue.handle(), ());
        app.surface.commit();
        app.sync();
        let result = match case {
            0 => draw(&f, &root, true, false),
            1 => draw(&f, &root, false, true),
            2 => draw(&f, &foreign.root(), false, false),
            3 => {
                let root = root.clone();
                f.backend(move |s| {
                    s.render_window_controls(
                        &mut Target::default(),
                        Some(WindowControlFrame {
                            surface: &root,
                            origin: (i32::MAX, 0),
                            position: None,
                            scheme: Scheme::Dark,
                        }),
                        88,
                    )
                })
                .map(|count| (count, None))
            }
            _ => f
                .backend(|s| {
                    s.render(
                        &mut Target {
                            fail: true,
                            ..Default::default()
                        },
                        88,
                    )
                })
                .map(|count| (count, None)),
        };
        match case {
            0 | 4 => assert!(matches!(result, Err(RenderError::Submission(_)))),
            1 => assert!(matches!(result, Err(RenderError::ControlTargetOmitted))),
            _ => assert!(matches!(result, Err(RenderError::ControlSnapshot(_)))),
        }
        app.sync();
        assert_eq!(app.events.frames.len(), before);
        assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
        assert_eq!(
            f.backend(|s| s.route_presented_window_control_pointer(CLOSE, UP, 2))?,
            Route::Released(Release::Cancelled)
        );
        draw(&f, &root, false, false)?;
        app.sync();
        assert_eq!(app.events.frames.len(), before + 1);
        assert_eq!(app.events.close_requests, 0);
    }
    Ok(())
}

#[test]
fn control_frame_removal_replacement_and_legacy_target_refusal() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let _other = mapped(&f);
    let original = root.clone();
    let replacement = f
        .backend(move |s| s.mapped_surfaces().find(|s| **s != original).cloned())
        .ok_or("replacement")?;
    for replace in [true, false] {
        draw(&f, &root, false, false)?;
        f.backend(|s| s.route_presented_window_control_pointer(CLOSE, DOWN, 1))?;
        if replace {
            draw(&f, &replacement, false, false)?;
        } else {
            f.backend(|s| s.render(&mut Target::default(), 88))?;
            assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
        }
        assert_eq!(
            f.backend(|s| s.route_presented_window_control_pointer(CLOSE, UP, 2))?,
            Route::Released(Release::Cancelled)
        );
    }
    struct Legacy {
        called: bool,
    }
    impl FrameTarget for Legacy {
        fn size(&self) -> Size<i32, Physical> {
            (320, 180).into()
        }
        fn submit(&mut self, _: &[WlSurface]) -> std::result::Result<Vec<WlSurface>, RenderError> {
            self.called = true;
            Err(RenderError::Submission(
                "unsupported controls reached submission".into(),
            ))
        }
    }
    draw(&f, &root, false, false)?;
    assert!(matches!(
        f.backend(move |s| {
            let mut target = Legacy { called: false };
            let result = s.render_window_controls(
                &mut target,
                Some(WindowControlFrame {
                    surface: &root,
                    origin: (3, 4),
                    position: None,
                    scheme: Scheme::Light,
                }),
                99,
            );
            assert!(!target.called);
            result
        }),
        Err(RenderError::ControlsUnsupported)
    ));
    assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
