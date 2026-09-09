//! Live reader composition, submission and publication without intervening dispatch.
use crate::window_control_reader_interaction::ReaderHitMap;
use crate::{
    FrameTarget, LabelGeometry, ReaderPointerHit, RenderError, Server, WindowControlLabels,
    WindowControlReader, WindowControlReaderInteraction, WindowControlReaderPointer,
    WindowControlReaderScene,
};
use alo_strings::Strings;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::sync::Arc;

/// Per-frame navigation preparation; page appearance remains fixed by the reader.
pub struct WindowControlReaderFrame<'a> {
    /// Explicit complete navigation vocabulary and source fallback policy.
    pub strings: &'a Strings,
    /// Reusable private shaper, using the bundled font.
    pub labels: &'a mut WindowControlLabels,
    /// Output-contained navigation capacity with room for feedback gutters.
    pub chrome: LabelGeometry,
    /// Live semantic feedback owner; cancellation keeps matching releases owned.
    pub pointer: &'a mut WindowControlReaderPointer,
}

/// Exact reader/page visit and painted geometry, stored only after submission.
pub(crate) struct ReaderPresentation {
    /// Exact reading session, distinct even for the same name.
    reader: Arc<()>,
    /// Exact page visit, including away-and-back transitions.
    selection: Arc<()>,
    /// Opaque areas from this successful composition.
    hits: ReaderHitMap,
}

/// Adapt one fresh reader to the existing callback/output transaction.
struct ReaderTarget<'a, T> {
    /// Actual backend, which must explicitly support this scene.
    target: &'a mut T,
    /// Complete native scene for this transaction only.
    scene: &'a WindowControlReaderScene<'a>,
    /// Live strip root that must be included in the submitted surface set.
    surface: &'a WlSurface,
}

impl<T: FrameTarget> FrameTarget for ReaderTarget<'_, T> {
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
        self.scene.validate(self.target.size())?;
        let submitted = self
            .target
            .submit_reader(roots, popups, cursor, self.scene)?;
        if !submitted.contains(self.surface) {
            return Err(RenderError::ControlTargetOmitted);
        }
        Ok(submitted)
    }
}

impl Server {
    /// Submit the selected live page, complete chrome, feedback and its original
    /// strip in one desktop frame. No dispatch occurs between preparation and
    /// publication. The backend must support readers explicitly; omitted roots
    /// and all errors preserve pending callbacks and retire old native authority.
    /// A failed reader is dismissed permanently; reopen explicitly after recovery.
    /// Success means backend submission, never physical presentation.
    ///
    /// Pump input first. This explicit host API installs no backend input filter.
    /// Hosts must cancel key/pointer ownership on removal or focus/seat/session
    /// loss, and drain releases while inactive. Ordinary control rendering removes
    /// reader publication; explicit strip retirement removes it as well.
    pub fn render_window_control_reader(
        &mut self,
        target: &mut impl FrameTarget,
        reader: &mut WindowControlReader,
        frame: WindowControlReaderFrame<'_>,
        time: u32,
    ) -> Result<usize, RenderError> {
        let previous = self.reader_presentation.take();
        let result = (|| {
            let feedback = frame.pointer.feedback(self, Some(reader));
            let identity = reader.identity.clone();
            let selection = reader.selection.clone();
            let style = reader.style;
            let index = reader.selected();
            let page = self
                .read_window_control_page(reader, index)
                .ok_or(RenderError::ControlScene)?;
            let snapshot = self
                .presented_window_controls(None)
                .ok_or(RenderError::ControlScene)?;
            let chrome = page.chrome(frame.strings)?.prepare(
                frame.labels,
                snapshot.layout(),
                page.page,
                frame.chrome,
                style.scheme,
                style.scale,
            )?;
            let mut scene = WindowControlReaderScene {
                layout: snapshot.layout(),
                interaction: WindowControlReaderInteraction::new(
                    page.page,
                    &chrome,
                    snapshot.layout(),
                )?,
                feedback,
                scheme: style.scheme,
            };
            let hits = scene.interaction.hit_map();
            if !previous.as_ref().is_some_and(|old| {
                Arc::ptr_eq(&old.reader, &identity)
                    && Arc::ptr_eq(&old.selection, &selection)
                    && old.hits == hits
            }) {
                frame.pointer.cancel();
                scene.feedback = crate::ReaderPointerFeedback::default();
            }
            scene.validate(target.size())?;
            let submitted = self.render_frame(
                &mut ReaderTarget {
                    target,
                    scene: &scene,
                    surface: snapshot.surface(),
                },
                time,
            )?;
            self.control_overlay.bounds = None;
            self.reader_presentation = Some(ReaderPresentation {
                reader: identity,
                selection,
                hits,
            });
            Ok(submitted)
        })();
        if result.is_err() {
            frame.pointer.cancel();
            reader.dismiss();
            self.retire_window_controls();
        }
        result
    }

    /// Whether this exact live reader/page visit has a successfully submitted frame.
    /// Stale identity, navigation (including away-and-back), removal or competing
    /// input retires the saved publication. Frozen geometry alone cannot revive it.
    pub fn window_control_reader_presented(&mut self, reader: &mut WindowControlReader) -> bool {
        let index = reader.selected();
        let live = self.read_window_control_page(reader, index).is_some();
        let matches = live
            && self.reader_presentation.as_ref().is_some_and(|view| {
                Arc::ptr_eq(&view.reader, &reader.identity)
                    && Arc::ptr_eq(&view.selection, &reader.selection)
            });
        if !matches {
            self.reader_presentation = None;
        }
        matches
    }

    /// Hit only the exact successfully submitted, still-live reader page.
    /// Call on every event; never cache this result across host events. The host
    /// still routes ownership through its semantic pointer/key transactions.
    pub fn presented_window_control_reader_hit(
        &mut self,
        reader: &mut WindowControlReader,
        position: (f64, f64),
    ) -> Option<ReaderPointerHit> {
        if !self.window_control_reader_presented(reader) {
            return None;
        }
        self.reader_presentation.as_ref()?.hits.hit(position)
    }
}
