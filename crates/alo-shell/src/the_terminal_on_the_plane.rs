//! **alo's terminal is a window on the canvas**: where it is on the plane, and
//! where that puts it on a display.
//!
//! [ADR 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md),
//! the owner's decision of 2026-11-01. Until then the terminal was drawn fixed to
//! the display, centred, at a share of its size, so zooming the canvas out left
//! it full size over every application that had shrunk, and *Show all* could not
//! see it. A booted machine showed exactly that.
//!
//! # A window like an application's, in the same arithmetic
//!
//! Its position and size are **plane units**, the units an application's buffer
//! origin is in, and the camera turns them into a display's pixels the way
//! `crate::scene::trees` does for a client: `(plane - camera.at) * zoom`. So the
//! terminal pans and zooms with everything else, and a press is tested against
//! the same answer the drawing used.
//!
//! **The picture is laid out at the window's own size and scaled when it is
//! drawn**, as an application's buffer is. The shell is told one grid, and
//! zooming never changes it: a person zooming out to find a window must not
//! re-flow every line their shell has printed.
//!
//! # Placed once, where a new window goes
//!
//! A terminal is given its place on the first frame after it opens, by
//! `crate::where_a_window_opens` — the owner's rules in
//! `docs/design/where-a-new-window-opens.md`: in the current view, beside the
//! window in front, never exactly on top of another. Its size is the share of
//! the display the design draws it at. From then on it stays where it is, on the
//! Place it opened on, until something moves it.
//!
//! # What this does not do yet
//!
//! The terminal cannot be dragged or resized: it had no handle before this and
//! has none now. The window edge, title and focused look of canvas windows are
//! the window-look lane's, and reach this window when it has one.

use alo_canvas::Camera;
use alo_strings::Direction;
use smithay::backend::renderer::Frame;
use smithay::utils::{Logical, Physical, Rectangle, Size};

use crate::painted::Washed;
use crate::the_terminals_cells_are_drawn::TerminalPicture;

/// alo's terminal, flattened into one picture, and where a display draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheTerminalOnTheScreen {
    /// The whole picture — body, transcript, cells and strip — at the window's
    /// own size, with its origin at the shadow's top-left corner.
    pub(crate) raster: Washed,
    /// Where the picture lands on this display, scaled by the camera's zoom.
    /// May reach past the display on any side, or miss it altogether.
    pub(crate) drawn_at: Rectangle<i32, Physical>,
}

impl TheTerminalOnTheScreen {
    /// The raster to upload before a frame opens.
    pub(crate) const fn washed(&self) -> &Washed {
        &self.raster
    }

    /// Refuse a picture whose pixels are not the size it says.
    ///
    /// # Errors
    /// [`crate::RenderError::TerminalScene`] when they are not.
    pub(crate) fn validate(&self) -> Result<(), crate::RenderError> {
        let (w, h) = (self.raster.area.size.w, self.raster.area.size.h);
        let wanted = usize::try_from(w)
            .ok()
            .zip(usize::try_from(h).ok())
            .and_then(|(w, h)| w.checked_mul(h)?.checked_mul(4));
        if w > 0 && h > 0 && wanted == Some(self.raster.pixels.len()) {
            return Ok(());
        }
        Err(crate::RenderError::TerminalScene)
    }

    /// Paint the uploaded picture where the camera puts it.
    ///
    /// # Errors
    /// Whatever the renderer refused.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        uploaded: Option<&(F::TextureId, Rectangle<i32, Physical>)>,
    ) -> Result<(), crate::RenderError> {
        let Some((texture, _)) = uploaded else {
            return Ok(());
        };
        crate::painted::wash_scaled(
            frame,
            texture,
            (self.raster.area.size.w, self.raster.area.size.h),
            self.drawn_at,
        )
    }
}

