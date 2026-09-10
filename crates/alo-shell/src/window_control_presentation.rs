//! One mapping-bound native strip and its explicit label focus, owned by the host.
use std::sync::Arc;

use alo_shortcuts::Action;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    PaintedWindowControls, Server, WindowControlLabelError, WindowControlLabelSelection,
    WindowControlLabelTarget, WindowControlPointerEvent, WindowControlRoute,
    WindowControlRouteError, WindowControlSnapshot, WindowControlSnapshotError,
};

/// Never exported as an authority token or transferable to another display.
#[derive(Clone)]
pub(crate) struct Presentation {
    /// Unique publication lifetime, including removal and identical republication.
    generation: Arc<()>,
    /// Exact root explicitly composed by this display's host.
    surface: WlSurface,
    /// Identity changes on every hide or mapping retirement, not just destruction.
    visibility: Arc<()>,
    /// Scale-one output bounds used for the composed strip.
    viewport: (i32, i32),
    /// Output-local strip origin; movement retires held authority.
    origin: (i32, i32),
    /// Presented maximize/restore intent, never a late toggle.
    restoring: bool,
    /// Explicit native label focus; no client keyboard authority.
    focus: Option<Action>,
    /// Every explicit focus request retires pending name-opening gestures.
    selection: Arc<()>,
}

impl Presentation {
    /// Borrow geometry for existing selectors and routing after live validation.
    fn painted(&self) -> PaintedWindowControls<'_> {
        PaintedWindowControls {
            surface: &self.surface,
            viewport: self.viewport,
            origin: self.origin,
        }
    }

    /// Only identical live presentation may keep focus and held execution.
    fn same_frame(&self, other: &Self) -> bool {
        self.surface == other.surface
            && Arc::ptr_eq(&self.visibility, &other.visibility)
            && self.viewport == other.viewport
            && self.origin == other.origin
            && self.restoring == other.restoring
    }
}

impl Server {
    /// Native focus identity, distinct from client focus and hover selection.
    pub(crate) fn control_name_focus(&mut self) -> Option<(Action, Arc<()>)> {
        self.control_reader_binding()?;
        let view = self.control_presentation.as_ref()?;
        Some((view.focus?, view.selection.clone()))
    }

    /// Fresh explicit reader selection without changing native or client focus.
    pub(crate) fn control_reader_target(
        &mut self,
        action: Action,
        size: (i32, i32),
    ) -> Result<Option<WindowControlLabelTarget>, WindowControlLabelError> {
        let Some(view) = self.live_window_controls() else {
            return Ok(None);
        };
        self.window_control_label_target(
            Some(view.painted()),
            WindowControlLabelSelection::Focus(action),
            size,
        )
    }

    /// Validated reader binding; competing gestures refuse reader access.
    pub(crate) fn control_reader_binding(&mut self) -> Option<Arc<()>> {
        self.live_window_controls()?;
        if self.window_control_input_busy()
            || self.control_press.is_some()
            || self.control_overlay.held()
        {
            return None;
        }
        Some(self.control_presentation.as_ref()?.generation.clone())
    }

