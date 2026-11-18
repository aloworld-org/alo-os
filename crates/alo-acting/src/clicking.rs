//! The words on the compositor's door for **alo clicking once at a point of the
//! window in front** — the screen road's click
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*) and
//! `docs/contracts/application-door.md`, *`click-at`*.
//!
//! ```text
//! → click-at net.supertuxkart.SuperTuxKart 500 400
//! ← clicked
//!   (or: not-in-front, not-alos-copy, the-person-is-pressing, could-not-click)
//! ```
//!
//! **A point of that one window, as proportions of it**: thousandths of its
//! width from its left edge and of its height from its top edge, `0` to
//! [`ToClick::THE_WHOLE`]. Never pixels, never a point outside the window, and
//! no place for a second window, a drag or a held button: one press and one
//! release of the primary button, there.

use alo_applications::Application;

use crate::request::Request;

/// The word that asks for a click.
pub const CLICK_AT: &str = "click-at";

/// alo asking for one click at one point of the window in front.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToClick {
    /// The application whose window must have the keyboard.
    pub application: Application,
    /// Thousandths of the window's width, from its left edge.
    pub across: u16,
    /// Thousandths of the window's height, from its top edge.
    pub down: u16,
}

impl ToClick {
    /// The whole width or height, in the thousandths a point is given in.
    pub const THE_WHOLE: u16 = 1000;

    /// A click at this point, or [`None`] for a point outside the window.
    #[must_use]
    pub const fn at(application: Application, across: u16, down: u16) -> Option<Self> {
        if across > Self::THE_WHOLE || down > Self::THE_WHOLE {
            return None;
        }
        Some(Self {
            application,
            across,
            down,
        })
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{CLICK_AT} {} {} {}",
            self.application.identifier(),
            self.across,
            self.down
        )
    }

    /// One line, read — [`None`] when it is not about a click at all, so the
    /// door can read it as one of the other acts.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotToClick>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next()? != CLICK_AT {
            return None;
        }
        if line.len() > Request::THE_MOST {
            return Some(Err(NotToClick::TooLong));
        }
        let (Some(named), Some(across), Some(down), None) =
            (words.next(), words.next(), words.next(), words.next())
        else {
            return Some(Err(NotToClick::NotOnePoint));
        };
        let Ok(application) = Application::identified(named) else {
            return Some(Err(NotToClick::NotOnePoint));
        };
        let (Some(across), Some(down)) = (a_thousandth(across), a_thousandth(down)) else {
            return Some(Err(NotToClick::NotOnePoint));
        };
        Some(Self::at(application, across, down).ok_or(NotToClick::NotOnePoint))
    }
}

/// Why a line that began as a click is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotToClick {
    /// Longer than any line on this door may be.
    #[error("a line on this door is at most {} bytes", Request::THE_MOST)]
    TooLong,
    /// It named no application, not two numbers inside the window, or
    /// something after them.
    #[error(
        "a click names exactly one application and one point of its window, in thousandths from \
         0 to 1000, and nothing else"
    )]
    NotOnePoint,
}

/// What the door answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clicked {
    /// The primary button was pressed and released once at that point.
    Clicked,
    /// The window that has the keyboard is not one of that application's, or
    /// no window has it. Nothing was clicked.
    NotInFront,
    /// It is the person's own window of that application, and at their setting
    /// alo works only in its own copy. Nothing was clicked.
    NotAlosCopy,
    /// The person is holding a button down. Nothing was clicked, so alo never
    /// presses into the middle of what the person is doing.
    ThePersonIsPressing,
    /// The window was that application's and no click could be delivered to
    /// it: no pointer, or a window with no size. Nothing was clicked.
    CouldNotClick,
}

impl Clicked {
    /// The words on the wire.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Clicked => "clicked",
            Self::NotInFront => "not-in-front",
            Self::NotAlosCopy => "not-alos-copy",
            Self::ThePersonIsPressing => "the-person-is-pressing",
            Self::CouldNotClick => "could-not-click",
        }
    }

    /// One answer's line, read — [`None`] for a line that is not one.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        match line.trim_end_matches(['\r', '\n']) {
            "clicked" => Some(Self::Clicked),
            "not-in-front" => Some(Self::NotInFront),
            "not-alos-copy" => Some(Self::NotAlosCopy),
            "the-person-is-pressing" => Some(Self::ThePersonIsPressing),
            "could-not-click" => Some(Self::CouldNotClick),
            _ => None,
        }
    }
}

/// A number from 0 to 1000 in digits alone: `+5`, ` 5` and `05` are not one
/// the door wrote.
fn a_thousandth(said: &str) -> Option<u16> {
    if said.is_empty() || !said.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if said.len() > 1 && said.starts_with('0') {
        return None;
    }
    said.parse::<u16>()
        .ok()
        .filter(|number| *number <= ToClick::THE_WHOLE)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "clicking_tests.rs"]
mod tests;
