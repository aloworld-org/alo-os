//! When the display this compositor draws to arrives, changes and goes.
//!
//! `crate::server_desk` holds a session's divisions and desktops and has an
//! arrival and a departure. **Something has to call them**, and until this file
//! existed nothing did: a chord asking to divide the display was answered *this
//! session has no display to divide* on a machine that was drawing a frame at
//! the time. A test found it, which is what the test was for.
//!
//! # This compositor advertises one output, so a session has one display
//!
//! `crate::presentation` creates the output global after the first successful
//! frame with valid metadata and a usable size, and withdraws it on retirement.
//! That is the arrival and the departure, so this hangs off those two moments
//! rather than inventing a third. The display is numbered 1 because there is
//! only ever one; the moment a second output is advertised, the number comes
//! from whatever advertises it and this constant goes.
//!
//! # A display that resizes leaves and comes straight back
//!
//! Neither `alo-desktops` nor `alo-dividing` takes a new area for a display it
//! already holds — a plug-in and an unplug are the whole of both lifecycles.
//! Rather than keep a stale area, or teach this crate to resize a tree (which
//! would be deciding a layout), a changed extent is the display **leaving and
//! returning at the new size**: the arrangement is remembered on the way out
//! and `alo_dividing::Remembered::restored` lays it out into the new area on
//! the way in, which is the road that already exists for a screen unplugged at
//! one desk and plugged in at another.
//!
//! # What a window is held by, for a screen that comes back
//!
//! A remembered division is remembered per application, not per window number,
//! so that a screen returning tomorrow finds today's windows. The application
//! is the XDG `app_id` its client set. A window whose client set none is left
//! out of what is remembered rather than remembered under a made-up name: an
//! invented key would match some other nameless window next time and put
//! somebody's terminal in their editor's share.

use alo_desktops::{DisplayId, Promises};
use alo_dividing::{Area, HeldBy, Point, Window, WindowId, area::Size};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;

/// The display a single-output compositor draws to.
const THE_DISPLAY: DisplayId = DisplayId::from_compositor(1);

/// The extent a display is described in, both ends agreeing.
///
/// `crate::window_mode` publishes an output extent only inside this range, and
/// `alo_dividing::Area` refuses an empty one. A frame outside it is a frame
/// with no output, which is a state the rest of this crate already has.
const DESCRIBABLE: std::ops::RangeInclusive<i32> = 1..=1_000_000;

impl crate::Server {
    /// A frame was submitted to a display of this size, called this.
    ///
    /// The first one is the display arriving. A later one of the same size is
    /// nothing happening, which is the usual case and costs a comparison. A
    /// later one of a different size is the display leaving and returning, for
    /// the reason in this file's header.
    ///
    /// A size outside [`DESCRIBABLE`] is not a display: whatever was here goes,
    /// and nothing arrives. That is the same answer
    /// `crate::window_mode::update_window_mode_output` gives such a frame, so
    /// the two cannot disagree about whether there is an output.
    pub(crate) fn the_display_submitted(
        &mut self,
        metadata: &crate::OutputMetadata,
        size: (i32, i32),
    ) {
        let named = &the_screen(metadata);
        let Some(area) = describable(size) else {
            self.the_display_retired();
            return;
        };
        match self.desk.area_of(THE_DISPLAY) {
            Some(already) if already == area => return,
            Some(_) => self.the_display_retired(),
            None => {}
        }
        // Three numbers for the three surfaces this compositor draws itself,
        // taken from the counter every window's number comes from so one of
        // them can never be a client's.
        let Ok(promises) = Promises::of(
            WindowId::from_compositor(crate::window_number::reserve()),
            WindowId::from_compositor(crate::window_number::reserve()),
            WindowId::from_compositor(crate::window_number::reserve()),
        ) else {
            // Three numbers from a counter that never repeats cannot be equal,
            // so this is unreachable rather than handled. Refusing is still the
            // answer: a display whose three promises are not three is one
            // `alo-desktops` would rightly not give desktops to.
            return;
        };
        let held = self.what_each_application_has_open();
        // A refusal here is `alo-desktops` saying this display is already
        // here, which the match above has just ruled out.
        let _ = self
            .desk
            .display_arrived(THE_DISPLAY, named, area, promises, &|by| {
                held.iter().find(|(who, _)| who == by).map(|(_, it)| *it)
            });
    }

