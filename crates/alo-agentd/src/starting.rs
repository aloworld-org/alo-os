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
//! 3. **The session this process was started into**, `crate::session` — right
//!    after the description, because the description is what says who the
//!    person is, and before anything is opened, because a machine wired into
//!    another login's session is one to stop on rather than one to serve from.
//! 4. **The vocabulary** — before the record, and that is not tidiness. A record
//!    that will not open is refused in `alo-keeping`'s own words, and a process
//!    that had not loaded a vocabulary yet would have nothing to render them
//!    with; see [`NotStarted::NoRecord`].
//! 5. **The record**, which is the one thing a service refuses to run without.
//! 6. **Whatever this person granted before**, which `alo-remembering` kept
//!    while nobody was signed in. After the record, so that a machine whose
//!    grants will not read has somewhere to be refused into; before the
//!    boundary and the socket, because a list of grants that cannot be believed
//!    is a machine to stop on rather than one to open a door on. A machine that
//!    has simply never been granted anything is not that machine, and starts.
//! 7. **The stop, and the handler that causes one.** `crate::signalling`.
//! 8. **The boundary** — the map `alo-boundaryd` pinned at boot, opened by
//!    path, and this service's own control group subtree, `crate::bounding`.
//!    Before the socket, because ADR 0015 says a turn that cannot be bounded
//!    does not run and a service that cannot bound one has nothing to offer
//!    anybody; and after the record, because a machine with no boundary is a
//!    machine that will not serve rather than one that cannot write down why.
//!    Nothing here is privileged: ADR 0018 moved the loading out of this
//!    process, so what this step needs is permission on a file.
//! 9. **The person's door, and the socket in it** — last, because it is the
//!    only thing anybody else on the machine can see. Nothing knocks on a
//!    service that is still deciding whether it can run.
//! 10. **The machine, and the serving.** [`until_stopped`].
//!
//! # What answers a question is not decided here either
//!
//! [`until_stopped`] makes one `crate::questions::Questions` and hands it to
//! the service. Nothing is read while it is made — no settings file is opened
//! and no runtime is probed — because a machine that reached for a model
//! before anybody had asked it anything would be doing exactly what ADR 0001
//! forbids. The first question of the first turn is what opens the file.
//!
//! Two of the three things it is made of come from somewhere and one is built
//! in. The catalogue is the one built into this image, and a malformed one stops
//! the process ([`NotStarted::NoCatalogue`]) rather than becoming a machine that
//! quietly offers nothing. **The bound is the description's**, which is where
//! ADR 0016's rule finally arrives: `[questions]` in `/etc/alo/agentd.toml`
//! states where an organisation permits a question to be answered, the file's
//! owner says whether it is an organisation's rule or the person's own, and a
//! machine with no such section is `TheBound::Nobodys` — *absent*, which is
//! every personal machine and is not the same thing as permissive.
//!
//! Nothing about **which** model or provider answers is decided from here. That
//! is the person's, in their own settings, and a bound refuses a choice without
//! ever replacing one.
//!
//! # The grants are handed in, and this file cannot reach the file they were in
//!
//! [`until_stopped`] takes the grants rather than reading them. `src/main.rs`
//! reads `alo_remembering::THE_GRANTS` before anything is opened and hands over
//! an `alo_capability::Grants` — **a value, with no path in it**.
//!
//! Since the person's door gained a way to say *what is granted has changed*, it
//! also hands over a [`WhatIsGranted`] — the list, and a way to read that file
//! **again**, and nothing else. There is still no way to write it anywhere below
//! this line, and `crate::rereading` is where that is argued: what the socket can
//! now reach is a road to reading the grants, and a message that arrived from
//! anywhere at all could cause nothing more than the file being read a second time.
//!
//! This is where the sentence *it starts with no grants at all* used to be, and
//! it was honest while it was true — nothing on this machine could make a
//! grant, then nothing could keep one. `alo-picking` answered the first
//! (ADR 0001 §3: a grant is made by a person picking a folder) and
//! `alo-remembering` the second, so what a person granted yesterday is what
//! this machine serves under today.
//!
//! What has **not** changed: nothing on this socket grants anything.
//! `alo-protocol` has three requests from an agent and two from a person, and
//! none of the five makes, widens or keeps a grant. A machine on which nobody
//! has picked a folder still refuses every verb in the grants' own words and
//! still writes every refusal down, which is the capability model running
//! rather than missing, and there is a test below that says so.

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
use crate::rereading::WhatIsGranted;
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
/// The grants are the caller's, for the reason the record is: `src/main.rs`
/// reads the file this machine keeps them in, so that the order in that file
/// stays the order in this file's header — and so that a test can hand this the
/// same machine with a different list. Nothing below this line has a path to
/// the file, which is the whole of what makes it unreachable from the socket.
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
    granted: &mut WhatIsGranted<'_>,
    bounding: &mut dyn Bounding,
    kept: &mut dyn Shortening,
) -> Result<Served, NotStarted> {
    let mut indicator = Indicator::default();
    let mut machine =
        Machine::carrying_out_file_verbs(strings, &OnThisMachine, bounding, &mut indicator, kept)?;
    // Nothing is read or probed here: the environment is copied, and the first
    // question of the first turn is what opens the person's file.
    //
    // **The bound is the description's**, which is `[questions]` in
    // `/etc/alo/agentd.toml` where an organisation wrote one and
    // `TheBound::Nobodys` on every machine none manages. Nothing is decided
    // here: `crate::describing` refused every way that section does not hold
    // before this machine was a `Described` at all, and who owns the file is
    // what makes it an organisation's rather than the person's.
    let mut questions = Questions::of_this_process(
        Catalogue::built_in().map_err(|why| NotStarted::NoCatalogue {
            why: why.to_string(),
        })?,
        described.questions().clone(),
    );
    Ok(Serving::of(
        knocking,
        waking,
        described.agent(),
        described.turn().duration(),
        described.proposal().duration(),
        described.keeping(),
    )
    .until_stopped(&mut machine, granted, &mut questions)?)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::lasting::Lasting;
    use crate::questions::TheBound;
    use crate::side::Side;
    use crate::testing::{
        NothingIsRemembered, Pretending, a_directory_of_our_own, a_folder_with_an_invoice,
        a_message, granting, ourselves,
    };
    use crate::words::A_TURN_IS_UNDER_WAY;
    use alo_capability::Grants;
    use alo_keeping::Keeping;
    use alo_record::Record;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    /// A machine described the ordinary way, with the agent these tests use.
    fn an_ordinary_machine() -> Described {
        Described::of(
            ourselves(),
            "@files",
            Lasting::of_seconds(3600, "agent.turn-seconds").unwrap(),
            Lasting::of_seconds(3600, "agent.proposal-seconds").unwrap(),
            Path::new("/var/lib/alo/record.jsonl"),
            Keeping::Forever,
            TheBound::Nobodys,
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
            &mut WhatIsGranted::of(&mut Grants::default(), &NothingIsRemembered),
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

    /// Where a test's own grants file goes: a directory of its own, never the
    /// folder being granted — a file inside the grant would be a file the agent
    /// can list, and these tests are about what it cannot touch.
    fn a_grants_file_of_our_own(what: &str) -> PathBuf {
        a_directory_of_our_own(what).join("grants.toml")
    }

    /// One agent connection, one message, and the answer it was given.
    ///
    /// The whole shape of the two tests below: a real socket, a real turn, and
    /// a service that was handed the grants somebody else had already read off
    /// a disk.
    fn what_the_agent_is_told(
        what: &str,
        grants: &mut Grants,
        asks: &[String],
    ) -> (Vec<String>, usize) {
        let described = an_ordinary_machine();
        let said = what_this_machine_says().unwrap();
        let strings = said.into_strings();
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out(what, &[Some(Side::Agent)]);
        let at = knocking.at();
        let mut record = Record::default();
        let asking = asks.to_vec();

        let client = std::thread::spawn(move || {
            let connection = UnixStream::connect(&at).unwrap();
            let mut reading = BufReader::new(connection.try_clone().unwrap());
            let mut writing = connection;
            let mut answers = Vec::new();
            for one in &asking {
                writing.write_all(a_message(one).as_bytes()).unwrap();
                writing.write_all(b"\n").unwrap();
                let mut back = String::new();
                reading.read_line(&mut back).unwrap();
                answers.push(back);
            }
            stop.stop();
            answers
        });

        until_stopped(
            &described,
            &knocking,
            &waking,
            &strings,
            &mut WhatIsGranted::of(grants, &NothingIsRemembered),
            &mut crate::testing::NothingIsBounded,
            &mut record,
        )
        .unwrap();
        (client.join().unwrap(), record.len())
    }

    /// One read of a folder, as an agent asks for it.
    fn listing(folder: &Path) -> String {
        format!(
            r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
            folder.display()
        )
    }

    /// **A grant a person made before this process existed is honoured by it.**
    ///
    /// The restart, end to end: a grant is kept on a disk, this process is
    /// handed nothing but what `alo-remembering` read back out of that file,
    /// and the verb the machine before it would have refused is carried out.
    /// The same read on a machine that was handed no grants is the refusal
    /// tested above, which is what makes this one about the file.
    #[test]
    fn a_grant_made_before_a_restart_is_honoured_after_one() {
        let (folder, _invoice) = a_folder_with_an_invoice("kept-grants");
        let at = a_grants_file_of_our_own("kept-grants-file");
        // The real clock, because a running service reads one: a grant made at
        // a fixed noon ran out decades ago.
        let now = SystemTime::now();
        alo_remembering::kept(&at, &granting(&folder, now), now).unwrap();

        // Everything this machine knows about what is granted, and nothing else.
        let mut grants = alo_remembering::remembered(&at, now).unwrap();
        assert_eq!(grants.len(), 1, "the grant did not survive the disk");

        let (answers, entries) =
            what_the_agent_is_told("kept-grants", &mut grants, &[listing(&folder)]);

        let back = answers.first().unwrap();
        assert!(back.contains("listed"), "{back}");
        assert!(!back.contains("refused"), "{back}");
        assert_eq!(entries, 1, "what was carried out was not written down");
    }

    /// **Nothing an agent can send over the socket writes a byte of the file
    /// the grants came out of.**
    ///
    /// Every request an agent has: a read it is permitted, a read it is not,
    /// and a verb that does not exist. The file is compared byte for byte
    /// afterwards, and so is the directory it is in — a service that had
    /// written a new list, or left a staging file behind, would fail this even
    /// if it had written the same grants back.
    ///
    /// It cannot be otherwise: `until_stopped` is handed an
    /// `alo_capability::Grants` and there is no path anywhere below it. The
    /// test is here because *there is no way to* is worth a measurement rather
    /// than a comment somebody may one day be tempted to make untrue.
    #[test]
    fn nothing_an_agent_says_writes_a_byte_of_the_grants() {
        let (folder, invoice) = a_folder_with_an_invoice("untouched-grants");
        let at = a_grants_file_of_our_own("untouched-grants-file");
        let now = SystemTime::now();
        alo_remembering::kept(&at, &granting(&folder, now), now).unwrap();

        let before = std::fs::read(&at).unwrap();
        let mut grants = alo_remembering::remembered(&at, now).unwrap();

        let (answers, _entries) = what_the_agent_is_told(
            "untouched-grants",
            &mut grants,
            &[
                listing(&folder),
                listing(Path::new("/etc")),
                format!(
                    r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
                    invoice.display()
                ),
                r#"{"read":{"verb":"grant_everything","given":[]}}"#.to_owned(),
            ],
        );
        assert_eq!(
            answers.len(),
            4,
            "the service stopped answering: {answers:?}"
        );
        assert!(answers.get(1).unwrap().contains("refused"), "{answers:?}");
        assert!(answers.get(3).unwrap().contains("refused"), "{answers:?}");

        assert_eq!(
            std::fs::read(&at).unwrap(),
            before,
            "the grants file was written while the service was running"
        );
        let alongside: Vec<PathBuf> = std::fs::read_dir(at.parent().unwrap())
            .unwrap()
            .map(|one| one.unwrap().path())
            .collect();
        assert_eq!(alongside, vec![at], "something else was written beside it");
    }
}
