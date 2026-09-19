//! The three verbs an agent proposes a change to the printers through.
//!
//! ★ *Printers, solved* is on the agent's list, and a printer is a change to the
//! whole machine: a place documents can go, for everybody who uses it. So there
//! are three verbs and every one of them **is a change** — it waits for one
//! approval of the sentence naming the printer, and that approval makes the
//! change once, through the privileged broker, and by no other road:
//!
//! | verb | the sentence a person approves |
//! |---|---|
//! | `add_printer` | *set up the printer Brother HL-L2350DW series, so this machine can print on it* |
//! | `remove_printer` | *remove the printer Brother HL-L2350DW series from this machine* |
//! | `set_default_printer` | *print on Brother HL-L2350DW series from now on* |
//!
//! # A printer by its own name, and never by where it is
//!
//! The argument is the name the printer gave itself — one name, no path, no
//! control character, as `alo_capability::Takes::Name` holds every name to — so
//! the sentence reads the way the printer is listed, and no address, queue or
//! driver a model could have written ever reaches the sentence or anything
//! after it. Which printer that name is, is asked of the printing service at the
//! moment the change is carried out ([`crate::chosen`]); a name no printer has,
//! or two printers share, changes nothing.
//!
//! # Why none needs a grant
//!
//! A grant is over a path or an application, and a printer is neither: the
//! change is over the machine's own list of printers, which no grant covers.
//! What protects the person is the approval itself — one sentence, naming the
//! printer, answered once — and the reason is written into each declaration
//! where the next reader of the verb list finds it, and in ADR 0055 §2, as
//! `docs/contracts/agent-verbs.md` rule 5 asks.
//!
//! # What is deliberately not a verb
//!
//! **Finding printers.** A list of the devices near a machine is a fingerprint
//! of where it is, and `alo-printing` makes finding a person's act in Settings;
//! an agent proposes a printer the person named. **Printing** is
//! `alo-printing`'s own `print_document`, on a different list.

use alo_capability::{
    Arg, Authorised, Effect, Requires, Takes, Value, Verb, VerbError, Verbs, VerbsError,
};

use crate::wanted::{Change, Wanted};
use crate::words;

/// The name the verb that sets a printer up is asked for by.
pub const ADD_PRINTER: &str = "add_printer";

/// The name the verb that removes a printer is asked for by.
pub const REMOVE_PRINTER: &str = "remove_printer";

/// The name the verb that chooses the printer this machine prints on is asked
/// for by.
pub const SET_DEFAULT_PRINTER: &str = "set_default_printer";

/// The most characters a printer's name may be — `alo-printing`'s own bound on
/// a name it shows.
pub const LONGEST_NAME: usize = 127;

/// Why a printer verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// An authority that is not an approved change to the printers.
///
/// English and a `Display`, like `alo_capability::VerbError`: it cannot happen
/// because of anything a person did, only because an executor handed this
/// crate the wrong authority, and its reader is fixing that executor.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAPrinterChange {
    /// The authority is for another verb.
    #[error("{verb} is not one of the printers' verbs, so it changes no printer")]
    AnotherVerb {
        /// The verb it was for.
        verb: String,
    },
    /// The authority did not come from an approval.
    #[error("a change to the printers came with no approval, so it changes nothing")]
    NotApproved,
    /// The printer's name is missing or of the wrong kind.
    #[error("a change to the printers arrived without a printer's name")]
    Malformed,
}

/// The three verbs, as a list of their own.
///
/// # Errors
/// [`Declaring`], which the verbs as written cannot cause.
pub fn printer_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the three verbs on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of one of these names.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    for (name, purpose, sentence) in [
        (ADD_PRINTER, words::ADD_PURPOSE, words::ADD_SENTENCE),
        (
            REMOVE_PRINTER,
            words::REMOVE_PURPOSE,
            words::REMOVE_SENTENCE,
        ),
        (
            SET_DEFAULT_PRINTER,
            words::DEFAULT_PURPOSE,
            words::DEFAULT_SENTENCE,
        ),
    ] {
        verbs.declare(Verb::checked(
            name,
            purpose,
            Effect::Change,
            vec![Arg::taking(
                words::PRINTER,
                words::THE_PRINTER,
                Takes::name(LONGEST_NAME),
            )],
            Requires::nothing_because(
                "a printer is neither a path nor an application, the change is over this \
                 machine's own list of printers, and the person approves this one change by the \
                 sentence naming the printer",
            ),
            sentence,
        )?)?;
    }
    Ok(())
}

