//! Read back live selected labels and their disappearance when a root is hidden.
use alo_appearance::{Scheme, TextScale};
use alo_shell::{
    PaintedWindowControls, RowOrder, Server, WindowControlLabelSelection as Selection,
    WindowControlLabelTarget, WindowControlLabels, readback_xrgb,
};
use alo_shortcuts::{Action, shortcut_words};
use alo_strings::Strings;
use drm::buffer::DrmFourcc;
use smithay::{
    backend::renderer::{
        Bind, Color32F, Frame, Offscreen, Renderer,
        gles::{GlesRenderbuffer, GlesRenderer},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Rectangle, Transform},
};

/// Each call checks two entire 5,760-pixel frames, including untouched background.
pub fn paint(
    server: &Server,
    renderer: &mut GlesRenderer,
    root: &WlSurface,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let choice = server.window_control_label_target(
        Some(PaintedWindowControls {
            surface: root,
            viewport: (120, 48),
            origin: (3, 4),
        }),
        Selection::Pointer(4.0, 5.0),
        (100, 40),
    )?;
    paint_choice(renderer, choice, visible)
}

/// Ten full frames exercise retained focus, invisible transitions and retirement.
pub fn presentation(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    root: &WlSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let publish = |server: &mut Server| {
        server.present_window_controls(Some(PaintedWindowControls {
            surface: root,
            viewport: (120, 48),
            origin: (3, 4),
        }))
    };
    publish(server)?;
    assert!(server.focus_window_control(Some(Action::MinimiseWindow)));
    publish(server)?;
    paint_choice(
        renderer,
        server.presented_window_control_label(None, (100, 40))?,
        true,
    )?;
    server.set_window_minimized(root, true)?;
    server.set_window_minimized(root, false)?;
    paint_choice(
        renderer,
        server.presented_window_control_label(None, (100, 40))?,
        false,
    )?;
    publish(server)?;
    paint_choice(
        renderer,
        server.presented_window_control_label(None, (100, 40))?,
        false,
    )?;
    assert!(server.focus_window_control(Some(Action::MinimiseWindow)));
    paint_choice(
        renderer,
        server.presented_window_control_label(None, (100, 40))?,
        true,
    )?;
    server.retire_window_controls();
    paint_choice(
        renderer,
        server.presented_window_control_label(None, (100, 40))?,
        false,
    )?;
    println!("Mapping-bound presentation: ten complete label lifetime frames passed");
    nested_input(server, renderer, root)?;
    Ok(())
}

/// Exercise the event-pump adapter through live label pixels and minimization.
fn nested_input(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    root: &WlSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{NestedControlInput, NestedPointerEvent};
    use smithay::backend::input::ButtonState;
    server.present_window_controls(Some(PaintedWindowControls {
        surface: root,
        viewport: (120, 48),
        origin: (3, 4),
    }))?;
    let mut input = NestedControlInput::default();
    input.route(
        server,
        true,
        Some(NestedPointerEvent::Motion {
            x: 4.0,
            y: 5.0,
            time: 1,
        }),
    )?;
    paint_choice(
        renderer,
        server.presented_window_control_label(input.position(), (100, 40))?,
        true,
    )?;
    input.route(
        server,
        true,
        Some(NestedPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 2,
        }),
    )?;
    paint_choice(
        renderer,
        server.presented_window_control_label(input.position(), (100, 40))?,
        false,
    )?;
    input.route(
        server,
        true,
        Some(NestedPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Released,
            time: 3,
        }),
    )?;
    assert!(!server.mapped_surfaces().any(|surface| surface == root));
    paint_choice(
        renderer,
        server.presented_window_control_label(input.position(), (100, 40))?,
        false,
    )?;
    server.set_window_minimized(root, false)?;
    server.present_window_controls(Some(PaintedWindowControls {
        surface: root,
        viewport: (120, 48),
        origin: (3, 4),
    }))?;
    input.route(server, false, None)?;
    assert_eq!(input.position(), None);
    paint_choice(
        renderer,
        server.presented_window_control_label(input.position(), (100, 40))?,
        false,
    )?;
    println!("Nested control input: eight complete label frames and live minimization passed");
    Ok(())
}

/// Compare every output pixel to the selected raster or cleared background.
fn paint_choice(
    renderer: &mut GlesRenderer,
    choice: Option<WindowControlLabelTarget>,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(choice.is_some(), visible);
    if let Some(choice) = choice {
        assert_eq!(choice.control.action(), Action::MinimiseWindow);
        assert_eq!(choice.geometry.origin, (3, 8));
        assert_eq!(choice.geometry.size, (100, 40));
    }
    let mut labels = WindowControlLabels::new()?;
    let strings = Strings::of(shortcut_words()?);
    let scale = TextScale::percent(100).map_err(|_| "invalid fixture scale")?;
    let mut buffer: GlesRenderbuffer =
        renderer.create_buffer(DrmFourcc::Abgr8888, (120, 48).into())?;
    let mut target = renderer.bind(&mut buffer)?;
    for scheme in [Scheme::Light, Scheme::Dark] {
        let label = choice
            .map(|choice| labels.prepare(&choice.control, &strings, choice.geometry, scheme, scale))
            .transpose()?;
        let mut frame = renderer.render(&mut target, (120, 48).into(), Transform::Normal)?;
        frame.clear(
            Color32F::new(0.0, 0.0, 0.0, 1.0),
            &[Rectangle::from_size((120, 48).into())],
        )?;
        if let Some(label) = &label {
            label.paint(&mut frame)?;
        }
        let _sync = frame.finish()?;
        let pixels = readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
        let (pixels, remainder) = pixels.pixels().as_chunks::<4>();
        assert!(remainder.is_empty());
        assert_eq!(pixels.len(), 120 * 48);
        for (index, pixel) in pixels.iter().enumerate() {
            let x = i32::try_from(index % 120)? - 3;
            let y = i32::try_from(index / 120)? - 8;
            let expected = if let Some(label) = &label
                && (0..100).contains(&x)
                && (0..40).contains(&y)
            {
                let rgba = label
                    .pixels()
                    .get(usize::try_from(y * 100 + x)?)
                    .ok_or("missing prepared pixel")?;
                [rgba[2], rgba[1], rgba[0], 0]
            } else {
                [0; 4]
            };
            assert_eq!(
                *pixel, expected,
                "live label visible={visible} pixel {index}"
            );
        }
    }
    println!("Live label visible={visible}: two complete 5,760-pixel GLES frames passed");
    Ok(())
}
