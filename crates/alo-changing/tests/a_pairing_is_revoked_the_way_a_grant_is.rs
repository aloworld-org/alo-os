//! A pairing is revoked the way a grant is — one call, the same answer, and
//! the daemon's own words when it will not — measured, refusals beside the
//! answers.
//!
//! Task 3 of `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`.
//! Every wire test here puts a thread where `alo-agentd` stands on a real Unix
//! socket, reads the one line [`TheDaemonsDoor`] sends, holds it to being the
//! request the contract documents, and answers the way the daemon does. The
//! rows a surface lists come from the two places the plan names: the daemon's
//! `pairings` answer and `alo_remembering::pairings_remembered` on a real
//! file.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use alo_capability::Grants;
use alo_changing::{
    Changing, Gone, Knocking, NotChanged, RevokingPairings, Row, SeenPairing, Stood,
    TheDaemonsDoor, Unpaired,
};
use alo_granted::Listing;
use alo_nearby::{Deliberating, Keying, MachineId, MayAskIts, Pairings, Proposal, Side};
use alo_picking::{Granting, OnThisDisk, Picker};
use alo_protocol::{AfterRevoking, FromAPerson, Paired, ToAPerson};
use alo_strings::{Filling, Key, Said, Strings, Vocabulary, Word};

/// The machine this one is paired with.
const RECEPTION: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";

/// This machine.
const STUDIO: &str = "aaaabbbbccccddddeeeeffff00001111";

/// The moment this file is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A folder of this test's own, empty, on the real disk.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir()
        .join("alo-changing-pairing-tests")
        .join(format!("{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// The machine's one vocabulary.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Reception's row, as the daemon's `pairings` answer lists it.
fn receptions_row() -> SeenPairing {
    SeenPairing::told(&Paired::of(RECEPTION, Vec::new(), 60, 3_600)).unwrap()
}

/// The studio's pairings with reception, made the way two machines make one.
fn the_studios_pairings() -> Pairings {
    let reception = MachineId::read(RECEPTION).unwrap();
    let studio = MachineId::read(STUDIO).unwrap();
    let proposal = Proposal::checked(
        reception,
        studio,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        Keying::fresh().unwrap().offer().clone(),
    )
    .unwrap();
    let on_studio = Deliberating::asked(proposal, Keying::fresh().unwrap())
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    pairings
}

/// A thread standing where the daemon stands: it reads one line, holds it to
/// being `expected`, and answers with `answer` — or closes without a word when
/// there is none.
fn a_daemon_at(socket: &Path, expected: FromAPerson, answer: Option<ToAPerson>) -> JoinHandle<()> {
    let listener = UnixListener::bind(socket).unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        assert_eq!(
            FromAPerson::read(line.trim_end()).unwrap(),
            expected,
            "the person's side asked something other than the contract documents"
        );
        if let Some(answer) = answer {
            let written = answer.written().unwrap();
            (&stream).write_all(written.as_bytes()).unwrap();
            (&stream).write_all(b"\n").unwrap();
        }
    })
}

/// The daemon's sentence for revoking a machine it is not paired with, as
/// `alo-agentd` declares it — spoken out of a vocabulary of this test's own,
/// because the daemon's words are its own and not collected here.
fn the_daemons_refusal() -> Said {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            Word::saying(
                "agentd.nothing-is-paired-with-that-machine",
                "this machine is not paired with that one, so there is nothing to revoke",
            )
            .phrase()
            .unwrap(),
        )
        .unwrap();
    Strings::of(vocabulary).say(
        &Key::named("agentd.nothing-is-paired-with-that-machine").unwrap(),
        &Filling::nothing(),
    )
}

/// The request revoking reception's pairing is.
fn revoking_reception() -> FromAPerson {
    FromAPerson::RevokePairing {
        machine: RECEPTION.to_owned(),
    }
}

/// A door of this file's own that counts both conversations and answers each
/// the way a daemon that did it would.
#[derive(Debug, Default)]
struct CountingDoor {
    /// How many knocks arrived.
    knocks: Cell<u32>,
    /// How many pairings it was asked to revoke.
    unpairings: Cell<u32>,
}

impl Knocking for CountingDoor {
    fn knock(&self) -> Stood {
        self.knocks.set(self.knocks.get() + 1);
        Stood::Heard { holding: 0 }
    }
}

impl RevokingPairings for CountingDoor {
    fn revoke_pairing(&self, _with: &MachineId) -> Unpaired {
        self.unpairings.set(self.unpairings.get() + 1);
        Unpaired::Revoked
    }
}

/// The one call a surface makes for a row of the one list — written as a
/// function with a signature, so *both kinds answer the same type* is a
/// fact the compiler checks rather than one this file asserts.
fn the_surface_revokes(changing: &mut Changing<'_>, row: Row) -> Result<Gone, NotChanged> {
    changing.revoked(&row, noon())
}

