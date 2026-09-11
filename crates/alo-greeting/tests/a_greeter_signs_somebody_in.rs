//! The greeter's whole job, against the accounts and the description this
//! repository really ships.
//!
//! Every acceptance criterion of *what the greeter does, before there is
//! anything to draw* is here except the ones that need a real socket, which
//! are in `the_greeter_knocks_at_a_real_door.rs`, and the two that are checks
//! on this crate's own source.
//!
//! The door is a fixture with a counter behind it, because *one knock, for
//! that uid, and only after a password verified* is a statement about how many
//! times something happened and for whom — which a real service can show once
//! and a counter can show for every road through the composition, including
//! the roads where the right answer is that nothing was knocked on at all.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::time::{Duration, Instant};

use alo_accounts::Accounts;
use alo_greeting::{Greeted, Greeting, Knocking, NotAnswered, Standing};
use alo_sessiond::{ALREADY_SIGNED_IN, Answered, Knock, NOBODY_HERE};
use alo_strings::{Language, Said, Strings, Translation, Vocabulary};

/// Hers, and it is right.
const HERS: &str = "correct horse battery staple";

/// A door that remembers every knock and answers what it was told to.
#[derive(Debug)]
struct ADoor {
    /// Every number knocked for, in order.
    heard: RefCell<Vec<u32>>,
    /// What it answers each time.
    answers: Answered,
}

impl ADoor {
    /// A door that opens whatever it is asked for.
    fn willing() -> Self {
        Self {
            heard: RefCell::new(Vec::new()),
            answers: Answered::Opened,
        }
    }

    /// A door that refuses with this key.
    fn refusing(word: alo_strings::Word) -> Self {
        Self {
            heard: RefCell::new(Vec::new()),
            answers: Answered::Refused(word.key()),
        }
    }

    /// Every number it was knocked for.
    fn heard(&self) -> Vec<u32> {
        self.heard.borrow().clone()
    }
}

impl Knocking for ADoor {
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered> {
        self.heard.borrow_mut().push(knock.person());
        Ok(self.answers.clone())
    }
}

/// A door nothing ever reaches, which answers the way it was told to fail.
#[derive(Debug)]
struct ABrokenDoor(NotAnswered);

impl Knocking for ABrokenDoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Err(self.0.clone())
    }
}

/// The number this machine's description names, read with the reader
/// `alo-agentd` is told by — so a greeting is built against the number a real
/// machine uses rather than against a copy of it.
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

/// Everything this machine can say, which is what a real surface holds.
fn everything_this_machine_can_say() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// The machine's vocabulary, read in English.
fn in_english() -> Strings {
    Strings::of(everything_this_machine_can_say())
}

/// **A name and password that match an account produce a knock for that
/// account's uid and nothing else** — one knock, for the number the machine
/// description names, and a session back that carries it.
#[test]
fn a_name_and_password_that_match_produce_one_knock_for_that_account() {
    let person = this_machines_person();
    let door = ADoor::willing();
    let greeting = Greeting::of(a_machine_with_ada_at(person), person, door);

    assert_eq!(greeting.standing(), Standing::SignIn);
    let greeted = greeting.signs_in("ada", HERS);

    let session = greeted.session().unwrap();
    assert_eq!(session.uid(), person);
    assert_eq!(session.name(), "ada");
    assert!(greeted.said(&in_english()).is_none());

    // **Exactly one knock, for exactly that number.** A second would be a
    // second session asked for, and one for any other number would be this
    // machine asking for somebody nobody authenticated.
    assert_eq!(knocks_of(&greeting), vec![person]);
}

/// What a greeting's fixture door heard, for the assertions above and below.
fn knocks_of(greeting: &Greeting<ADoor>) -> Vec<u32> {
    greeting.door().heard()
}

