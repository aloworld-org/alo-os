//! Fresh native strip submission and mapping-bound input publication.

use alo_appearance::{Scheme, TextScale};
use alo_strings::Strings;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

use crate::{FrameTarget, PaintedWindowControls, RenderError, Server, WindowControlScene};

/// Trusted host selection for one frame; no saved layout or authority token.
/// The viewport comes from the actual target; optional labels use the same frame.
pub struct WindowControlFrame<'a> {
    /// Explicit live visible root, never inferred from client keyboard focus.
    pub surface: &'a WlSurface,
    /// Scale-one output-local origin for the strip.
    pub origin: (i32, i32),
    /// Current backend pointer position, absent after input loss.
    pub position: Option<(f64, f64)>,
    /// Existing appearance tokens.
    pub scheme: Scheme,
}

/// Person-owned text settings and reusable shaper for a fresh label each frame.
pub struct WindowControlLabelFrame<'a> {
    /// Explicit vocabulary, including its source fallback.
    pub strings: &'a Strings,
    /// Private native font/shaping state.
    pub labels: &'a mut crate::WindowControlLabels,
    /// Requested label box; incomplete text refuses submission.
    pub size: (i32, i32),
    /// Existing person-selected text scale.
    pub scale: TextScale,
}

/// Borrow the native scene for exactly one ordinary presentation transaction.
struct ControlTarget<'a, T> {
    /// Backend whose successful submission controls callback publication.
    target: &'a mut T,
    /// Fresh scene, or explicit removal.
    scene: Option<WindowControlScene<'a>>,
    /// A strip cannot authorize a root the backend omitted from the frame.
    surface: Option<&'a WlSurface>,
}

impl<T: FrameTarget> FrameTarget for ControlTarget<'_, T> {
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        self.target.metadata()
    }
    fn size(&self) -> Size<i32, Physical> {
        self.target.size()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &crate::Cursor::Default)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let submitted = self
            .target
            .submit_controls(roots, popups, cursor, self.scene)?;
        if self
            .surface
            .is_some_and(|surface| !submitted.contains(surface))
        {
            return Err(RenderError::ControlTargetOmitted);
        }
        Ok(submitted)
    }
}

impl Server {
    /// Submit fresh controls with the desktop, then publish their input lifetime.
    ///
    /// No dispatch occurs between snapshot, submission and publication. Validation
    /// and submission failures retire old native authority while preserving owned
    /// releases and pending client callbacks. None removes controls; ordinary
    /// `render` uses that path too. Unchanged successful frames preserve gestures
    /// and explicit label focus. Pump input first. Success means backend submission,
    /// never physical presentation. Labels/navigation/target selection are separate.
    pub fn render_window_controls(
        &mut self,
        target: &mut impl FrameTarget,
        controls: Option<WindowControlFrame<'_>>,
        time: u32,
    ) -> Result<usize, RenderError> {
        self.render_labeled_window_controls(target, controls, None, time)
    }

    /// Submit freshly selected/shaped labels with controls and publish their
    /// pointer exclusion only on success. Native focus is mapping-bound; otherwise
    /// current hover selects the name. Held gestures dismiss labels. Clipped text,
    /// overlapping controls, invalid geometry and shaping failures refuse the frame.
    /// None removes labels, retaining any owned button releases. No keyboard grab.
    pub fn render_labeled_window_controls(
        &mut self,
        target: &mut impl FrameTarget,
        controls: Option<WindowControlFrame<'_>>,
        labels: Option<WindowControlLabelFrame<'_>>,
        time: u32,
    ) -> Result<usize, RenderError> {
        let result = (|| {
            let Some(view) = controls else {
                self.retire_window_controls();
                return self.render_frame(
                    &mut ControlTarget {
                        target,
                        scene: None,
                        surface: None,
                    },
                    time,
                );
            };
            let size = target.size();
            let viewport = (size.w, size.h);
            let snapshot =
                self.window_control_feedback(view.surface, viewport, view.origin, view.position)?;
            let painted = PaintedWindowControls {
                surface: view.surface,
                viewport,
                origin: view.origin,
            };
            let focus = self.control_frame_focus(&painted);
            let label = if let Some(text) = labels {
                let selection = focus
                    .map(crate::WindowControlLabelSelection::Focus)
                    .unwrap_or_else(|| {
                        view.position
                            .map(|(x, y)| crate::WindowControlLabelSelection::Pointer(x, y))
                            .unwrap_or(crate::WindowControlLabelSelection::Dismissed)
                    });
                self.window_control_label_target(Some(painted), selection, text.size)?
                    .map(|selected| {
                        text.labels.prepare(
                            &selected.control,
                            text.strings,
                            selected.geometry,
                            view.scheme,
                            text.scale,
                        )
                    })
                    .transpose()?
            } else {
                None
            };
            let scene = WindowControlScene {
                layout: snapshot.layout(),
                label: label.as_ref(),
                scheme: view.scheme,
            };
            scene.validate(size)?;
            let submitted = self.render_frame(
                &mut ControlTarget {
                    target,
                    scene: Some(scene),
                    surface: Some(view.surface),
                },
                time,
            )?;
            self.present_window_controls(Some(PaintedWindowControls {
                surface: view.surface,
                viewport,
                origin: view.origin,
            }))?;
            self.control_overlay.bounds = label.as_ref().map(crate::WindowControlLabel::bounds);
            Ok(submitted)
        })();
        if result.is_err() {
            self.retire_window_controls();
        }
        result
    }
}

impl crate::Nested {
    /// Compose labels using this backend's fresh parent position and the server's
    /// mapping-bound native focus. Pump input first; failures retire authority.
    pub fn render_labeled_window_controls(
        &mut self,
        server: &mut Server,
        selected: Option<(&WlSurface, (i32, i32))>,
        scheme: Scheme,
        labels: Option<WindowControlLabelFrame<'_>>,
        time: u32,
    ) -> Result<usize, RenderError> {
        let position = self.control_position();
        server.render_labeled_window_controls(
            self,
            selected.map(|(surface, origin)| WindowControlFrame {
                surface,
                origin,
                position,
                scheme,
            }),
            labels,
            time,
        )
    }

    /// Submit an explicit native strip using the current parent pointer position.
    /// Pump input first. None removes controls; failures retire their authority.
    /// Labels require host overlay input policy and are not painted here.
    pub fn render_window_controls(
        &mut self,
        server: &mut Server,
        selected: Option<(&WlSurface, (i32, i32))>,
        scheme: Scheme,
        time: u32,
    ) -> Result<usize, RenderError> {
        let position = self.control_position();
        server.render_window_controls(
            self,
            selected.map(|(surface, origin)| WindowControlFrame {
                surface,
                origin,
                position,
                scheme,
            }),
            time,
        )
    }
}
