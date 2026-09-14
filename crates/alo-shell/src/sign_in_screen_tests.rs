//! The sign-in screen against real accounts and a door that counts its knocks.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use alo_accounts::Accounts;
use alo_sessiond::{Answered, Knock};
use std::cell::RefCell;
use std::rc::Rc;

/// The uid the machine description names, and the account made for it.
const PERSON: u32 = 1000;
/// The one account's name.
const NAME: &str = "ada";
/// Its password, which nothing else in the process has ever seen.
const PASSWORD: &str = "hunter2-tourniquet-bramble";

/// How a door answers, and every knock it heard.
#[derive(Debug, Clone)]
struct CountingDoor {
    /// What it says to every knock.
    answers: Result<Answered, NotAnswered>,
    /// The uid of every knock, in order.
    heard: Rc<RefCell<Vec<u32>>>,
}

impl Knocking for CountingDoor {
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered> {
        self.heard.borrow_mut().push(knock.person());
        self.answers.clone()
    }
}

/// A door that answers so, and the list of what it heard.
fn a_door(answers: Result<Answered, NotAnswered>) -> (CountingDoor, Rc<RefCell<Vec<u32>>>) {
    let heard = Rc::new(RefCell::new(Vec::new()));
    (
        CountingDoor {
            answers,
            heard: Rc::clone(&heard),
        },
        heard,
    )
}

/// Everything this machine can say, as the shell would load it.
fn the_machines_words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A machine with one account on it, described for `person`.
fn a_screen(person: u32, door: CountingDoor) -> SignInScreen<CountingDoor> {
    let mut store = Accounts::none().unwrap();
    store.created(NAME, PERSON, PASSWORD).unwrap();
    SignInScreen::of(Ok(Greeting::of(store, person, door)), the_machines_words())
}

/// Type this, one key at a time, and require the screen to still be there.
fn typed(mut screen: SignInScreen<CountingDoor>, text: &str) -> SignInScreen<CountingDoor> {
    for letter in text.chars() {
        screen = still(screen.pressed(SignInKey::Letter(letter)));
    }
    screen
}

/// The screen, which must not have handed over.
fn still(signing: Signing<CountingDoor>) -> SignInScreen<CountingDoor> {
    match signing {
        Signing::Still(screen) => Some(*screen),
        Signing::HandedOver(_) => None,
    }
    .unwrap()
}

/// The session, which must have been handed over.
fn handed_over(signing: Signing<CountingDoor>) -> Session {
    match signing {
        Signing::HandedOver(session) => Some(session),
        Signing::Still(_) => None,
    }
    .unwrap()
}

/// A name and a password, typed the way a person does and entered.
fn signs_in(
    screen: SignInScreen<CountingDoor>,
    name: &str,
    password: &str,
) -> Signing<CountingDoor> {
    let screen = typed(screen, name);
    let screen = still(screen.pressed(SignInKey::Enter));
    let screen = typed(screen, password);
    screen.pressed(SignInKey::Enter)
}

/// The sentence a screen shows under its fields.
fn sentence(screen: &SignInScreen<CountingDoor>) -> Option<String> {
    match screen.shows() {
        SignInShows::Fields { said, .. } => said.map(|said| said.text().to_owned()),
        SignInShows::Sentence(said) => Some(said.text().to_owned()),
    }
}

/// **The right name and password open a session, and the screen hands over
/// to it**: one knock, for the described person, and no screen left to draw.
#[test]
fn the_right_password_opens_a_session_and_the_screen_hands_over() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let session = handed_over(signs_in(a_screen(PERSON, door), NAME, PASSWORD));
    assert_eq!(session.uid(), PERSON);
    assert_eq!(session.name(), NAME);
    assert_eq!(*heard.borrow(), [PERSON], "one approval, one knock");
}

/// **A wrong password says what `alo-greeting` says, and nothing more**: the
/// same sentence every time, the same sentence as a name nobody has, no
/// number in it, and no knock at all.
#[test]
fn a_wrong_password_says_what_the_greeting_says_and_never_how_many_or_whether_the_name_exists() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = a_screen(PERSON, door);

    let mut sentences = Vec::new();
    for (name, password) in [
        (NAME, "not-the-password"),
        (NAME, "still-not-it"),
        ("nobody-by-this-name", PASSWORD),
        ("", ""),
        (NAME, "third-time"),
    ] {
        screen = still(signs_in(screen, name, password));
        sentences.push(sentence(&screen).unwrap());
    }

    let greetings_own = alo_accounts::NOT_SIGNED_IN.says();
    for said in &sentences {
        assert_eq!(said, greetings_own, "the screen reworded the refusal");
        assert!(
            !said.chars().any(|letter| letter.is_ascii_digit()),
            "the refusal counts something: {said}"
        );
        assert!(!said.contains("nobody-by-this-name"), "{said}");
    }
    assert!(heard.borrow().is_empty(), "a refused sign-in knocked");
    assert_eq!(screen.for_the_maintainer(), None);
}

