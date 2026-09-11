//! The composition against a real socket, with a real `alo-sessiond` door on
//! the other side of it.
//!
//! Everything in `a_greeter_signs_somebody_in.rs` knocks at a fixture, which is
//! what makes those roads testable on any host. This is the half that cannot
//! be: a socket really bound by the opener's own `Listening`, a connection
//! really made, a line really written and read — and, beside it, the three
//! doors that are not a door at all, because *the service is not running* and
//! *the service answered nonsense* are exactly the states a fixture cannot
//! prove anything about.
//!
//! Unix only, because there is no Unix socket anywhere else. Everything this
//! crate decides is arithmetic and runs on any host; this file is the part
//! that touches a machine, which is the division `alo-sessiond` itself makes.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a value that is not the one expected names itself in the failure, which a matches! \
              assertion could not"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;

use alo_accounts::Accounts;
use alo_greeting::{Greeted, Greeting, NotAnswered, TheOpenersDoor};
use alo_sessiond::listening::Listening;
use alo_sessiond::{
    Answered, Door, Knock, Logind, NOBODY_HERE, NotAsked, NotOpened, Opening, TheAccounts,
    our_group,
};
use alo_strings::Strings;

/// Hers, and it is right.
const HERS: &str = "correct horse battery staple";

/// The number this machine's description names.
fn this_machines_person() -> u32 {
    alo_image::Image::at(std::path::Path::new(alo_image::THE_IMAGE))
        .unwrap()
        .description()
        .person()
}

/// A machine with one account on it, at this number.
fn a_machine_with_ada_at(uid: u32) -> Accounts {
    let mut store = Accounts::none().unwrap();
    store.created("ada", uid, HERS).unwrap();
    store
}

/// Everything this machine can say, read in English.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Somewhere this test may bind a socket.
fn somewhere(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-greeting-{}-{what}", std::process::id()));
    drop(std::fs::create_dir_all(&at));
    at.join("sign-in.sock")
}

/// The accounts the opener is asked about — its own question, which is a
/// number in and a yes or no out.
struct Numbered(Vec<u32>);

impl TheAccounts for Numbered {
    fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked> {
        Ok(self.0.contains(&person))
    }
}

/// A `logind` that opens whatever it is asked for, and remembers.
#[derive(Default)]
struct Willing(Vec<u32>);

impl Logind for Willing {
    fn open_a_session_for(&mut self, person: u32) -> Result<(), NotOpened> {
        self.0.push(person);
        Ok(())
    }
}

/// **A correct password reaches a real opener as one knock, and the session it
/// opens is this machine's person's.**
///
/// The whole chain, end to end and over a socket: `alo-accounts` verified,
/// `alo_accounts::Session` agreed with the machine description,
/// `alo_sessiond::Knock` crossed a real Unix socket, the opener's own
/// `Opening` decided, and what came back became an
/// [`alo_greeting::Greeted::SignedIn`].
#[test]
fn a_correct_password_opens_a_session_through_a_real_door() {
    let person = this_machines_person();
    let at = somewhere("opened");
    let ours = our_group();
    let listening = Listening::at(&at, ours).unwrap();
    let mut opening = Opening::at(
        Door::handed_to(ours),
        Numbered(vec![person]),
        Willing::default(),
    );

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let signing_in = std::thread::spawn(move || greeting.signs_in("ada", HERS));
    let answered = listening.answer_one(&mut opening).unwrap();

    assert_eq!(answered, Answered::Opened);
    let greeted = signing_in.join().unwrap();
    assert_eq!(greeted.session().unwrap().uid(), person);
    assert!(opening.is_signed_in());
    drop(std::fs::remove_file(&at));
}

/// **A number the opener has no account for is refused over the real wire, in
/// the opener's own words.** The greeter renders the key it was sent and does
/// not reword it, and nothing about the refusal is invented on this side.
#[test]
fn a_refusal_comes_back_as_the_openers_own_sentence() {
    let person = this_machines_person();
    let at = somewhere("refused");
    let ours = our_group();
    let listening = Listening::at(&at, ours).unwrap();
    // The opener's accounts do not hold this person — a machine whose sign-in
    // surface and whose accounts disagree, which is the state `NOBODY_HERE`
    // exists for.
    let mut opening = Opening::at(
        Door::handed_to(ours),
        Numbered(Vec::new()),
        Willing::default(),
    );

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let signing_in = std::thread::spawn(move || greeting.signs_in("ada", HERS));
    let answered = listening.answer_one(&mut opening).unwrap();

    assert_eq!(answered, Answered::Refused(NOBODY_HERE.key()));
    let greeted = signing_in.join().unwrap();
    assert_eq!(greeted, Greeted::Refused(NOBODY_HERE.key()));
    assert_eq!(
        greeted.said(&in_english()).unwrap().text(),
        NOBODY_HERE.says(),
        "the greeter reworded the opener's own sentence"
    );
    assert!(!opening.is_signed_in());
    drop(std::fs::remove_file(&at));
}

/// **A machine whose opener is not running says so**, rather than taking a
/// correct password and appearing to do nothing. It is the failure a person
/// actually meets on a machine where the service did not start, and it is one
/// of the two the fixture doors cannot show.
#[test]
fn a_door_nobody_is_behind_is_told_in_words() {
    let person = this_machines_person();
    let at = somewhere("nobody");
    drop(std::fs::remove_file(&at));

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let greeted = greeting.signs_in("ada", HERS);

    let Greeted::NotAnswered(why) = &greeted else {
        panic!("a door nobody is behind was not told apart: {greeted:?}");
    };
    assert!(matches!(why, NotAnswered::NobodyThere { .. }), "{why:?}");
    let said = greeted.said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(said.text(), alo_greeting::NOTHING_LISTENING.says());
}

