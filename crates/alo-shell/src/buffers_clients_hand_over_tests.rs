//! Whether the loop asks, and whether the renderer answers from itself.
use super::*;
use crate::{
    FrameTarget, RenderError, direct_loop::LoopTarget, direct_target::ScenePainter,
    scene_native::NativeLayers,
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::{cell::RefCell, os::unix::fs::PermissionsExt, rc::Rc};

type Log = Rc<RefCell<Vec<&'static str>>>;

/// A target that records being asked, and refuses to draw this shell's screens.
///
/// It deliberately does not override `validate_handed_buffers`, so what is
/// recorded is the loop reaching for the buffers and not a fake pretending to
/// import. The default body runs underneath, which with an empty queue does
/// nothing.
struct Asked {
    log: Log,
}
impl FrameTarget for Asked {
    fn size(&self) -> Size<i32, Physical> {
        (32, 32).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.log.borrow_mut().push("drew");
        Ok(vec![])
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
}
impl crate::presentation::NativeTarget for Asked {}
impl LoopTarget for Asked {
    fn check(&self) -> Result<(), RenderError> {
        Ok(())
    }
    fn validate_handed_buffers(
        &mut self,
        handed: Vec<crate::buffers_clients_hand_over::HandedOver>,
    ) {
        self.log.borrow_mut().push("asked for buffers");
        for held in handed {
            held.failed();
        }
    }
}

struct Quiet;
impl crate::direct_input_loop::LoopInput for Quiet {
    fn dispatch(
        &mut self,
        _: &mut Server,
        _: &mut dyn FnMut() -> Result<(), crate::SessionError>,
    ) -> Result<(), crate::DirectLoopError> {
        Ok(())
    }
    fn shutdown(self, _: &mut Server) -> Option<std::io::Result<()>> {
        Some(Ok(()))
    }
}

/// **The seam.** Delete the drain in `crate::direct_loop::run_with_input` and
/// this fails; nothing else in the suite does, because every other test about
/// held buffers can hand them over itself.
///
/// It also fixes the order, which is the half that matters: a client is answered
/// *before* the frame is drawn, so a buffer is never discovered to be unusable
/// by a frame that already used it.
#[test]
fn the_loop_asks_for_handed_buffers_before_it_draws() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind(dir.path(), "handed-buffers")?;
    let log = Log::default();
    let mut steps = [DirectFrame::Render(1), DirectFrame::Stop].into_iter();
    let result = crate::direct_loop::run_with_input(
        &mut server,
        Asked { log: log.clone() },
        &mut || Ok(()),
        &mut || steps.next().unwrap_or(DirectFrame::Stop),
        Quiet,
    );
    assert!(
        result.outcome.is_ok(),
        "the loop refused: {:?}",
        result.outcome
    );
    assert_eq!(
        log.borrow().as_slice(),
        ["asked for buffers", "drew"],
        "the loop must ask about handed buffers, and ask before it draws"
    );
    Ok(())
}

/// A painter that paints and imports nothing, for the default half of the
/// narrow interface.
struct NoImporter;
impl ScenePainter for NoImporter {
    fn paint(
        &mut self,
        _: Size<i32, Physical>,
        _: &[WlSurface],
        _: &[crate::Popup],
        _: &crate::Cursor,
        _: NativeLayers<'_>,
        _: alo_canvas::Camera,
    ) -> Result<(crate::ScanoutPixels, Vec<WlSurface>), RenderError> {
        Err(RenderError::Submission("this painter draws nothing".into()))
    }
}

/// A painter that has not overridden the interface advertises nothing.
///
/// Which is what keeps its refusal unreachable rather than merely unlikely:
/// `crate::surfaces::Surfaces::advertise_importable_buffers` puts up no global
/// for an empty set, so no client is ever in a position to offer such a painter
/// a buffer.
///
/// **The refusal itself is not exercised here and cannot be.** `Dmabuf` has no
/// public constructor — one arrives from a client — so the only honest way to
/// reach `validate_import`'s default body is a real client offering a real
/// buffer, which belongs to the interoperability pass and not to this suite.
#[test]
fn a_painter_with_no_importer_advertises_nothing() {
    assert!(
        NoImporter.importable_formats().iter().next().is_none(),
        "a painter with no importer must advertise no format at all"
    );
}

/// **The software renderer answers from itself, not from a list written here.**
///
/// This is the measurement that settles what a note of this lane's got wrong:
/// buffer sharing was recorded as something that would have to be invented,
/// because `crate::software_scanout`'s header said this renderer cannot import.
/// pixman implements `ImportDma`, and the formats below come out of it.
///
/// The count is deliberately not asserted. Which formats pixman offers is
/// pixman's business and may change when the pin moves; that there is at least
/// one, and that it was asked rather than assumed, is this shell's business.
#[test]
fn the_software_renderer_says_what_it_can_import() -> Result<(), Box<dyn std::error::Error>> {
    let painter = crate::software_scanout::SoftwarePainter::new()?;
    let formats: Vec<_> = painter.importable_formats().iter().copied().collect();
    assert!(
        !formats.is_empty(),
        "pixman implements ImportDma, so it must name at least one format it can take"
    );
    Ok(())
}