/// **A grant and a pairing are revoked with one call and answered with the
/// same `Gone`** — and each reaches only its own door: the grant is written
/// and knocked for, the pairing is asked of the daemon and writes nothing.
#[test]
fn both_kinds_of_row_are_revoked_with_one_call_and_answer_the_same_gone() {
    let folder = a_folder_of_our_own("one-call");
    let home = folder.join("home");
    std::fs::create_dir_all(home.join("Invoices")).unwrap();
    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();
    picker.go_into("Invoices", &OnThisDisk).unwrap();
    let picked = picker.pick().unwrap();
    let at = folder.join("grants.toml");
    let door = CountingDoor::default();

    let mut grants = Grants::default();
    let mut changing = Changing::of(&mut grants, &at, &door);
    changing
        .granted(
            &Granting::to("@files", Duration::from_secs(3_600)),
            &picked,
            noon(),
        )
        .unwrap();
    let grant = Row::from(
        Listing::of(changing.holding(), noon())
            .rows()
            .first()
            .unwrap()
            .clone(),
    );
    let file_after_the_grant = std::fs::read(&at).unwrap();

    let pairing: Gone = the_surface_revokes(&mut changing, Row::from(receptions_row())).unwrap();
    assert_eq!(
        pairing,
        Gone::Revoked {
            stood: Stood::KeptByTheDaemon
        }
    );
    assert_eq!(door.unpairings.get(), 1, "one revocation, one question");
    assert_eq!(door.knocks.get(), 1, "a pairing's revocation knocked");
    assert_eq!(
        std::fs::read(&at).unwrap(),
        file_after_the_grant,
        "a pairing's revocation wrote the grants file"
    );

    let revoked: Gone = the_surface_revokes(&mut changing, grant).unwrap();
    assert_eq!(
        revoked,
        Gone::Revoked {
            stood: Stood::Heard { holding: 0 }
        }
    );
    assert_eq!(
        door.unpairings.get(),
        1,
        "a grant's revocation asked the daemon about a pairing"
    );
    assert_eq!(door.knocks.get(), 2);
}

/// **Over a real socket, a pairing's revocation is the documented
/// `revoke-pairing` naming the machine, and `revoked` is done** — kept by the
/// daemon, which serves under it already.
#[test]
fn a_pairing_the_daemon_revoked_is_gone_and_kept() {
    let folder = a_folder_of_our_own("revoked");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(
        &socket,
        revoking_reception(),
        Some(ToAPerson::revoked(AfterRevoking::Revoked)),
    );

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let at = folder.join("grants.toml");
    let gone = Changing::of(&mut grants, &at, &door)
        .revoked(&Row::from(receptions_row()), noon())
        .unwrap();
    daemon.join().unwrap();

    let Gone::Revoked { stood } = gone else {
        unreachable!("the pairing was not revoked: {gone:?}");
    };
    assert_eq!(stood, Stood::KeptByTheDaemon);
    assert!(stood.was_heard());
    assert_eq!(stood.explained(&in_english()), None);
    assert!(!at.exists(), "a pairing's revocation wrote the grants file");
}

/// **A revocation the daemon could not write down says it lasts until a
/// restart** — done now, and not reported as though it were kept.
#[test]
fn a_pairing_revoked_until_a_restart_says_so() {
    let folder = a_folder_of_our_own("until-a-restart");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(
        &socket,
        revoking_reception(),
        Some(ToAPerson::revoked(AfterRevoking::RevokedUntilARestart)),
    );

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let at = folder.join("grants.toml");
    let gone = Changing::of(&mut grants, &at, &door)
        .revoked(&Row::from(receptions_row()), noon())
        .unwrap();
    daemon.join().unwrap();

    let Gone::Revoked { stood } = gone else {
        unreachable!("the pairing was not revoked: {gone:?}");
    };
    assert_eq!(stood, Stood::UntilARestart);
    let explained = stood.explained(&in_english()).unwrap();
    assert!(explained.contains("restart"), "{explained}");
    assert!(explained.contains("revoked again"), "{explained}");
}

/// **A revocation the daemon refuses is a refusal in the daemon's own
/// words** — never `Gone`, and never reworded.
#[test]
fn a_pairing_the_daemon_refuses_is_told_in_its_words_and_not_as_done() {
    let folder = a_folder_of_our_own("refused");
    let socket = folder.join("agentd.sock");
    let strings = in_english();
    let daemons_sentence = the_daemons_refusal();
    assert!(!daemons_sentence.is_a_bug(), "{daemons_sentence}");
    let daemon = a_daemon_at(
        &socket,
        revoking_reception(),
        Some(ToAPerson::refused(&daemons_sentence)),
    );

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let at = folder.join("grants.toml");
    let refused = Changing::of(&mut grants, &at, &door)
        .revoked(&Row::from(receptions_row()), noon())
        .unwrap_err();
    daemon.join().unwrap();

    let NotChanged::PairingRefused { told } = &refused else {
        unreachable!("a refusal was read as something else: {refused:?}");
    };
    assert_eq!(
        told,
        daemons_sentence.text(),
        "the daemon's words were reworded"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("was not revoked"), "{said}");
    assert!(said.text().contains(daemons_sentence.text()), "{said}");
}

