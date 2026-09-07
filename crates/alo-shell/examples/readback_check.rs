//! Real WSLg GLES offscreen pixels; not successful DRM or hardware evidence.

/// Return an explicit failure if any graphics or pixel check fails.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("scanout readback check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "linux"))]
/// Refuse platforms without the native GLES backend.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

#[cfg(target_os = "linux")]
/// Verify offscreen rendering and readback with an actual Wayland EGL context.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{RowOrder, readback_xrgb};
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
    // Main-thread event loop retained until all graphics resources are dropped.
    let (mut backend, _events) = winit::init_from_attributes::<GlesRenderer>(
        Window::default_attributes().with_title("alo scanout readback check"),
    )?;
    if !matches!(
        backend.window().display_handle()?.as_raw(),
        RawDisplayHandle::Wayland(_)
    ) {
        return Err("requires a Wayland parent".into());
    }
    let (renderer, _window) = backend.bind()?;
    // Odd width checks packing; asymmetric rows and columns detect reversal.
    let size = (3, 2);
    let mut buffer: GlesRenderbuffer = renderer.create_buffer(DrmFourcc::Abgr8888, size.into())?;
    let mut target = renderer.bind(&mut buffer)?;
    let pattern = [
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [0, 255, 255],
        [255, 0, 255],
    ];
    for (transform, order) in [
        (Transform::Normal, RowOrder::TopToBottom),
        (Transform::Flipped180, RowOrder::BottomToTop),
    ] {
        let mut frame = renderer.render(&mut target, size.into(), transform)?;
        frame.clear(
            Color32F::new(0.0, 0.0, 0.0, 1.0),
            &[Rectangle::from_size(size.into())],
        )?;
        for (index, rgb) in pattern.iter().enumerate() {
            frame.draw_solid(
                Rectangle::new(
                    ((index % 3) as i32, (index / 3) as i32).into(),
                    (1, 1).into(),
                ),
                &[Rectangle::from_size((1, 1).into())],
                Color32F::new(
                    rgb[0] as f32 / 255.0,
                    rgb[1] as f32 / 255.0,
                    rgb[2] as f32 / 255.0,
                    1.0,
                ),
            )?;
        }
        let _sync = frame.finish()?;
        let result = readback_xrgb(renderer, &target, order)?;
        let expected: Vec<u8> = pattern
            .iter()
            .flat_map(|rgb| [rgb[2], rgb[1], rgb[0], 0])
            .collect();
        if result.pixels() != expected {
            return Err(format!("{transform:?} pixel mismatch: {:?}", result.pixels()).into());
        }
        result.frame()?;
        println!(
            "{transform:?}: six exact XRGB pixels; channels, odd-width packing and orientation verified"
        );
    }
    Ok(())
}