/// **What was typed is forgotten straight after it is asked about**, whatever
/// the answer: the password field is empty and the name field waiting, so
/// nothing a refused attempt typed is on the screen for the next person.
#[test]
fn what_was_typed_is_forgotten_once_it_is_asked_about() {
    let (door, _) = a_door(Ok(Answered::Opened));
    let screen = still(signs_in(a_screen(PERSON, door), NAME, "wrong"));
    assert!(matches!(
        screen.shows(),
        SignInShows::Fields {
            name: "",
            password_typed: false,
            field: SignInField::Name,
            said: Some(_),
        }
    ));
}

/// **Editing never asks anybody anything**: typing, erasing and moving
/// between fields knock on nothing, and Enter in the name field only moves
/// to the password.
#[test]
fn editing_asks_nobody_anything() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = typed(a_screen(PERSON, door), "adx");
    screen = still(screen.pressed(SignInKey::Erase));
    screen = typed(screen, "a");
    screen = still(screen.pressed(SignInKey::Enter));
    screen = typed(screen, "pw");
    screen = still(screen.pressed(SignInKey::OtherField));
    screen = still(screen.pressed(SignInKey::Nothing));
    assert!(matches!(
        screen.shows(),
        SignInShows::Fields {
            name: "ada",
            password_typed: true,
            field: SignInField::Name,
            said: None,
        }
    ));
    assert!(heard.borrow().is_empty());
}

/// **A door that refuses is shown in the door's own words**, one sentence
/// per refusal `alo-sessiond` can make.
#[test]
fn a_door_that_refuses_is_shown_in_its_own_words() {
    for word in alo_sessiond::EVERY_WORD {
        let (door, heard) = a_door(Ok(Answered::Refused(word.key())));
        let screen = still(signs_in(a_screen(PERSON, door), NAME, PASSWORD));
        assert_eq!(sentence(&screen).unwrap(), word.says(), "{}", word.named());
        assert_eq!(heard.borrow().len(), 1);
    }
}

/// **A door nobody is behind says it was not the password**, and keeps the
/// door's English for whoever maintains the machine — never anything typed.
#[test]
fn a_door_nobody_is_behind_says_so_and_tells_the_maintainer_where() {
    for (why, word) in [
        (
            NotAnswered::NobodyThere {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "No such file or directory".to_owned(),
            },
            alo_greeting::NOTHING_LISTENING,
        ),
        (
            NotAnswered::NothingCameBack {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "the door closed without answering".to_owned(),
            },
            alo_greeting::NOTHING_SAID,
        ),
    ] {
        let (door, _) = a_door(Err(why));
        let screen = still(signs_in(a_screen(PERSON, door), NAME, PASSWORD));
        assert_eq!(sentence(&screen).unwrap(), word.says());
        let english = screen.for_the_maintainer().unwrap();
        assert!(
            english.contains("/run/alo-sessiond/sign-in.sock"),
            "{english}"
        );
        assert!(!english.contains(PASSWORD), "{english}");
        assert!(!english.contains(NAME), "{english}");
    }
}

/// **A machine described for somebody else refuses in `alo-accounts`' words**
/// and knocks for nobody.
#[test]
fn a_machine_described_for_somebody_else_knocks_for_nobody() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let screen = still(signs_in(a_screen(PERSON + 1, door), NAME, PASSWORD));
    assert_eq!(
        sentence(&screen).unwrap(),
        alo_accounts::NOT_THIS_MACHINES_PERSON.says()
    );
    assert!(heard.borrow().is_empty());
}

/// **A machine nobody has an account on shows *make an account* and takes no
/// keys** — nothing is created here, and nothing is knocked on.
#[test]
fn a_machine_with_nobody_on_it_takes_no_keys_and_makes_no_account() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = SignInScreen::of(
        Ok(Greeting::of(Accounts::none().unwrap(), PERSON, door)),
        the_machines_words(),
    );
    for key in [
        SignInKey::Letter('a'),
        SignInKey::Enter,
        SignInKey::OtherField,
        SignInKey::Enter,
    ] {
        screen = still(screen.pressed(key));
    }
    assert!(matches!(screen.shows(), SignInShows::Sentence(_)));
    assert_eq!(
        sentence(&screen).unwrap(),
        alo_greeting::MAKE_AN_ACCOUNT.says()
    );
    assert!(heard.borrow().is_empty());
    assert!(matches!(screen.stands, Stands::MakeAnAccount(_)));
}

