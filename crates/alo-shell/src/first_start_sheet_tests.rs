//! A machine that knows nobody, made usable.
//!
//! **The test this whole road exists for** is the last one: a fresh machine
//! takes a name and a secret at its own screen and ends with somebody signed
//! in. `alo_greeting::makes_an_account` could do that from 2026-10-10 and
//! nothing called it, so a fresh machine had nobody to sign in as — the one
//! thing standing between alo OS and a computer somebody could use.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::{SignInKey, SignInScreen, Signing, WindowControlLabels, sign_in_raster::WhoseScreen};
use alo_accounts::Accounts;
use alo_appearance::{Scheme, TextScale};
use alo_greeting::{Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};

/// A door that opens for anybody who reaches it.
#[derive(Debug)]
struct ADoor;

impl Knocking for ADoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// The ordinary light look.
fn light() -> crate::SignInLook {
    crate::SignInLook {
        contrast: crate::Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    }
}

/// A machine with a store on a disk and nobody in it.
fn a_fresh_machine() -> (tempfile::TempDir, SignInScreen<ADoor>) {
    let kept = tempfile::tempdir().unwrap();
    let store = kept.path().join("accounts.toml");
    // An empty store that `found` will read, so the greeting has somewhere to
    // write the account it makes.
    alo_accounts::kept(&store, &Accounts::none().unwrap()).unwrap();
    let greeting = Greeting::at(&store, 1000, ADoor).expect("a fresh store reads");
    let words = crate::what_this_machine_can_say().unwrap();
    // On a clock in Ireland, whose one time zone Continue sets without asking
    // which (the owner, 2026-11-14).
    (
        kept,
        SignInScreen::of(Ok(greeting), words)
            .where_the_machine_is(alo_formats::RegionCode::named("IE").ok()),
    )
}

/// A fresh machine, walked past the first-start overview to the name sheet.
///
/// **The language sheet came first from 2026-10-13, and since 2026-10-28 the
/// overview of frame `09` does**, so a test about the account sheet has to
/// reach it the way a person does: Tab to *Continue*, and take it. Walking the
/// real journey rather than adding a constructor that skips it — a test that
/// built the name sheet directly would keep passing on a machine where the
/// overview never handed over.
fn at_the_name_sheet(screen: SignInScreen<ADoor>) -> SignInScreen<ADoor> {
    let mut screen = screen;
    for key in [crate::SignInKey::OtherField, crate::SignInKey::Enter] {
        screen = match screen.pressed(key) {
            Signing::Still(screen) => *screen,
            Signing::HandedOver(_) => panic!("the first-start overview signed somebody in"),
        };
    }
    screen
}

/// The picture a fresh machine draws.
fn drawn(screen: &SignInScreen<ADoor>) -> crate::sign_in_raster::SignInPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    crate::sign_in_raster::picture(
        screen.shows(),
        &mut labels,
        screen.words(),
        (1440, 960),
        light(),
        WhoseScreen::ItsOwn,
        crate::display_scale::DisplayScale::ONE,
    )
    .expect("a fresh machine's sheet draws")
}

/// How many pixels of an inked box differ from the ground it was shaped on.
fn marked(ink: &crate::painted::Inked) -> usize {
    let Some(ground) = ink.pixels.first().copied() else {
        return 0;
    };
    ink.pixels
        .iter()
        .filter(|p| {
            u32::from(p[0]).abs_diff(u32::from(ground[0]))
                + u32::from(p[1]).abs_diff(u32::from(ground[1]))
                + u32::from(p[2]).abs_diff(u32::from(ground[2]))
                > 24
        })
        .count()
}

/// **A machine with no accounts offers somewhere to make one.**
///
/// It showed one sentence and took no keys at all until 2026-10-13: the
/// greeting that can make an account was dropped on the way into the screen,
/// so *make an account* was advice with nothing behind it.
#[test]
fn a_machine_with_no_accounts_offers_a_sheet_to_make_one() {
    let (_kept, screen) = a_fresh_machine();
    let screen = at_the_name_sheet(screen);
    assert!(
        matches!(screen.shows(), crate::SignInShows::FirstAccount { .. }),
        "a fresh machine does not offer the first-start sheet"
    );
    let sheet = drawn(&screen);
    assert!(
        sheet.surface.is_some(),
        "the sheet has no raster, so it drew no wallpaper and no cards"
    );
    let ink: usize = sheet.inked.iter().map(marked).sum();
    assert!(
        ink > 300,
        "only {ink} inked pixels on the whole sheet: it is laid out and not lettered"
    );
}