/// **A wrong password and an unknown name are refused identically** — the same
/// sentence, `alo-accounts`' own, and neither of them knocks at anything, so
/// neither can be told from the other by a clock either.
#[test]
fn a_wrong_password_and_an_unknown_name_are_one_refusal_in_words_and_in_time() {
    let person = this_machines_person();
    let greeting = Greeting::of(a_machine_with_ada_at(person), person, ADoor::willing());
    let strings = in_english();

    let wrong = greeting.signs_in("ada", "not her password");
    let unknown = greeting.signs_in("grace", HERS);

    assert_eq!(wrong, unknown);
    assert_eq!(
        wrong,
        Greeted::Refused(alo_accounts::NOT_SIGNED_IN.key()),
        "the greeter reworded a refusal that alo-accounts already declares"
    );
    let said_of_wrong = wrong.said(&strings).unwrap();
    let said_of_unknown = unknown.said(&strings).unwrap();
    assert_eq!(said_of_wrong.text(), said_of_unknown.text());
    assert!(!said_of_wrong.is_a_bug(), "{said_of_wrong}");
    assert!(!said_of_wrong.text().contains("ada"), "{said_of_wrong}");
    assert!(
        !said_of_unknown.text().contains("grace"),
        "{said_of_unknown}"
    );

    // Nothing was asked of the door for either of them. A knock for a known
    // name and none for an unknown one would tell an attacker at this screen
    // which names have accounts here, over a wire, whatever the sentence said.
    assert!(knocks_of(&greeting).is_empty());

    // And in time: interleaved so drift hits both alike, medians so one
    // scheduler hiccup decides nothing. The same measurement `alo-accounts`
    // makes of itself, made of the composition above it — which is where it
    // would be lost, because this is the layer that could branch.
    let mut wrong_took = Vec::new();
    let mut unknown_took = Vec::new();
    for _ in 0..9 {
        wrong_took.push(refused_in(&greeting, "ada", "not her password"));
        unknown_took.push(refused_in(&greeting, "grace", HERS));
    }
    let wrong_median = median(wrong_took);
    let unknown_median = median(unknown_took);
    assert!(
        unknown_median * 3 >= wrong_median,
        "an unknown name ({unknown_median:?}) answers faster than a wrong password \
         ({wrong_median:?}), so names can be enumerated by timing"
    );
    assert!(
        wrong_median * 3 >= unknown_median,
        "a wrong password ({wrong_median:?}) answers faster than an unknown name \
         ({unknown_median:?}), so names can be enumerated by timing"
    );
}

/// One refusal, timed — and asserted to be the indistinguishable one, so a
/// measurement of some other failure cannot pass as this measurement.
fn refused_in(greeting: &Greeting<ADoor>, name: &str, secret: &str) -> Duration {
    let began = Instant::now();
    let greeted = greeting.signs_in(name, secret);
    let took = began.elapsed();
    assert_eq!(greeted, Greeted::Refused(alo_accounts::NOT_SIGNED_IN.key()));
    took
}

/// The middle value, which one scheduler hiccup cannot move.
fn median(mut took: Vec<Duration>) -> Duration {
    took.sort_unstable();
    took.get(took.len() / 2).copied().unwrap()
}

/// **A machine with no store answers *make an account* rather than a sign-in**,
/// which is ADR 0024's first-boot sentence and the state every alo OS arrives
/// in, because the image ships no accounts at all.
#[test]
fn a_machine_with_no_store_answers_make_an_account() {
    let person = this_machines_person();
    let greeting = Greeting::of(Accounts::none().unwrap(), person, ADoor::willing());

    assert_eq!(greeting.standing(), Standing::MakeAnAccount);
    let said = greeting.standing().said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Make one to sign in"), "{said}");

    // And nothing on that machine signs anybody in, including nobody with an
    // empty password — the refusal is the ordinary one, so a first-boot
    // machine gives away no more than any other.
    let greeted = greeting.signs_in("ada", "");
    assert_eq!(greeted, Greeted::Refused(alo_accounts::NOT_SIGNED_IN.key()));
    assert!(knocks_of(&greeting).is_empty());
}

