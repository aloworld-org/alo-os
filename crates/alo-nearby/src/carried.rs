//! How a proposal is spelt on the wire, and how one that arrived is read.
//!
//! One line:
//!
//! ```text
//! alo-os/1 proposal <asking> <asked> <seconds> <offer> <may,may>
//! ```
//!
//! Seven fields, and every one of them is on the list task 7 of the
//! local-network plan names: both identities, the enumerated list, the
//! duration in whole seconds, and the asking machine's [`Offer`]. Nothing
//! else has a place. A line with an eighth field is refused rather than read
//! around, for the reason `reading.rs` refuses an advertisement that says
//! more than presence: reading around it would let a later version quietly
//! start saying more, and everything on a network can read this line.
//!
//! What is *not* here is as deliberate as what is: no name the person gave
//! either machine, no address, no clock, no account, and nothing about what
//! either machine holds. The name a person reads for the other machine is
//! theirs to give on their own machine, and never crosses.

use std::time::Duration;

use crate::deliberating::Proposal;
use crate::keying::Offer;
use crate::machine::MachineId;
use crate::pairing::NotPaired;
use crate::permitting::MayAskIts;
use crate::refusing::NotNearby;

/// What every proposal begins with on the wire, and which version it speaks.
const ON_THE_WIRE: &str = "alo-os/1";

/// The word that says what kind of line this is.
const PROPOSAL: &str = "proposal";

/// How the arms of the list are kept apart inside one field.
const BETWEEN_ARMS: char = ',';

impl Proposal {
    /// The proposal as it travels: one line.
    #[must_use]
    pub fn said(&self) -> String {
        let may = self
            .may()
            .iter()
            .map(|arm| arm.said())
            .collect::<Vec<_>>()
            .join(&BETWEEN_ARMS.to_string());
        format!(
            "{ON_THE_WIRE} {PROPOSAL} {} {} {} {} {may}",
            self.asking(),
            self.asked(),
            self.lasting().as_secs(),
            self.offered().said()
        )
    }

