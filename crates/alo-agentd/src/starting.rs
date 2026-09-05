//! Everything between a process beginning and a service running.
//!
//! `crate::serving` is the service once it is running, and every other file here
//! is one decision it is made of. This is the join: what has to be true before
//! any of it happens, and what a machine is assembled out of once it is.
//!
//! It is a module rather than the `main` itself because of what a `main` cannot
//! be — a `main` reads `/etc/alo/agentd.toml`, opens a socket in the session it
//! was started inside, and writes into a record on a real disk, so a decision
//! left in one is a decision with a comment instead of a test. What is in
//! `src/main.rs` is the order these happen in and what an exit code means.
//!
//! # The order, and why it is that order
//!
//! 1. **Who this process is, and that it is not root.** ADR 0001 §2, and it is
//!    first because nothing else is worth deciding if the answer is no.
//! 2. **The description.** Everything the service is told rather than decides,
//!    and `crate::trusting` has already refused a file that could have been
//!    written by somebody else.
//! 3. **The vocabulary** — before the record, and that is not tidiness. A record
//!    that will not open is refused in `alo-keeping`'s own words, and a process
//!    that had not loaded a vocabulary yet would have nothing to render them
//!    with; see [`NotStarted::NoRecord`].
//! 4. **The record**, which is the one thing a service refuses to run without.
//! 5. **The stop, and the handler that causes one.** `crate::signalling`.
//! 6. **The boundary** — the map `alo-boundaryd` pinned at boot, opened by
//!    path, and this service's own control group subtree, `crate::bounding`.
//!    Before the socket, because ADR 0015 says a turn that cannot be bounded
//!    does not run and a service that cannot bound one has nothing to offer
//!    anybody; and after the record, because a machine with no boundary is a
//!    machine that will not serve rather than one that cannot write down why.
//!    Nothing here is privileged: ADR 0018 moved the loading out of this
//!    process, so what this step needs is permission on a file.
//! 7. **The session, and the socket** — last, because it is the only thing
//!    anybody else on the machine can see. Nothing knocks on a service that is
//!    still deciding whether it can run.
//! 8. **The machine, and the serving.** [`until_stopped`].
//!
//! # What answers a question is not decided here either
//!
//! [`until_stopped`] makes one `crate::questions::Questions` and hands it to
//! the service. Nothing is read while it is made — no settings file is opened
//! and no runtime is probed — because a machine that reached for a model
//! before anybody had asked it anything would be doing exactly what ADR 0001
//! forbids. The first question of the first turn is what opens the file.
//!
//! Two of the three things it is made of are settled and one is not. The
//! catalogue is the one built into this image, and a malformed one stops the
//! process ([`NotStarted::NoCatalogue`]) rather than becoming a machine that
//! quietly offers nothing. **The bound is `None`**, because no file on this
//! machine states what an organisation permits:
//! `docs/contracts/machine-description.md` has no policy key, whether it gains
//! one is ADR 0016's subject and a queue item of its own, and today no
//! `alo_models::SourcePolicy` refuses this machine answering on itself — so
//! every machine passes the same value and none is affected by it either way.
//!
//! # What is not read from anywhere, and is not a stub
//!
//! [`until_stopped`] begins with **no grants at all**, and that is the honest
//! state of this machine rather than a gap. A grant is made by a person picking
//! a folder (ADR 0001 §3), the surface they pick it in is the shell, and nothing
//! on this socket can make one: `alo-protocol` has three requests from an agent
//! and two from a person, and none of the five grants anything. So a list read
//! from a file would be a list nothing writes, which is a worse answer than an
//! empty one — where a machine's grants are kept is a question for whoever
//! writes the first one, and it is a queue item of its own.
//!
//! What that means while it is true is worth being plain about: every verb an
//! agent asks for is refused, in the grants' own words, and every refusal is
//! written down. That is the capability model running rather than the capability
//! model missing, and there is a test below that says so.

use alo_capability::Grants;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_models::Catalogue;
use alo_saying::{Loaded, everything_this_machine_can_say, the_translations};
use alo_strings::Strings;
use alo_turn::{Bounding, Machine, Shortening};

use crate::caller::Uid;
use crate::described::Described;
use crate::knocking::Knocking;
use crate::questions::Questions;
use crate::refusing::NotStarted;
use crate::serving::{Served, Serving};
use crate::stopping::Waking;
use crate::words::declare_into;