/// **An account that is not this machine's described person knocks at
/// nothing.** The password was right and the session is still refused, in
/// `alo-accounts`' own words, because a session under a number the description
/// does not name would be a machine whose daemon and whose sign-in disagree
/// about who is signed in.
#[test]
fn an_account_that_is_not_this_machines_person_knocks_at_nothing() {
    let person = this_machines_person();
    let greeting = Greeting::of(a_machine_with_ada_at(person + 1), person, ADoor::willing());

    let greeted = greeting.signs_in("ada", HERS);
    assert_eq!(
        greeted,
        Greeted::Refused(alo_accounts::NOT_THIS_MACHINES_PERSON.key())
    );
    assert!(knocks_of(&greeting).is_empty());

    let said = greeted.said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("different account"), "{said}");
}

/// **What the door refuses is looked up and rendered in the person's own
/// language**, rather than handed on as the key it crossed the wire as. The
/// opener has a service log and no person in front of it; this side has the
/// person, the vocabulary and the language.
#[test]
fn what_the_door_refuses_is_shown_in_the_persons_own_language() {
    let person = this_machines_person();
    let german = Language::written("de").unwrap();
    let vocabulary = everything_this_machine_can_say();
    let translation = Translation::into_language(german.clone()).says(
        NOBODY_HERE.key(),
        "Die Anmeldung wurde nicht abgeschlossen: Dieser Rechner hat kein Konto mit dieser Nummer",
    );
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);

    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        ADoor::refusing(NOBODY_HERE),
    );
    let greeted = greeting.signs_in("ada", HERS);

    assert_eq!(greeted, Greeted::Refused(NOBODY_HERE.key()));
    let said: Said = greeted.said(&strings).unwrap();
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("Die Anmeldung"), "{said}");
    // The door was reached and it answered: this is the machine refusing, not
    // the conversation failing to happen.
    assert_eq!(knocks_of(&greeting), vec![person]);
}

/// **A second knock while somebody is signed in is the opener's sentence, not
/// this crate's.** The greeter has no memory of a session and must not grow
/// one: the component that opens sessions is the one that knows whether there
/// is one, and a second opinion here would be a screen that could disagree
/// with the machine.
#[test]
fn a_door_that_says_somebody_is_signed_in_already_is_carried_word_for_word() {
    let person = this_machines_person();
    let greeting = Greeting::of(
        a_machine_with_ada_at(person),
        person,
        ADoor::refusing(ALREADY_SIGNED_IN),
    );

    let greeted = greeting.signs_in("ada", HERS);
    assert_eq!(greeted, Greeted::Refused(ALREADY_SIGNED_IN.key()));
    assert_eq!(
        greeted.said(&in_english()).unwrap().text(),
        ALREADY_SIGNED_IN.says()
    );
}

/// **Every way the conversation can fail to happen is told in words**, rather
/// than as a screen that appears to have done nothing. Four values, two
/// sentences, and the English kept for whoever maintains the machine.
#[test]
fn every_way_the_conversation_can_fail_is_told_in_words() {
    let person = this_machines_person();
    let strings = in_english();
    let at = "/run/alo-sessiond/sign-in.sock".to_owned();

    for why in [
        NotAnswered::NobodyThere {
            at: at.clone(),
            why: "No such file or directory".to_owned(),
        },
        NotAnswered::WentAway {
            at: at.clone(),
            why: "Broken pipe".to_owned(),
        },
        NotAnswered::NothingCameBack {
            at: at.clone(),
            why: "the door closed without answering".to_owned(),
        },
        NotAnswered::NotAnAnswer {
            at: at.clone(),
            why: "`hello` is not an answer".to_owned(),
        },
    ] {
        let greeting = Greeting::of(
            a_machine_with_ada_at(person),
            person,
            ABrokenDoor(why.clone()),
        );
        let greeted = greeting.signs_in("ada", HERS);
        assert_eq!(greeted, Greeted::NotAnswered(why.clone()));

        let said = greeted.said(&strings).unwrap();
        assert!(!said.is_a_bug(), "{why} reached a person as a key");
        assert!(
            said.text().contains("Nothing you typed was wrong"),
            "{said}"
        );
        // And the detail a person is not shown is kept for whoever can act on
        // it, naming the door.
        assert!(why.to_string().contains("sign-in.sock"), "{why}");
    }
}