/// **Typing a name changes the sheet**, so a person can see their keystrokes
/// arrived.
#[test]
fn what_is_typed_reaches_the_sheet() {
    let (_kept, screen) = a_fresh_machine();
    let screen = at_the_name_sheet(screen);
    let empty = drawn(&screen);
    let screen = match screen.pressed(SignInKey::Letter('a')) {
        Signing::Still(screen) => *screen,
        Signing::HandedOver(_) => panic!("a letter signed somebody in"),
    };
    assert_ne!(
        drawn(&screen).inked.len() + drawn(&screen).solids.len(),
        0,
        "nothing is drawn after a letter"
    );
    assert_ne!(
        empty.inked.iter().map(marked).sum::<usize>(),
        drawn(&screen).inked.iter().map(marked).sum::<usize>(),
        "the sheet is the same whatever is typed"
    );
}

/// **The secret is never drawn, however much of it is typed.**
///
/// The rule `crate::sign_in_raster` keeps for the sign-in screen, kept here:
/// two secrets of two lengths are the same picture, so nothing about one
/// reaches a screen somebody can read over a shoulder.
#[test]
fn two_secrets_of_different_lengths_are_the_same_sheet() {
    let typed = |secret: &str| {
        let (_kept, mut screen) = a_fresh_machine();
        let mut keys: Vec<SignInKey> = "ada".chars().map(SignInKey::Letter).collect();
        keys.push(SignInKey::Enter);
        keys.extend(secret.chars().map(SignInKey::Letter));
        for key in keys {
            screen = match screen.pressed(key) {
                Signing::Still(screen) => *screen,
                Signing::HandedOver(_) => panic!("nothing should sign in before the secret ends"),
            };
        }
        let picture = drawn(&screen);
        (picture.inked.len(), picture.solids.len())
    };
    assert_eq!(
        typed("short"),
        typed("a very much longer one indeed"),
        "the sheet changed with the length of the secret typed into it"
    );
}

/// **The PIN sheet promises no password it never asks for.**
///
/// It said *Keep your password for the situations that require it*, as the
/// design does, on a road where a person who chooses a PIN is never asked for
/// a password (the walk on `main` `8263ddcb`, 2026-11-01). Walked the way a
/// person does: the name, then the PIN sheet, and what it is told there.
#[test]
fn the_pin_sheet_promises_no_password_it_never_asks_for() {
    let (_kept, screen) = a_fresh_machine();
    let mut screen = at_the_name_sheet(screen);
    for key in "ada"
        .chars()
        .map(SignInKey::Letter)
        .chain([SignInKey::Enter])
    {
        screen = match screen.pressed(key) {
            Signing::Still(screen) => *screen,
            Signing::HandedOver(_) => panic!("nothing should sign in at the name"),
        };
    }
    let crate::SignInShows::FirstAccount { account, .. } = screen.shows() else {
        panic!("the name did not lead to the PIN sheet");
    };
    assert_eq!(
        account.sheet,
        crate::TheAccountSheet::Secret(crate::ASecret::Pin)
    );
    let said = screen
        .words()
        .say(
            &crate::first_start_words::PIN_INTRODUCTION.key(),
            &alo_strings::Filling::nothing(),
        )
        .into_text();
    assert_eq!(said, "Use a PIN to sign in on this computer.");
    assert!(
        !said.to_lowercase().contains("password"),
        "the PIN sheet speaks of a password this road never asks for: {said}"
    );
}