/// Refuse to be root.
///
/// ADR 0001 §2: `alo-agentd` runs as the signed-in person and never with
/// authority they do not have themselves. `crate::side` refuses a *description*
/// that names the agent as root, which is a number in a file; this is the other
/// half, and it is about the process that is really running.
///
/// The person's own login is not otherwise checked here. Which login the person
/// is, is the description's, and `crate::Listening` refuses a process that is
/// not the person it was told about — one question asked in one place.
///
/// # Errors
///
/// [`NotStarted::AsRoot`], and nothing has been read, opened or bound.
pub const fn not_as_root(us: Uid) -> Result<(), NotStarted> {
    if us.is_root() {
        return Err(NotStarted::AsRoot);
    }
    Ok(())
}

/// Everything this machine can say, with whatever translations it has.
///
/// Three things in order, and the order is `alo-saying`'s: the fifteen crates
/// that have words in them, then this crate's own three on top — `alo-agentd` is
/// Linux, so it is not on the collected list and the process that runs it is
/// what declares it — and then every translation in
/// `alo_saying::THE_TRANSLATIONS`, checked against the vocabulary they are being
/// loaded into.
///
/// Nothing about a translation stops a machine. What did not load travels in
/// [`Loaded::damage`] and belongs in the service log, which is `alo-saying`'s
/// decision and this is the caller that honours it.
///
/// # Errors
///
/// [`NotStarted::NotCollected`] or [`NotStarted::NotDeclared`] — both of them
/// alo OS's own words contradicting each other, which cannot be fixed on the
/// machine it happens on and is caught by a test in CI rather than by somebody's
/// morning.
pub fn what_this_machine_says() -> Result<Loaded, NotStarted> {
    let mut vocabulary = everything_this_machine_can_say()?;
    declare_into(&mut vocabulary)?;
    Ok(Loaded::at(vocabulary, the_translations()))
}

