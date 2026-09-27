//! What a frame is called, which is also what moves it.
//!
//! ADR 0071 settles that **the name above a frame is what moves it** and the edge
//! and corners resize — so a name stopped being decoration here and became the
//! affordance. It is the same name a reader says: the canvas plan's task 7 asks
//! that every frame be *reached, focused and named*, and its constraint is that
//! this is not a second interface.
//!
//! # The shell had a name all along and never asked for it
//!
//! `xdg_toplevel.set_title` was not read anywhere in this crate. Smithay keeps it
//! in the toplevel's own role attributes, so nothing had to be stored — what was
//! missing was a question, which is why *a frame has no name* was true and cheap
//! to stop being true.
//!
//! # Three answers, and this crate translates none of them
//!
//! A title and an app id are the application's own strings and must never be
//! translated. Only the third case is the machine speaking, and the word for it is
//! `alo_access::words::AN_APPLICATION` — the same phrase the accessibility tree
//! already reads for a window, rather than a second vocabulary invented here. So
//! this returns which of the three it is and lets the caller say it, which keeps a
//! client's string and a translated word in different shapes and makes it hard to
//! print one where the other belongs.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;

/// What a frame is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameName {
    /// What the application calls this window, in its own words. Never translated.
    Given(String),
    /// Its class, where it named the window nothing. Never translated.
    ItsClass(String),
    /// The application named neither, so the machine says what it is.
    ///
    /// Said with `alo_access::words::AN_APPLICATION`. This crate holds no wording.
    AnApplication,
}

impl FrameName {
    /// The application's own string, where it gave one.
    ///
    /// [`None`] for [`Self::AnApplication`] — and that is the point of the
    /// distinction: a caller that wants something to draw must decide what to do
    /// about the third case rather than being handed an English word.
    #[must_use]
    pub fn its_own_words(&self) -> Option<&str> {
        match self {
            Self::Given(name) | Self::ItsClass(name) => Some(name),
            Self::AnApplication => None,
        }
    }
}

impl crate::Server {
    /// What this frame is called.
    ///
    /// The title if the application set one, then its app id, then
    /// [`FrameName::AnApplication`]. An empty string counts as unset: an
    /// application that sets a title to nothing has not named its window, and a
    /// frame whose handle is a blank label is a frame nobody can grab.
    ///
    /// Reads committed state and sends nothing. Safe to ask every frame.
    #[must_use]
    pub fn the_name_of(&self, frame: &WlSurface) -> FrameName {
        let named = with_states(frame, |states| {
            let data = states.data_map.get::<XdgToplevelSurfaceData>();
            data.and_then(|data| data.lock().ok())
                .map(|state| (state.title.clone(), state.app_id.clone()))
        });
        let Some((title, app_id)) = named else {
            return FrameName::AnApplication;
        };
        let said = |it: Option<String>| it.filter(|it| !it.trim().is_empty());
        said(title)
            .map(FrameName::Given)
            .or_else(|| said(app_id).map(FrameName::ItsClass))
            .unwrap_or(FrameName::AnApplication)
    }
}
