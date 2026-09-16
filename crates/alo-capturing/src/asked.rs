//! An application asking for a picture of the screen, through the portal.
//!
//! The plan's last acceptance for this task: **an application asking for a
//! screenshot through the portal is judged by `alo-portals` against its grant
//! before anything is taken.** Both halves of that sentence are held here, and
//! by the shape of the types rather than by the order of some lines.
//!
//! **Judged by `alo-portals`.** [`Asked`] holds an `alo_portals::Request` for
//! `alo_portals::Portal::Screenshot` and asks it `judged`. Nothing in this
//! crate decides whether an application may photograph a screen, nothing here
//! holds a `&mut Grants`, and there is no list of trusted applications
//! anywhere: what a grant covers, whose it is and when it ends are ADR 0040's
//! and `alo-capability`'s, and a second opinion here could disagree with the
//! one the machine actually enforces.
//!
//! **Before anything is taken.** [`ForAnApplication`] is the only value with a
//! `take` on it for an application's picture, and [`Asked::judged`] is its only
//! constructor. An application's screenshot that was not judged is therefore
//! not a thing that can be attempted — the rented mechanism is not reachable
//! from a refusal — and `nothing_is_taken_before_the_grants_are_asked` watches
//! a mechanism that counts how often it was asked and finds it at zero.
//!
//! # A person is not an application
//!
//! [`crate::Screenshot`] has a `take` of its own and needs no grant, because a
//! person photographing their own screen is not an agent or an application
//! doing something: ADR 0001's capability model is about what acts *on* a
//! person's behalf without them. That is the same line `alo-picking` and
//! `alo-granted` draw, and it is why there is no verb in this crate at all —
//! the agent's picture of the screen is the plan's task 6, through a verb whose
//! proposal says *a picture of your screen* and which is approved like any
//! other change.

use std::time::SystemTime;

use alo_capability::Grants;
use alo_clipboard::Clipboard;
use alo_portals::{Allowed, NotARequest, Portal, Request};

use crate::grabs::Grabs;
use crate::on_this_day::OnThisDay;
use crate::refusing::NotTaken;
use crate::taken::Taken;
use crate::taking::Screenshot;

/// An application asking this machine for a picture of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// The request, in the portal's own terms.
    request: Request,
}

impl Asked {
    /// The application with this identifier, asking for a picture of the
    /// screen.
    ///
    /// # Errors
    /// `alo_portals::NotARequest`, when what arrived was never a request —
    /// an identifier that is not one, which that crate words and this one does
    /// not restate.
    pub fn of(application: &str) -> Result<Self, NotARequest> {
        Ok(Self {
            request: Request::of(application, Portal::Screenshot)?,
        })
    }

    /// The request, in the portal's own terms.
    #[must_use]
    pub const fn request(&self) -> &Request {
        &self.request
    }

    /// This picture, if the application's grant covers one.
    ///
    /// # Errors
    /// [`NotTaken::NotAllowed`], carrying `alo-portals`' own refusal — the
    /// application that holds no grant at all, or the grant that expired or was
    /// never made. Nothing has been captured: this is the only road to an
    /// application's [`ForAnApplication::take`].
    pub fn judged(
        &self,
        taking: Screenshot,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<ForAnApplication, NotTaken> {
        match self.request.judged(grants, now) {
            Ok(against) => Ok(ForAnApplication { taking, against }),
            Err(why) => Err(NotTaken::NotAllowed(why)),
        }
    }
}

/// A picture of the screen an application was allowed.
///
/// No public constructor: [`Asked::judged`] is the only one, so an
/// application's picture cannot be taken without its grant having been asked
/// first.
#[derive(Debug, Clone)]
pub struct ForAnApplication {
    /// What was allowed.
    taking: Screenshot,
    /// Which grants allowed it, kept so that whatever writes the answer down
    /// can say what it was allowed against.
    against: Allowed,
}

impl ForAnApplication {
    /// What was allowed.
    #[must_use]
    pub const fn taking(&self) -> &Screenshot {
        &self.taking
    }

    /// Which grants allowed it.
    #[must_use]
    pub const fn against(&self) -> &Allowed {
        &self.against
    }

