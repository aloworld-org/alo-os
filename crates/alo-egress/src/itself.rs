//! One thing alo OS is about to do on its own, and the line a person reads
//! while it happens.
//!
//! [`Leaving`](crate::Leaving) is an egress an agent causes: it carries a
//! [`Grantee`](alo_capability::Grantee), because everything an agent does is
//! under somebody's authority and the record says whose. This is the other
//! kind, and the difference is a missing field rather than an extra one.
//!
//! **There is no agent here, and inventing one would be the mistake.** The
//! obvious shape — give alo OS a name and let it hold grants like anything else
//! — would say that the system acts under a grant, and it does not: nobody
//! granted their machine permission to sign them in. Worse, it would put alo OS
//! inside the capability model, where every question about what an agent may do
//! would then have an answer about the system beside it. So an errand has a
//! reason and a destination and nothing else: there is no `agent()` here
//! answering `None`, because the type has no room for one to be missing from.
//!
//! **It is on the same indicator, and that is the point.** A person watching
//! one light for *nothing has left this machine* must not have a second, unlit
//! one somewhere else. So [`Indicator::beginning_on_its_own`] puts this on the
//! list [`Indicator::beginning`] puts an agent's egress on, and
//! [`Indicator::is_quiet`] is false while alo OS is fetching a model. The
//! promise being kept is stronger than *no telemetry*: it is *nothing at all
//! that you cannot see*.
//!
//! [`Indicator::beginning`]: crate::Indicator::beginning
//! [`Indicator::beginning_on_its_own`]: crate::Indicator::beginning_on_its_own
//! [`Indicator::is_quiet`]: crate::Indicator::is_quiet

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::destination::Destination;
use crate::errand::Errand;

/// One errand, about to happen.
///
/// Serialises so the indicator's contents can be shown or written down, and
/// deliberately does **not** deserialise — like [`Leaving`](crate::Leaving),
/// and for the same reason. One read back off a disk would be an errand nothing
/// decided about, holding a destination nobody checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnItsOwn {
    /// Why alo OS is doing this.
    errand: Errand,
    /// Where it is reaching.
    destination: Destination,
}

impl OnItsOwn {
    /// An errand alo OS is about to run, reaching here.
    #[must_use]
    pub fn for_(errand: Errand, destination: Destination) -> Self {
        Self {
            errand,
            destination,
        }
    }

    /// Why alo OS is doing this.
    #[must_use]
    pub fn errand(&self) -> Errand {
        self.errand
    }

    /// Where it is reaching.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// The line a person reads at the moment it happens.
    ///
    /// One sentence per reason, whole, with the preposition inside it — item
    /// 9h's decision about the three an agent causes, kept for these three.
    /// English wants *at* before an identity service and *from* before a
    /// catalogue, and a language that inflects the place needs the whole
    /// sentence in front of it to choose.
    ///
    /// The place goes in through [`Destination::fills`], not as text, so the
    /// line is only as translated as the place named in the middle of it.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::egress_words`] answers with the
    /// key, marked, and `Said::is_a_bug`. **What alo OS is doing never depends
    /// on the string table** — this describes an errand that is already
    /// happening, and calling it cannot change what it is.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &self.errand.word().key(),
            &self
                .destination
                .fills("destination", Filling::nothing(), strings),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use crate::words;
    use alo_strings::{Strings, Vocabulary};

    fn catalogue() -> Destination {
        Destination::at("models.alo.example").unwrap()
    }

    /// Each of the three says what alo OS is doing and where, in a sentence a
    /// person can act on. *Something is happening* would be a diagnostic.
    #[test]
    fn the_line_a_person_reads_says_what_the_machine_is_doing_and_where() {
        let strings = in_english();
        assert_eq!(
            OnItsOwn::for_(Errand::FetchingAModel, catalogue())
                .said(&strings)
                .text(),
            "alo OS is fetching a model from models.alo.example"
        );
        assert_eq!(
            OnItsOwn::for_(
                Errand::SigningIn,
                Destination::at("identity.alo.example").unwrap()
            )
            .said(&strings)
            .text(),
            "alo OS is signing you in at identity.alo.example"
        );
        assert_eq!(
            OnItsOwn::for_(
                Errand::CheckingForAnUpdate,
                Destination::at("updates.alo.example").unwrap()
            )
            .said(&strings)
            .text(),
            "alo OS is checking for an update at updates.alo.example"
        );
    }