/// The picture's parts painted into one straight-RGBA raster, in the order
/// they are drawn: the body, then the shapes, then the cells, then the strip.
///
/// **One raster rather than many runs**, because a run of text cannot be
/// scaled by the camera one pixel row at a time and stay a run of text. The
/// shapes and the cells are opaque, so each simply replaces what is under it.
///
/// [`None`] for a picture with nothing in it, with no size, or with a part
/// outside its own room, which `TerminalPicture::validate` refuses.
pub(crate) fn flattened(picture: &TerminalPicture) -> Option<Washed> {
    let (width, height) = picture.size;
    if picture.is_empty() {
        return None;
    }
    picture.validate((width, height).into()).ok()?;
    let (w, h) = (usize::try_from(width).ok()?, usize::try_from(height).ok()?);
    let mut pixels = vec![0_u8; w.checked_mul(h)?.checked_mul(4)?];
    let at = |x: i32, y: i32| -> Option<usize> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        (x < w && y < h).then(|| (y * w + x) * 4)
    };
    if let Some(panel) = &picture.panel {
        let across = usize::try_from(panel.area.size.w).ok()?;
        for (row, line) in panel.pixels.chunks(across.max(1) * 4).enumerate() {
            let y = panel.area.loc.y + i32::try_from(row).ok()?;
            let Some(start) = at(panel.area.loc.x, y) else {
                continue;
            };
            let end = start + line.len();
            if let Some(into) = pixels.get_mut(start..end) {
                into.copy_from_slice(line);
            }
        }
    }
    for solid in &picture.solids {
        let [r, g, b] = solid.colour;
        for y in solid.area.loc.y..solid.area.loc.y + solid.area.size.h {
            for x in solid.area.loc.x..solid.area.loc.x + solid.area.size.w {
                if let Some(i) = at(x, y)
                    && let Some(into) = pixels.get_mut(i..i + 4)
                {
                    into.copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    for inked in picture.inked.iter().chain(&picture.strip) {
        let across = usize::try_from(inked.area.size.w).ok()?.max(1);
        for (n, [r, g, b]) in inked.pixels.iter().enumerate() {
            let x = inked.area.loc.x + i32::try_from(n % across).ok()?;
            let y = inked.area.loc.y + i32::try_from(n / across).ok()?;
            if let Some(i) = at(x, y)
                && let Some(into) = pixels.get_mut(i..i + 4)
            {
                into.copy_from_slice(&[*r, *g, *b, 255]);
            }
        }
    }
    Some(Washed {
        area: Rectangle::from_size((width, height).into()),
        pixels,
    })
}

/// Where a window on the plane is drawn on a display looking through `camera`.
///
/// `scene::trees`' arithmetic for a client's buffer, applied to a rectangle:
/// the corner moves by the pan and both are multiplied by the zoom. The far
/// corner is worked out on its own and rounded, so two windows that meet on the
/// plane meet on the screen.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "a plane coordinate times a zoom of at most ten fits an i32 for any window this \
              canvas holds"
)]
pub(crate) fn on_the_screen(
    window: Rectangle<i32, Logical>,
    camera: Camera,
) -> Rectangle<i32, Physical> {
    let zoom = crate::scene::drawn_at(camera);
    let to = |plane: i32, from: i32| -> i32 {
        ((f64::from(plane) - f64::from(from)) * zoom).round() as i32
    };
    let (left, top) = (
        to(window.loc.x, camera.at().x),
        to(window.loc.y, camera.at().y),
    );
    let (right, bottom) = (
        to(window.loc.x + window.size.w, camera.at().x),
        to(window.loc.y + window.size.h, camera.at().y),
    );
    Rectangle::new((left, top).into(), (right - left, bottom - top).into())
}

impl crate::Server {
    /// Where alo's terminal is on the canvas, while it is on the Place being
    /// looked at.
    ///
    /// [`None`] with no terminal, before it is placed, and while it is on
    /// another Place — which is a terminal nobody can see from here, exactly
    /// as an application's window on another Place is.
    #[must_use]
    pub(crate) fn where_the_terminal_is_on_the_plane(&self) -> Option<Rectangle<i32, Logical>> {
        self.alos_own_terminal.as_ref()?;
        let (window, place) = self.the_terminal_on_the_plane?;
        (place == self.the_place_now()).then_some(window)
    }

    /// Where alo's terminal would be if it had never been placed: centred on
    /// what this camera sees, the share of the display the design draws it at.
    ///
    /// **Only a draw that cannot place it asks this** — a fixture driving the
    /// nested backend, which holds the server immutably — so it is drawn where
    /// it will be put rather than not at all.
    fn where_the_terminal_would_open(
        display: (i32, i32),
        scale: f64,
        camera: Camera,
    ) -> Option<Rectangle<i32, Logical>> {
        let (window, _) =
            crate::the_terminals_cells_are_drawn::where_the_window_goes(display, scale)?;
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "half a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let middle = (
            camera.at().x + (f64::from(display.0) / 2.0 / zoom) as i32,
            camera.at().y + (f64::from(display.1) / 2.0 / zoom) as i32,
        );
        Some(Rectangle::new(
            (middle.0 - window.size.w / 2, middle.1 - window.size.h / 2).into(),
            (window.size.w, window.size.h).into(),
        ))
    }

    /// Give alo's terminal its place on the canvas, if it is open and has none.
    ///
    /// **Once**, on the first frame after it opens: in the view the person is
    /// looking at, by the owner's rules for a new window, beside the window in
    /// front and never exactly on top of another. The size is the share of this
    /// display the design draws a terminal at.
    ///
    /// Answers whether it placed one.
    pub(crate) fn the_terminal_is_placed(
        &mut self,
        display: (i32, i32),
        scale: f64,
        reading: Direction,
    ) -> bool {
        if self.alos_own_terminal.is_none() {
            self.the_terminal_on_the_plane = None;
            return false;
        }
        if self.the_terminal_on_the_plane.is_some() {
            return false;
        }
        let Some((window, _)) =
            crate::the_terminals_cells_are_drawn::where_the_window_goes(display, scale)
        else {
            return false;
        };
        let camera = self.the_camera();
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return false;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let view = Rectangle::<i32, Logical>::new(
            (camera.at().x, camera.at().y).into(),
            (
                (f64::from(display.0) / zoom) as i32,
                (f64::from(display.1) / zoom) as i32,
            )
                .into(),
        );
        let place = self.the_place_now();
        let open: Vec<Rectangle<i32, Logical>> = self
            .mapped_surfaces()
            .filter(|surface| crate::canvas_place::the_place_of(surface) == Some(place))
            .map(crate::where_a_window_is)
            .collect();
        let at = crate::where_a_window_opens(
            view,
            &open,
            open.first().copied(),
            Size::from((window.size.w, window.size.h)),
            reading,
            &[],
        );
        self.the_terminal_on_the_plane = Some((
            Rectangle::new(at, (window.size.w, window.size.h).into()),
            place,
        ));
        true
    }

    /// alo's terminal as a frame on the canvas, for *Show all* to count.
    #[must_use]
    pub(crate) fn the_terminal_as_a_frame(&self) -> Option<alo_canvas::Frame> {
        let window = self.where_the_terminal_is_on_the_plane()?;
        let at = alo_canvas::At::checked(window.loc.x, window.loc.y)?;
        let size = alo_canvas::Size::checked(
            u32::try_from(window.size.w).ok()?,
            u32::try_from(window.size.h).ok()?,
        )?;
        Some(alo_canvas::Frame::of(
            self.the_terminals_number,
            self.the_place_now(),
            at,
            size,
        ))
    }

    /// Where alo's terminal is drawn on the display named, through that
    /// display's camera — the window itself, not its shadow.
    ///
    /// Falls back to where it will open, for a draw that cannot place it; see
    /// `Server::where_the_terminal_would_open`.
    #[must_use]
    pub(crate) fn where_the_terminal_is_drawn_on(
        &self,
        named: &str,
        display: (i32, i32),
        scale: f64,
    ) -> Option<(Rectangle<i32, Logical>, Camera)> {
        self.alos_own_terminal.as_ref()?;
        let camera = self.the_camera_of_the_display(named);
        let window = match self.the_terminal_on_the_plane {
            Some((window, place)) => (place == self.the_place_now()).then_some(window)?,
            None => Self::where_the_terminal_would_open(display, scale, camera)?,
        };
        Some((window, camera))
    }

    /// The camera a display named this is drawn through.
    ///
    /// The key `Server::render_frame` reads it under: the display's own name
    /// where the session has an arrangement, and the one unnamed camera where
    /// it has none.
    pub(crate) fn the_camera_of_the_display(&self, named: &str) -> Camera {
        let key = if self.the_screens().is_some() {
            named
        } else {
            ""
        };
        self.surfaces.camera_of(key)
    }

    /// Bring alo's terminal into view if it is out of it: the view travels to
    /// the window, at the zoom the person has, rather than the window being
    /// pulled to where the person is looking.
    ///
    /// `docs/design/the-canvas-as-a-workspace.md` §11, *focus it without
    /// silently moving it to the current camera position*. Answers whether the
    /// view moved.
    pub(crate) fn the_view_goes_to_the_terminal(&mut self) -> bool {
        if self.alos_own_terminal.is_none() {
            return false;
        }
        let Some((_, its_place)) = self.the_terminal_on_the_plane else {
            return false;
        };
        // **Its Place first**, as `Server::the_view_travels_to` does for an
        // application's window: the Place the person is on becomes the
        // terminal's, and nothing about the terminal moves.
        let somewhere_else = its_place != self.the_place_now();
        if somewhere_else {
            self.surfaces.place = its_place;
        }
        let Some(window) = self.where_the_terminal_is_on_the_plane() else {
            return somewhere_else;
        };
        let Some(room) = self.surfaces.popups.output_size else {
            return somewhere_else;
        };
        let camera = self.the_camera();
        let seen = on_the_screen(window, camera);
        let output = Rectangle::<i32, Physical>::from_size(room);
        if output.contains_rect(seen) {
            return somewhere_else;
        }
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return somewhere_else;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "half a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let at = alo_canvas::At::checked(
            window.loc.x + window.size.w / 2 - (f64::from(room.w) / 2.0 / zoom) as i32,
            window.loc.y + window.size.h / 2 - (f64::from(room.h) / 2.0 / zoom) as i32,
        );
        at.and_then(|at| self.look_at_the_canvas(at)).is_some() || somewhere_else
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminal_on_the_plane_tests.rs"]
mod tests;
