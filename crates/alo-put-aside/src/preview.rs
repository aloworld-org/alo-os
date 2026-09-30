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
//! A title, an application, and the patch the window will go back to. **No size, no
//! thumbnail, no screen coordinate** — what the preview looks like is the compositor's,
//! and a crate that cannot measure a font has no business deciding it.

use alo_dock::on_the_canvas::Patch;
use alo_dock::window::{AppId, Window, WindowId};

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
}

impl Preview {
    /// The preview for a window being put aside.
    #[must_use]
    pub fn of(window: &Window) -> Self {
        Self {
            window: window.id(),
            app: window.app().clone(),
            called: window.called().to_owned(),
            at: window.at(),
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

    /// Where it goes back to.
    #[must_use]
    pub const fn at(&self) -> Patch {
        self.at
    }
}
