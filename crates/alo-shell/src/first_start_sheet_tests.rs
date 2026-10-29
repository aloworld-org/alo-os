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
    (kept, SignInScreen::of(Ok(greeting), words))
}

/// A fresh machine, walked past the language and region sheets to the name
/// sheet.
///
/// **The language sheet comes first since 2026-10-13, and the region sheet
/// after it since 2026-10-28**, so a test about the account sheet has to reach
/// it the way a person does: on each, one Enter chooses the row under the
/// cursor and a second takes the way on. Walking the real journey rather than
/// adding a constructor that skips it — a test that built the name sheet
/// directly would keep passing on a machine where a sheet never handed over.
fn at_the_name_sheet(screen: SignInScreen<ADoor>) -> SignInScreen<ADoor> {
    let mut screen = screen;
    for _ in 0..4 {
        screen = match screen.pressed(crate::SignInKey::Enter) {
            Signing::Still(screen) => *screen,
            Signing::HandedOver(_) => panic!("a first-start sheet signed somebody in"),
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

    let mut keys: Vec<SignInKey> = "ada".chars().map(SignInKey::Letter).collect();
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
