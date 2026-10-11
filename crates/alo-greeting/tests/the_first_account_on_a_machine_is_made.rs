//! The road past *nobody has an account on this machine yet*.
//!
//! A machine renders that sentence from `how_many() == 0` and, until
//! `makes_an_account` existed, could do nothing about it: a boot on 2026-10-10
//! settled on that screen for three identical frames. These are the checks
//! that the road exists, that it ends where a person needs it to, and - the
//! longer half - that every way it can fail says so and leaves nothing behind.
//!
//! **The store is a real file in a real directory**, because the promise that
//! matters is not that an account exists in memory: it is that the machine
//! signs the same person in tomorrow. Half of these tests read the store back
//! from the disk with a second greeting, which is what a restart is.
//!
//! **No `panic!` in here**, which this crate forbids even in a test: a
//! `Greeted` is examined through the two readers below and compared, so a
//! wrong variant fails as an unequal value rather than as a thrown message.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;

use alo_accounts::Accounts;
use alo_greeting::{Greeted, Greeting, Knocking, NotAnswered, Standing};
use alo_sessiond::{Answered, Knock};

/// Theirs, and it is right.
const THEIRS: &str = "correct horse battery staple";

/// A door that opens for whatever it is knocked for, and remembers.
#[derive(Debug)]
struct ADoor(RefCell<Vec<u32>>);

impl ADoor {
    /// A door that opens whatever it is asked for.
    fn willing() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    /// Every number it was knocked for.
    fn heard(&self) -> Vec<u32> {
        self.0.borrow().clone()
    }
}

impl Knocking for ADoor {
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered> {
        self.0.borrow_mut().push(knock.person());
        Ok(Answered::Opened)
    }
}

/// The uid a greeting signed somebody in as, if it signed anybody in.
fn signed_in_as(greeted: &Greeted) -> Option<u32> {
    match greeted {
        Greeted::SignedIn(session) => Some(session.uid()),
        Greeted::Refused(_) | Greeted::NotAnswered(_) => None,
    }
}

/// The key a greeting refused with, if it refused.
fn refused_with(greeted: &Greeted) -> Option<alo_strings::Key> {
    match greeted {
        Greeted::Refused(key) => Some(key.clone()),
        Greeted::SignedIn(_) | Greeted::NotAnswered(_) => None,
    }
}

/// The number this machine's description names, read with the reader
/// `alo-agentd` is told by rather than a copy of it.
fn this_machines_person() -> u32 {
    alo_image::Image::at(std::path::Path::new(alo_image::THE_IMAGE))
        .unwrap()
        .description()
        .person()
}

/// A directory of this test's own, and the store path inside it.
fn somewhere_to_keep(what: &str) -> std::path::PathBuf {
    let folder =
        std::env::temp_dir().join(format!("alo-greeting-first-{what}-{}", std::process::id()));
    let _gone = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder.join("accounts.toml")
}

/// **A machine nobody has an account on makes one, and signs that person in.**
///
/// The whole point, in one test: the state the greeter has been stuck in since
/// it was built now has a road out of it, and the road ends in a session
/// rather than in a sentence.
#[test]
fn the_first_account_is_made_and_the_person_is_signed_in() {
    let store = somewhere_to_keep("made");
    let person = this_machines_person();
    let mut greeting = Greeting::at(&store, person, ADoor::willing()).unwrap();
    assert_eq!(greeting.standing(), Standing::MakeAnAccount);

    let greeted = greeting.makes_an_account("ada", THEIRS);

    assert_eq!(
        signed_in_as(&greeted),
        Some(person),
        "the first account did not sign this machine's person in: {greeted:?}"
    );
    assert_eq!(
        greeting.door().heard(),
        vec![person],
        "one knock, for this machine's person"
    );
}

/// **And the machine knows them tomorrow**, which is the promise a store is
/// for. A second greeting reads the disk the way a restart does.
#[test]
fn the_account_is_still_there_after_a_restart() {
    let store = somewhere_to_keep("restart");
    let person = this_machines_person();
    let mut first = Greeting::at(&store, person, ADoor::willing()).unwrap();
    let made = first.makes_an_account("ada", THEIRS);
    assert_eq!(signed_in_as(&made), Some(person), "{made:?}");

    let next_boot = Greeting::at(&store, person, ADoor::willing()).unwrap();

    assert_eq!(
        next_boot.standing(),
        Standing::SignIn,
        "a machine that made an account still asks for one to be made"
    );
    let again = next_boot.signs_in("ada", THEIRS);
    assert_eq!(
        signed_in_as(&again),
        Some(person),
        "the account did not survive a restart: {again:?}"
    );
}

/// **A second account is refused**, by the door and not by the screen.
#[test]
fn a_machine_that_has_an_account_refuses_another() {
    let store = somewhere_to_keep("second");
    let person = this_machines_person();
    let mut greeting = Greeting::at(&store, person, ADoor::willing()).unwrap();
    let _first = greeting.makes_an_account("ada", THEIRS);

    let again = greeting.makes_an_account("bo", THEIRS);

    assert_eq!(
        refused_with(&again),
        Some(alo_greeting::ALREADY_HAS_AN_ACCOUNT.key()),
        "a second account was made: {again:?}"
    );
}

