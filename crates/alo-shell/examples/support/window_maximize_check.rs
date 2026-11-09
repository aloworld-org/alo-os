//! Full-frame GLES checks through client maximize/restore requests.
use alo_shell::{Cursor, FrameTarget, RenderError, Server, render_scanout};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

/// How tall a window's band is at scale 1: `crate::window_edge::THE_REGION_IS_TALL`
/// in the shell, 54 in every frame of the design. Maximise keeps the band on the
/// screen, so the window goes this far below the top of the work area.
const THE_BAND: i32 = 54;

/// The window a maximise gives here: the display's whole width, and its height
/// less the band. 33x32 is the output-sized buffer the client commits
/// (`attach_maximized`), so the display is made exactly that plus the band.
const MAXIMISED: (i32, i32) = (33, 32);

/// The display the maximise stages present to: tall enough for the band.
///
/// Until maximise kept the band the probe's only display was 33x32, and a
/// maximised window filled it from the origin. Now a work area 32 tall holds no
/// band and anything under it, so maximise is refused there — which is
/// intended, and `tests/window_controls` holds that refusal. This display
/// leaves exactly the 33x32 under a 54-tall band.
const DISPLAY: (i32, i32) = (MAXIMISED.0, MAXIMISED.1 + THE_BAND);

/// The white 32x24 window the interactive resize stages leave at (20, 20).
const RESTORED: ((i32, i32), (i32, i32)) = ((20, 20), (32, 24));

/// Fixture-only sink of the taller display: it prepares the real scene at its
/// size and checks every pixel, and never claims a DRM commit.
struct ATallerDisplay<'a> {
    renderer: &'a mut GlesRenderer,
}

impl FrameTarget for ATallerDisplay<'_> {
    fn size(&self) -> Size<i32, Physical> {
        DISPLAY.into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &Cursor::Hidden)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[alo_shell::Popup],
        _cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        // The pointer is not what this frame is evidence of; stages 1, 4, 6
        // and 7 hold the cursor's pixels.
        let prepared = render_scanout(self.renderer, DISPLAY.into(), roots, popups, &Cursor::Hidden)?;
        let ((x0, y0), (w, h)) = RESTORED;
        for (index, pixel) in prepared.pixels().pixels().as_chunks::<4>().0.iter().enumerate() {
            let x = (index % DISPLAY.0 as usize) as i32;
            let y = (index / DISPLAY.0 as usize) as i32;
            let inside = (x0..x0 + w).contains(&x) && (y0..y0 + h).contains(&y);
            let expected = if inside { [255, 255, 255, 0] } else { [0; 4] };
            assert_eq!(*pixel, expected, "taller display pixel ({x},{y})");
        }
        Ok(prepared.into_parts().1)
    }
}

/// Present one frame to [`DISPLAY`], so the maximise the client asks for next
/// is sized from a display with room for the band.
pub fn a_display_with_room_for_the_band(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let presented = server.render(&mut ATallerDisplay { renderer }, 78)?;
    assert_eq!(presented, 1, "the one window is presented to the taller display");
    Ok(())
}

/// Check the entire actual client buffer and unchanged background at each boundary.
pub fn stage(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    stage: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots.first().ok_or("maximize root missing")?;
    let controls = server.window_control_snapshot(root, (120, 48), (3, 4))?;
    assert_eq!(controls.surface(), root);
    assert_eq!(controls.layout().restoring(), stage <= 18);
    assert_eq!(controls.maximize_refusal(), None);
    assert!(
        controls
            .layout()
            .controls()
            .iter()
            .all(|control| control.enabled())
    );
    crate::window_control_snapshot_check::paint(renderer, controls.layout(), stage <= 18)?;
    // Maximised: the top-left of the work area, which has nothing reserved
    // on this display, moved down by the band; the client's own buffer, no scaling.
    let maximised = ((0, THE_BAND), MAXIMISED);
    let (origin, size) = match stage {
        17 => RESTORED,
        18 => RESTORED, // Acknowledgement has no pixel effect.
        19 => maximised,
        20 => maximised, // Restore acknowledgement also waits for commit.
        21 => RESTORED,
        _ => return Err("unknown maximize stage".into()),
    };
    assert_eq!(
        alo_shell::window_buffer_origin(root),
        (f64::from(origin.0), f64::from(origin.1)).into()
    );
    let prepared = render_scanout(
        renderer,
        (80, 80).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    for (index, pixel) in prepared
        .pixels()
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let x = (index % 80) as i32;
        let y = (index / 80) as i32;
        let expected = if (origin.0..origin.0 + size.0).contains(&x)
            && (origin.1..origin.1 + size.1).contains(&y)
        {
            [255, 255, 255, 0]
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "maximize stage {stage} pixel {index}");
    }
    println!("Maximize/restore stage {stage}: all 6400 GLES pixels passed");
    Ok(())
}