/// **A door that takes the knock and closes without answering is told apart
/// from a door that is not there**, because the two send whoever maintains the
/// machine to two different places — one service that never started, and one
/// that did and did not finish.
#[test]
fn a_door_that_says_nothing_is_told_apart_from_a_door_that_is_not_there() {
    let person = this_machines_person();
    let at = somewhere("silent");
    drop(std::fs::remove_file(&at));
    let listening = UnixListener::bind(&at).unwrap();

    let hanging_up = std::thread::spawn(move || {
        let (connection, _) = listening.accept().unwrap();
        let mut line = String::new();
        drop(BufReader::new(&connection).read_line(&mut line));
        // Read the knock and go away without answering it.
        drop(connection);
        line
    });

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let greeted = greeting.signs_in("ada", HERS);

    assert_eq!(
        hanging_up.join().unwrap().trim_end(),
        Knock::on_behalf_of(person).written(),
        "the knock that reached the door was not the one this machine's person earns"
    );
    let Greeted::NotAnswered(why) = &greeted else {
        panic!("a door that said nothing was not told apart: {greeted:?}");
    };
    assert!(
        matches!(why, NotAnswered::NothingCameBack { .. }),
        "{why:?}"
    );
    assert_eq!(
        greeted.said(&in_english()).unwrap().text(),
        alo_greeting::NOTHING_SAID.says()
    );
    drop(std::fs::remove_file(&at));
}

/// **A door that answers something that is not an answer is refused rather
/// than read leniently**, and the person is told the machine did not finish
/// rather than shown whatever arrived.
#[test]
fn a_door_that_answers_nonsense_is_not_read_leniently() {
    let person = this_machines_person();
    let at = somewhere("nonsense");
    drop(std::fs::remove_file(&at));
    let listening = UnixListener::bind(&at).unwrap();

    let talking = std::thread::spawn(move || {
        let (connection, _) = listening.accept().unwrap();
        let mut line = String::new();
        drop(BufReader::new(&connection).read_line(&mut line));
        drop((&connection).write_all(b"opened 1000 and welcome\n"));
    });

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let greeted = greeting.signs_in("ada", HERS);
    talking.join().unwrap();

    let Greeted::NotAnswered(why) = &greeted else {
        panic!("a line that is not an answer was read as one: {greeted:?}");
    };
    assert!(matches!(why, NotAnswered::NotAnAnswer { .. }), "{why:?}");
    assert!(greeted.session().is_none());
    assert_eq!(
        greeted.said(&in_english()).unwrap().text(),
        alo_greeting::NOTHING_SAID.says()
    );
    drop(std::fs::remove_file(&at));
}

/// **A wrong password never reaches the door at all.** The refusal happens
/// before anything is connected to, which is the timing promise from the other
/// end: nothing on this machine can learn from a socket whether a name has an
/// account here.
#[test]
fn a_wrong_password_never_reaches_a_real_door() {
    let person = this_machines_person();
    let at = somewhere("untouched");
    drop(std::fs::remove_file(&at));
    let listening = UnixListener::bind(&at).unwrap();
    listening.set_nonblocking(true).unwrap();

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        TheOpenersDoor::at(&at),
    );
    let greeted = greeting.signs_in("ada", "not her password");
    assert_eq!(greeted, Greeted::Refused(alo_accounts::NOT_SIGNED_IN.key()));

    // Nobody connected: the listener has nothing waiting on it.
    assert!(
        matches!(listening.accept(), Err(why) if why.kind() == std::io::ErrorKind::WouldBlock),
        "something connected to the door for a password that was wrong"
    );
    drop(std::fs::remove_file(&at));
}

/// **A greeter built from a machine's own store reads that store**, and a
/// machine with no store at all is the first-boot state rather than a failure.
#[test]
fn a_greeter_reads_the_store_a_machine_keeps_and_first_boot_is_not_a_failure() {
    let person = this_machines_person();
    let folder = std::env::temp_dir().join(format!("alo-greeting-store-{}", std::process::id()));
    drop(std::fs::create_dir_all(&folder));
    let store = folder.join("accounts.toml");
    drop(std::fs::remove_file(&store));

    // Nothing there: the state every alo OS arrives in, because the image
    // ships no accounts.
    let greeting = Greeting::at(&store, person, TheOpenersDoor::on_this_machine()).unwrap();
    assert_eq!(greeting.standing(), alo_greeting::Standing::MakeAnAccount);

    // And with somebody in it, written by `alo-accounts`' own writer, the same
    // greeter stands at a sign-in.
    alo_accounts::kept(&store, &a_machine_with_ada_at(person)).unwrap();
    let greeting = Greeting::at(&store, person, TheOpenersDoor::on_this_machine()).unwrap();
    assert_eq!(greeting.standing(), alo_greeting::Standing::SignIn);
    drop(std::fs::remove_file(&store));
}

/// **A store that is there and will not be believed is neither state**: it is
/// a refusal, because a password box on that machine would be asking for a
/// keystroke nothing can check.
#[test]
fn a_store_that_will_not_be_believed_is_refused_rather_than_read_as_first_boot() {
    let person = this_machines_person();
    let folder = std::env::temp_dir().join(format!("alo-greeting-refused-{}", std::process::id()));
    drop(std::fs::create_dir_all(&folder));
    let store = folder.join("accounts.toml");
    std::fs::write(&store, "this is not a list of accounts").unwrap();

    let refused = Greeting::at(&store, person, TheOpenersDoor::on_this_machine()).unwrap_err();
    assert!(refused.at.ends_with("accounts.toml"), "{refused}");
    assert!(!refused.why.is_empty(), "{refused}");

    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Nobody can sign in here"), "{said}");
    drop(std::fs::remove_file(&store));
}