/// **A greeting with nowhere to keep an account will not make one.**
///
/// `Greeting::of` is the fixture shape and holds no store. An account made in
/// memory would sign somebody in once and be gone by morning, with nothing on
/// the screen to say so, so it is refused instead.
#[test]
fn a_greeting_with_no_store_refuses_rather_than_keeping_it_in_memory() {
    let person = this_machines_person();
    let mut greeting = Greeting::of(Accounts::none().unwrap(), person, ADoor::willing());

    let greeted = greeting.makes_an_account("ada", THEIRS);

    assert_eq!(
        refused_with(&greeted),
        Some(alo_greeting::THE_ACCOUNT_WAS_NOT_KEPT.key()),
        "an account was made with nowhere to keep it: {greeted:?}"
    );
    assert!(
        greeting.door().heard().is_empty(),
        "nothing should have been knocked on"
    );
}

/// **A name that cannot be a login is refused, and says which.**
#[test]
fn a_name_that_is_not_a_name_is_refused() {
    let store = somewhere_to_keep("badname");
    let mut greeting = Greeting::at(&store, this_machines_person(), ADoor::willing()).unwrap();

    let greeted = greeting.makes_an_account("Ada Lovelace", THEIRS);

    assert_eq!(
        refused_with(&greeted),
        Some(alo_accounts::NOT_A_NAME.key()),
        "a name with a space was accepted: {greeted:?}"
    );
    assert_eq!(
        greeting.standing(),
        Standing::MakeAnAccount,
        "a refused name must leave the machine still asking for an account"
    );
}

/// **An account that could not be written is not an account in memory
/// either.**
///
/// The refusal says *it was not made*, and this is what makes that sentence
/// true rather than kind: if the store in memory kept the account after the
/// disk refused it, the greeter would move to *sign in*, the person would sign
/// in once, and the machine would have forgotten them by morning with nothing
/// having said so.
///
/// The disk is made to refuse by putting a **directory** where the store file
/// goes, which is a write that cannot succeed and needs no permissions trick
/// to arrange.
///
/// **Added after a mutation found nothing was checking it.** Breaking the
/// rollback left every other test passing.
#[test]
fn an_account_the_disk_refused_is_not_left_in_memory() {
    let store = somewhere_to_keep("unwritable");
    let person = this_machines_person();
    let mut greeting = Greeting::at(&store, person, ADoor::willing()).unwrap();
    // Where the file belongs, a directory now stands.
    std::fs::create_dir_all(&store).unwrap();

    let greeted = greeting.makes_an_account("ada", THEIRS);

    assert_eq!(
        refused_with(&greeted),
        Some(alo_greeting::THE_ACCOUNT_WAS_NOT_KEPT.key()),
        "a store that cannot be written did not refuse: {greeted:?}"
    );
    assert_eq!(
        greeting.standing(),
        Standing::MakeAnAccount,
        "the account survived in memory after the disk refused it, so the greeter moved on \
         from a machine that still has no account"
    );
    assert!(
        greeting.door().heard().is_empty(),
        "somebody was signed in on an account that was never written down"
    );
}

/// **An empty password is refused**, and is its own sentence rather than the
/// name's.
#[test]
fn an_account_with_no_password_is_refused() {
    let store = somewhere_to_keep("nopassword");
    let mut greeting = Greeting::at(&store, this_machines_person(), ADoor::willing()).unwrap();

    let greeted = greeting.makes_an_account("ada", "");

    assert_eq!(
        refused_with(&greeted),
        Some(alo_accounts::AN_ACCOUNT_NEEDS_A_PASSWORD.key()),
        "an account was made with no password: {greeted:?}"
    );
}

/// **The four refusals are four sentences**, so a person is never told to fix
/// the wrong thing. Three of them they can act on; the fourth says the machine
/// is at fault, which is the one case where retyping cannot help.
#[test]
fn the_refusals_do_not_say_the_same_thing() {
    let mut keys = vec![
        alo_accounts::NOT_A_NAME.key(),
        alo_accounts::AN_ACCOUNT_NEEDS_A_PASSWORD.key(),
        alo_accounts::NAME_IS_TAKEN.key(),
        alo_accounts::THE_ACCOUNT_WAS_NOT_MADE.key(),
        alo_greeting::ALREADY_HAS_AN_ACCOUNT.key(),
        alo_greeting::THE_ACCOUNT_WAS_NOT_KEPT.key(),
    ];
    let how_many = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), how_many, "two refusals share a sentence");
}

/// **Nothing is left behind by a refusal.** A machine that refused an account
/// is a machine with no account, on the disk as well as in the sentence.
#[test]
fn a_refused_account_leaves_the_machine_with_none() {
    let store = somewhere_to_keep("nothing-left");
    let person = this_machines_person();
    let mut greeting = Greeting::at(&store, person, ADoor::willing()).unwrap();

    let _refused = greeting.makes_an_account("Ada Lovelace", THEIRS);

    assert_eq!(greeting.standing(), Standing::MakeAnAccount);
    let next_boot = Greeting::at(&store, person, ADoor::willing()).unwrap();
    assert_eq!(
        next_boot.standing(),
        Standing::MakeAnAccount,
        "a refusal wrote something to the disk"
    );
}

/// **The screen moves on**, which is the whole of what a person sees. Before
/// the account the machine stands at *make an account*; after it, at *sign
/// in* - and that transition is what the boot of 2026-10-10 could not make.
#[test]
fn making_an_account_moves_the_machine_from_make_to_sign_in() {
    let store = somewhere_to_keep("standing");
    let mut greeting = Greeting::at(&store, this_machines_person(), ADoor::willing()).unwrap();
    assert_eq!(greeting.standing(), Standing::MakeAnAccount);

    let _made = greeting.makes_an_account("ada", THEIRS);

    assert_eq!(greeting.standing(), Standing::SignIn);
}
