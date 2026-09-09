//! Actual nested EGL strip transactions without synthetic backend success.
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    Nested, Server, WindowControlLabelFrame, WindowControlLabels,
    WindowControlPointerEvent as Event, WindowControlRelease as Release,
    WindowControlRoute as Route,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use smithay::backend::input::ButtonState;

/// Use a live root between dispatches; never close or resize the client fixture.
pub fn run(
    server: &mut Server,
    nested: &mut Nested,
    time: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = server
        .mapped_surfaces()
        .next()
        .cloned()
        .ok_or("missing root")?;
    let mut labels = WindowControlLabels::new()?;
    let strings = Strings::of(shortcut_words()?);
    let scale = TextScale::percent(100).map_err(|_| "invalid scale")?;
    for scheme in [Scheme::Light, Scheme::Dark] {
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        nested.render_labeled_window_controls(
            server,
            Some((&root, (3, 4))),
            scheme,
            Some(WindowControlLabelFrame {
                labels: &mut labels,
                strings: &strings,
                size: (160, 40),
                scale,
            }),
            time,
        )?;
        assert_eq!(
            server.route_presented_window_control_pointer((80.0, 45.0), Event::Motion, time)?,
            Route::Consumed
        );
        assert_eq!(
            server.route_presented_window_control_pointer(
                (80.0, 45.0),
                Event::Button(0x111, ButtonState::Pressed),
                time
            )?,
            Route::Consumed
        );
        nested.render_labeled_window_controls(
            server,
            Some((&root, (3, 4))),
            scheme,
            Some(WindowControlLabelFrame {
                labels: &mut labels,
                strings: &strings,
                size: (160, 40),
                scale,
            }),
            time,
        )?;
        assert_eq!(
            server.route_presented_window_control_pointer(
                (80.0, 45.0),
                Event::Button(0x111, ButtonState::Released),
                time
            )?,
            Route::Consumed
        );
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        nested.render_labeled_window_controls(
            server,
            Some((&root, (3, 4))),
            scheme,
            Some(WindowControlLabelFrame {
                labels: &mut labels,
                strings: &strings,
                size: (9, 9),
                scale,
            }),
            time,
        )?;
        // The 9px preferred box expands to full output width without clipping.
        assert_eq!(
            server.route_presented_window_control_pointer((1.0, 45.0), Event::Motion, time)?,
            Route::Consumed
        );
        assert!(server.focus_window_control(Some(Action::CloseWindow)));
        let mut impossible = Strings::of(shortcut_words()?);
        let language = alo_strings::Language::written("de")?;
        let translation = shortcut_words()?.check(
            alo_strings::Translation::into_language(language.clone())
                .says(Action::CloseWindow.word().key(), "long ".repeat(100)),
        )?;
        impossible.speaks(translation)?;
        impossible.prefers(&[language]);
        assert!(matches!(
            nested.render_labeled_window_controls(
                server,
                Some((&root, (3, 4))),
                scheme,
                Some(WindowControlLabelFrame {
                    labels: &mut labels,
                    strings: &impossible,
                    size: (9, 9),
                    scale
                }),
                time
            ),
            Err(alo_shell::RenderError::ControlScene)
        ));
        assert!(server.presented_window_controls(None).is_none());
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        assert_eq!(
            server
                .presented_window_controls(None)
                .ok_or("missing published strip")?
                .surface(),
            &root
        );
        assert_eq!(
            server.route_presented_window_control_pointer(
                (76.0, 5.0),
                Event::Button(0x110, ButtonState::Pressed),
                time
            )?,
            Route::Consumed
        );
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        // Geometry refusal must retire the previous successful EGL frame's authority.
        assert!(
            nested
                .render_window_controls(server, Some((&root, (i32::MAX, 0))), scheme, time)
                .is_err()
        );
        assert!(server.presented_window_controls(None).is_none());
        assert_eq!(
            server.route_presented_window_control_pointer(
                (76.0, 5.0),
                Event::Button(0x110, ButtonState::Released),
                time
            )?,
            Route::Released(Release::Cancelled)
        );
        nested.render_window_controls(server, Some((&root, (3, 4))), scheme, time)?;
        server.render(nested, time)?;
        assert!(server.presented_window_controls(None).is_none());
    }
    println!(
        "Nested control transactions: eight EGL strip submissions, two label submissions, two expanded full-name submissions, two label dismissals, two ordinary removals, exhausted-space and strip refusal/recovery sequences passed"
    );
    Ok(())
}