/// **A fresh machine makes its first account and signs somebody in.**
///
/// The whole road, end to end, through the real screen: a name, a secret, and
/// a session. Nothing had ever called `makes_an_account` before this, so a
/// machine with no accounts could not be used at all.
#[test]
fn a_fresh_machine_makes_its_first_account_and_signs_in() {
    let (kept, screen) = a_fresh_machine();
    // Past the language sheet first: this test is about the name sheet, and a
    // person reaches it by choosing a language.
    let mut screen = at_the_name_sheet(screen);
    let store = kept.path().join("accounts.toml");

    // **The PIN typed twice**, as the approved sheet asks (`562:508`).
    let mut keys: Vec<SignInKey> = "ada".chars().map(SignInKey::Letter).collect();
    keys.push(SignInKey::Enter);
    keys.extend("the-one-they-chose".chars().map(SignInKey::Letter));
    keys.push(SignInKey::Enter);
    keys.extend("the-one-they-chose".chars().map(SignInKey::Letter));
    keys.push(SignInKey::Enter);

    let mut opened = None;
    for key in keys {
        match screen.pressed(key) {
            Signing::Still(still) => screen = *still,
            Signing::HandedOver(session) => {
                opened = Some(session);
                break;
            }
        }
    }
    assert!(
        opened.is_some(),
        "a name and a secret at the first-start sheet did not open a session"
    );

    // **It is on the disk, not only in memory.** An account that signed
    // somebody in and was not kept is one they cannot use tomorrow.
    let found = alo_accounts::found(&store).expect("the store reads after the account was made");
    assert_eq!(
        found.how_many(),
        1,
        "the account was not written to the disk"
    );
    assert_eq!(
        found.name_of(1000),
        Some("ada"),
        "the account on the disk is not the one that was typed"
    );
}

/// A fresh machine's first account made through the real sheets — with a PIN,
/// or after *Use password only* — and the sign-in screen the **next boot**
/// draws from the store it left on the disk.
///
/// *Use password only* is reached by the keyboard, as a person without a
/// pointer reaches it: Tab past the two fields and *Save PIN*, then Enter.
fn the_next_boot_after_choosing(password_only: bool) -> (tempfile::TempDir, SignInScreen<ADoor>) {
    let (kept, screen) = a_fresh_machine();
    let mut screen = at_the_name_sheet(screen);
    let mut keys: Vec<SignInKey> = "dis".chars().map(SignInKey::Letter).collect();
    keys.push(SignInKey::Enter);
    if password_only {
        keys.extend([
            SignInKey::OtherField,
            SignInKey::OtherField,
            SignInKey::OtherField,
            SignInKey::Enter,
        ]);
    }
    for _ in 0..2 {
        keys.extend("4821-or-a-password".chars().map(SignInKey::Letter));
        keys.push(SignInKey::Enter);
    }
    let mut opened = false;
    for key in keys {
        match screen.pressed(key) {
            Signing::Still(still) => screen = *still,
            Signing::HandedOver(_) => {
                opened = true;
                break;
            }
        }
    }
    assert!(opened, "the first account did not sign anybody in");

    let store = kept.path().join("accounts.toml");
    let greeting = Greeting::at(&store, 1000, ADoor).expect("the store reads at the next boot");
    let words = crate::what_this_machine_can_say().unwrap();
    (kept, SignInScreen::of(Ok(greeting), words))
}

/// What the sign-in screen's one field is called, to a screen reader.
fn the_field_is_called(screen: &SignInScreen<ADoor>) -> String {
    let tree = crate::ReadAloudTree::before_sign_in(screen.words(), screen.shows());
    let in_front = tree.in_front().expect("the sign-in screen is in front");
    tree.nodes()
        .get(in_front.keyboard)
        .expect("the keyboard is on a node")
        .name
        .clone()
}

/// One of the screen's own words, said.
fn said(screen: &SignInScreen<ADoor>, word: alo_strings::Word) -> String {
    screen
        .words()
        .say(&word.key(), &alo_strings::Filling::nothing())
        .text()
        .to_owned()
}