/// Assemble the machine every turn happens against, and serve until stopped.
///
/// The five things `alo_turn::Machine` is made of are made here and nowhere
/// else, which is what `crate::serving` means by *the service is handed a
/// machine rather than building one*: the verbs are the six this machine can
/// carry out, the resolver is the real one, the indicator is the single one law
/// 1's surface is drawn from, and the record is whatever the caller opened at
/// the path the description named.
///
/// The record and the boundary are taken rather than made here, and for one
/// reason each. The record, so that the order in `src/main.rs` stays the order
/// in this file's header — and so that a test can hand this the same machine
/// with somewhere else to write. The boundary, because making one changes the
/// machine outside this process: it loads a programme into the kernel and moves
/// this service into a control group of its own, so what owns it is what has to
/// give it back, and that is `src/main.rs`.
///
/// # Errors
///
/// [`NotStarted::NoVerbs`] if the six will not declare, which they cannot, and
/// [`NotStarted::NotServed`] for every way a running service stops that is the
/// machine's rather than a client's. A service that ends because somebody asked
/// it to is not an error and answers with what it did.
pub fn until_stopped(
    described: &Described,
    knocking: &dyn Knocking,
    waking: &Waking,
    strings: &Strings,
    bounding: &mut dyn Bounding,
    kept: &mut dyn Shortening,
) -> Result<Served, NotStarted> {
    let mut indicator = Indicator::default();
    let mut machine =
        Machine::carrying_out_file_verbs(strings, &OnThisMachine, bounding, &mut indicator, kept)?;
    // Nothing has been granted on this machine, and nothing on this socket can
    // grant anything; the header says what that means and why it is a state
    // rather than a hole.
    let mut grants = Grants::default();
    // Nothing is read or probed here: the environment is copied, and the first
    // question of the first turn is what opens the person's file. The bound is
    // `None` because nothing on this machine states one — the header says why,
    // and `crate::questions` says what changes the day something does.
    let mut questions = Questions::of_this_process(
        Catalogue::built_in().map_err(|why| NotStarted::NoCatalogue {
            why: why.to_string(),
        })?,
        None,
    );
    Ok(Serving::of(
        knocking,
        waking,
        described.agent(),
        described.turn().duration(),
        described.proposal().duration(),
        described.keeping(),
    )
    .until_stopped(&mut machine, &mut grants, &mut questions)?)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::lasting::Lasting;
    use crate::side::Side;
    use crate::testing::{Pretending, a_folder_with_an_invoice, a_message, ourselves};
    use crate::words::A_TURN_IS_UNDER_WAY;
    use alo_keeping::Keeping;
    use alo_record::Record;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixStream;
    use std::path::Path;

    /// A machine described the ordinary way, with the agent these tests use.
    fn an_ordinary_machine() -> Described {
        Described::of(
            ourselves(),
            "@files",
            Lasting::of_seconds(3600, "agent.turn-seconds").unwrap(),
            Lasting::of_seconds(3600, "agent.proposal-seconds").unwrap(),
            Path::new("/var/lib/alo/record.jsonl"),
            Keeping::Forever,
        )
        .unwrap()
    }

    /// **Root is refused**, which is ADR 0001 §2 about the process rather than
    /// about a number somebody wrote in a file.
    #[test]
    fn a_process_running_as_root_does_not_start() {
        assert!(matches!(
            not_as_root(Uid::of(0).unwrap()).unwrap_err(),
            NotStarted::AsRoot
        ));
    }

    /// And an ordinary person starts, which is the other half of the same
    /// question: this refuses root and nothing else.
    #[test]
    fn an_ordinary_person_starts() {
        assert!(not_as_root(Uid::of(1000).unwrap()).is_ok());
    }

    /// **The machine's vocabulary holds this service's own words as well as
    /// everything else's.** `alo-agentd` is Linux and is not on `alo-saying`'s
    /// collected list, so the process is what puts its three strings on — and a
    /// service that had forgotten to would refuse a second agent with a key.
    #[test]
    fn the_machines_vocabulary_holds_what_this_service_says() {
        let said = what_this_machine_says().unwrap();
        let strings = said.strings();

        let refusal = strings.say(&A_TURN_IS_UNDER_WAY.key(), &alo_strings::Filling::nothing());
        assert_eq!(refusal.text(), A_TURN_IS_UNDER_WAY.says());

        // And somebody else's, so that this is the machine's vocabulary rather
        // than this crate's three in a wrapper.
        assert!(
            strings.vocabulary().how_many() > crate::words::EVERY_WORD.len(),
            "only this crate's own words were collected"
        );
    }

    /// **A translation is never a reason not to start**, which is
    /// `alo-saying`'s rule asked of the process that honours it: this machine
    /// has no translations directory at all, and it still comes back with a
    /// vocabulary to serve from.
    ///
    /// What did not load is reported rather than swallowed, so whatever went
    /// wrong has a line in the service log for each — the assertion is that
    /// count, because damage nobody can read is the same as damage nobody
    /// reported.
    #[test]
    fn a_translation_is_never_a_reason_not_to_start() {
        let said = what_this_machine_says().unwrap();
        assert_eq!(said.damage().lines().len(), said.damage().how_many());
        assert!(
            said.strings()
                .say(&A_TURN_IS_UNDER_WAY.key(), &alo_strings::Filling::nothing())
                .text()
                .contains("turn"),
            "the service cannot say anything"
        );
    }

    /// **A machine that has just started has granted nothing, and says so in
    /// the grants' own words rather than by going quiet.**
    ///
    /// The end-to-end shape of everything above: a real socket, a real turn, a
    /// real read asked for, refused by the capability model, and one entry
    /// written down. It is the refusal path, and on a machine where nobody has
    /// picked a folder yet it is the only path there is.
    #[test]
    fn a_machine_that_has_just_started_has_granted_nothing_and_writes_the_refusal_down() {
        let described = an_ordinary_machine();
        let said = what_this_machine_says().unwrap();
        let strings = said.into_strings();
        let (folder, _invoice) = a_folder_with_an_invoice("nothing-granted");
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out("nothing-granted", &[Some(Side::Agent)]);
        let at = knocking.at();
        let mut record = Record::default();

        let client = std::thread::spawn(move || {
            let connection = UnixStream::connect(&at).unwrap();
            let mut reading = BufReader::new(connection.try_clone().unwrap());
            let mut writing = connection;
            writing
                .write_all(
                    a_message(&format!(
                        r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                        folder.display()
                    ))
                    .as_bytes(),
                )
                .unwrap();
            writing.write_all(b"\n").unwrap();
            let mut back = String::new();
            reading.read_line(&mut back).unwrap();
            stop.stop();
            back
        });

        let served = until_stopped(
            &described,
            &knocking,
            &waking,
            &strings,
            &mut crate::testing::NothingIsBounded,
            &mut record,
        )
        .unwrap();
        let back = client.join().unwrap();

        assert!(back.contains("refused"), "{back}");
        assert_eq!(served.turns(), 1);
        assert_eq!(served.messages(), 1);
        assert_eq!(
            record.len(),
            1,
            "a refusal on a machine that has granted nothing is still evidence"
        );
    }
}
