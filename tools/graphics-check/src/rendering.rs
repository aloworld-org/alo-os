//! A single nested GLES frame, using the unmodified pinned Smithay backend.

use smithay::backend::renderer::{Color32F, Frame, Renderer, gles::GlesRenderer};
use smithay::backend::winit;
use smithay::reexports::winit::platform::pump_events::PumpStatus;
use smithay::reexports::winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use smithay::reexports::winit::window::Window;
use smithay::utils::{Rectangle, Transform};

/// Initialize, clear, finish and submit once, then release the window and context.
pub(super) fn frame() -> Result<(), String> {
    let attributes = Window::default_attributes()
        .with_title("alo graphics development probe")
        .with_inner_size(smithay::reexports::winit::dpi::LogicalSize::new(320, 200));
    let (mut backend, mut events) = winit::init_from_attributes::<GlesRenderer>(attributes)
        .map_err(|error| format!("Smithay graphics initialization: {error:?}"))?;
    let display = backend
        .window()
        .display_handle()
        .map_err(|error| format!("cannot identify nested display: {error}"))?;
    if !matches!(display.as_raw(), RawDisplayHandle::Wayland(_)) {
        return Err("nested backend selected X11; a Wayland rendering check is required".into());
    }
    if let PumpStatus::Exit(code) = events.dispatch_new_events(|_| {}) {
        return Err(format!("nested display exited before rendering: {code}"));
    }
    let size = backend.window_size();
    if size.w <= 0 || size.h <= 0 {
        return Err("nested display has an empty framebuffer".into());
    }
    let damage = Rectangle::from_size(size);
    {
        let (renderer, mut framebuffer) = backend
            .bind()
            .map_err(|error| format!("bind nested framebuffer: {error}"))?;
        let mut frame = renderer
            .render(&mut framebuffer, size, Transform::Normal)
            .map_err(|error| format!("begin GLES frame: {error}"))?;
        frame
            .clear(Color32F::new(0.1, 0.2, 0.3, 1.0), &[damage])
            .map_err(|error| format!("clear GLES frame: {error}"))?;
        // The parent compositor synchronizes the submitted buffer, as in
        // Smithay's nested backend example; this is not direct scanout evidence.
        let _sync = frame
            .finish()
            .map_err(|error| format!("finish GLES frame: {error}"))?;
    }
    backend
        .submit(Some(&[damage]))
        .map_err(|error| format!("submit nested frame: {error}"))?;
    println!("Submitted framebuffer: {}x{}", size.w, size.h);
    Ok(())
}
