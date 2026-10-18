//! The sign-in screen against real accounts and a door that counts its knocks.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
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
    _name: &str,
    password: &str,
) -> Signing<CountingDoor> {
    // **A returning person types a password and nothing else.**
    //
    // `a_screen` makes a machine that already has an account, so this is the
    // screen a person comes back to: it draws one field, `Enter your PIN`,
    // over a name it already knows, and the name reaching `signs_in` is
    // `whose_name` rather than anything typed.
    //
    // This typed the name first, then Enter, then the password -- the
    // interaction of the screen that *asks* for a name, driven at a screen
    // that does not draw one. Eleven tests held that shape, and on a booted
    // machine on 2026-10-16 a person typing their PIN at this screen watched
    // nothing happen, because the keys were going into a field they could
    // not see. The tests passed throughout.
    //
    // `_name` stays in the signature so each caller still reads as the
    // account it means, and so a screen that one day asks for one again is a
    // change here rather than at every call.
    let screen = typed(screen, password);
    screen.pressed(SignInKey::Enter)
}

/// Sign in at a screen that **asks for a name**, typing both fields.
///
/// `a_screen(PERSON + 1, …)` is that screen: the store holds an account at
/// `PERSON`, so `whose_name` for `PERSON + 1` is `None`, the machine cannot
/// say whose it is, and both fields are drawn and asked for.
///
/// [`signs_in`] is the other screen — the one a person comes back to, which
/// knows the name and asks only for a password.
fn signs_in_with_a_name(
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
        SignInShows::Fields { said, .. } | SignInShows::FirstAccount { said, .. } => {
            said.map(|said| said.text().to_owned())
        }
        SignInShows::Sentence(said) => Some(said.text().to_owned()),
        // The language sheet shows no sentence of its own: it offers a list
        // and two actions, and what it would say about a refusal belongs to
        // the sheet that asks for a name.
        SignInShows::Language { .. } => None,
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
/// same sentence every time, no number in it, and no knock at all.
///
/// **The name is not among the things tried, and that is the change.** This
/// test used to type `nobody-by-this-name` to prove that an unknown name says
/// what a wrong password says. The screen a person comes back to **draws no
/// name field** — it knows whose machine it is — so a name cannot be offered
/// to it at all, and the property is now structural rather than behavioural.
/// `a_name_cannot_be_offered_to_the_screen_that_knows_whose_it_is` holds that
/// half, and the screen that *does* ask for a name holds the other.
#[test]
fn a_wrong_password_says_what_the_greeting_says_and_never_how_many() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = a_screen(PERSON, door);

    let mut sentences = Vec::new();
    for password in ["not-the-password", "still-not-it", "", "third-time"] {
        screen = still(signs_in(screen, NAME, password));
        sentences.push(sentence(&screen).unwrap());
    }

    let greetings_own = alo_accounts::NOT_SIGNED_IN.says();
    for said in &sentences {
        assert_eq!(said, greetings_own, "the screen reworded the refusal");
        assert!(
            !said.chars().any(|letter| letter.is_ascii_digit()),
            "the refusal counts something: {said}"
        );
    }
    assert!(heard.borrow().is_empty(), "a refused sign-in knocked");
    assert_eq!(screen.for_the_maintainer(), None);
}

/// **A name nobody has says what a wrong password says**, at the screen that
/// asks for one.
///
/// `PERSON + 1` is nobody this machine can name, so it draws both fields and
/// a name can be offered. This is where enumeration would show itself, and
/// the two sentences must be the one sentence.
#[test]
fn an_unknown_name_says_what_a_wrong_password_says_where_a_name_can_be_typed() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = a_screen(PERSON + 1, door);

    let mut sentences = Vec::new();
    for (name, password) in [
        (NAME, "not-the-password"),
        ("nobody-by-this-name", PASSWORD),
    ] {
        screen = still(signs_in_with_a_name(screen, name, password));
        sentences.push(sentence(&screen).unwrap());
    }

    assert_eq!(
        sentences[0], sentences[1],
        "a name nobody has is told apart from a password nobody typed"
    );
    for said in &sentences {
        assert!(!said.contains("nobody-by-this-name"), "{said}");
    }
    assert!(heard.borrow().is_empty(), "a refused sign-in knocked");
}