/// **With no daemon running, nothing is revoked and the person is told so**
/// — nothing on this side writes the pairings file instead.
#[test]
fn with_nobody_at_the_door_no_pairing_is_revoked() {
    let folder = a_folder_of_our_own("nobody");
    let door = TheDaemonsDoor::at(&folder.join("nobody-listens.sock"));
    let mut grants = Grants::default();
    let at = folder.join("grants.toml");

    let refused = Changing::of(&mut grants, &at, &door)
        .revoked(&Row::from(receptions_row()), noon())
        .unwrap_err();

    assert!(
        matches!(refused, NotChanged::NobodyKeepsPairings),
        "{refused:?}"
    );
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("was not revoked"), "{said}");
    let written: Vec<_> = std::fs::read_dir(&folder).unwrap().collect();
    assert!(written.is_empty(), "something was written: {written:?}");
}

/// **A daemon that hears the request and never answers is not a
/// revocation** — the person is asked to look again, not told it is done.
#[test]
fn a_daemon_that_never_answers_is_not_read_as_a_revocation() {
    let folder = a_folder_of_our_own("silent");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(&socket, revoking_reception(), None);

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let at = folder.join("grants.toml");
    let refused = Changing::of(&mut grants, &at, &door)
        .revoked(&Row::from(receptions_row()), noon())
        .unwrap_err();
    daemon.join().unwrap();

    assert!(
        matches!(refused, NotChanged::PairingNotAnswered),
        "{refused:?}"
    );
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Look at the list"), "{said}");
}

/// **An answer this request cannot earn is not a revocation either** — a
/// daemon answering `granted` to `revoke-pairing` has said nothing about
/// the pairing.
#[test]
fn an_answer_to_another_question_is_not_read_as_a_revocation() {
    let folder = a_folder_of_our_own("wrong-answer");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(&socket, revoking_reception(), Some(ToAPerson::granted(0)));

    let door = TheDaemonsDoor::at(&socket);
    let unpaired = door.revoke_pairing(&MachineId::read(RECEPTION).unwrap());
    daemon.join().unwrap();

    assert_eq!(unpaired, Unpaired::NotAnswered);
}

/// **The rows a surface lists come from the daemon's `pairings` answer**,
/// carrying the name the person gave — and a pairing the daemon names by
/// something that is not a machine identity is never a row.
#[test]
fn the_daemons_pairings_answer_is_the_list_of_rows() {
    let folder = a_folder_of_our_own("listed");
    let socket = folder.join("agentd.sock");
    let paired = vec![
        Paired::of(RECEPTION, Vec::new(), 60, 3_600).that_is_called(Some("reception")),
        Paired::of("../../etc/passwd", Vec::new(), 60, 3_600),
    ];
    let daemon = a_daemon_at(
        &socket,
        FromAPerson::Pairings,
        Some(ToAPerson::pairings(paired, Vec::new())),
    );

    let rows = TheDaemonsDoor::at(&socket).pairings().unwrap();
    daemon.join().unwrap();

    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = rows.first().unwrap();
    assert_eq!(row.machine().as_str(), RECEPTION);
    assert_eq!(row.called(), Some("reception"));
}

/// **With no daemon there is no answer to list from**, and a surface is told
/// so rather than handed an empty list that would read as *nothing paired*.
#[test]
fn with_nobody_at_the_door_there_is_no_daemons_list() {
    let folder = a_folder_of_our_own("no-list");
    assert_eq!(
        TheDaemonsDoor::at(&folder.join("nobody-listens.sock")).pairings(),
        None
    );
}

/// **The rows a surface lists come from the daemon's file, read through
/// `pairings_remembered`** — on a real disk, the pairing's row is the machine
/// it is with, and it revokes the same way.
#[test]
fn the_daemons_file_read_back_is_the_list_of_rows() {
    let folder = a_folder_of_our_own("remembered");
    let at = folder.join("pairings.toml");
    // The daemon's own write, standing in for the daemon: this test is not
    // shipped, and the file it makes is the one the daemon would have made.
    alo_remembering::pairings_kept(&at, &the_studios_pairings(), noon()).unwrap();

    let rows = SeenPairing::remembered(&at, noon()).unwrap();

    assert_eq!(rows, vec![receptions_row()]);
}

/// **A machine that has paired with nothing has no file, and that is an
/// empty list** — the reading the daemon gives its own start.
#[test]
fn no_pairings_file_is_an_empty_list() {
    let folder = a_folder_of_our_own("never-paired");
    assert_eq!(
        SeenPairing::remembered(&folder.join("pairings.toml"), noon()).unwrap(),
        Vec::new()
    );
}

/// **A pairings file that is there and does not read is refused, never
/// listed as nothing** — *nothing is paired* would be a claim nobody checked.
#[test]
fn a_pairings_file_that_does_not_read_is_refused_rather_than_empty() {
    let folder = a_folder_of_our_own("unreadable");
    let at = folder.join("pairings.toml");
    std::fs::write(&at, "this is not a list of pairings").unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o600)).unwrap();

    assert!(SeenPairing::remembered(&at, noon()).is_err());
}