    /// **There is no agent, and the type has no room for one.** An errand
    /// carries a reason and a place; whose authority it is under is a question
    /// with no answer here, rather than an answer of *the system*.
    #[test]
    fn an_errand_carries_a_reason_and_a_place_and_nothing_else() {
        let errand = OnItsOwn::for_(Errand::FetchingAModel, catalogue());
        assert_eq!(errand.errand(), Errand::FetchingAModel);
        assert_eq!(errand.destination(), &catalogue());

        let written = serde_json::to_string(&errand).unwrap();
        assert!(written.contains("fetching-a-model"), "{written}");
        assert!(!written.contains("agent"), "{written}");
        assert!(!written.contains("grantee"), "{written}");
    }

    /// **The line and the place named in it are one language**, as an agent's
    /// line is: a German machine does not read a German sentence with an
    /// English clause in the middle of it.
    #[test]
    fn an_errand_line_and_the_place_in_it_are_one_language() {
        let strings = translated(&[
            (
                words::ALO_IS_FETCHING_A_MODEL,
                "alo OS holt ein Modell von {destination}",
            ),
            (words::A_PAIRED_MACHINE, "{machine}, in Ihrem Netz"),
        ]);
        let said = OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::paired("the studio workstation").unwrap(),
        )
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("in Ihrem Netz"), "{said}");
        assert!(!said.text().contains("on your network"), "{said}");
    }

    /// **A line is only as translated as the place named in the middle of it**,
    /// which is item 15's rule and holds here for the same reason it holds for
    /// an agent's line: the half a reader could not read is the half saying
    /// where their machine is reaching.
    #[test]
    fn an_errand_line_naming_an_untranslated_place_does_not_claim_to_be_translated() {
        let half = translated(&[(
            words::ALO_IS_FETCHING_A_MODEL,
            "alo OS holt ein Modell von {destination}",
        )]);
        let said = OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::paired("the studio workstation").unwrap(),
        )
        .said(&half);
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().starts_with("alo OS holt"), "{said}");
        assert!(said.text().contains("on your network"), "{said}");
    }

    /// **A host is data, and data cannot make a line untranslated.** Nobody
    /// translates `models.alo.example`, so a German line naming one is a German
    /// line — the same rule an agent's line is held to.
    #[test]
    fn a_line_naming_a_host_is_as_translated_as_it_reads() {
        let strings = translated(&[(
            words::ALO_IS_CHECKING_FOR_AN_UPDATE,
            "alo OS sucht bei {destination} nach einer Aktualisierung",
        )]);
        let said = OnItsOwn::for_(
            Errand::CheckingForAnUpdate,
            Destination::at("updates.alo.example").unwrap(),
        )
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert_eq!(
            said.text(),
            "alo OS sucht bei updates.alo.example nach einer Aktualisierung"
        );
    }

    /// **The indicator does not go blank because nobody declared the words.**
    /// The line still says something is happening and names the key, which is
    /// how whoever has to fix it finds out.
    #[test]
    fn a_line_without_the_words_still_says_the_machine_is_doing_something() {
        let said =
            OnItsOwn::for_(Errand::SigningIn, catalogue()).said(&Strings::of(Vocabulary::empty()));
        assert!(said.is_a_bug());
        assert!(said.text().contains("egress.itself.signing-in"), "{said}");
    }

    /// **The refusal is at the door, and it is the destination's.** An address
    /// that cannot be shown on one line never becomes an errand, because it
    /// never becomes a [`Destination`] — so the indicator cannot be handed a
    /// line it could not draw.
    #[test]
    fn an_address_that_cannot_be_shown_never_becomes_an_errand() {
        assert!(Destination::at("updates.alo.example\u{1b}[2K").is_err());
        assert!(Destination::at("   ").is_err());
    }
}
