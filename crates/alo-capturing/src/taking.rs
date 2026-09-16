//! Taking the picture: the act, and everything refused before it.
//!
//! [`Screenshot::of`] is the constructor and it is where the refusals are.
//! That ordering is the design: **a picture that may not be taken is not a
//! value that exists and then declines to be taken — it is a value that cannot
//! be built.** [`Screenshot::take`] is the only thing that can reach
//! [`crate::Grabs`], and nothing can hold a [`Screenshot`] of the lock screen
//! or of another person's window to call it with.
//!
//! # The four questions, in this order
//!
//! 1. **Is the lock screen up?** Then nothing is captured, whatever was asked
//!    for: the whole screen is the lock screen, a region of it is a region of
//!    the lock screen, and a window underneath it belongs to somebody who has
//!    not proved they are back at the keyboard.
//! 2. **Is this window the lock screen's own?** A compositor can hold that
//!    surface while the session behind it is already unlocked, and a capture
//!    asked for in that instant must not get it.
//! 3. **Is this window somebody else's?** Several people are signed in to one
//!    alo machine; a window in another person's session is theirs.
//! 4. **Is this rectangle on the screen?** A selection with no width, or one
//!    hanging over an edge, is not an area of the screen.
//!
//! # The clipboard is handed over whatever the destination
//!
//! [`Screenshot::take`] always takes the machine's one clipboard, and reaches
//! it only when the destination says to. That is deliberate: *never both
//! unasked* is a property somebody can watch — the clipboard goes in, comes
//! back untouched, and a test says so — rather than one they have to take on
//! trust because the code never had the opportunity.
//!
//! # Nothing is sent anywhere
//!
//! There are two destinations, a file and the clipboard, and both are on this
//! machine. Nothing here opens a connection, and nothing this crate depends on
//! can: `tests/nothing_leaves_when_a_picture_is_taken.rs` reads the manifests
//! and says so. The plan's constraint — *nothing is uploaded, shared or sent
//! anywhere by taking a screenshot* — has no code to enforce it because there
//! is no code that could break it.

use alo_clipboard::Clipboard;

use crate::grabs::Grabs;
use crate::on_this_day::OnThisDay;
use crate::refusing::NotTaken;
use crate::screen::Screen;
use crate::session::Session;
use crate::taken::Taken;
use crate::to_the_clipboard;
use crate::what::What;
use crate::where_it_goes::WhereItGoes;
use crate::writing;

/// A picture of the screen that may be taken.
///
/// Holding one is knowing that the lock screen is down, that what is being
/// captured belongs to the session that asked, and that the rectangle is on the
/// screen. There is no other constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screenshot {
    /// What is being captured.
    what: What,
    /// The screen it is on.
    on: Screen,
    /// Where the picture is to go.
    where_it_goes: WhereItGoes,
}

impl Screenshot {
    /// A picture of this, in this session, going here.
    ///
    /// # Errors
    /// [`NotTaken::TheLockScreen`], [`NotTaken::AnotherPersonsWindow`] and
    /// [`NotTaken::NotAnAreaOfTheScreen`], asked in that order. Nothing has
    /// been captured when any of them is answered: the rented mechanism is not
    /// reachable from here at all.
    pub fn of(
        what: What,
        in_session: &Session,
        on: Screen,
        where_it_goes: WhereItGoes,
    ) -> Result<Self, NotTaken> {
        if in_session.the_lock_screen_is_up() {
            return Err(NotTaken::TheLockScreen);
        }
        if let Some(window) = what.window() {
            if window.is_the_lock_screen() {
                return Err(NotTaken::TheLockScreen);
            }
            if window.whose() != in_session.whose() {
                return Err(NotTaken::AnotherPersonsWindow);
            }
        }
        if !what.across(on).is_on(on) {
            return Err(NotTaken::NotAnAreaOfTheScreen);
        }
        Ok(Self {
            what,
            on,
            where_it_goes,
        })
    }

    /// What is being captured.
    #[must_use]
    pub const fn what(&self) -> &What {
        &self.what
    }

    /// Where the picture is to go.
    #[must_use]
    pub const fn where_it_goes(&self) -> &WhereItGoes {
        &self.where_it_goes
    }

