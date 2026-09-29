//! One window: which application it belongs to, where it sits, and how.
//!
//! **A window has a remembered location.** It is not something the system
//! places wherever it happens to have room — it occupies a
//! [`crate::Patch`] of the workspace plane, it keeps that patch when it is put
//! aside, and it is still there when it comes back. Everything the Dock does
//! about travel rests on that.
//!
//! # Put aside is a state, not a place
//!
//! A window the person has put away keeps its patch. That is what lets *restore
//! it at its saved canvas position* be a sentence rather than a wish, and it is
//! why [`HowItSits::PutAside`] carries nothing: there is nothing to remember
//! separately, because nothing was forgotten.
//!
//! The name is the owner's. The right panel answers *what did I put aside*, so
//! that is what the state is called here rather than *minimised*, which
//! describes what happened to the pixels instead of what the person meant.
//!
//! # Nothing here draws, and nothing here is a handle
//!
//! A [`WindowId`] is a number the compositor already uses for a window; this
//! crate holds it and compares it and never asks what is inside. So a test can
//! lay out a desk of nine windows without a display, which is the whole reason
//! the decision in [`crate::clicking`] can be tested at all.

use crate::on_the_canvas::Patch;

/// Which window, as the compositor numbers them.
///
/// A number rather than a name: two windows of one application are commonly the
/// same document opened twice, so a name would not tell them apart and the
/// thing that does is already in the compositor's hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u64);

impl WindowId {
    /// The window the compositor calls this.
    #[must_use]
    pub const fn numbered(number: u64) -> Self {
        Self(number)
    }

    /// The number the compositor calls it.
    #[must_use]
    pub const fn number(self) -> u64 {
        self.0
    }
}

/// Which application, as this machine names it.
///
/// Its own type rather than a `String` at every call site, so that *which app*
/// and *which window* cannot be handed over the wrong way round — they are both
/// identifiers and they read alike in an argument list.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppId(String);

/// Why something is not an application's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnApp {
    /// Nothing, or only blank space.
    ///
    /// An application with no name is one a person cannot be told about: the
    /// screen reader announces the name and the previews are titled with it, so
    /// an empty one is a row somebody is asked to choose from with nothing in
    /// it.
    Unnamed,
}

impl AppId {
    /// The application this machine calls this.
    ///
    /// # Errors
    /// [`NotAnApp::Unnamed`] for a name that is empty or only blank space.
    pub fn named(name: &str) -> Result<Self, NotAnApp> {
        let name = name.trim();
        if name.is_empty() {
            return Err(NotAnApp::Unnamed);
        }
        Ok(Self(name.to_owned()))
    }

    /// What this machine calls it.
    ///
    /// **It is never translated.** An application's name came off this machine
    /// rather than out of a vocabulary, which is `alo-strings`' rule for such
    /// things: whoever puts it in a sentence fills it in rather than looking it
    /// up.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// How a window sits at this moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HowItSits {
    /// On the canvas, in its patch, where the person left it.
    OnTheCanvas,
    /// Put aside. It keeps its patch and the right panel holds its preview.
    PutAside,
    /// Filling a display, over everything including the Dock.
    ///
    /// The canvas is not what is showing it, so travelling to it is not a pan —
    /// which is why this is a state of its own rather than a very large patch.
    FillingTheScreen,
}

impl HowItSits {
    /// Whether the person would have to bring it back before they could use it.
    #[must_use]
    pub const fn was_put_aside(self) -> bool {
        matches!(self, Self::PutAside)
    }
}

/// One window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Which window.
    id: WindowId,
    /// Which application it belongs to.
    app: AppId,
    /// What the person calls it — the title a preview is named with.
    called: String,
    /// The part of the plane it occupies, kept whatever state it is in.
    at: Patch,
    /// How it sits at this moment.
    sits: HowItSits,
}

impl Window {
    /// A window of this application, called this, sitting here like this.
    #[must_use]
    pub fn of(id: WindowId, app: AppId, called: &str, at: Patch, sits: HowItSits) -> Self {
        Self {
            id,
            app,
            called: called.to_owned(),
            at,
            sits,
        }
    }

