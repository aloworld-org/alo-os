//! The egress indicator's place in the status area: the compositor half of
//! `alo_indicator`'s seam.
//!
//! `alo-indicator` decides what the indicator says and hands a compositor one
//! `alo_indicator::Drawn` whenever that changes; this is what it hands it to.
//! [`EgressStatus`] keeps the last picture it accepted so every frame can draw
//! it, and refuses — in `alo-indicator`'s own words — when the session has no
//! output for it to be drawn on.
//!
//! # It shows; it never decides
//!
//! Nothing here asks a policy, begins or ends a departure, or counts one. The
//! only way a picture arrives is `alo_indicator::Indicating::show`, which makes
//! it from the machine's own `alo_egress::Indicator`, so there is nothing this
//! type could draw that was not a departure — and nothing it could leave out,
//! because it has no way to choose which lines to keep.
//!
//! # No dismissing, no hiding, no setting that turns it off
//!
//! There is no method here that takes a picture away. One `EgressStatus`
//! belongs to one output for as long as that output exists. When the output
//! goes, the shell drops it and asks `Indicating::show` with no compositor —
//! `alo-indicator`'s refusal, which has a sentence and forgets what was drawn
//! — and a new output gets a new `EgressStatus`, which is told again on the
//! next call. A toggle on this type would be the one thing that could not
//! work: `Indicating` asks a compositor only when what is leaving changes, so
//! a status area switched back on would wait, untold, for the machine to move.

use alo_indicator::{Compositor, Drawn, SurfaceRefused};

/// The egress indicator as the status area holds it, for one output.
///
/// ```
/// use alo_indicator::Indicating;
/// use alo_shell::EgressStatus;
///
/// let mut status = EgressStatus::on_an_output();
/// let mut indicating = Indicating::nowhere();
/// let indicator = alo_egress::Indicator::default();
/// assert_eq!(
///     indicating.show(Some(&mut status), &indicator),
///     alo_indicator::Drew::Shown,
/// );
/// assert!(status.is_told());
/// ```
#[derive(Debug)]
pub struct EgressStatus {
    /// The last picture accepted, or `None` before the first.
    showing: Option<Drawn>,
    /// Whether there is an output to draw it on.
    output: bool,
}

impl EgressStatus {
    /// The status area of an output that exists, not yet told what is leaving.
    #[must_use]
    pub fn on_an_output() -> Self {
        Self {
            showing: None,
            output: true,
        }
    }

    /// A session with no output at all: every picture is refused, so the
    /// person is told in words rather than left looking at nothing.
    #[must_use]
    pub fn with_no_output() -> Self {
        Self {
            showing: None,
            output: false,
        }
    }

    /// Whether this has been told what is leaving, and can therefore be drawn.
    ///
    /// A frame is refused until it has: a status area drawn before anyone told
    /// it would look exactly like a machine on which nothing is leaving.
    #[must_use]
    pub fn is_told(&self) -> bool {
        self.showing.is_some()
    }

    /// The picture to draw, when it has been told one.
    pub(crate) fn showing(&self) -> Option<&Drawn> {
        self.showing.as_ref()
    }
}

impl Compositor for EgressStatus {
    fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused> {
        if !self.output {
            return Err(SurfaceRefused::NothingToShowOn);
        }
        self.showing = Some(drawn);
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::egress_status_testing::{asking_a_provider, noon, words};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_indicator::{Drew, Indicating, NotShown};

    /// **What the status area holds is what the machine's indicator says**,
    /// the moment it changes.
    #[test]
    fn it_holds_what_the_indicator_hands_it() {
        let mut status = EgressStatus::on_an_output();
        let mut indicating = Indicating::nowhere();
        let mut indicator = Indicator::default();
        assert!(!status.is_told());

        assert_eq!(indicating.show(Some(&mut status), &indicator), Drew::Shown);
        assert!(status.showing().unwrap().lamp().is_dark());

        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        assert_eq!(indicating.show(Some(&mut status), &indicator), Drew::Shown);
        assert_eq!(status.showing().unwrap().lines().len(), 1);

        assert!(indicator.ended(departing));
        assert_eq!(indicating.show(Some(&mut status), &indicator), Drew::Shown);
        assert!(status.showing().unwrap().lamp().is_dark());
    }

    /// **With no output the answer is `alo-indicator`'s refusal**, which has a
    /// sentence in the machine's vocabulary, and nothing is left believing it
    /// is on a screen.
    #[test]
    fn with_no_output_it_refuses_in_the_indicators_words() {
        let mut status = EgressStatus::with_no_output();
        let mut indicating = Indicating::nowhere();
        let refused = indicating.show(Some(&mut status), &Indicator::default());
        assert_eq!(
            refused,
            Drew::Refused(NotShown::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert!(!status.is_told());
        assert!(!indicating.is_showing());
        if let Drew::Refused(refusal) = refused {
            let said = refusal.said(&words());
            assert!(!said.is_a_bug(), "{said}");
        }
    }

    /// **An output that went away while something was leaving, and came back,
    /// is drawn lit again** — not left untold because nothing about the
    /// machine changed while it was gone.
    #[test]
    fn an_output_that_came_back_is_told_again() {
        let mut indicating = Indicating::nowhere();
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        let mut first = EgressStatus::on_an_output();
        assert_eq!(indicating.show(Some(&mut first), &indicator), Drew::Shown);

        // The output goes: its status area goes with it, and the indicator is
        // asked with nowhere to show it.
        drop(first);
        assert_eq!(
            indicating.show(None, &indicator),
            Drew::Refused(NotShown::NoCompositor)
        );

        let mut back = EgressStatus::on_an_output();
        assert!(!back.is_told());
        assert_eq!(indicating.show(Some(&mut back), &indicator), Drew::Shown);
        assert!(back.showing().unwrap().lamp().is_lit());
        assert!(indicator.ended(departing));
    }
}
