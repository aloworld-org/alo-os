//! Full-frame GLES acceptance for the native control view, not desktop input.

#[cfg(target_os = "linux")]
#[path = "support/window_controls_pixels.rs"]
mod window_controls_pixels;

/// Report graphics failure to the development gate.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("window control graphics check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// The native graphical fixture requires Linux.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Compare every pixel to independently specified original glyph masks.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::Scheme;
    use alo_shell::{RowOrder, WindowControlLayout, readback_xrgb};
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
        Window::default_attributes().with_title("alo window control view check"),
    )?;
    if !matches!(
        backend.window().display_handle()?.as_raw(),
        RawDisplayHandle::Wayland(_)
    ) {
        return Err("requires a Wayland parent".into());
    }
    let (renderer, _window) = backend.bind()?;
    let mut buffer: GlesRenderbuffer =
        renderer.create_buffer(DrmFourcc::Abgr8888, (120, 48).into())?;
    let mut target = renderer.bind(&mut buffer)?;
    let mut frames = 0;
    for scheme in [Scheme::Light, Scheme::Dark] {
        for restoring in [false, true] {
            for mask in 0_u8..8 {
                let enabled = [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0];
                for origin in [(3, 4), (-5, -7), (100, 35), (120, 48)] {
                    for interaction in [
                        None,
                        Some((0, false)),
                        Some((1, false)),
                        Some((2, false)),
                        Some((0, true)),
                        Some((1, true)),
                        Some((2, true)),
                    ] {
                        let position = interaction.map(|(slot, _)| {
                            (
                                f64::from(origin.0 + slot * 36 + 10),
                                f64::from(origin.1 + 10),
                            )
                        });
                        let active = interaction
                            .filter(|_| {
                                position.is_some_and(|(x, y)| {
                                    (0.0..120.0).contains(&x) && (0.0..48.0).contains(&y)
                                })
                            })
                            .map(|(slot, pressed)| (usize::try_from(slot).unwrap_or(3), pressed));
                        let layout =
                            WindowControlLayout::new((120, 48), origin, enabled, restoring)?
                                .with_pointer_feedback(
                                    position,
                                    interaction.is_some_and(|(_, pressed)| pressed),
                                );
                        let mut frame =
                            renderer.render(&mut target, (120, 48).into(), Transform::Normal)?;
                        frame.clear(
                            Color32F::new(0.0, 0.0, 0.0, 1.0),
                            &[Rectangle::from_size((120, 48).into())],
                        )?;
                        layout.paint(&mut frame, scheme)?;
                        let _sync = frame.finish()?;
                        let pixels = readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
                        assert_eq!(pixels.size(), (120, 48));
                        assert_eq!(pixels.pixels().len(), 120 * 48 * 4);
                        let (rows, remainder) = pixels.pixels().as_chunks::<4>();
                        assert!(remainder.is_empty());
                        for (index, pixel) in rows.iter().enumerate() {
                            let x = i32::try_from(index % 120)?;
                            let y = i32::try_from(index / 120)?;
                            let expected = if interaction.is_none() {
                                window_controls_pixels::expected(
                                    (x, y),
                                    origin,
                                    scheme,
                                    enabled,
                                    restoring,
                                )
                            } else {
                                window_controls_pixels::expected_feedback(
                                    (x, y),
                                    origin,
                                    scheme,
                                    enabled,
                                    restoring,
                                    active,
                                )
                            };
                            assert_eq!(*pixel, expected, "frame {frames} at {x},{y}");
                            // Painted grounds are never black, so this independently
                            // checks every paint/hit boundary, including disabled hits.
                            assert_eq!(
                                layout.hit(f64::from(x) + 0.5, f64::from(y) + 0.5).is_some(),
                                expected != [0; 4]
                            );
                        }
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 896);
    println!(
        "Native window controls: 896 full 5,760-pixel GLES frames passed (idle/hover/pressed, light/dark, all availability combinations, maximize/restore, four clipping positions); view-only evidence"
    );
    Ok(())
}
