//! The two measuring verbs, declared.
//!
//! `docs/features.md` promises *what is running, and what it is using* and
//! *what is filling the disk* at v0.5, and the agent's *"why is it slow?"* is
//! a verb — which ADR 0001 says is a thing on a closed list, checked against
//! a grant, recorded. This file is those two verbs, declared the way
//! `docs/contracts/agent-verbs.md` says a verb has to be and the way
//! `alo-files` declares its six; every rule about *how* is
//! [`alo_capability::verb`]'s, and this file would not compile past its own
//! tests if a declaration broke one.
//!
//! **Nothing is declared by default.** [`measuring_verbs`] hands back a
//! list; [`declare_into`] puts them on somebody else's. A registry nobody
//! gave these to cannot ask what is running, which is the honest starting
//! state for a machine where every capability is somebody's decision.
//!
//! # Both are reads, and both need a grant over the folder they read
//!
//! Each answers a question and changes nothing, so each is
//! [`Effect::Read`]: it runs inside the turn, nobody is asked to approve it,
//! and the record's entry has no approval to name. Each still requires a
//! grant — *read inside the turn* is about approval and never about reach —
//! and the grant is over the one folder the verb reads.
//!
//! For `what_is_filling` that is the folder being counted, which is what a
//! person picks. For `what_is_running` it is the kernel's own directory of
//! what is running, `/proc`, and the verb takes it as an argument rather
//! than assuming it for two reasons. The first is honesty: every number the
//! answer holds is read from a file under that directory, so the grant that
//! permits the verb names exactly what the verb reads, and revoking it stops
//! the verb the same instant it would stop `list_folder`. The second is
//! testability: a kernel a test wrote out into a directory of its own is
//! measured by exactly the code that measures the real one. What a person
//! grants is the process list, and the grants panel says so in the one
//! vocabulary it has — a folder, until a time.
//!
//! # No interval on the verb
//!
//! *What is using the machine* is a rate, and a rate is two readings with
//! time between them. That time is not an argument here: a count of seconds
//! the model chooses would be a way to make a turn wait as long as the model
//! likes, and the interval a person's window refreshes at is not the
//! model's to set. Whoever carries the verb out chooses it and passes it in,
//! which is [`crate::Measured::of`].
//!
//! # Where the words come from
//!
//! Every sentence in this file is [`crate::words`]'. The purpose, each
//! argument's purpose and the sentence a person reads are the constants a
//! translator is handed, passed to `alo_capability::Verb::checked` so that
//! the string being translated is the string that was checked.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// Why the measuring verbs could not be declared.
///
/// Neither can happen to the two as they are written — the tests in this
/// file say so. It is a `Result` because a library that panics on its own
/// declaration takes the daemon with it, and because [`declare_into`] can
/// genuinely fail against a list that already holds one of these names.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The measuring verbs, as a list of their own.
///
/// ```
/// use alo_capability::Given;
/// use alo_measuring::{measuring_verbs, measuring_words};
/// use alo_strings::Strings;
///
/// let verbs = measuring_verbs()?;
/// let call = verbs.call("what_is_running", &[("proc", Given::text("/proc"))])
///     .expect("the verb takes the kernel's directory");
///
/// let strings = Strings::of(measuring_words()?);
/// assert_eq!(
///     call.sentence(&strings).text(),
///     "list what is running and what it is using, read from /proc",
/// );
/// // A read: it answers inside the turn, and nobody is asked to approve it.
/// assert!(!call.waits_for_approval());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// [`Declaring`], which the two as written cannot cause.
pub fn measuring_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the measuring verbs on an existing list.
///
/// Both or neither, as `alo-files` does with its six: a name already taken
/// is found before anything is added, so a list never ends up holding half
/// a set of capabilities that nobody chose.
///
/// # Errors
/// [`Declaring::List`] if the list already holds one of these names — a name
/// means one thing, so whoever took it first keeps it and nothing is
/// silently replaced.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let declaring = [what_is_running()?, what_is_filling()?];
    for verb in &declaring {
        if verbs.of(verb.name()).is_some() {
            return Err(Declaring::List(VerbsError::AlreadyDeclared {
                name: verb.name().to_owned(),
            }));
        }
    }
    for verb in declaring {
        verbs.declare(verb)?;
    }
    Ok(())
}

