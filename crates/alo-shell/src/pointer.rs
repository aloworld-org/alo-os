//! Trusted pointer routing and drag lifetime; no agent input interface.

use crate::{InputError, Server, surfaces::Surfaces};
use smithay::{
    backend::{input::ButtonState, renderer::utils::with_renderer_surface_state},
    desktop::{WindowSurfaceType, utils::under_from_surface_tree},
    input::pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerHandle},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point, SERIAL_COUNTER, Serial},
    wayland::compositor::get_parent,
};

/// Backend bookkeeping complements Smithay's protocol and implicit click grab.
pub(crate) struct Pointer {
    /// Protocol handle on the existing keyboard seat.
    pub(crate) handle: PointerHandle<Surfaces>,
    /// Accepted buttons only; duplicates never reach Smithay's grab state.
    pub(crate) buttons: Vec<u32>,
    /// Last accepted location and timestamp for cancellation.
    pub(crate) location: Point<f64, Logical>,
    /// Timestamp used for synthetic cancellation events.
    time: u32,
    /// Scroll a canvas pan has not been able to spend yet, in screen pixels.
    ///
    /// See `crate::canvas_pan`: the plane is measured in whole units and a scroll
    /// arrives as a fraction, so a trackpad's tenths are kept here instead of
    /// being truncated away one event at a time.
    pub(crate) unspent_scroll: (f64, f64),
    /// Latest matched real release, valid only while its exact recipient keeps focus.
    pub(crate) popup_release: Option<(Serial, WlSurface)>,
}

impl Server {
    /// Enable pointer capability on the keyboard seat, idempotently.
    ///
    /// Backends must route motion, buttons, axes and leave after enabling this.
    /// This internal API creates no agent verb; cursor presentation is separate.
    pub fn enable_pointer(&mut self) -> Result<(), InputError> {
        if self.surfaces.pointer.is_none() {
            let keyboard = self
                .surfaces
                .keyboard
                .as_mut()
                .ok_or(InputError::Unavailable)?;
            self.surfaces.pointer = Some(Pointer {
                handle: keyboard.seat.add_pointer(),
                buttons: Vec::new(),
                location: (0.0, 0.0).into(),
                time: 0,
                unspent_scroll: (0.0, 0.0),
                popup_release: None,
            });
        }
        Ok(())
    }