/// The change a person approved, from the authority that approval became.
///
/// # Errors
/// [`NotAPrinterChange`] for another verb's authority, one that came from no
/// approval, or one without a printer's name.
pub fn approved(authorised: &Authorised) -> Result<Wanted, NotAPrinterChange> {
    let change = match authorised.verb() {
        ADD_PRINTER => Change::Add,
        REMOVE_PRINTER => Change::Remove,
        SET_DEFAULT_PRINTER => Change::MakeDefault,
        other => {
            return Err(NotAPrinterChange::AnotherVerb {
                verb: other.to_owned(),
            });
        }
    };
    if authorised.from_approval().is_none() {
        return Err(NotAPrinterChange::NotApproved);
    }
    let Some(Value::Name(called)) = authorised.call().value(words::PRINTER) else {
        return Err(NotAPrinterChange::Malformed);
    };
    Ok(Wanted::of(change, called))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::{CallError, Given};

    /// **Three verbs, every one a change, none needing a grant, each saying
    /// why.**
    #[test]
    fn three_verbs_every_one_a_change_that_needs_no_grant_and_says_why() {
        let verbs = printer_verbs().unwrap();
        assert_eq!(verbs.len(), 3);
        for name in [ADD_PRINTER, REMOVE_PRINTER, SET_DEFAULT_PRINTER] {
            let verb = verbs.of(name).unwrap();
            assert_eq!(verb.effect(), Effect::Change, "{name}");
            assert!(
                matches!(verb.requires(), Requires::Nothing { reason } if reason.contains("approves")),
                "{name}"
            );
        }
    }

    /// **The sentence names the printer**, filled from the validated argument.
    #[test]
    fn the_sentence_a_person_approves_names_the_printer() {
        let verbs = printer_verbs().unwrap();
        let strings = in_english();
        for (name, sentence) in [
            (
                ADD_PRINTER,
                "set up the printer Brother HL-L2350DW series, so this machine can print on it",
            ),
            (
                REMOVE_PRINTER,
                "remove the printer Brother HL-L2350DW series from this machine",
            ),
            (
                SET_DEFAULT_PRINTER,
                "print on Brother HL-L2350DW series from now on",
            ),
        ] {
            let call = verbs
                .call(
                    name,
                    &[(words::PRINTER, Given::text(" Brother HL-L2350DW series "))],
                )
                .unwrap();
            assert!(call.waits_for_approval(), "{name}");
            assert_eq!(call.sentence(&strings).text(), sentence);
        }
    }

    /// **Nothing shaped like an address, a path, a command or a second line
    /// becomes a call**, so none of it reaches a sentence, let alone the broker.
    #[test]
    fn a_printer_named_by_anything_but_a_name_never_becomes_a_call() {
        let verbs = printer_verbs().unwrap();
        for given in [
            Given::text("ipp://192.168.1.20/ipp/print"),
            Given::text("/dev/usb/lp0"),
            Given::text("../../etc/cups/printers.conf"),
            Given::text("Brother\nreboot"),
            Given::text(""),
            Given::text("x".repeat(LONGEST_NAME + 1)),
            Given::number(7),
        ] {
            for name in [ADD_PRINTER, REMOVE_PRINTER, SET_DEFAULT_PRINTER] {
                assert!(
                    verbs
                        .call(name, &[(words::PRINTER, given.clone())])
                        .is_err(),
                    "{name} took {given:?}"
                );
            }
        }
    }

    /// **No verb finds printers, prints, or configures anything else**, so an
    /// agent can ask for none of them here.
    #[test]
    fn no_verb_finds_printers_or_configures_anything_else() {
        let verbs = printer_verbs().unwrap();
        for name in [
            "find_printers",
            "list_printers",
            "configure_printer",
            "set_printer_driver",
            "add_printer_by_address",
            "print_document",
        ] {
            assert!(matches!(
                verbs.call(name, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// A name already taken is not replaced.
    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = printer_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }
}