    /// Take it.
    ///
    /// The moment is handed in, as `now` is everywhere in this repository:
    /// nothing here reads a clock, and the picture's name is made from the
    /// moment the caller says it was taken.
    ///
    /// # Errors
    /// [`NotTaken::NotGrabbed`] when the rented mechanism would not hand a
    /// picture over, and [`NotTaken::NoRoomForAName`],
    /// [`NotTaken::NotWritten`] or [`NotTaken::NotCopied`] when the picture was
    /// taken and could not be kept.
    pub fn take(
        &self,
        at: OnThisDay,
        screen: &mut dyn Grabs,
        clipboard: &mut Clipboard,
    ) -> Result<Taken, NotTaken> {
        let picture = screen.grab(self.what.across(self.on), self.on)?;
        match &self.where_it_goes {
            WhereItGoes::AFile(folder) => {
                let at = writing::write(&picture, folder, at)?;
                Ok(Taken::Saved { at })
            }
            WhereItGoes::TheClipboard => {
                to_the_clipboard::copy(clipboard, picture)?;
                Ok(Taken::Copied)
            }
            // Written first: a picture that reached the disk and then could not
            // reach the clipboard is still somebody's picture, and the refusal
            // names the half that did not happen.
            WhereItGoes::AFileAndTheClipboard(folder) => {
                let at = writing::write(&picture, folder, at)?;
                to_the_clipboard::copy(clipboard, picture)?;
                Ok(Taken::SavedAndCopied { at })
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::region::Region;
    use crate::session::WhoseSession;
    use crate::testing::{
        a_moment, a_picture, a_screen, an_empty_folder, anna, nothing_reads_the_screen,
        one_frame_of_the_screen,
    };
    use crate::window::{Window, WindowId};

    /// **The whole screen, one window and a region are all pictures that can be
    /// taken** — the plan's three, through one constructor.
    #[test]
    fn the_whole_screen_one_window_and_a_region_can_all_be_taken() {
        let session = Session::of(&anna());
        let hers = Window::of(
            WindowId::recorded(7),
            &anna(),
            Region::of(100, 50, 400, 300).unwrap(),
        );
        for what in [
            What::TheWholeScreen,
            What::OneWindow(hers),
            What::APartOfIt(Region::of(0, 0, 10, 10).unwrap()),
        ] {
            let taking = Screenshot::of(
                what.clone(),
                &session,
                a_screen(),
                WhereItGoes::the_clipboard(),
            );
            assert!(taking.is_ok(), "{what:?} was refused");
        }
    }

    /// **The lock screen cannot be captured, whatever was asked for.** The
    /// whole screen, a window and a region are all refused while it is up, and
    /// the rented mechanism is never reached.
    #[test]
    fn nothing_is_captured_while_the_lock_screen_is_up() {
        let locked = Session::behind_the_lock_screen(&anna());
        let hers = Window::of(
            WindowId::recorded(7),
            &anna(),
            Region::of(100, 50, 400, 300).unwrap(),
        );
        for what in [
            What::TheWholeScreen,
            What::OneWindow(hers),
            What::APartOfIt(Region::of(0, 0, 10, 10).unwrap()),
        ] {
            assert_eq!(
                Screenshot::of(what, &locked, a_screen(), WhereItGoes::the_clipboard()),
                Err(NotTaken::TheLockScreen)
            );
        }
    }

    /// **The lock screen's own window cannot be captured either**, even in a
    /// session that is not locked — which is the instant a compositor still
    /// holds that surface while the lock screen comes down.
    #[test]
    fn the_lock_screens_own_window_cannot_be_captured() {
        let session = Session::of(&anna());
        let lock = Window::the_lock_screen(
            WindowId::recorded(1),
            &anna(),
            Region::of(0, 0, 1920, 1080).unwrap(),
        );
        assert_eq!(
            Screenshot::of(
                What::OneWindow(lock),
                &session,
                a_screen(),
                WhereItGoes::the_clipboard()
            ),
            Err(NotTaken::TheLockScreen)
        );
    }

    /// **A window in another person's session cannot be captured.** Several
    /// people are signed in to one machine, and a window that is not yours is
    /// not yours.
    #[test]
    fn another_persons_window_cannot_be_captured() {
        let session = Session::of(&anna());
        let bos = Window::of(
            WindowId::recorded(9),
            &WhoseSession::of("bo"),
            Region::of(0, 0, 400, 300).unwrap(),
        );
        assert_eq!(
            Screenshot::of(
                What::OneWindow(bos),
                &session,
                a_screen(),
                WhereItGoes::the_clipboard()
            ),
            Err(NotTaken::AnotherPersonsWindow)
        );
    }

    /// **A rectangle that is not on the screen is refused**, so nothing asks
    /// the rented mechanism to cut a shape that has no answer.
    #[test]
    fn a_rectangle_off_the_screen_is_refused() {
        let session = Session::of(&anna());
        let off = Region::of(1900, 0, 100, 100).unwrap();
        assert_eq!(
            Screenshot::of(
                What::APartOfIt(off),
                &session,
                a_screen(),
                WhereItGoes::the_clipboard()
            ),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );

        // A window whose rectangle hangs over an edge is refused the same way.
        let hanging = Window::of(WindowId::recorded(7), &anna(), off);
        assert_eq!(
            Screenshot::of(
                What::OneWindow(hanging),
                &session,
                a_screen(),
                WhereItGoes::the_clipboard()
            ),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );
    }

    /// **A picture to a file goes nowhere near the clipboard.** The machine's
    /// one clipboard is handed in and comes back holding nothing, which is what
    /// *never both unasked* means when somebody is watching.
    #[test]
    fn a_picture_to_a_file_leaves_the_clipboard_alone() {
        let folder = an_empty_folder("to-a-file");
        let taking = Screenshot::of(
            What::TheWholeScreen,
            &Session::of(&anna()),
            a_screen(),
            WhereItGoes::a_file(&folder),
        )
        .unwrap();

        let mut clipboard = Clipboard::nothing_copied_yet();
        let taken = taking
            .take(a_moment(), &mut one_frame_of_the_screen(), &mut clipboard)
            .unwrap();

        assert!(matches!(taken, Taken::Saved { .. }));
        assert!(clipboard.offered().is_none(), "the clipboard was touched");
        assert_eq!(
            std::fs::read(taken.at().unwrap()).unwrap(),
            a_picture().bytes()
        );
    }

    /// **A picture to the clipboard writes no file.** Somebody taking one to
    /// paste into a message leaves nothing behind in a folder.
    #[test]
    fn a_picture_to_the_clipboard_writes_no_file() {
        let folder = an_empty_folder("to-the-clipboard");
        let taking = Screenshot::of(
            What::TheWholeScreen,
            &Session::of(&anna()),
            a_screen(),
            WhereItGoes::the_clipboard(),
        )
        .unwrap();

        let mut clipboard = Clipboard::nothing_copied_yet();
        let taken = taking
            .take(a_moment(), &mut one_frame_of_the_screen(), &mut clipboard)
            .unwrap();

        assert_eq!(taken, Taken::Copied);
        assert!(clipboard.offered().is_some());
        assert_eq!(
            std::fs::read_dir(folder.at()).unwrap().count(),
            0,
            "something was written"
        );
    }

    /// **Both happens when somebody asked for both**, and then the file and the
    /// clipboard hold the same picture.
    #[test]
    fn both_happens_when_somebody_asked_for_both() {
        let folder = an_empty_folder("to-both");
        let taking = Screenshot::of(
            What::TheWholeScreen,
            &Session::of(&anna()),
            a_screen(),
            WhereItGoes::a_file_and_the_clipboard(&folder),
        )
        .unwrap();

        let mut clipboard = Clipboard::nothing_copied_yet();
        let taken = taking
            .take(a_moment(), &mut one_frame_of_the_screen(), &mut clipboard)
            .unwrap();

        assert!(matches!(taken, Taken::SavedAndCopied { .. }));
        assert!(taken.is_on_the_clipboard());
        assert_eq!(
            std::fs::read(taken.at().unwrap()).unwrap(),
            a_picture().bytes()
        );
    }

    /// **A mechanism that would not hand a picture over is a refusal**, and
    /// nothing is written and nothing is copied.
    #[test]
    fn a_mechanism_that_hands_nothing_over_writes_nothing_and_copies_nothing() {
        let folder = an_empty_folder("no-mechanism");
        let taking = Screenshot::of(
            What::TheWholeScreen,
            &Session::of(&anna()),
            a_screen(),
            WhereItGoes::a_file_and_the_clipboard(&folder),
        )
        .unwrap();

        let mut clipboard = Clipboard::nothing_copied_yet();
        let refused = taking
            .take(a_moment(), &mut nothing_reads_the_screen(), &mut clipboard)
            .unwrap_err();

        assert!(matches!(refused, NotTaken::NotGrabbed(_)), "{refused:?}");
        assert!(clipboard.offered().is_none());
        assert_eq!(std::fs::read_dir(folder.at()).unwrap().count(), 0);
    }
}