/// **A name cannot be offered to the screen that knows whose machine it is.**
///
/// It draws one field and `Tab` does not reach a second, so every key a
/// person presses goes into the password they can see. This is the structural
/// half of the property above, and the fault it closes: on a booted machine
/// on 2026-10-16 a returning person typed their PIN into a name field that
/// was not on the screen and watched nothing happen.
#[test]
fn a_name_cannot_be_offered_to_the_screen_that_knows_whose_it_is() {
    let (door, _heard) = a_door(Ok(Answered::Opened));
    let mut screen = a_screen(PERSON, door);

    screen = still(screen.pressed(SignInKey::OtherField));
    screen = typed(screen, "nobody-by-this-name");

    let SignInShows::Fields { name, field, .. } = screen.shows() else {
        panic!("the screen a person comes back to stopped showing fields")
    };
    assert_eq!(
        name, "",
        "a name reached a screen that draws no field for one"
    );
    assert_eq!(
        field,
        SignInField::Password,
        "Tab moved the waiting field away from the only field drawn"
    );
}

/// **What was typed is forgotten straight after it is asked about**, whatever
/// the answer, and the screen comes back to **the field it draws**.
///
/// Nothing a refused attempt typed is left for the next person to see.
///
/// **This asserted `SignInField::Name` until 2026-10-16, and that was the
/// bug.** `a_screen` is the screen a person comes back to: it draws one
/// field, `Enter your PIN`, over a name it already knows. Returning to the
/// name after a refusal sent every attempt but the first into a field nobody
/// can see — so a person who mistyped their PIN once could never sign in
/// again, and this test held that behaviour in place.
#[test]
fn what_was_typed_is_forgotten_once_it_is_asked_about() {
    let (door, _) = a_door(Ok(Answered::Opened));
    let screen = still(signs_in(a_screen(PERSON, door), NAME, "wrong"));

    let SignInShows::Fields {
        name,
        password_typed,
        field,
        said,
        ..
    } = screen.shows()
    else {
        panic!("the screen stopped showing fields after a refusal")
    };
    assert_eq!(name, "", "a refused name was left on the screen");
    assert!(!password_typed, "a refused password was left on the screen");
    assert_eq!(
        field,
        SignInField::Password,
        "a second attempt would go into a field this screen does not draw"
    );
    assert!(said.is_some(), "a refusal said nothing");
}

/// **Editing never asks anybody anything**: typing, erasing and moving
/// between fields knock on nothing, and Enter in the name field only moves
/// to the password.
#[test]
fn editing_asks_nobody_anything() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    // **To the name field first.** The returning screen starts on the
    // password it draws; this test is about editing both fields, and a person
    // reaches the other one the same way.
    // The screen that asks for a name: `PERSON + 1` is nobody this machine
    // can name, so both fields are drawn and this test can edit both.
    let mut screen = typed(a_screen(PERSON + 1, door), "adx");
    screen = still(screen.pressed(SignInKey::Erase));
    screen = typed(screen, "a");
    screen = still(screen.pressed(SignInKey::Enter));
    screen = typed(screen, "pw");
    screen = still(screen.pressed(SignInKey::OtherField));
    screen = still(screen.pressed(SignInKey::Nothing));
    // **Named one at a time rather than through `matches!`.** A `matches!`
    // that fails says only that it failed, which costs whoever reads it the
    // work of finding out which field moved.
    let SignInShows::Fields {
        name,
        password_typed,
        field,
        said,
        ..
    } = screen.shows()
    else {
        panic!("a screen with two fields stopped showing fields")
    };
    assert_eq!(name, "ada", "the name field");
    assert!(password_typed, "the password field");
    assert_eq!(field, SignInField::Name, "which field is waiting");
    assert!(said.is_none(), "editing said something: {said:?}");
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
    let screen = still(signs_in_with_a_name(
        a_screen(PERSON + 1, door),
        NAME,
        PASSWORD,
    ));
    assert_eq!(
        sentence(&screen).unwrap(),
        alo_accounts::NOT_THIS_MACHINES_PERSON.says()
    );
    assert!(heard.borrow().is_empty());
}

