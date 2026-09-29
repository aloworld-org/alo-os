//! **The canvas is where they left it** — task 9, on the shell's side.
//!
//! `alo_arranging` owns the file and refuses anything `alo-canvas` would not
//! allow. This is the other half: reading it when the desktop stands up, giving a
//! window its place when its application comes back, and writing it down again
//! when the session ends.
//!
//! # A place is claimed once, by the first window of its application
//!
//! A `wl_surface` does not survive a session, so a remembered place is keyed by
//! `app_id` — the thing that is the same tomorrow. A place is **taken** when it is
//! given out, so a second window of the same application is placed as any new
//! window is rather than landing exactly on top of the first.
//!
//! # An application that did not come back is absent, not empty
//!
//! The acceptance's second sentence, and it needs no code at all: the arrangement
//! is a list of places waiting to be claimed, and a place nothing claims is never
//! used. Nothing here draws a frame, reserves room for one, or tells anybody a
//! window is missing — which is what *absent rather than drawn empty* means.
//!
//! # A place that is no longer reachable is not given out
//!
//! A file written on a wide display can hold a place that is off a narrow one, and
//! a person who signed in on the smaller machine would get a window they could not
//! reach. So a remembered place is offered to `crate::canvas_never_lost`'s own
//! question first — the same rule a drag is held to — and a place that fails it is
//! dropped rather than used. Task 8 is why that question exists to ask.

use alo_arranging::Arrangement;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle},
};

/// The arrangement this session started from, and what is left of it.
#[derive(Debug, Clone)]
pub struct WhereTheyLeftIt {
    /// What was read at sign-in.
    arrangement: Arrangement,
    /// Which applications have already claimed their place this session.
    claimed: Vec<String>,
}

impl WhereTheyLeftIt {
    /// Start a session from this arrangement.
    #[must_use]
    pub fn from(arrangement: Arrangement) -> Self {
        Self {
            arrangement,
            claimed: Vec::new(),
        }
    }

    /// The camera it was left looking through.
    #[must_use]
    pub fn camera(&self) -> alo_canvas::Camera {
        self.arrangement.camera()
    }

    /// Whether any place is still waiting to be claimed.
    #[must_use]
    pub fn anything_left(&self) -> bool {
        self.arrangement.how_many() > self.claimed.len()
    }
}

impl crate::Server {
    /// Give this window the place its application was left in, if it has one.
    ///
    /// Whether a place was taken. `false` where the application was not
    /// remembered, where it has already claimed its place this session, or where
    /// the place is one task 8 would not let a person drag to — on a display
    /// narrower than the one the file was written on, a remembered place can be
    /// somewhere nothing could get it back from.
    pub fn put_back_where_it_was(
        &mut self,
        frame: &WlSurface,
        left: &mut WhereTheyLeftIt,
        dock: Rectangle<i32, Physical>,
    ) -> bool {
        let Some(app_id) = self.the_app_id_of(frame) else {
            return false;
        };
        if left.claimed.contains(&app_id) {
            return false;
        }
        let Some((at, _)) = left.arrangement.where_it_was(&app_id) else {
            return false;
        };
        // The same question a drag is held to. A place that would leave the name
        // under the dock, or the arrangement wider than Show all reaches, is not
        // one to put somebody's window back into.
        let here = crate::window_number::Numbers::given_to(frame)
            .and_then(|_| self.the_frames_on_the_plane().into_iter().next())
            .map_or(alo_canvas::At::origin(), |frame| frame.at());
        if self.as_far_as_a_frame_may_be_dragged(frame, here, at, dock) != at {
            return false;
        }
        if self.place_window(frame, (at.x, at.y)).is_err() {
            return false;
        }
        left.claimed.push(app_id);
        true
    }

    /// The arrangement as it is right now, ready to be written down.
    ///
    /// Read from the frames actually open and the camera actually looked through,
    /// rather than from anything kept alongside them: a second record of where a
    /// window is would be a second answer to the question `scene::trees` already
    /// has.
    #[must_use]
    pub fn the_arrangement_now(&self) -> Arrangement {
        let mut arrangement = Arrangement::fresh();
        arrangement.looking(self.the_camera());
        for frame in self.mapped_surfaces() {
            let (Some(app_id), Some(place)) = (
                self.the_app_id_of(frame),
                self.the_frames_on_the_plane()
                    .into_iter()
                    .find(|it| Some(it.id()) == crate::window_number::Numbers::given_to(frame)),
            ) else {
                continue;
            };
            arrangement.window_was(app_id, place.at(), place.size());
        }
        arrangement
    }

    /// The `app_id` this window gave, if it gave one.
    ///
    /// **A title is not a key.** `FrameName` falls back from a title to a class to
    /// the machine's own words, which is right for something a person reads and
    /// wrong for something a place is remembered by: a person can change a title
    /// mid-session, and a window would come back somewhere else because they
    /// renamed a document.
    #[must_use]
    pub fn the_app_id_of(&self, frame: &WlSurface) -> Option<String> {
        smithay::wayland::compositor::with_states(frame, |states| {
            states
                .data_map
                .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()
                .and_then(|data| data.lock().ok())
                .and_then(|state| state.app_id.clone())
        })
        .filter(|app_id| !app_id.trim().is_empty())
    }
}