    /// The output was retired: the display has gone.
    ///
    /// Its division is remembered under the name it arrived with, so the same
    /// screen coming back finds it. Nothing is closed and no window is moved.
    pub(crate) fn the_display_retired(&mut self) {
        let Some(named) = self.desk.name_of(THE_DISPLAY).map(str::to_owned) else {
            return;
        };
        let held = self.what_each_window_is_held_by();
        // Unknown is the only refusal, and the name above proves it is known.
        let _ = self.desk.display_left(THE_DISPLAY, &named, &|window| {
            held.iter()
                .find(|(number, _)| *number == window)
                .map(|(_, by)| by.clone())
        });
    }

    /// Which window each application that is running has open.
    ///
    /// One window per application: a division remembers a share as *the mail
    /// application's*, and handing back two windows for it would be this crate
    /// choosing between them. The first of that application's mapped windows is
    /// the one. Which of two windows of the same application a share is given
    /// back to is a question `alo-dividing` does not ask and this cannot
    /// answer, so it is the first rather than a guess dressed as a rule.
    fn what_each_application_has_open(&mut self) -> Vec<(HeldBy, Window)> {
        let mapped: Vec<WlSurface> = self.surfaces.mapped().cloned().collect();
        let mut open: Vec<(HeldBy, Window)> = Vec::new();
        for surface in mapped {
            let Some(by) = held_by(&surface) else {
                continue;
            };
            if open.iter().any(|(who, _)| *who == by) {
                continue;
            }
            let number = self.desk.number_of(&surface);
            open.push((by, Window::any_size(number)));
        }
        open
    }

    /// Which application holds each window that has a number.
    fn what_each_window_is_held_by(&mut self) -> Vec<(WindowId, HeldBy)> {
        let mapped: Vec<WlSurface> = self.surfaces.mapped().cloned().collect();
        mapped
            .into_iter()
            .filter_map(|surface| {
                let by = held_by(&surface)?;
                let number = self.desk.number_given_to(&surface)?;
                Some((WindowId::from_compositor(number), by))
            })
            .collect()
    }
}

/// What `alo-dividing` remembers this screen by.
///
/// **The screen and the port both, because neither alone is the screen.** A
/// connector name is the socket on this machine, so remembering by it alone
/// would lay a person's laptop arrangement over whatever monitor they plugged
/// into that port next; a make and model alone would move an arrangement
/// between two ports, which is right, but this compositor draws to one output
/// and has no second port to move it to. Together they are the strongest
/// identity the backend gives us.
///
/// **What it cannot tell apart** is two identical monitors swapped between two
/// ports — the metadata carries no serial number. That is a limit of what a
/// backend reports and is recorded in `docs/quirks.md` rather than guessed at.
fn the_screen(metadata: &crate::OutputMetadata) -> String {
    format!("{}/{}/{}", metadata.make, metadata.model, metadata.name)
}

/// This size as an area a display can be, or [`None`] where it is not one.
fn describable(size: (i32, i32)) -> Option<Area> {
    let (width, height) = size;
    if !DESCRIBABLE.contains(&width) || !DESCRIBABLE.contains(&height) {
        return None;
    }
    Area::of(
        Point::at(0, 0),
        Size::of(u32::try_from(width).ok()?, u32::try_from(height).ok()?),
    )
    .ok()
}

/// The application this window belongs to, as its own client named it.
///
/// [`None`] for a window whose client set no `app_id`, or set one
/// `alo-dividing` will not hold as a key — an empty one, a spaced one, an
/// overlong one or one with a control character in it. A name that cannot be a
/// key is not turned into one here; what it means is that this window's share
/// is not remembered, which is honest and loses only the arrangement.
fn held_by(surface: &WlSurface) -> Option<HeldBy> {
    let named: Option<String> = with_states(surface, |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|data| {
                data.lock()
                    .unwrap_or_else(|_| std::process::abort())
                    .app_id
                    .clone()
            })
    });
    HeldBy::named(&named?).ok()
}
