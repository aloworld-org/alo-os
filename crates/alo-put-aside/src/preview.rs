//! One preview: one window a person put aside, named so they can choose between
//! several of the same application.
//!
//! **A preview is one window, never one application.** That is the owner's second
//! behaviour rule and it is the whole reason this panel is not the Dock: the Dock shows
//! that Browser is open, and three minimised Browser windows here are three previews
//! with three titles. A panel that grouped them would answer *which applications did I
//! put aside*, which is a question the Dock already answers.
//!
//! # It carries what a person reads, and nothing about pixels
//!
//! A title, an application, and the view the window will go back to. **No size, no
//! thumbnail, no screen coordinate** — what the preview looks like is the compositor's,
//! and a crate that cannot measure a font has no business deciding it.
//!
//! A [`Zoom`] is not a pixel and not a screen coordinate: it is how far into the plane
//! the person was, which is part of *where the window was* rather than part of how
//! anything is drawn. It is stored as `alo-canvas`'s checked type rather than as a number
//! of thousandths, because [`Zoom::of`] is the thing that refuses 0 and 50_000 — and a
//! panel holding a value nobody validated would hand a caller a view it could not build.

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::Patch;
use alo_dock::window::{AppId, Window, WindowId};

use crate::where_it_goes_back::WhereItGoesBack;

/// One window put aside, as the panel holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// Which window. The compositor's own number.
    window: WindowId,
    /// Which application it belongs to.
    app: AppId,
    /// What the person calls it — the name the preview is headed with.
    called: String,
    /// The part of the plane it goes back to.
    ///
    /// **Kept here as well as on the window**, because restoring has to name the saved
    /// patch rather than wherever the person is looking, and a panel that asked the
    /// canvas *where is this window now* would get an answer about a window that is not
    /// on it.
    at: Patch,
    /// How far in the person was when they put it aside.
    ///
    /// **Handed in rather than read.** This crate cannot ask a camera anything, so the
    /// zoom arrives from whoever performed the gesture — which is the right direction
    /// anyway: what is saved is the view the person was looking at when they chose to put
    /// the window away, and only they were there.
    zoom: Zoom,
}

impl Preview {
    /// The preview for a window being put aside, at the zoom the person was at.
    #[must_use]
    pub fn of(window: &Window, zoom: Zoom) -> Self {
        Self {
            window: window.id(),
            app: window.app().clone(),
            called: window.called().to_owned(),
            at: window.at(),
            zoom,
        }
    }

    /// Which window this is.
    #[must_use]
    pub const fn window(&self) -> WindowId {
        self.window
    }

    /// Which application it belongs to.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        &self.app
    }

    /// What the person calls it.
    ///
    /// **Never empty.** `alo-dock` gives a window with no title of its own the
    /// application's name when it is made, for the reason its own file states: a
    /// preview headed with a number is a row a person cannot choose between.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// The part of the plane it goes back to.
    #[must_use]
    pub const fn at(&self) -> Patch {
        self.at
    }

    /// How far in the person was.
    #[must_use]
    pub const fn zoom(&self) -> Zoom {
        self.zoom
    }

    /// The whole view it goes back to, as one value.
    ///
    /// The two accessors above stay because a caller listing the panel wants the patch
    /// without deciding anything about a camera. **This is the one that restoring uses**,
    /// and it exists so that a caller cannot take the patch and forget the zoom — which
    /// is the failure that would make a restored window look right in the wrong place.
    #[must_use]
    pub const fn where_it_goes_back(&self) -> WhereItGoesBack {
        WhereItGoesBack::of(self.at, self.zoom)
    }
}