    /// Publish one explicitly painted strip after successful host composition.
    ///
    /// This records presentation, never paints it or chooses a focused window.
    /// Supply None on removal, failed submission or backend deactivation. A failed
    /// candidate retires the previous strip too. Replacement, relayout, changed
    /// maximize/restore intent and hide/remap clear native label focus and cancel
    /// held execution while preserving ownership of the matching release.
    /// Repeating the same live frame preserves focus and a held gesture.
    ///
    /// Do not use a retained surface handle to assert that an old frame is current:
    /// the host must have composed this mapping and geometry. Until a backend uses
    /// this lifecycle API, its existing low-level control methods remain explicit.
    pub fn present_window_controls(
        &mut self,
        painted: Option<PaintedWindowControls<'_>>,
    ) -> Result<(), WindowControlSnapshotError> {
        let Some(view) = painted else {
            self.retire_window_controls();
            return Ok(());
        };
        let candidate = (|| {
            let snapshot =
                self.window_control_snapshot(view.surface, view.viewport, view.origin)?;
            Ok::<_, WindowControlSnapshotError>(Presentation {
                generation: Arc::new(()),
                surface: view.surface.clone(),
                visibility: self
                    .surfaces
                    .window_visibility(view.surface)
                    .ok_or(WindowControlSnapshotError::Unmapped)?,
                viewport: view.viewport,
                origin: view.origin,
                restoring: snapshot.layout().restoring(),
                focus: None,
                selection: Arc::new(()),
            })
        })();
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(error) => {
                self.retire_window_controls();
                return Err(error);
            }
        };
        if self
            .control_presentation
            .as_ref()
            .is_none_or(|old| !old.same_frame(&candidate))
        {
            self.retire_window_controls();
            self.control_presentation = Some(candidate);
        }
        Ok(())
    }

    /// Remove native presentation and focus without forwarding an owned release.
    /// Idempotent, including on seats lacking pointer or keyboard capability.
    pub fn retire_window_controls(&mut self) {
        self.reader_presentation = None;
        self.control_overlay.bounds = None;
        self.control_presentation = None;
        self.cancel_window_control();
    }

    /// Select explicit native label focus within the current visible strip.
    ///
    /// Disabled names are eligible; non-strip or completely clipped actions refuse.
    /// None or a refusal clears old focus. This never changes client keyboard focus
    /// or dispatches an operation. Host keyboard navigation is separate integration.
    pub fn focus_window_control(&mut self, action: Option<Action>) -> bool {
        if let Some(view) = &mut self.control_presentation {
            view.focus = None;
            view.selection = Arc::new(());
        }
        let Some(view) = self.live_window_controls() else {
            return false;
        };
        if self.window_control_input_busy()
            || self.control_press.is_some()
            || self.control_overlay.held()
        {
            return false;
        }
        let Ok(snapshot) = self.window_control_snapshot(&view.surface, view.viewport, view.origin)
        else {
            return false;
        };
        let selected = action.filter(|action| {
            snapshot.layout().controls().iter().any(|control| {
                control.action() == *action
                    && control
                        .bounds()
                        .intersection(snapshot.layout().viewport)
                        .is_some()
            })
        });
        if let Some(view) = &mut self.control_presentation {
            view.focus = selected;
        }
        selected.is_some()
    }

    /// Fresh feedback for the published mapping; stale presentation retires itself.
    /// The returned snapshot remains a frozen view, never permission to execute.
    pub fn presented_window_controls(
        &mut self,
        position: Option<(f64, f64)>,
    ) -> Option<WindowControlSnapshot> {
        let view = self.live_window_controls()?;
        self.window_control_feedback(&view.surface, view.viewport, view.origin, position)
            .ok()
    }

    /// Select a label using mapping-bound native focus, otherwise fresh hover.
    ///
    /// None/error means discard the old label. No hover position is retained.
    /// Competing ownership clears native focus, so ending a grab cannot restore it.
    /// Overlay hit policy and full clipped-text access still belong to composition.
    pub fn presented_window_control_label(
        &mut self,
        hover: Option<(f64, f64)>,
        size: (i32, i32),
    ) -> Result<Option<WindowControlLabelTarget>, WindowControlLabelError> {
        let Some(view) = self.live_window_controls() else {
            return Ok(None);
        };
        let selection = view
            .focus
            .map(WindowControlLabelSelection::Focus)
            .unwrap_or_else(|| {
                hover
                    .map(|(x, y)| WindowControlLabelSelection::Pointer(x, y))
                    .unwrap_or(WindowControlLabelSelection::Dismissed)
            });
        self.window_control_label_target(Some(view.painted()), selection, size)
    }

    /// Route once against the published mapping, retiring stale authority first.
    ///
    /// Pointer events dismiss native label focus. Actual position/time remain
    /// required for every event; this does not cache motion or pump a backend.
    /// Absent presentation uses ordinary client fallback; cancelled native releases
    /// remain consumed by the existing transaction router, even after retirement.
    pub fn route_presented_window_control_pointer(
        &mut self,
        position: (f64, f64),
        event: WindowControlPointerEvent,
        time: u32,
    ) -> Result<WindowControlRoute, WindowControlRouteError> {
        if let Some(view) = &mut self.control_presentation {
            view.focus = None;
        }
        let view = self.live_window_controls();
        if self.route_control_overlay(position, event)? {
            return Ok(WindowControlRoute::Consumed);
        }
        self.route_window_control_pointer(
            view.as_ref().map(Presentation::painted),
            position,
            event,
            time,
        )
    }

    /// Route a scroll frame once, consuming opaque-label hits and owned gestures.
    /// Existing client grabs retain priority. Returns false when not delivered.
    pub fn route_presented_window_control_axis(
        &mut self,
        position: (f64, f64),
        frame: smithay::input::pointer::AxisFrame,
    ) -> Result<bool, crate::InputError> {
        if !crate::pointer::bounded(position.0)
            || !crate::pointer::bounded(position.1)
            || !crate::pointer::bounded(frame.axis.0)
            || !crate::pointer::bounded(frame.axis.1)
        {
            return Err(crate::InputError::InvalidPointer);
        }
        self.focus_window_control(None);
        self.live_window_controls();
        if self.control_overlay_axis(position)? {
            return Ok(false);
        }
        self.pointer_axis(frame)
    }

    /// Revalidate even when no backend event observed an intervening hide/remap.
    fn live_window_controls(&mut self) -> Option<Presentation> {
        let view = self.control_presentation.as_ref()?;
        let current = self
            .surfaces
            .window_visibility(&view.surface)
            .is_some_and(|visibility| Arc::ptr_eq(&visibility, &view.visibility))
            && self
                .window_control_snapshot(&view.surface, view.viewport, view.origin)
                .is_ok_and(|snapshot| snapshot.layout().restoring() == view.restoring);
        if !current {
            self.retire_window_controls();
            return None;
        }
        if (self.window_control_input_busy()
            || self.control_press.is_some()
            || self.control_overlay.held())
            && let Some(view) = &mut self.control_presentation
        {
            view.focus = None;
            view.generation = Arc::new(());
        }
        self.control_presentation.clone()
    }

    /// Focus survives only an identical live candidate; never carry it to a new root.
    pub(crate) fn control_frame_focus(
        &mut self,
        painted: &PaintedWindowControls<'_>,
    ) -> Option<Action> {
        let old = self.live_window_controls()?;
        (old.surface == *painted.surface
            && old.viewport == painted.viewport
            && old.origin == painted.origin)
            .then_some(old.focus)
            .flatten()
    }
}
