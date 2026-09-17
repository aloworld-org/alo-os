//! Why something that asked to keep this machine awake is not keeping it awake.

use alo_capability::Applicant;
use alo_strings::{Filling, Said, Strings};

use crate::logind::NotHeld;
use crate::words;

/// A hold that was not taken.
///
/// There is no `Display`: the only road to words is [`NotKeptAwake::said`], in
/// the language the person reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotKeptAwake {
    /// What was offered was an application's permission for something other
    /// than keeping the machine awake.
    NotAskingToStayAwake(Applicant),
    /// The application's grant does not allow it at this moment — revoked,
    /// expired, or never made. `alo-portals`' own refusal, carried whole.
    NotAllowed(alo_portals::Refused),
    /// The turn is over, or has stopped, so there is nothing running to keep
    /// the machine awake for.
    TheAgentIsNotWorking,
    /// The machine would not take the hold.
    NotHeld(NotHeld),
}

impl NotKeptAwake {
    /// What a person is told.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAskingToStayAwake(application) => strings.say(
                &words::NOT_ASKING_TO_STAY_AWAKE.key(),
                &Filling::of("application", application.as_str().to_owned()),
            ),
            Self::NotAllowed(refused) => refused.said(strings),
            Self::TheAgentIsNotWorking => {
                strings.say(&words::THE_AGENT_IS_NOT_WORKING.key(), &Filling::nothing())
            }
            Self::NotHeld(not_held) => not_held.said(strings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// Every refusal this crate words itself is declared and filled.
    #[test]
    fn every_refusal_this_crate_words_is_said() {
        let strings = in_english();
        for refused in [
            NotKeptAwake::NotAskingToStayAwake(Applicant::named("org.gnome.Cheese")),
            NotKeptAwake::TheAgentIsNotWorking,
            NotKeptAwake::NotHeld(NotHeld {
                machine: "no bus".to_owned(),
            }),
        ] {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
        }
    }
}