    /// Take it.
    ///
    /// # Errors
    /// Everything [`Screenshot::take`] can answer with. Nothing about the
    /// grants is asked again: it was asked once, and asking twice would be two
    /// answers to one question.
    pub fn take(
        &self,
        at: OnThisDay,
        screen: &mut dyn Grabs,
        clipboard: &mut Clipboard,
    ) -> Result<Taken, NotTaken> {
        self.taking.take(at, screen, clipboard)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::session::Session;
    use crate::testing::{
        THE_HOUR, a_moment, a_screen, anna, counting_what_it_was_asked, granted_a_screenshot, noon,
        nothing_granted, one_frame_of_the_screen,
    };
    use crate::what::What;
    use crate::where_it_goes::WhereItGoes;
    use alo_portals::Refused;

    /// A picture of the whole screen, to the clipboard.
    fn a_picture_of_the_screen() -> Screenshot {
        Screenshot::of(
            What::TheWholeScreen,
            &Session::of(&anna()),
            a_screen(),
            WhereItGoes::the_clipboard(),
        )
        .unwrap()
    }

    /// **An application the person granted a picture of the screen may take
    /// one**, and the grants that allowed it come back with it.
    #[test]
    fn an_application_that_was_granted_this_may_take_it() {
        let grants = granted_a_screenshot("com.example.Notes");
        let asked = Asked::of("com.example.Notes").unwrap();
        let allowed = asked
            .judged(a_picture_of_the_screen(), &grants, noon())
            .unwrap();

        assert!(!allowed.against().against().is_empty());
        let mut clipboard = Clipboard::nothing_copied_yet();
        let taken = allowed
            .take(a_moment(), &mut one_frame_of_the_screen(), &mut clipboard)
            .unwrap();
        assert_eq!(taken, Taken::Copied);
    }

    /// **An application nobody granted anything is refused**, in the grants'
    /// own words, before any question reaches a person.
    #[test]
    fn an_application_nobody_granted_anything_is_refused() {
        let asked = Asked::of("com.example.Stranger").unwrap();
        let refused = asked
            .judged(a_picture_of_the_screen(), &nothing_granted(), noon())
            .unwrap_err();

        assert!(matches!(
            refused.refused(),
            Some(Refused::NothingGranted { .. })
        ));
    }

    /// **An application granted something else is refused this.** A camera
    /// grant is not a grant to photograph the screen, which is ADR 0040's
    /// table, decided in `alo-portals` and never restated here.
    #[test]
    fn an_application_granted_something_else_is_refused_this() {
        let grants = crate::testing::granted_the_camera("com.example.VideoCall");
        let asked = Asked::of("com.example.VideoCall").unwrap();
        let refused = asked
            .judged(a_picture_of_the_screen(), &grants, noon())
            .unwrap_err();

        assert!(matches!(
            refused.refused(),
            Some(Refused::NotAllowed { .. })
        ));
    }

    /// **A grant that has expired is not a grant.** The same application, the
    /// same request, an hour later.
    #[test]
    fn a_grant_that_ended_is_not_a_grant() {
        let grants = granted_a_screenshot("com.example.Notes");
        let asked = Asked::of("com.example.Notes").unwrap();
        assert!(
            asked
                .judged(a_picture_of_the_screen(), &grants, noon() + THE_HOUR)
                .is_err()
        );
    }

    /// **Nothing is taken before the grants are asked.** The mechanism counts
    /// how many times it was asked for a picture, and after a refused request
    /// it is at zero — which is what *before anything is taken* means when
    /// somebody checks.
    #[test]
    fn nothing_is_taken_before_the_grants_are_asked() {
        let mut mechanism = counting_what_it_was_asked();
        let asked = Asked::of("com.example.Stranger").unwrap();
        let refused = asked.judged(a_picture_of_the_screen(), &nothing_granted(), noon());

        assert!(refused.is_err());
        assert_eq!(mechanism.how_often(), 0);

        // And the refusal carries no way of taking one anyway: there is no
        // value here with a `take` on it. The one below is the allowed road.
        let grants = granted_a_screenshot("com.example.Notes");
        let allowed = Asked::of("com.example.Notes")
            .unwrap()
            .judged(a_picture_of_the_screen(), &grants, noon())
            .unwrap();
        let mut clipboard = Clipboard::nothing_copied_yet();
        allowed
            .take(a_moment(), &mut mechanism, &mut clipboard)
            .unwrap();
        assert_eq!(mechanism.how_often(), 1);
    }

    /// **It is the screenshot portal that is asked**, and not another: a
    /// request for one picture of the screen is not a request to record it.
    #[test]
    fn the_portal_asked_is_the_screenshot_portal() {
        let asked = Asked::of("com.example.Notes").unwrap();
        assert_eq!(asked.request().portal(), Portal::Screenshot);
        assert_ne!(asked.request().portal(), Portal::ScreenCapture);
    }

    /// **An identifier that is not one is not a request**, and is refused in
    /// `alo-portals`' words rather than in a second set written here.
    #[test]
    fn an_identifier_that_is_not_one_is_not_a_request() {
        assert!(Asked::of("").is_err());
        assert!(Asked::of("   ").is_err());
    }
}