/// **A person who pressed *Use password only* is asked for their password on
/// every boot after, and never for a PIN.**
///
/// Found on booted machines on three walks running (2026-11-12, -13, -14):
/// the first start took a password, and every later sign-in screen said
/// *Enter your PIN* — a PIN the person never set. The kind of secret was not
/// kept, so the screen could not know. This walks the whole road: the sheets,
/// the store on the disk, a fresh greeting read from it, and the screen.
#[test]
fn a_password_chosen_at_the_first_start_is_asked_for_at_every_sign_in() {
    let (_kept, screen) = the_next_boot_after_choosing(true);
    let crate::SignInShows::Fields { secret, .. } = screen.shows() else {
        panic!("the next boot did not open on the sign-in screen");
    };
    assert_eq!(secret, crate::SecretKind::Password);
    assert_eq!(
        the_field_is_called(&screen),
        said(&screen, crate::sign_in_words::ENTER_YOUR_PASSWORD)
    );
}

/// **And a person who chose a PIN is still asked for their PIN.**
#[test]
fn a_pin_chosen_at_the_first_start_is_asked_for_at_every_sign_in() {
    let (_kept, screen) = the_next_boot_after_choosing(false);
    let crate::SignInShows::Fields { secret, .. } = screen.shows() else {
        panic!("the next boot did not open on the sign-in screen");
    };
    assert_eq!(secret, crate::SecretKind::Pin);
    assert_eq!(
        the_field_is_called(&screen),
        said(&screen, crate::sign_in_words::ENTER_YOUR_PIN)
    );
}

/// **The screen is read from the store at every start, so a secret that
/// changes kind is asked for as its new kind** — a PIN where there was a
/// password, and a password where a recovery set one again over a PIN.
///
/// Nothing on alo OS sets a PIN after the first start yet; this holds the
/// half that would have to be true when something does, which is that the
/// screen does not remember what it asked for last time.
#[test]
fn a_secret_that_changes_kind_is_asked_for_as_its_new_kind() {
    let kept = tempfile::tempdir().unwrap();
    let store = kept.path().join("accounts.toml");
    for kind in [crate::SecretKind::Password, crate::SecretKind::Pin] {
        let mut accounts = Accounts::none().unwrap();
        accounts.created_with("ada", 1000, "theirs", kind).unwrap();
        alo_accounts::kept(&store, &accounts).unwrap();
        let greeting = Greeting::at(&store, 1000, ADoor).expect("the store reads");
        let words = crate::what_this_machine_can_say().unwrap();
        let screen = SignInScreen::of(Ok(greeting), words);
        let crate::SignInShows::Fields { secret, .. } = screen.shows() else {
            panic!("a machine with an account did not open on the sign-in screen");
        };
        assert_eq!(
            secret, kind,
            "the screen asked for what the store used to hold"
        );
    }

    // And a password set again with the recovery key replaces the PIN.
    let mut accounts = alo_accounts::found(&store).unwrap();
    accounts
        .password_set_again("ada", "a new password")
        .unwrap();
    alo_accounts::kept(&store, &accounts).unwrap();
    let greeting = Greeting::at(&store, 1000, ADoor).expect("the store reads");
    let words = crate::what_this_machine_can_say().unwrap();
    let screen = SignInScreen::of(Ok(greeting), words);
    let crate::SignInShows::Fields { secret, .. } = screen.shows() else {
        panic!("a machine with an account did not open on the sign-in screen");
    };
    assert_eq!(secret, crate::SecretKind::Password);
}

/// **A machine that already has an account never offers the sheet.**
///
/// The refusal `makes_an_account` holds from its own side, held here from the
/// surface's: a second account cannot be made because the screen never asks
/// for one.
#[test]
fn a_machine_with_an_account_is_not_offered_the_sheet() {
    let kept = tempfile::tempdir().unwrap();
    let store = kept.path().join("accounts.toml");
    let mut accounts = Accounts::none().unwrap();
    accounts.created("ada", 1000, "theirs").unwrap();
    alo_accounts::kept(&store, &accounts).unwrap();

    let greeting = Greeting::at(&store, 1000, ADoor).expect("the store reads");
    let words = crate::what_this_machine_can_say().unwrap();
    let screen = SignInScreen::of(Ok(greeting), words);
    assert!(
        matches!(screen.shows(), crate::SignInShows::Fields { .. }),
        "a machine with an account was offered the first-start sheet"
    );
}
