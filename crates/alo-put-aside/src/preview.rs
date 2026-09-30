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

use crate::what_a_preview_is_headed_with::Headline;
use crate::what_alo_is_doing::WhatAloIsDoing;
use crate::where_it_goes_back::WhereItGoesBack;
use crate::whether_it_is_private::Privacy;

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
    /// What alo is doing in this window, if anything.
    ///
    /// **Not a construction-time value**, because it changes while the window is away: that
    /// is the whole point of leaving alo working in something you put aside. So it starts as
    /// [`WhatAloIsDoing::Nothing`] and is replaced by [`Self::alo_is_now`].
    ///
    /// Starting at `Nothing` is what makes the default safe. A window put aside on a machine
    /// with no agent carries no agent report and can never be drawn with an empty one, which
    /// is ADR 0009 held by the field's initial value rather than by a caller remembering.
    alo: WhatAloIsDoing,
    /// Whether the title may be shown, and what to show if not.
    privacy: Privacy,
}

impl Preview {
    /// The preview for a window being put aside, at the zoom the person was at.
    ///
    /// Carries no agent report. One is attached by [`Self::alo_is_now`] if alo is given
    /// something to do here, and **never by this constructor** — a machine with no agent must
    /// not have to pass an argument saying so.
    #[must_use]
    pub fn of(window: &Window, zoom: Zoom, privacy: Privacy) -> Self {
        Self {
            window: window.id(),
            app: window.app().clone(),
            called: window.called().to_owned(),
            at: window.at(),
            zoom,
            alo: WhatAloIsDoing::Nothing,
            privacy,
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

    /// What to write at the top of this preview's row.
    ///
    /// **The only way to get a heading, and a private window's title is not reachable at
    /// all.** This replaced a `called()` that returned the title, because a public accessor
    /// returning the title is the whole of what a surface needs to leak a private window, and
    /// no documentation beside it stops the one caller who draws the obvious field.
    ///
    /// The same correction as `Zoom` and `Camera`, made before somebody else had to measure
    /// it: a rule held by a value being out of reach beats a rule held by a caller not asking
    /// for it.
    ///
    /// **Never empty.** `alo-dock` gives a window with no title of its own the application's
    /// name when it is made, for the reason its own file states — a preview headed with a
    /// number is a row a person cannot choose between — and a safe name refuses to be blank.
    #[must_use]
    pub fn headline(&self) -> Headline<'_> {
        match self.privacy.safe_name() {
            Some(safe) => Headline::PreviewHidden(safe),
            None => Headline::ItsTitle(&self.called),
        }
    }

    /// Whether this window's contents may be described at all.
    #[must_use]
    pub const fn privacy(&self) -> &Privacy {
        &self.privacy
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

    /// What alo is doing in this window, if anything.
    ///
    /// A surface asks this **before** drawing anything agent-shaped. [`WhatAloIsDoing::Nothing`]
    /// means draw no heading and no control, which is ADR 0009 in this panel: the agent's
    /// surfaces disappear rather than nag, and a greyed-out feature is an advertisement.
    #[must_use]
    pub const fn alo(&self) -> &WhatAloIsDoing {
        &self.alo
    }

    /// Say what alo is doing here now.
    ///
    /// Replaces the report wholesale rather than amending it, so a preview cannot hold a
    /// half-updated one — the same reason [`crate::alo_at_work::AtWork`] takes every part at once.
    ///
    /// Passing [`WhatAloIsDoing::Nothing`] is how work **ends**, and it is the same value a
    /// preview starts with: a finished task and a machine with no agent leave a surface with
    /// exactly the same thing to draw, which is nothing.
    pub fn alo_is_now(&mut self, what: WhatAloIsDoing) {
        self.alo = what;
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
