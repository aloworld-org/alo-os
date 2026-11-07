//! **A person's own command, carrying out an adapter's declared ability** —
//! what `alo-app <application> <ability> [values]` stands on
//! (`docs/contracts/terminal-commands.md`, section 2, *Declared*).
//!
//! The person typed it, so nothing here asks for a grant or an approval: law 2
//! binds the agent, not the person. **Everything else is the agent's road
//! exactly.** The verb is found by the same lookup, the application is asked
//! about by the same question, the message is built from the same validated
//! [`Call`] by the same code, and it is sent once by the same send
//! (`crate::driving`). A value alo's call would refuse is refused here too,
//! because the [`Call`] this takes exists only once the same validation has
//! passed it.

use alo_applications::Installed;
use alo_capability::Call;
use alo_strings::{Said, Strings};

use crate::adapters::Adapters;
use crate::delivering::{Delivers, NotDelivered};
use crate::driving::{NotMessaged, sent, the_message, whose_verb};

/// A declared ability a person's command sent to its application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrivenByHand {
    /// Whether the application did not answer in time.
    unanswered: bool,
}

impl DrivenByHand {
    /// Whether it was sent and the application did not answer, so whether it
    /// happened is not known (`terminal-commands.md`, exit 75).
    #[must_use]
    pub const fn unanswered(self) -> bool {
        self.unanswered
    }
}

/// Why a declared ability was not carried out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotDrivenByHand {
    /// No loaded adapter declares it.
    NotDeclared(Said),
    /// Its application is not installed.
    NotInstalled(Said),
    /// The call could not be put into the application's terms, which a loaded
    /// adapter cannot cause.
    NotBuilt(Said),
    /// It was sent and nothing was done: the application is not there, does
    /// not offer it, or refused.
    NotDelivered(NotDelivered, Said),
}

impl NotDrivenByHand {
    /// What a person is told — the same sentence the agent's road says.
    #[must_use]
    pub fn said(&self) -> &Said {
        match self {
            Self::NotDeclared(said)
            | Self::NotInstalled(said)
            | Self::NotBuilt(said)
            | Self::NotDelivered(_, said) => said,
        }
    }

    /// Whether the application itself was reached and answered — as opposed to
    /// refused before anything was sent to it.
    #[must_use]
    pub const fn reached_the_application(&self) -> bool {
        matches!(
            self,
            Self::NotDelivered(NotDelivered::Refused | NotDelivered::DoesNotOffer, _)
        )
    }
}

/// Carry out one validated call of an adapter's declared verb, once.
///
/// # Errors
/// [`NotDrivenByHand`]: nothing was done in the application.
pub fn drive_by_hand(
    adapters: &Adapters,
    installed: &Installed,
    call: &Call,
    to: &dyn Delivers,
    strings: &Strings,
) -> Result<DrivenByHand, NotDrivenByHand> {
    let (adapter, verb) =
        whose_verb(adapters, call.verb(), strings).map_err(NotDrivenByHand::NotDeclared)?;
    let message = the_message(adapter, verb, call, installed, strings).map_err(|(why, said)| {
        match why {
            NotMessaged::NotInstalled => NotDrivenByHand::NotInstalled(said),
            NotMessaged::NotBuilt => NotDrivenByHand::NotBuilt(said),
        }
    })?;
    match sent(&message, to) {
        Ok(unanswered) => Ok(DrivenByHand { unanswered }),
        Err(why) => Err(NotDrivenByHand::NotDelivered(
            why,
            why.said(adapter.application, strings),
        )),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::RefCell;

    use alo_applications::Application;
    use alo_capability::Given;
    use alo_strings::Vocabulary;

    use super::*;
    use crate::message::Message;
    use crate::verbs::{adapter_verbs, shipped_adapters};

    /// A bus that keeps every message and answers as it was told to.
    struct ABus {
        /// What it answers.
        answering: Result<(), NotDelivered>,
        /// Everything sent to it.
        sent: RefCell<Vec<Message>>,
    }

    impl Delivers for ABus {
        fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
            self.sent.borrow_mut().push(message.clone());
            self.answering
        }
    }

    fn answering(answering: Result<(), NotDelivered>) -> ABus {
        ABus {
            answering,
            sent: RefCell::new(Vec::new()),
        }
    }

    fn strings() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        alo_applications::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    fn opening(document: &str) -> Call {
        adapter_verbs()
            .unwrap()
            .call(
                "text_editor.open_document",
                &[("document", Given::text(document))],
            )
            .unwrap()
    }

    fn the_editor_installed() -> Installed {
        Installed::holding([Application::identified("org.gnome.TextEditor").unwrap()])
    }

    /// **One call, one message, and no grant asked for**: the person typed it.
    #[test]
    fn a_persons_call_sends_exactly_one_message_and_asks_no_grant() {
        let bus = answering(Ok(()));
        let driven = drive_by_hand(
            &shipped_adapters().unwrap(),
            &the_editor_installed(),
            &opening("/home/anna/today.txt"),
            &bus,
            &strings(),
        )
        .unwrap();
        assert!(!driven.unanswered());
        let sent = bus.sent.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent.first().unwrap().destination(), "org.gnome.TextEditor");
        assert_eq!(sent.first().unwrap().method(), "Open");
    }

    /// **Not installed is refused before anything is sent**, in the words the
    /// agent's road says.
    #[test]
    fn an_application_not_installed_is_refused_and_nothing_is_sent() {
        let bus = answering(Ok(()));
        let refused = drive_by_hand(
            &shipped_adapters().unwrap(),
            &Installed::nothing(),
            &opening("/home/anna/today.txt"),
            &bus,
            &strings(),
        )
        .unwrap_err();
        assert!(matches!(refused, NotDrivenByHand::NotInstalled(_)));
        assert!(!refused.reached_the_application());
        assert!(bus.sent.borrow().is_empty());
    }

    /// **No answer is not a refusal**: it was sent and may have happened.
    #[test]
    fn an_application_that_does_not_answer_may_have_done_it() {
        let bus = answering(Err(NotDelivered::DidNotAnswer));
        let driven = drive_by_hand(
            &shipped_adapters().unwrap(),
            &the_editor_installed(),
            &opening("/home/anna/today.txt"),
            &bus,
            &strings(),
        )
        .unwrap();
        assert!(driven.unanswered());
        for (answer, reached) in [
            (NotDelivered::Refused, true),
            (NotDelivered::DoesNotOffer, true),
            (NotDelivered::NotThere, false),
        ] {
            let refused = drive_by_hand(
                &shipped_adapters().unwrap(),
                &the_editor_installed(),
                &opening("/home/anna/today.txt"),
                &answering(Err(answer)),
                &strings(),
            )
            .unwrap_err();
            assert_eq!(refused.reached_the_application(), reached, "{answer:?}");
            assert!(!refused.said().text().is_empty());
        }
    }

    /// **A verb no adapter declares sends nothing.**
    #[test]
    fn a_verb_no_adapter_declares_is_refused() {
        let bus = answering(Ok(()));
        let reading = alo_capability::Verbs::default();
        assert!(reading.call("text_editor.open_document", &[]).is_err());
        let refused = drive_by_hand(
            &crate::adapters::Adapters::default(),
            &the_editor_installed(),
            &opening("/home/anna/today.txt"),
            &bus,
            &strings(),
        )
        .unwrap_err();
        assert!(matches!(refused, NotDrivenByHand::NotDeclared(_)));
        assert!(bus.sent.borrow().is_empty());
    }
}
