//! Full GLES readback of every prepared native-name page, with explicit fonts.

/// Propagate all graphical and pixel failures to the component gate.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

/// Native rendering is Linux-only.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Compare complete page frames on the actual Wayland/GLES backend.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_shell::{
        LabelGeometry, RowOrder, WindowControlLabelTarget, WindowControlLabels,
        WindowControlLayout, readback_xrgb,
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
        Window::default_attributes().with_title("alo paged native name check"),
    )?;
    if !matches!(
        backend.window().display_handle()?.as_raw(),
        RawDisplayHandle::Wayland(_)
    ) {
        return Err("requires a Wayland parent".into());
    }
    let (renderer, _) = backend.bind()?;
    let mut buffer: GlesRenderbuffer =
        renderer.create_buffer(DrmFourcc::Abgr8888, (320, 180).into())?;
    let mut target = renderer.bind(&mut buffer)?;
    let vocabulary = shortcut_words()?;
    let german = Language::written("de")?;
    let translation = vocabulary
        .check(Translation::into_language(german.clone()).says(
            Action::CloseWindow.word().key(),
            "Schließen\nΚλείσιμο\nЗатваряне",
        ))
        .map_err(|error| error.to_string())?;
    let mut translated = Strings::of(vocabulary);
    translated.speaks(translation)?;
    translated.prefers(&[german]);
    let fallback = Strings::of(shortcut_words()?);
    let mut shaper = WindowControlLabels::new()?;
    let layout = WindowControlLayout::new((320, 180), (0, 0), [false; 3], false)?;
    let mut frames = 0;
    for strings in [&translated, &fallback] {
        for scheme in [Scheme::Light, Scheme::Dark] {
            for percent in [100, 125, 200, 300] {
                let selected = WindowControlLabelTarget {
                    control: layout.controls()[2],
                    geometry: LabelGeometry {
                        viewport: (320, 180),
                        origin: (8, 40),
                        size: (240, 20 * percent as i32 / 100 + 8),
                    },
                };
                let pages = shaper.prepare_pages(
                    selected,
                    &layout,
                    strings,
                    scheme,
                    TextScale::percent(percent).map_err(|_| "invalid fixture scale")?,
                )?;
                assert_eq!(pages.said(), &selected.control.action().said(strings));
                assert!(!pages.pages().is_empty());
                // Revisit in reverse too: no page paint may leave the preceding raster.
                for page in pages.pages().iter().chain(pages.pages().iter().rev()) {
                    let mut frame =
                        renderer.render(&mut target, (320, 180).into(), Transform::Normal)?;
                    frame.clear(
                        Color32F::new(0., 0., 0., 1.),
                        &[Rectangle::from_size((320, 180).into())],
                    )?;
                    page.paint(&mut frame)?;
                    let _sync = frame.finish()?;
                    let readback = readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
                    let (pixels, remainder) = readback.pixels().as_chunks::<4>();
                    assert!(remainder.is_empty());
                    assert_eq!(pixels.len(), 320 * 180);
                    for (index, pixel) in pixels.iter().enumerate() {
                        let bounds = page.bounds();
                        let x = index as i32 % 320 - bounds.loc.x;
                        let y = index as i32 / 320 - bounds.loc.y;
                        let expected =
                            if (0..bounds.size.w).contains(&x) && (0..bounds.size.h).contains(&y) {
                                let rgba = *page
                                    .pixels()
                                    .get((y * bounds.size.w + x) as usize)
                                    .ok_or("incomplete page raster")?;
                                [rgba[2], rgba[1], rgba[0], 0]
                            } else {
                                [0; 4]
                            };
                        assert_eq!(*pixel, expected, "page frame {frames}, pixel {index}");
                    }
                    frames += 1;
                }
            }
        }
    }
    assert!(frames >= 64);
    println!(
        "Native name pages: {frames} complete 57,600-pixel GLES frames, forward/reverse, translation/fallback, light/dark, four scales"
    );
    Ok(())
}
