//! What may keep this machine awake: a closed list of three, each named.
//!
//! *Why won't this laptop sleep* is a question every operating system makes
//! hard to answer, because anything may hold a machine awake and nothing has to
//! say so. Here the list is closed and every entry on it has a sentence:
//!
//! 1. **the person's own setting** ([`Keeper::YourSetting`]);
//! 2. **an application holding the inhibit portal under a grant**
//!    ([`Keeper::AnApplication`], named by its identifier);
//! 3. **a turn that is running**, for its own length and no longer
//!    ([`Keeper::TheAgent`]).
//!
//! There is no fourth variant a service, a daemon or an agent's request could
//! become, and no variant that holds the machine awake without being named.

use alo_capability::{Applicant, Grantee};
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// One thing holding this machine awake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keeper {
    /// The person set the machine to stay awake.
    YourSetting,
    /// An application asked through the inhibit portal, and a grant allowed it.
    AnApplication(Applicant),
    /// A turn the person started is still running, for this agent.
    TheAgent(Grantee),
}

impl Keeper {
    /// The string this crate declares for this keeper.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::YourSetting => words::KEPT_AWAKE_BY_YOUR_SETTING,
            Self::AnApplication(_) => words::KEPT_AWAKE_BY_AN_APPLICATION,
            Self::TheAgent(_) => words::KEPT_AWAKE_BY_THE_AGENT,
        }
    }

    /// Why this machine is staying awake, in the language the person reads.
    ///
    /// An application is named by its identifier and by nothing it said about
    /// itself. The agent is not named at all: the shell shows its mark and word
    /// beside this (ADR 0010), and a name here would be a second one.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::AnApplication(application) => {
                Filling::of("application", application.as_str().to_owned())
            }
            Self::YourSetting | Self::TheAgent(_) => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **Every keeper is named**, with nothing left unfilled, and an
    /// application by its identifier.
    #[test]
    fn every_keeper_says_why_the_machine_is_awake() {
        let strings = in_english();
        for keeper in [
            Keeper::YourSetting,
            Keeper::AnApplication(Applicant::named("org.libreoffice.Impress")),
            Keeper::TheAgent(Grantee::named("@files")),
        ] {
            let said = keeper.said(&strings);
            assert!(!said.is_a_bug(), "{keeper:?}: {said}");
            assert!(said.unfilled().is_empty(), "{keeper:?}: {said}");
            assert!(
                said.text()
                    .starts_with("This machine is staying awake because"),
                "{said}"
            );
        }
        assert!(
            Keeper::AnApplication(Applicant::named("org.libreoffice.Impress"))
                .said(&strings)
                .text()
                .contains("org.libreoffice.Impress")
        );
        assert!(
            !Keeper::TheAgent(Grantee::named("@files"))
                .said(&strings)
                .text()
                .contains("@files")
        );
    }
}