/// **Accounts that would not be read show `alo-greeting`'s sentence, take no
/// keys, and give the maintainer the file and the reason.**
#[test]
fn accounts_that_would_not_be_read_take_no_keys() {
    let refused = NotReadable {
        at: "/etc/alo/accounts.toml".to_owned(),
        why: "it is a symbolic link".to_owned(),
    };
    let mut screen: SignInScreen<CountingDoor> =
        SignInScreen::of(Err(refused), the_machines_words());
    screen = still(screen.pressed(SignInKey::Letter('a')));
    screen = still(screen.pressed(SignInKey::Enter));
    assert_eq!(
        sentence(&screen).unwrap(),
        alo_greeting::ACCOUNTS_UNREADABLE.says()
    );
    assert!(
        screen
            .for_the_maintainer()
            .unwrap()
            .contains("/etc/alo/accounts.toml")
    );
}

/// **Every sentence the screen can show is in the machine's vocabulary, and
/// none of them is this crate's.** Each road to a sentence is walked against
/// the vocabulary `alo-saying` collects; what comes back must not be marked
/// as a bug, must be one of the words of the three crates that decide what
/// is shown here, and must be none of the words the shell declares itself.
#[test]
fn every_sentence_the_screen_can_show_is_collected_and_none_is_written_here() {
    let deciders: Vec<&str> = alo_greeting::EVERY_WORD
        .iter()
        .chain(alo_accounts::EVERY_WORD.iter())
        .chain(alo_sessiond::EVERY_WORD.iter())
        .map(|word| word.says())
        .collect();
    let the_shells_own: Vec<&str> = crate::window_control_reader_words::READER_WORDS
        .iter()
        .map(|word| word.says())
        .collect();

    let mut shown: Vec<Said> = Vec::new();
    let mut collect = |screen: &SignInScreen<CountingDoor>| match screen.shows() {
        SignInShows::Fields { said, .. } => shown.extend(said.cloned()),
        SignInShows::Sentence(said) => shown.push(said.clone()),
    };

    // Standing at a sentence: nobody on the machine, and accounts refused.
    let (door, _) = a_door(Ok(Answered::Opened));
    collect(&SignInScreen::of(
        Ok(Greeting::of(Accounts::none().unwrap(), PERSON, door)),
        the_machines_words(),
    ));
    collect(&SignInScreen::of(
        Err(NotReadable {
            at: "/etc/alo/accounts.toml".to_owned(),
            why: "refused".to_owned(),
        }),
        the_machines_words(),
    ));
    // Refused before the door, both ways.
    let (door, _) = a_door(Ok(Answered::Opened));
    collect(&still(signs_in(a_screen(PERSON, door), NAME, "wrong")));
    let (door, _) = a_door(Ok(Answered::Opened));
    collect(&still(signs_in(a_screen(PERSON + 1, door), NAME, PASSWORD)));
    // Refused at the door, every way it can.
    for word in alo_sessiond::EVERY_WORD {
        let (door, _) = a_door(Ok(Answered::Refused(word.key())));
        collect(&still(signs_in(a_screen(PERSON, door), NAME, PASSWORD)));
    }
    // The door not answering, every way it can.
    for why in [
        NotAnswered::NobodyThere {
            at: "a".to_owned(),
            why: "b".to_owned(),
        },
        NotAnswered::WentAway {
            at: "a".to_owned(),
            why: "b".to_owned(),
        },
        NotAnswered::NothingCameBack {
            at: "a".to_owned(),
            why: "b".to_owned(),
        },
        NotAnswered::NotAnAnswer {
            at: "a".to_owned(),
            why: "b".to_owned(),
        },
    ] {
        let (door, _) = a_door(Err(why));
        collect(&still(signs_in(a_screen(PERSON, door), NAME, PASSWORD)));
    }

    assert_eq!(
        shown.len(),
        2 + 2 + 4 + 4,
        "a road to a sentence was not walked"
    );
    for said in &shown {
        assert!(!said.is_a_bug(), "not in the vocabulary: {said}");
        assert!(
            deciders.contains(&said.text()),
            "a sentence none of the deciding crates declares: {said}"
        );
        assert!(
            !the_shells_own.contains(&said.text()),
            "a sentence this crate declares: {said}"
        );
    }
}
