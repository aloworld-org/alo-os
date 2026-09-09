//! Full-frame native text readback using WSLg's GLES development fixture.

/// Propagate every graphics/refusal assertion to the component gate.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

/// This native renderer requires Linux.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Compare complete frames, including clipped label edges and untouched ground.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_shell::{
        LabelGeometry, RowOrder, WindowControlLabels, WindowControlLayout, readback_xrgb,
    };
    use alo_shortcuts::{Action, shortcut_words};
    use alo_strings::{Language, Strings, Translation};
    use drm::buffer::DrmFourcc;
    use smithay::{
        backend::{
            renderer::{
                Bind, Color32F, Frame, Offscreen, Renderer,
                gles::{GlesRenderbuffer, GlesRenderer},
            },
            winit,
        },
        reexports::winit::{
            raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
            window::Window,
        },
        utils::{Rectangle, Transform},
    };
    let (mut backend, _events) = winit::init_from_attributes::<GlesRenderer>(
        Window::default_attributes().with_title("alo native label check"),
    )?;
    if !matches!(
        backend.window().display_handle()?.as_raw(),
        RawDisplayHandle::Wayland(_)
    ) {
        return Err("requires a Wayland parent".into());
    }
    let (renderer, _window) = backend.bind()?;
    let mut buffer: GlesRenderbuffer =
        renderer.create_buffer(DrmFourcc::Abgr8888, (320, 180).into())?;
    let mut target = renderer.bind(&mut buffer)?;
    let vocabulary = shortcut_words()?;
    let german = Language::written("de")?;
    let translation = vocabulary
        .check(Translation::into_language(german.clone()).says(
            Action::CloseWindow.word().key(),
            "Schließen — Κλείσιμο — Затваряне",
        ))
        .map_err(|error| error.to_string())?;
    let mut strings = Strings::of(vocabulary);
    strings.speaks(translation)?;
    strings.prefers(&[german]);
    let mut labels = WindowControlLabels::new()?;
    let mut frames = 0;
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [100, 200, 300] {
            // Disabled controls must retain exactly the same readable names.
            let layout = WindowControlLayout::new((320, 180), (0, 0), [false; 3], false)?;
            for control in layout.controls() {
                for origin in [(8, 8), (-30, -10), (280, 155), (320, 180)] {
                    let label = labels.prepare(
                        control,
                        &strings,
                        LabelGeometry {
                            viewport: (320, 180),
                            origin,
                            size: (240, 140),
                        },
                        scheme,
                        TextScale::percent(percent).map_err(|_| "invalid fixture scale")?,
                    )?;
                    assert_eq!(label.said(), &control.action().said(&strings));
                    let ground = match scheme {
                        Scheme::Light => [248, 246, 242, 255],
                        Scheme::Dark => [31, 37, 41, 255],
                    };
                    assert!(
                        label.pixels().iter().filter(|p| **p != ground).count() > 30,
                        "rendered text is present"
                    );
                    let mut frame =
                        renderer.render(&mut target, (320, 180).into(), Transform::Normal)?;
                    frame.clear(
                        Color32F::new(0.0, 0.0, 0.0, 1.0),
                        &[Rectangle::from_size((320, 180).into())],
                    )?;
                    label.paint(&mut frame)?;
                    let _sync = frame.finish()?;
                    let pixels = readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
                    let (pixels, remainder) = pixels.pixels().as_chunks::<4>();
                    assert!(remainder.is_empty());
                    assert_eq!(pixels.len(), 320 * 180);
                    for (index, pixel) in pixels.iter().enumerate() {
                        let x = i32::try_from(index % 320)? - origin.0;
                        let y = i32::try_from(index / 320)? - origin.1;
                        let expected = if (0..240).contains(&x) && (0..140).contains(&y) {
                            let rgba = label
                                .pixels()
                                .get(usize::try_from(y * 240 + x)?)
                                .ok_or("incomplete label raster")?;
                            [rgba[2], rgba[1], rgba[0], 0]
                        } else {
                            [0; 4]
                        };
                        assert_eq!(*pixel, expected, "frame {frames}, pixel {index}");
                    }
                    frames += 1;
                }
            }
        }
    }
    assert_eq!(frames, 72);
    println!(
        "Native labels: 72 complete 57,600-pixel GLES frames; translated/source fallback, three actions, light/dark, 100/200/300% text, four clipping origins"
    );
    Ok(())
}