    /// Hit-test mapped trees in renderer order at logical scale one.
    ///
    /// Toplevels use their compositor placement; popups align their XDG geometries
    /// above their own parent, matching the renderer. Input regions,
    /// buffer dimensions and subsurface stacking/offsets are respected. A held
    /// button retains the original recipient until release, including outside
    /// its bounds. Invalid coordinates leave routing state unchanged.
    /// An accepted XDG move consumes motion and places the owning root; a resize
    /// negotiates dimensions and anchors only acknowledged committed geometry.
    /// Both consume drag buttons until release. Keyboard routing is independent.
    pub fn pointer_motion(&mut self, x: f64, y: f64, time: u32) -> Result<(), InputError> {
        if !bounded(x) || !bounded(y) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        // `location` is the screen's, which is what is stored; `on_the_plane` is
        // the same point in a frame's own units, which is what a frame, a drag and
        // a resize are told. Converted once, here, rather than at each reader.
        let location: Point<f64, Logical> = (x, y).into();
        let on_the_plane = self.surfaces.on_the_plane(location);
        if self.surfaces.move_window_pointer(on_the_plane)?
            || self.surfaces.resize_window_pointer(on_the_plane)?
        {
            if let Some(pointer) = self.surfaces.pointer.as_mut() {
                pointer.location = location;
                pointer.time = time;
            }
            return Ok(());
        }
        let focus = self.pointer_target(location);
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        pointer.location = location;
        pointer.time = time;
        let handle = pointer.handle.clone();
        // Plane units on both sides: the focus origin above came back in them, and
        // Smithay sends a client the difference. A screen position against a plane
        // origin would put the pointer inside the window by the wrong amount at
        // every zoom but life size, which is the one zoom a test forgets.
        handle.motion(
            &mut self.surfaces,
            focus,
            &MotionEvent {
                location: on_the_plane,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        handle.frame(&mut self.surfaces);
        self.surfaces.prune_pointer_release();
        Ok(())
    }

    /// Route a Linux mouse button (BTN_LEFT through BTN_TASK).
    ///
    /// No focus, duplicate presses and unmatched releases return false. Buttons
    /// pressed outside a client never authorize a later release into a client.
    /// During a popup grab, outside presses dismiss the chain and return false;
    /// neither the press nor its later release is redirected to another client.
    /// A matched real release may authorize one popup while its recipient stays
    /// focused. New accepted button events and cancellation invalidate it.
    /// Interactive move/resize buttons are consumed and return false, including the
    /// final release; that release restores hit testing without client delivery.
    pub fn pointer_button(
        &mut self,
        button: u32,
        state: ButtonState,
        time: u32,
    ) -> Result<bool, InputError> {
        if !(0x110..=0x117).contains(&button) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        self.surfaces.prune_window_move();
        if self.surfaces.window_resize_button(button, state) {
            if !self.surfaces.resizing_pointer()
                && let Some(location) = self.surfaces.pointer.as_ref().map(|p| p.location)
            {
                self.pointer_motion(location.x, location.y, time)?;
            }
            return Ok(false);
        }
        if self.surfaces.window_move_button(button, state) {
            if self.surfaces.window_move.is_none()
                && let Some(location) = self.surfaces.pointer.as_ref().map(|p| p.location)
            {
                self.pointer_motion(location.x, location.y, time)?;
            }
            return Ok(false);
        }
        if state == ButtonState::Pressed
            && self.surfaces.popup_grab.is_some()
            && self
                .surfaces
                .pointer
                .as_ref()
                .is_some_and(|p| !p.buttons.contains(&button))
            && let Some(location) = self.surfaces.pointer.as_ref().map(|p| p.location)
        {
            // Re-hit the scene even without motion: a menu may have mapped
            // beneath a stationary pointer, or an implicit drag may hide an
            // outside target while another button remains held.
            if self.pointer_target(location).is_none() {
                self.surfaces.dismiss_popup_grab();
                return Ok(false);
            }
            self.pointer_motion(location.x, location.y, time)?;
        }
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        if pointer.handle.current_focus().is_none()
            || pointer.buttons.contains(&button) == (state == ButtonState::Pressed)
        {
            return Ok(false);
        }
        if state == ButtonState::Pressed {
            pointer.buttons.push(button);
        } else {
            pointer.buttons.retain(|held| *held != button);
        }
        pointer.time = time;
        pointer.popup_release = None;
        let recipient = pointer.handle.current_focus();
        let serial = SERIAL_COUNTER.next_serial();
        let handle = pointer.handle.clone();
        handle.button(
            &mut self.surfaces,
            &ButtonEvent {
                button,
                state,
                time,
                serial,
            },
        );
        handle.frame(&mut self.surfaces);
        // Smithay ends the implicit grab on the final release but retains its
        // focus until the next motion. Re-hit now so moving away during a drag
        // cannot leave release authority on that stale recipient.
        let release_target = self.surfaces.pointer.as_ref().is_some_and(|pointer| {
            !pointer.buttons.is_empty()
                || self
                    .pointer_target(pointer.location)
                    .map(|(surface, _)| surface)
                    == recipient
        });
        if state == ButtonState::Released
            && release_target
            && let Some(pointer) = self.surfaces.pointer.as_mut()
        {
            pointer.popup_release = recipient.map(|surface| (serial, surface));
        }
        self.surfaces.prune_pointer_release();
        Ok(true)
    }

    /// Forward one finite scroll frame, preserving source, v120 and stop flags.
    /// Timestamps use the backend's monotonic clock.
    ///
    /// # Over a frame it is the frame's; over the canvas it pans
    ///
    /// The canvas plan's task 5 states the rule, and the hit test already answers
    /// it: with a client focused, this is that application's scroll and nothing
    /// moves the plane; with none, the arrow is on the canvas and the scroll pans
    /// it. See `crate::canvas_pan`. `true` means the scroll did one of those two
    /// things, and a caller that needs to know which asks the camera.
    ///
    /// `false` where a scroll reached nobody: no pointer, a fraction too small to
    /// spend yet — the remainder is kept — or a pan the plane's own edge refused.
    /// This used to be the answer whenever nothing was focused, because then a
    /// scroll over empty canvas did nothing at all.
    ///
    /// **Nothing focused is not quite the same as the arrow being on the canvas.**
    /// It is also true in the moment after the pointer has left the output, and
    /// this crate keeps no separate record of whether a pointer is present — so a
    /// scroll arriving then pans. Named in the plan under task 5 rather than fixed
    /// with a flag invented here.
    pub fn pointer_axis(&mut self, frame: AxisFrame) -> Result<bool, InputError> {
        if !bounded(frame.axis.0) || !bounded(frame.axis.1) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        // **Over a frame, or over the plane** — task 5's whole rule, and the hit
        // test has already answered it: a focus means the arrow is on a frame and
        // the scroll is that application's, and no focus means the arrow is on the
        // canvas and the scroll moves the canvas. See `crate::canvas_pan`.
        if pointer.handle.current_focus().is_none() {
            pointer.time = frame.time;
            return Ok(self.pan_the_plane_by_scroll(&frame));
        }
        pointer.time = frame.time;
        let handle = pointer.handle.clone();
        handle.axis(&mut self.surfaces, frame);
        handle.frame(&mut self.surfaces);
        Ok(true)
    }

    /// Cancel held buttons and clear focus on parent leave, deactivation or close.
    /// A subsequent motion is required before another client can receive input.
    /// Native presentation/focus retire and execution cancels even without a pointer;
    /// its matching primary release remains owned by the native transaction.
    pub fn pointer_leave(&mut self) -> Result<(), InputError> {
        self.retire_window_controls();
        self.surfaces.dismiss_popup_grab();
        self.surfaces.clear_pointer()
    }

    /// Scene hits are shared by motion and explicit-grab outside-click policy.
    ///
    /// # Two spaces, and this is the one conversion between them
    ///
    /// `location` arrives in **screen pixels**, which is what a backend reports
    /// and the space the arrow and the window-control strip live in. What comes
    /// back is a surface and that surface's origin in **the plane's own units**,
    /// because that origin is subtracted from a client's pointer position and a
    /// client is never told about the canvas — at 40 % its window is 16 of its
    /// own units wide and about 6 pixels tall on the glass, and only one of those
    /// two numbers means anything inside the application.
    ///
    /// The camera's pan needs no term here: `crate::scene::trees` has already put
    /// it into every origin, and dividing the pointer and the origins by the same
    /// zoom cancels it exactly. So a pan is held in one place and a zoom in one
    /// place, and neither is applied twice.
    fn pointer_target(
        &self,
        location: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        let on_the_plane = self.surfaces.on_the_plane(location);
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        crate::scene::trees(&roots, &self.popup_surfaces(), self.camera)
            .into_iter()
            .find_map(|(root, origin)| {
                let origin = self.surfaces.on_the_plane(origin);
                under_from_surface_tree(
                    &root,
                    on_the_plane - origin,
                    (0, 0),
                    WindowSurfaceType::ALL,
                )
                .map(|(surface, offset)| (surface, origin + offset.to_f64()))
            })
            .filter(|(surface, _)| self.surfaces.popup_allows_pointer(surface))
    }
}

impl Surfaces {
    /// A screen point in the units a frame on the plane is measured in.
    ///
    /// **The inverse of the zoom `crate::scene::trees` applied, and nothing
    /// else.** The pan is already in the origins this is compared against, so
    /// adding the camera's own position here would apply it twice.
    ///
    /// This is the seam `docs/autonomy/the-smallest-canvas-worth-showing.md`
    /// task 2 is about. A viewport surface never calls it — the dock and the
    /// status area are laid out from the output's size, a zoom does not resize
    /// them, and a control that converted a pointer would be compensating for a
    /// transform it is not under.
    pub(crate) fn on_the_plane(&self, screen: Point<f64, Logical>) -> Point<f64, Logical> {
        screen.downscale(crate::scene::drawn_at(self.popups.camera))
    }
}

/// Wayland fixed-point values have a signed 24-bit integer part.
pub(crate) fn bounded(value: f64) -> bool {
    value.is_finite() && (-8_388_608.0..8_388_608.0).contains(&value)
}

impl Surfaces {
    /// Last validated pointer event, used to re-hit after an explicit scene edit.
    pub(crate) fn pointer_position(&self) -> Option<(Point<f64, Logical>, u32)> {
        self.pointer
            .as_ref()
            .and_then(|p| p.handle.current_focus().map(|_| (p.location, p.time)))
    }

    /// A release cannot authorize a menu after its pointer recipient changes.
    fn prune_pointer_release(&mut self) {
        if let Some(pointer) = self.pointer.as_mut()
            && pointer.popup_release.as_ref().is_some_and(|(_, surface)| {
                pointer.handle.current_focus().as_ref() != Some(surface)
            })
        {
            pointer.popup_release = None;
        }
    }

    /// Clear stale focus even if its root survives a child unmap/destruction.
    pub(crate) fn prune_pointer_focus(&mut self) {
        let focus = self.pointer.as_ref().and_then(|p| p.handle.current_focus());
        if focus.is_some_and(|surface| !self.pointer_surface_mapped(surface)) {
            let _ = self.clear_pointer();
        }
    }

    /// Require buffered ancestors ending in a live mapped root.
    fn pointer_surface_mapped(&self, mut surface: WlSurface) -> bool {
        loop {
            if !surface.is_alive()
                || !with_renderer_surface_state(&surface, |s| s.buffer().is_some()).unwrap_or(false)
            {
                return false;
            }
            match get_parent(&surface) {
                Some(parent) => surface = parent,
                None => {
                    return self.mapped().any(|root| *root == surface)
                        || self.popups.mapped().any(|popup| popup.surface == surface);
                }
            }
        }
    }

    /// Clear the pending grab target before synthesizing button releases.
    pub(crate) fn clear_pointer(&mut self) -> Result<(), InputError> {
        self.window_move = None;
        self.cancel_window_resize();
        // Plane units, like every other motion this crate dispatches. The focus is
        // `None` in both sends below, so no client reads this coordinate — but a
        // location in the other space here would be the one that survived a later
        // change to these two calls. Read before the mutable borrow, because the
        // conversion asks this same `Surfaces` where the camera is.
        let location = self
            .pointer
            .as_ref()
            .map(|pointer| self.on_the_plane(pointer.location))
            .ok_or(InputError::PointerUnavailable)?;
        let pointer = self
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        let handle = pointer.handle.clone();
        let time = pointer.time;
        let buttons = std::mem::take(&mut pointer.buttons);
        pointer.popup_release = None;
        // Remove the pending recipient before releasing the implicit grab.
        handle.motion(
            self,
            None,
            &MotionEvent {
                location,
                time,
                serial: SERIAL_COUNTER.next_serial(),
            },
        );
        for button in buttons {
            handle.button(
                self,
                &ButtonEvent {
                    button,
                    state: ButtonState::Released,
                    time,
                    serial: SERIAL_COUNTER.next_serial(),
                },
            );
        }
        handle.unset_grab(self, SERIAL_COUNTER.next_serial(), time);
        handle.motion(
            self,
            None,
            &MotionEvent {
                location,
                time,
                serial: SERIAL_COUNTER.next_serial(),
            },
        );
        handle.frame(self);
        Ok(())
    }
}