    /// Which window.
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Which application it belongs to.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        &self.app
    }

    /// What the person calls it.
    ///
    /// **Named previews need this and there is no fallback.** A preview headed
    /// with a number is a row a person cannot choose between, so a window with
    /// no title of its own is given the application's name by whoever makes it
    /// rather than being allowed to arrive empty.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// The part of the plane it occupies.
    #[must_use]
    pub const fn at(&self) -> Patch {
        self.at
    }

    /// How it sits at this moment.
    #[must_use]
    pub const fn sits(&self) -> HowItSits {
        self.sits
    }

    /// The same window, sitting differently. Its patch does not move.
    #[must_use]
    pub fn now_sits(mut self, sits: HowItSits) -> Self {
        self.sits = sits;
        self
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::on_the_canvas::Spot;

    fn a_patch() -> Patch {
        Patch::of(Spot::at(120, 80), 800, 600).unwrap()
    }

    /// **An application with no name is refused**, because its name is what a
    /// screen reader announces and what a preview is titled with.
    #[test]
    fn an_application_must_have_a_name() {
        assert_eq!(AppId::named(""), Err(NotAnApp::Unnamed));
        assert_eq!(AppId::named("   "), Err(NotAnApp::Unnamed));
        assert_eq!(AppId::named("  Browser  ").unwrap().name(), "Browser");
    }

    /// **Putting a window aside does not move it.** The patch it comes back to
    /// is the patch it left, which is the whole of *restore it at its saved
    /// canvas position*.
    #[test]
    fn a_window_put_aside_keeps_the_place_it_came_from() {
        let window = Window::of(
            WindowId::numbered(7),
            AppId::named("Browser").unwrap(),
            "Research",
            a_patch(),
            HowItSits::OnTheCanvas,
        );
        let aside = window.clone().now_sits(HowItSits::PutAside);

        assert_eq!(aside.at(), window.at(), "it did not move");
        assert!(aside.sits().was_put_aside());
        assert!(!window.sits().was_put_aside());

        let back = aside.now_sits(HowItSits::OnTheCanvas);
        assert_eq!(back.at(), window.at(), "and it came back to the same place");
    }

    /// Two windows of one application are told apart by number, not by name,
    /// because the same document opened twice has the same name twice.
    #[test]
    fn two_windows_of_one_application_are_told_apart_by_number() {
        let browser = AppId::named("Browser").unwrap();
        let one = Window::of(
            WindowId::numbered(1),
            browser.clone(),
            "Research",
            a_patch(),
            HowItSits::OnTheCanvas,
        );
        let other = Window::of(
            WindowId::numbered(2),
            browser.clone(),
            "Research",
            a_patch(),
            HowItSits::OnTheCanvas,
        );

        assert_eq!(one.app(), other.app());
        assert_eq!(one.called(), other.called());
        assert_ne!(one.id(), other.id(), "and they are still two windows");
    }

    /// **A name that came off this machine is kept exactly**, because it is
    /// filled into a sentence rather than looked up in a vocabulary, and a name
    /// this crate had tidied would be a name the person does not recognise.
    #[test]
    fn an_application_name_is_kept_exactly_as_the_machine_gave_it() {
        for given in ["Blender", "LibreOffice Writer", "alo Docs", "Ptyxis"] {
            assert_eq!(AppId::named(given).unwrap().name(), given);
        }
    }

    /// Filling the screen is a state of its own rather than a very large patch,
    /// because the canvas is not what is showing such a window.
    #[test]
    fn filling_the_screen_is_its_own_state() {
        let window = Window::of(
            WindowId::numbered(3),
            AppId::named("Blender").unwrap(),
            "Opening scene",
            a_patch(),
            HowItSits::FillingTheScreen,
        );
        assert!(!window.sits().was_put_aside());
        assert_eq!(
            window.at(),
            a_patch(),
            "it still has a place to come back to"
        );
    }
}