    /// A proposal somebody sent, read back and checked for shape and for
    /// everything [`Proposal::checked`] checks of any proposal — and for
    /// nothing else: whether it is accepted where it arrived is
    /// [`crate::Proposals::arrived`]'s to answer.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAProposal`] for anything that is not exactly the line
    /// [`said`](Self::said) writes, including a field this crate has no place
    /// for and an arm this crate's list does not have;
    /// [`NotNearby::NotAnOffer`] for an offer that does not read; and
    /// [`NotNearby::NotAMachineIdentity`] for a machine that is not one.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        let fields: Vec<&str> = said.trim().split(' ').collect();
        let &[version, kind, asking, asked, seconds, offer, may] = fields.as_slice() else {
            return Err(NotNearby::NotAProposal(format!(
                "{} fields rather than seven",
                fields.len()
            )));
        };
        if version != ON_THE_WIRE {
            return Err(NotNearby::NotAProposal(format!(
                "`{version}` is not a version spoken here"
            )));
        }
        if kind != PROPOSAL {
            return Err(NotNearby::NotAProposal(format!(
                "`{kind}` is not a proposal"
            )));
        }
        let seconds: u64 = seconds
            .parse()
            .map_err(|_| NotNearby::NotAProposal("the seconds are not a number".to_owned()))?;
        let offer = Offer::read(offer)?;
        let may = may
            .split(BETWEEN_ARMS)
            .map(|arm| {
                MayAskIts::read(arm).ok_or_else(|| {
                    NotNearby::NotAProposal(format!(
                        "`{arm}` is not something a pairing may permit"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::checked(
            MachineId::read(asking)?,
            MachineId::read(asked)?,
            &may,
            Duration::from_secs(seconds),
            offer,
        )
        .map_err(|why| NotNearby::NotAProposal(refused_because(why).to_owned()))
    }
}

/// Why a line that is shaped like a proposal is still not one, in a sentence
/// for a log — the person's sentence is [`NotPaired::said`], and nobody is
/// shown a proposal that was refused here.
const fn refused_because(why: NotPaired) -> &'static str {
    match why {
        NotPaired::WithItself => "it names one machine as both asking and asked",
        NotPaired::NothingAsked => "it asks for nothing",
        NotPaired::NoTime => "it would last no time",
        NotPaired::TooLong => "it would last longer than a pairing may",
        NotPaired::OnlyOneSideAgreed
        | NotPaired::NoEnd
        | NotPaired::NotWithThatMachine
        | NotPaired::NoKey
        | NotPaired::NotTheOfferMade => "it was refused as a proposal",
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use crate::deliberating::{AT_MOST, Proposal};
    use crate::keying::Keying;
    use crate::permitting::MayAskIts;
    use crate::refusing::NotNearby;
    use crate::testing::{reception, studio};

    /// A proposal for a day, for both of the studio's things.
    fn a_proposal() -> Proposal {
        Proposal::checked(
            reception(),
            studio(),
            &[MayAskIts::Workspace, MayAskIts::Models],
            Duration::from_secs(86_400),
            Keying::fresh().unwrap().offer().clone(),
        )
        .unwrap()
    }

    /// **A proposal is read back as it was written**, with every field on the
    /// list and nothing beside it: exactly seven fields.
    #[test]
    fn a_proposal_written_here_is_read_back_and_has_exactly_seven_fields() {
        let proposal = a_proposal();
        let said = proposal.said();
        assert_eq!(said.split(' ').count(), 7, "{said}");
        assert_eq!(Proposal::read(&said).unwrap(), proposal);
        assert!(said.starts_with("alo-os/1 proposal "));
        assert!(said.contains(" 86400 "));
        assert!(said.ends_with(" models,workspace"));
    }

    /// **A field this crate has no place for is refused rather than read
    /// around**, and so is everything else that is not the line.
    #[test]
    fn a_proposal_with_a_field_not_on_the_list_is_refused() {
        let good = a_proposal().said();
        let fields: Vec<&str> = good.split(' ').collect();
        let with = |at: usize, replaced: &str| -> String {
            fields
                .iter()
                .enumerate()
                .map(|(n, field)| if n == at { replaced } else { field })
                .collect::<Vec<_>>()
                .join(" ")
        };
        let mut with_a_field_missing = good.clone();
        with_a_field_missing.truncate(good.rfind(' ').unwrap());
        for not_one in [
            String::new(),
            format!("{good} disan"),
            format!("{good} name=reception"),
            with_a_field_missing,
            with(0, "alo-os/2"),
            with(1, "confirmed"),
            with(2, "disan-laptop"),
            with(4, "a-day"),
            with(4, "0"),
            with(4, &(AT_MOST.as_secs() + 1).to_string()),
            with(5, "00"),
            with(6, "everything"),
            with(6, "models,verbs"),
            with(6, ""),
            with(3, fields.get(2).copied().unwrap_or_default()),
        ] {
            assert!(
                matches!(
                    Proposal::read(&not_one).unwrap_err(),
                    NotNearby::NotAProposal(_)
                        | NotNearby::NotAnOffer(_)
                        | NotNearby::NotAMachineIdentity(_)
                ),
                "`{not_one}` was read as a proposal"
            );
        }
    }

    /// An offer that does not read is refused as an offer, before anything
    /// else about the line is considered a proposal.
    #[test]
    fn an_offer_that_does_not_read_is_refused_as_an_offer() {
        let good = a_proposal().said();
        let fields: Vec<&str> = good.split(' ').collect();
        let with_a_bad_offer = fields
            .iter()
            .enumerate()
            .map(|(n, field)| if n == 5 { "not-an-offer" } else { field })
            .collect::<Vec<_>>()
            .join(" ");
        assert!(matches!(
            Proposal::read(&with_a_bad_offer).unwrap_err(),
            NotNearby::NotAnOffer(_)
        ));
    }
}