/// **A machine nobody has an account on offers a sheet to make one**, and a
/// greeting with nowhere to keep an account refuses rather than pretending.
///
/// This asserted *takes no keys and makes no account* until 2026-10-13, which
/// was true and was the fault: the screen said *make an account* and offered
/// nowhere to make one, so a fresh machine could not be used at all. What is
/// still true, and is what this now holds, is that **nothing is knocked on**
/// — this greeting was built from accounts in hand rather than read from a
/// store, so it has nowhere to write and says so instead of opening a session
/// against an account no restart would find.
#[test]
fn a_machine_with_nobody_on_it_is_offered_a_sheet_and_refused_without_a_store() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let mut screen = SignInScreen::of(
        Ok(Greeting::of(Accounts::none().unwrap(), PERSON, door)),
        the_machines_words(),
    );
    // Past the language sheet, then a name, a secret and submit — the walk a
    // person actually takes. The two Enters at the front choose the language
    // under the cursor and take `Use this language`.
    for key in [SignInKey::Enter, SignInKey::Enter] {
        screen = still(screen.pressed(key));
    }
    // A name, then a secret, then submit.
    // The old sequence moved between the fields without ever submitting,
    // which is how it proved keys were ignored.
    for key in [
        SignInKey::Letter('a'),
        SignInKey::Enter,
        SignInKey::Letter('b'),
        SignInKey::Enter,
    ] {
        screen = still(screen.pressed(key));
    }
    assert!(
        matches!(screen.shows(), SignInShows::FirstAccount { .. }),
        "a machine with nobody on it is not offered a sheet to make an account"
    );
    assert_eq!(
        sentence(&screen).unwrap(),
        alo_greeting::THE_ACCOUNT_WAS_NOT_KEPT.says(),
        "a greeting with nowhere to keep an account did not say so"
    );
    assert!(
        heard.borrow().is_empty(),
        "a refused account still knocked at the door"
    );
    assert!(matches!(screen.stands, Stands::MakeAnAccount(_, _)));
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
        SignInShows::Fields { said, .. } | SignInShows::FirstAccount { said, .. } => {
            shown.extend(said.cloned());
        }
        SignInShows::Sentence(said) => shown.push(said.clone()),
        // The language sheet has no sentence to collect.
        SignInShows::Language { .. } => {}
    };

    // **The first-start sheet says nothing until something is tried**, so it
    // is walked rather than merely built: a name and a secret at a greeting
    // with nowhere to keep an account answers the keeping refusal.
    let (door, _) = a_door(Ok(Answered::Opened));
    let mut fresh = SignInScreen::of(
        Ok(Greeting::of(Accounts::none().unwrap(), PERSON, door)),
        the_machines_words(),
    );
    for key in [
        // Past the language sheet: choose, then take `Use this language`.
        SignInKey::Enter,
        SignInKey::Enter,
        SignInKey::Letter('a'),
        SignInKey::Enter,
        SignInKey::Letter('b'),
        SignInKey::Enter,
    ] {
        fresh = still(fresh.pressed(key));
    }
    collect(&fresh);
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
    // `PERSON + 1` is nobody this machine can name, so it asks for one.
    collect(&still(signs_in_with_a_name(
        a_screen(PERSON + 1, door),
        NAME,
        PASSWORD,
    )));
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