/// List what is running and what each process is using, read from the
/// kernel's own directory of it.
fn what_is_running() -> Result<Verb, VerbError> {
    Verb::checked(
        "what_is_running",
        words::WHAT_IS_RUNNING,
        Effect::Read,
        vec![Arg::taking(
            "proc",
            words::WHAT_IS_RUNNING_PROC,
            Takes::Path,
        )],
        Requires::grants_over(["proc"]),
        words::WHAT_IS_RUNNING_SENTENCE,
    )
}

/// Count what is filling a folder, as a tree of sizes.
fn what_is_filling() -> Result<Verb, VerbError> {
    Verb::checked(
        "what_is_filling",
        words::WHAT_IS_FILLING,
        Effect::Read,
        vec![Arg::taking(
            "folder",
            words::WHAT_IS_FILLING_FOLDER,
            Takes::Path,
        )],
        Requires::grants_over(["folder"]),
        words::WHAT_IS_FILLING_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{Ask, CallError, Given};
    use alo_strings::Strings;

    /// This crate's words beside the capability model's, with nothing
    /// translated — a refusal at the door is worded by the deciding crate.
    fn in_english() -> Strings {
        let mut vocabulary = crate::measuring_words().unwrap();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// The two `docs/features.md` promised, and no third that arrived with
    /// them.
    #[test]
    fn the_list_is_the_two_measurements() {
        let verbs = measuring_verbs().unwrap();
        let names: Vec<_> = verbs.all().map(Verb::name).collect();
        assert_eq!(names, ["what_is_running", "what_is_filling"]);
    }

    /// **Both are reads.** Each runs inside the turn, nobody approves it,
    /// and that is a property of the verb rather than of whoever calls it.
    #[test]
    fn both_are_reads_that_nobody_is_asked_to_approve() {
        for verb in measuring_verbs().unwrap().all() {
            assert_eq!(verb.effect(), Effect::Read, "{}", verb.name());
            assert!(!verb.effect().waits_for_approval(), "{}", verb.name());
        }
    }

    /// **Each requires a grant over the one folder it reads**, and a call
    /// asks the grants about exactly that folder.
    #[test]
    fn each_requires_a_grant_over_the_folder_it_reads() {
        let verbs = measuring_verbs().unwrap();
        assert_eq!(
            verbs.of("what_is_running").unwrap().requires(),
            &Requires::Grants(vec!["proc".to_owned()])
        );
        assert_eq!(
            verbs.of("what_is_filling").unwrap().requires(),
            &Requires::Grants(vec!["folder".to_owned()])
        );
        let running = verbs
            .call("what_is_running", &[("proc", Given::text("/proc"))])
            .unwrap();
        assert_eq!(running.asks(), &[Ask::path("/proc")]);
        let filling = verbs
            .call(
                "what_is_filling",
                &[("folder", Given::text("/home/anna/Pictures"))],
            )
            .unwrap();
        assert_eq!(filling.asks(), &[Ask::path("/home/anna/Pictures")]);
    }

    /// The sentence a person reads while each runs names its one argument.
    #[test]
    fn what_a_person_reads_is_a_sentence_naming_the_folder() {
        let strings = in_english();
        let verbs = measuring_verbs().unwrap();
        let running = verbs
            .call("what_is_running", &[("proc", Given::text("/proc"))])
            .unwrap()
            .sentence(&strings);
        assert_eq!(
            running.text(),
            "list what is running and what it is using, read from /proc"
        );
        assert!(!running.is_a_bug() && running.unfilled().is_empty());
        let filling = verbs
            .call(
                "what_is_filling",
                &[("folder", Given::text("/home/anna/Pictures"))],
            )
            .unwrap()
            .sentence(&strings);
        assert_eq!(filling.text(), "count what is filling /home/anna/Pictures");
        assert!(!filling.is_a_bug() && filling.unfilled().is_empty());
    }

    /// A call that does not survive the door is not a call: a folder that is
    /// not a full path, one that steps upwards, a missing argument, an
    /// argument the verb does not take — each refused before any grant is
    /// consulted, and there is no interval to send.
    #[test]
    fn a_call_that_does_not_survive_the_door_is_not_a_call() {
        let strings = in_english();
        let verbs = measuring_verbs().unwrap();
        let relative = verbs
            .call("what_is_filling", &[("folder", Given::text("Pictures"))])
            .unwrap_err();
        assert!(matches!(relative, CallError::Argument(_)), "{relative:?}");
        assert!(relative.said(&strings).text().contains("full path"));

        let upwards = verbs
            .call("what_is_running", &[("proc", Given::text("/proc/../etc"))])
            .unwrap_err();
        assert!(matches!(upwards, CallError::Argument(_)), "{upwards:?}");

        let missing = verbs.call("what_is_running", &[]).unwrap_err();
        assert!(missing.said(&strings).text().contains("proc"));

        let interval = verbs
            .call(
                "what_is_running",
                &[("proc", Given::text("/proc")), ("over", Given::number(5))],
            )
            .unwrap_err();
        assert!(matches!(interval, CallError::NoSuchArgument { .. }));
    }

    /// The list is closed, and these two do not open it: nothing here
    /// signals, stops or renices anything.
    #[test]
    fn nothing_that_acts_on_a_process_is_on_the_list() {
        let verbs = measuring_verbs().unwrap();
        for asked in [
            "kill_process",
            "stop_process",
            "renice_process",
            "exec",
            "empty_folder",
        ] {
            assert!(verbs.of(asked).is_none(), "{asked}");
            assert!(matches!(
                verbs.call(asked, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// A list that already holds one of these names keeps its own, and gets
    /// none of the others either.
    #[test]
    fn a_name_already_taken_is_not_quietly_replaced_and_the_rest_do_not_arrive() {
        let mut verbs = measuring_verbs().unwrap();
        let again = declare_into(&mut verbs).unwrap_err();
        assert!(matches!(again, Declaring::List(_)), "{again}");
        assert_eq!(verbs.len(), 2);

        let theirs = measuring_verbs().unwrap();
        let mut mine = Verbs::default();
        mine.declare(theirs.of("what_is_filling").unwrap().clone())
            .unwrap();
        let clash = declare_into(&mut mine).unwrap_err();
        assert!(clash.to_string().contains("what_is_filling"), "{clash}");
        assert_eq!(mine.len(), 1);
        assert!(mine.of("what_is_running").is_none());
    }

    /// The two sit beside the file verbs on one list, which is the
    /// arrangement a daemon has: one registry, every crate declaring into it.
    #[test]
    fn they_join_a_list_that_already_holds_the_file_verbs() {
        let mut verbs = alo_files::file_verbs().unwrap();
        declare_into(&mut verbs).unwrap();
        assert_eq!(verbs.len(), 8);
        assert!(verbs.of("what_is_running").is_some());
        assert!(verbs.of("list_folder").is_some());
    }

    /// **Every word the two are declared with is one this crate declares.**
    /// A constant left out of [`crate::words`]' list would compile, declare
    /// and reach a person as a key in the place where the sentence belongs.
    #[test]
    fn everything_the_two_say_is_something_this_crate_declares() {
        let strings = in_english();
        for verb in measuring_verbs().unwrap().all() {
            let purpose = verb.purpose(&strings);
            assert!(!purpose.is_a_bug(), "{}: {purpose}", verb.name());
            for arg in verb.args() {
                let said = arg.purpose(&strings);
                assert!(!said.is_a_bug(), "{} {}: {said}", verb.name(), arg.name());
            }
        }
        let verbs = measuring_verbs().unwrap();
        assert_eq!(
            verbs.of("what_is_running").unwrap().sentence().key(),
            &words::WHAT_IS_RUNNING_SENTENCE.key()
        );
        assert_eq!(
            verbs.of("what_is_filling").unwrap().sentence().key(),
            &words::WHAT_IS_FILLING_SENTENCE.key()
        );
    }

    /// Nothing arrives declared.
    #[test]
    fn nothing_is_declared_until_somebody_declares_it() {
        let empty = Verbs::default();
        assert!(empty.of("what_is_running").is_none());
        let mut mine = Verbs::default();
        declare_into(&mut mine).unwrap();
        assert_eq!(mine.len(), 2);
    }
}
