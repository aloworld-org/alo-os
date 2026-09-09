//! Actual nested EGL strip transactions without synthetic backend success.
use alo_appearance::Scheme;
use alo_shell::{
    Nested, Server, WindowControlPointerEvent as Event, WindowControlRelease as Release,
    WindowControlRoute as Route,
};
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
    for scheme in [Scheme::Light, Scheme::Dark] {
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
        "Nested control transactions: six EGL strip submissions, two removals and two refusal/recovery sequences passed"
    );
    Ok(())
}