/// **A second Enter cannot submit the same credentials again.**
///
/// The owner's direction of 2026-10-12 asks that duplicate submissions be
/// prevented while authentication is pending. On this screen they are
/// prevented structurally rather than by a flag, and this writes that down so
/// a later change to an asynchronous door cannot quietly lose it.
///
/// **A successful attempt leaves no screen at all.** `pressed` takes the
/// screen by value and hands back a session, so there is nothing to press
/// again while an attempt is in flight — a person leaning on Enter cannot
/// knock twice, because the thing they would press has been consumed.
///
/// **And a refused one forgets what it sent.** What was typed is cleared
/// before the answer is handled, so the screen that comes back could not
/// resubmit the same credentials even if somebody pressed it again.
#[test]
fn a_submission_leaves_no_screen_to_submit_again() {
    let (door, heard) = a_door(Ok(Answered::Opened));
    let session = handed_over(signs_in(a_screen(PERSON, door), NAME, PASSWORD));
    assert_eq!(session.uid(), PERSON);
    assert_eq!(
        *heard.borrow(),
        [PERSON],
        "one submission must be exactly one knock"
    );

    // And the refused road: the screen comes back with nothing typed, so the
    // next press is a different attempt rather than the same one again.
    let (door, heard) = a_door(Ok(Answered::Opened));
    let screen = still(signs_in(a_screen(PERSON, door), NAME, "not the password"));
    assert!(
        heard.borrow().is_empty(),
        "a refused password knocked on the door: {:?}",
        heard.borrow()
    );
    assert!(matches!(
        screen.shows(),
        SignInShows::Fields {
            whose: _,
            name: "",
            password_typed: false,
            ..
        }
    ));
}

/// A screen at the first-account sheet, with a name typed and nothing else.
///
/// The two Enters at the front are the language sheet's: one chooses the row
/// under the cursor, one takes *Use this language*. Both are a person's own
/// presses and neither is hidden, which is why the fixture makes them rather
/// than reaching past them.
fn at_the_first_account_sheet() -> SignInScreen<CountingDoor> {
    let (door, _heard) = a_door(Ok(Answered::Opened));
    let mut screen = SignInScreen::of(
        Ok(Greeting::of(Accounts::none().unwrap(), PERSON, door)),
        the_machines_words(),
    );
    for key in [SignInKey::Enter, SignInKey::Enter] {
        screen = still(screen.pressed(key));
    }
    for key in [
        SignInKey::Letter('d'),
        SignInKey::Letter('i'),
        SignInKey::Letter('s'),
    ] {
        screen = still(screen.pressed(key));
    }
    screen
}

/// **A refused first account keeps the name the person typed.**
///
/// Measured on a booted machine on 2026-10-18 and it cost a person their
/// typing: Enter on the name moves to the secret, a second Enter submits an
/// empty one, and the refusal arrived with the name field wiped.
///
/// **The secret still goes**, which is the half that must not regress — a
/// refusal that left a password in memory would be a refusal that kept it.
/// Both are asserted, because keeping the name is only right while the secret
/// is still forgotten.
#[test]
fn a_refused_first_account_keeps_the_name_and_forgets_the_secret() {
    let mut screen = at_the_first_account_sheet();
    assert_eq!(screen.entry.name(), "dis", "the premise: a name is typed");

    // Enter moves to the secret; Enter again submits an empty one.
    screen = still(screen.pressed(SignInKey::Enter));
    screen = still(screen.pressed(SignInKey::Enter));

    assert!(
        sentence(&screen).is_some(),
        "an empty secret was not refused, so this test is not about a refusal"
    );
    assert_eq!(
        screen.entry.name(),
        "dis",
        "a refusal took the name away, so a person types it again because they \
         pressed Enter once too often"
    );
    assert!(
        !screen.entry.has_password(),
        "a refusal kept the secret in memory"
    );
}

/// **A refusal stops speaking the moment the person types again.**
///
/// A refusal describes the attempt that produced it and nothing else. On the
/// booted machine of 2026-10-18 it outlived its attempt: the empty secret was
/// refused, the person typed their name again, went on to the PIN row, and
/// *an account needs a password, and an empty one is not one* was still under
/// it — a sentence about a sheet they had already left.
#[test]
fn a_refusal_does_not_follow_the_person_to_the_next_thing_they_type() {
    let mut screen = at_the_first_account_sheet();
    screen = still(screen.pressed(SignInKey::Enter));
    screen = still(screen.pressed(SignInKey::Enter));
    assert!(
        sentence(&screen).is_some(),
        "the premise: something was refused and the screen is saying so"
    );

    screen = still(screen.pressed(SignInKey::Letter('a')));

    assert!(
        sentence(&screen).is_none(),
        "a refusal about an attempt the person has already moved on from is \
         still on their screen"
    );
}
